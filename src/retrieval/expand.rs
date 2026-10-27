//! Deterministic expansion: claim tokens + configured synonyms.

use std::collections::HashMap;
use std::path::Path;

use unicode_normalization::UnicodeNormalization;

use crate::domain::Expansion;

#[derive(Debug, Clone, Default)]
pub struct Expander {
    synonyms: HashMap<String, Vec<String>>,
}

impl Expander {
    pub fn load(path: &Path) -> Self {
        if path.as_os_str().is_empty() {
            return Self::default();
        }
        let Ok(raw) = std::fs::read_to_string(path) else {
            return Self::default();
        };
        let synonyms: HashMap<String, Vec<String>> = toml::from_str(&raw).unwrap_or_default();
        let synonyms = synonyms
            .into_iter()
            .map(|(key, values)| (key.to_lowercase(), values))
            .collect();
        Self { synonyms }
    }

    pub fn expand(&self, claim: &str) -> (Expansion, Vec<String>) {
        let claim_tokens = tokenize(claim);
        let mut expanded_terms = Vec::new();
        for token in &claim_tokens {
            if let Some(values) = self.synonyms.get(token) {
                for value in values {
                    if !expanded_terms
                        .iter()
                        .any(|term: &String| term.eq_ignore_ascii_case(value))
                    {
                        expanded_terms.push(value.to_lowercase());
                    }
                }
            }
        }
        let mut variants = claim_tokens.clone();
        for term in &expanded_terms {
            for token in tokenize(term) {
                if !variants.contains(&token) {
                    variants.push(token);
                }
            }
        }
        (
            Expansion {
                claim_tokens,
                expanded_terms,
            },
            variants,
        )
    }
}

pub fn tokenize(text: &str) -> Vec<String> {
    let normalized: String = text.nfkc().collect();
    let mut tokens = Vec::new();
    let mut current = String::new();
    for character in normalized.chars() {
        if character.is_alphabetic() || character.is_numeric() {
            current.extend(character.to_lowercase());
        } else if !current.is_empty() {
            tokens.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}
