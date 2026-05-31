//! Chunking strategies for splitting text into indexable chunks.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Strategy used to split text into chunks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChunkStrategy {
    /// Fixed-size chunks with optional overlap.
    Fixed {
        /// Maximum number of characters per chunk.
        size: usize,
        /// Number of overlapping characters between consecutive chunks.
        overlap: usize,
    },
    /// Semantic paragraph-based splitting (split on blank lines / paragraph boundaries).
    Semantic,
    /// Recursive splitting: tries paragraph boundaries first, then sentences,
    /// then falls back to fixed-size if a single unit exceeds `max_size`.
    Recursive {
        /// Maximum number of characters per chunk.
        max_size: usize,
    },
}

/// A chunk of text produced by the chunker.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Chunk {
    /// The chunk text.
    pub text: String,
    /// Byte offset of the chunk start in the original text.
    pub start: usize,
    /// Byte offset of the chunk end (exclusive) in the original text.
    pub end: usize,
    /// Arbitrary metadata attached to this chunk.
    pub metadata: HashMap<String, String>,
}

/// Chunker splits text into `Chunk`s according to a `ChunkStrategy`.
pub struct Chunker;

impl Chunker {
    /// Split `text` using the given `strategy`, returning a list of chunks.
    pub fn chunk(text: &str, strategy: ChunkStrategy) -> Vec<Chunk> {
        match strategy {
            ChunkStrategy::Fixed { size, overlap } => Self::chunk_fixed(text, size, overlap),
            ChunkStrategy::Semantic => Self::chunk_semantic(text),
            ChunkStrategy::Recursive { max_size } => Self::chunk_recursive(text, max_size),
        }
    }

    /// Fixed-size chunking with overlap.
    fn chunk_fixed(text: &str, size: usize, overlap: usize) -> Vec<Chunk> {
        if text.is_empty() || size == 0 {
            return Vec::new();
        }
        let effective_overlap = overlap.min(size.saturating_sub(1));
        let step = size - effective_overlap;
        let bytes = text.as_bytes();
        let len = bytes.len();
        let mut chunks = Vec::new();
        let mut pos = 0;

        while pos < len {
            let end = (pos + size).min(len);
            // Ensure we land on a char boundary
            let end = Self::snap_to_char_boundary(text, end);
            let start = Self::snap_to_char_boundary(text, pos);
            if start >= len {
                break;
            }
            let slice = &text[start..end];
            if !slice.is_empty() {
                chunks.push(Chunk {
                    text: slice.to_string(),
                    start,
                    end,
                    metadata: HashMap::new(),
                });
            }
            if end >= len {
                break;
            }
            pos += step;
        }
        chunks
    }

    /// Snap a byte position forward to the nearest UTF-8 char boundary.
    fn snap_to_char_boundary(text: &str, pos: usize) -> usize {
        let len = text.len();
        if pos >= len {
            return len;
        }
        let mut p = pos;
        while p < len && !text.is_char_boundary(p) {
            p += 1;
        }
        p
    }

    /// Semantic (paragraph-based) chunking. Splits on double newlines.
    fn chunk_semantic(text: &str) -> Vec<Chunk> {
        if text.is_empty() {
            return Vec::new();
        }
        let mut chunks = Vec::new();
        let mut search_start = 0;

        // Split on blank lines (two or more consecutive newlines)
        for segment in text.split("\n\n") {
            let trimmed = segment.trim();
            if trimmed.is_empty() {
                // advance past the separator
                search_start += segment.len() + 2; // +2 for the "\n\n"
                continue;
            }
            // Find this segment's position in the original text
            let start = match text[search_start..].find(trimmed) {
                Some(offset) => search_start + offset,
                None => search_start,
            };
            let end = start + trimmed.len();
            chunks.push(Chunk {
                text: trimmed.to_string(),
                start,
                end,
                metadata: HashMap::new(),
            });
            search_start = start + segment.len();
            // Account for the "\n\n" separator eaten by split
            if search_start < text.len() {
                search_start += 2;
            }
        }
        chunks
    }

    /// Recursive chunking: paragraph -> sentence -> fixed fallback.
    fn chunk_recursive(text: &str, max_size: usize) -> Vec<Chunk> {
        if text.is_empty() || max_size == 0 {
            return Vec::new();
        }

        // First, split by paragraphs
        let paragraphs = Self::chunk_semantic(text);
        let mut result = Vec::new();

        for para in paragraphs {
            if para.text.len() <= max_size {
                result.push(para);
            } else {
                // Split paragraph into sentences
                let sentences = Self::split_sentences(&para.text);
                let mut current_text = String::new();
                let mut current_start = para.start;

                for sentence in &sentences {
                    if current_text.len() + sentence.len() > max_size && !current_text.is_empty() {
                        let trimmed = current_text.trim().to_string();
                        let end = current_start + trimmed.len();
                        if !trimmed.is_empty() {
                            result.push(Chunk {
                                text: trimmed.clone(),
                                start: current_start,
                                end,
                                metadata: HashMap::new(),
                            });
                        }
                        current_start = end;
                        current_text.clear();
                    }
                    current_text.push_str(sentence);
                }

                // Flush remaining
                if !current_text.trim().is_empty() {
                    let trimmed = current_text.trim().to_string();
                    let end = current_start + trimmed.len();
                    result.push(Chunk {
                        text: trimmed,
                        start: current_start,
                        end,
                        metadata: HashMap::new(),
                    });
                }
            }
        }

        // Final pass: any chunk still over max_size gets fixed-split
        let mut final_chunks = Vec::new();
        for chunk in result {
            if chunk.text.len() <= max_size {
                final_chunks.push(chunk);
            } else {
                let sub = Self::chunk_fixed(&chunk.text, max_size, 0);
                for mut sc in sub {
                    sc.start += chunk.start;
                    sc.end += chunk.start;
                    final_chunks.push(sc);
                }
            }
        }

        final_chunks
    }

    /// Simple sentence splitter: splits on `. `, `! `, `? ` boundaries.
    pub fn split_sentences(text: &str) -> Vec<String> {
        let mut sentences = Vec::new();
        let mut current = String::new();
        let chars: Vec<char> = text.chars().collect();
        let len = chars.len();

        for i in 0..len {
            current.push(chars[i]);
            let is_terminator = chars[i] == '.' || chars[i] == '!' || chars[i] == '?';
            let followed_by_space = i + 1 < len && chars[i + 1] == ' ';
            if is_terminator && followed_by_space {
                sentences.push(current.clone());
                current.clear();
            }
        }
        if !current.is_empty() {
            sentences.push(current);
        }
        sentences
    }
}
