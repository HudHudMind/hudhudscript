//! Document model with format-aware processing.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::chunking::{Chunk, ChunkStrategy, Chunker};

/// Supported document formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocumentFormat {
    /// Plain text without any special structure.
    PlainText,
    /// Markdown with headings, lists, code blocks, etc.
    Markdown,
    /// Source code (any programming language).
    Code,
}

/// A document with its content, format, and metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    /// Unique identifier for this document.
    pub id: String,
    /// The document content.
    pub content: String,
    /// The document format.
    pub format: DocumentFormat,
    /// Arbitrary key-value metadata.
    pub metadata: HashMap<String, String>,
}

impl Document {
    /// Create a new document.
    pub fn new(id: impl Into<String>, content: impl Into<String>, format: DocumentFormat) -> Self {
        Self {
            id: id.into(),
            content: content.into(),
            format,
            metadata: HashMap::new(),
        }
    }

    /// Create a new document with metadata.
    pub fn with_metadata(
        id: impl Into<String>,
        content: impl Into<String>,
        format: DocumentFormat,
        metadata: HashMap<String, String>,
    ) -> Self {
        Self {
            id: id.into(),
            content: content.into(),
            format,
            metadata,
        }
    }

    /// Chunk this document using a format-aware strategy.
    ///
    /// If `strategy` is `None`, an appropriate default strategy is chosen
    /// based on the document format.
    pub fn chunk(&self, strategy: Option<ChunkStrategy>) -> Vec<Chunk> {
        let strategy = strategy.unwrap_or_else(|| self.default_strategy());
        let text = self.preprocess();
        let mut chunks = Chunker::chunk(&text, strategy);

        // Attach document metadata to each chunk
        for chunk in &mut chunks {
            chunk.metadata.insert("doc_id".to_string(), self.id.clone());
            chunk
                .metadata
                .insert("format".to_string(), format!("{:?}", self.format));
        }

        chunks
    }

    /// Return the default chunking strategy for this document's format.
    fn default_strategy(&self) -> ChunkStrategy {
        match self.format {
            DocumentFormat::PlainText => ChunkStrategy::Recursive { max_size: 512 },
            DocumentFormat::Markdown => ChunkStrategy::Semantic,
            DocumentFormat::Code => ChunkStrategy::Recursive { max_size: 1024 },
        }
    }

    /// Preprocess the document content based on its format.
    pub fn preprocess(&self) -> String {
        match self.format {
            DocumentFormat::PlainText => self.content.clone(),
            DocumentFormat::Markdown => self.preprocess_markdown(),
            DocumentFormat::Code => self.preprocess_code(),
        }
    }

    /// Preprocess markdown: strip HTML comments, normalize whitespace.
    fn preprocess_markdown(&self) -> String {
        let mut result = String::with_capacity(self.content.len());
        let mut in_comment = false;

        let chars: Vec<char> = self.content.chars().collect();
        let len = chars.len();
        let mut i = 0;

        while i < len {
            if !in_comment
                && i + 3 < len
                && chars[i] == '<'
                && chars[i + 1] == '!'
                && chars[i + 2] == '-'
                && chars[i + 3] == '-'
            {
                in_comment = true;
                i += 4;
                continue;
            }
            if in_comment
                && i + 2 < len
                && chars[i] == '-'
                && chars[i + 1] == '-'
                && chars[i + 2] == '>'
            {
                in_comment = false;
                i += 3;
                continue;
            }
            if !in_comment {
                result.push(chars[i]);
            }
            i += 1;
        }

        result
    }

    /// Preprocess code: keep as-is (preserve structure).
    fn preprocess_code(&self) -> String {
        self.content.clone()
    }
}
