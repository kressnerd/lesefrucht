//! Tantivy BM25 over passages, with German and English stemming fields.

use std::collections::HashSet;
use std::path::Path;
use std::sync::Mutex;

use rust_stemmers::{Algorithm, Stemmer};
use tantivy::collector::TopDocs;
use tantivy::query::{BooleanQuery, Occur, TermQuery, TermSetQuery};
use tantivy::schema::{
    Field, INDEXED, IndexRecordOption, STORED, STRING, Schema, TantivyDocument, TextFieldIndexing,
    TextOptions, Value,
};
use tantivy::{Index, IndexReader, IndexWriter, Term, doc};

#[derive(Debug, thiserror::Error)]
pub enum LexicalError {
    #[error("tantivy: {0}")]
    Tantivy(#[from] tantivy::TantivyError),
    #[error("index directory: {0}")]
    Io(#[from] std::io::Error),
    #[error("tantivy query: {0}")]
    Query(#[from] tantivy::query::QueryParserError),
}

#[derive(Debug, Clone)]
pub struct LexicalFields {
    pub uid: Field,
    pub work_id: Field,
    pub edition_id: Field,
    pub passage_index: Field,
    pub text: Field,
    pub stem_de: Field,
    pub stem_en: Field,
}

#[derive(Debug, Clone)]
pub struct LexicalDoc {
    pub uid: String,
    pub work_id: String,
    pub edition_id: String,
    pub passage_index: u32,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct LexicalHit {
    pub uid: String,
    pub work_id: String,
    pub edition_id: String,
    pub passage_index: u32,
    pub score: f32,
}

pub struct LexicalIndex {
    index: Index,
    fields: LexicalFields,
    writer: Mutex<IndexWriter>,
    reader: IndexReader,
}

impl LexicalIndex {
    pub fn open(dir: &Path) -> Result<Self, LexicalError> {
        std::fs::create_dir_all(dir)?;
        let (schema, fields) = schema();
        let index = match Index::open_in_dir(dir) {
            Ok(index) => index,
            Err(_) => Index::create_in_dir(dir, schema.clone())?,
        };
        let writer = index.writer(64_000_000)?;
        let reader = index.reader()?;
        Ok(Self {
            index,
            fields,
            writer: Mutex::new(writer),
            reader,
        })
    }

    pub fn add(&self, document: &LexicalDoc) -> Result<(), LexicalError> {
        let writer = self.writer.lock().expect("lexical writer");
        let (stem_de, stem_en) = (stem(&document.text, Algorithm::German), stem(&document.text, Algorithm::English));
        writer.add_document(doc!(
            self.fields.uid => document.uid.clone(),
            self.fields.work_id => document.work_id.clone(),
            self.fields.edition_id => document.edition_id.clone(),
            self.fields.passage_index => u64::from(document.passage_index),
            self.fields.text => document.text.clone(),
            self.fields.stem_de => stem_de,
            self.fields.stem_en => stem_en,
        ))?;
        Ok(())
    }

    pub fn commit(&self) -> Result<(), LexicalError> {
        let mut writer = self.writer.lock().expect("lexical writer");
        writer.commit()?;
        self.reader.reload()?;
        Ok(())
    }

    pub fn delete_edition(&self, edition_id: &str) -> Result<(), LexicalError> {
        let writer = self.writer.lock().expect("lexical writer");
        writer.delete_term(Term::from_field_text(self.fields.edition_id, edition_id));
        Ok(())
    }

    pub fn search(
        &self,
        tokens: &[String],
        allowed: Option<&HashSet<String>>,
        limit: usize,
    ) -> Result<Vec<LexicalHit>, LexicalError> {
        if tokens.is_empty() || limit == 0 {
            return Ok(Vec::new());
        }
        let mut clauses: Vec<(Occur, Box<dyn tantivy::query::Query>)> = Vec::new();
        for token in tokens {
            for algorithm in [Algorithm::German, Algorithm::English] {
                let term = Term::from_field_text(
                    if algorithm == Algorithm::German {
                        self.fields.stem_de
                    } else {
                        self.fields.stem_en
                    },
                    &stem_token(token, algorithm),
                );
                clauses.push((
                    Occur::Should,
                    Box::new(TermQuery::new(term, IndexRecordOption::WithFreqs)),
                ));
            }
        }
        let token_query = BooleanQuery::new(clauses);
        let query: Box<dyn tantivy::query::Query> = match allowed {
            Some(work_ids) if !work_ids.is_empty() => {
                let terms: Vec<Term> = work_ids
                    .iter()
                    .map(|work_id| Term::from_field_text(self.fields.work_id, work_id))
                    .collect();
                Box::new(BooleanQuery::new(vec![
                    (Occur::Must, Box::new(token_query)),
                    (Occur::Must, Box::new(TermSetQuery::new(terms))),
                ]))
            }
            Some(_) => return Ok(Vec::new()),
            None => Box::new(token_query),
        };
        let searcher = self.reader.searcher();
        let top = searcher.search(&query, &TopDocs::with_limit(limit).order_by_score())?;
        let mut hits = Vec::with_capacity(top.len());
        for (score, address) in top {
            let document: TantivyDocument = searcher.doc(address)?;
            let get = |field: Field| {
                document
                    .get_first(field)
                    .and_then(|value| value.as_str())
                    .unwrap_or_default()
                    .to_string()
            };
            hits.push(LexicalHit {
                uid: get(self.fields.uid),
                work_id: get(self.fields.work_id),
                edition_id: get(self.fields.edition_id),
                passage_index: document
                    .get_first(self.fields.passage_index)
                    .and_then(|value| value.as_u64())
                    .unwrap_or_default() as u32,
                score,
            });
        }
        Ok(hits)
    }

    pub fn index(&self) -> &Index {
        &self.index
    }
}

fn schema() -> (Schema, LexicalFields) {
    let mut builder = Schema::builder();
    let text_indexing = TextFieldIndexing::default()
        .set_tokenizer("default")
        .set_index_option(IndexRecordOption::WithFreqsAndPositions);
    let text_options = TextOptions::default()
        .set_indexing_options(text_indexing.clone())
        .set_stored();
    let stem_options = TextOptions::default().set_indexing_options(text_indexing);
    let uid = builder.add_text_field("uid", STRING | STORED);
    let work_id = builder.add_text_field("work_id", STRING | STORED);
    let edition_id = builder.add_text_field("edition_id", STRING | STORED);
    let passage_index = builder.add_u64_field("passage_index", INDEXED | STORED);
    let text = builder.add_text_field("text", text_options);
    let stem_de = builder.add_text_field("stem_de", stem_options.clone());
    let stem_en = builder.add_text_field("stem_en", stem_options);
    (
        builder.build(),
        LexicalFields {
            uid,
            work_id,
            edition_id,
            passage_index,
            text,
            stem_de,
            stem_en,
        },
    )
}

fn stem(text: &str, algorithm: Algorithm) -> String {
    let stemmer = Stemmer::create(algorithm);
    text.split_whitespace()
        .map(|token| stem_token(token, algorithm))
        .map(|token| {
            if token.is_empty() {
                String::new()
            } else {
                stemmer.stem(&token).into_owned()
            }
        })
        .filter(|token| !token.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn stem_token(token: &str, algorithm: Algorithm) -> String {
    let lowered = token.to_lowercase();
    let cleaned: String = lowered
        .chars()
        .filter(|character| character.is_alphanumeric())
        .collect();
    if cleaned.is_empty() {
        return cleaned;
    }
    Stemmer::create(algorithm).stem(&cleaned).into_owned()
}
