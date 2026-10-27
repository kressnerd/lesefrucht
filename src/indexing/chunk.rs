//! Sentence-aware, chapter-bounded passage chunks.

use sha2::{Digest, Sha256};
use text_splitter::{Characters, ChunkConfig, TextSplitter};

use crate::domain::{Chapter, ChapterRef};

#[derive(Debug, Clone)]
pub struct Chunk {
    pub chapter: ChapterRef,
    pub passage_index: u32,
    pub char_start: u32,
    pub char_end: u32,
    pub text: String,
    pub text_hash: String,
}

pub struct Chunker {
    splitter: TextSplitter<Characters>,
}

impl Chunker {
    pub fn new(max_chars: usize, overlap_chars: usize) -> Self {
        let config = ChunkConfig::new(max_chars)
            .with_overlap(overlap_chars)
            .unwrap_or_else(|_| ChunkConfig::new(max_chars));
        Self {
            splitter: TextSplitter::new(config),
        }
    }

    /// Chunk one chapter. `chapter_start` is the chapter's global scalar offset.
    pub fn chunk(
        &self,
        chapter: &Chapter,
        chapter_start: u32,
        next_index: &mut u32,
    ) -> Vec<Chunk> {
        let mut chunks = Vec::new();
        for index in self.splitter.chunk_char_indices(&chapter.text) {
            let local_start = index.char_offset;
            let text = index.chunk;
            let length = text.chars().count() as u32;
            if length == 0 {
                continue;
            }
            let char_start = chapter_start + local_start as u32;
            let char_end = char_start + length;
            let mut hasher = Sha256::new();
            hasher.update(text.as_bytes());
            chunks.push(Chunk {
                chapter: ChapterRef::from(chapter),
                passage_index: *next_index,
                char_start,
                char_end,
                text: text.to_string(),
                text_hash: hex::encode(hasher.finalize()),
            });
            *next_index += 1;
        }
        chunks
    }
}
