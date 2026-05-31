//! Tests extracted from hudhudscript-rag/src/document.rs

use hudhudscript_rag::chunking::ChunkStrategy;
use hudhudscript_rag::document::{Document, DocumentFormat};
use std::collections::HashMap;

#[test]
fn test_document_creation() {
    let doc = Document::new("doc1", "Hello world", DocumentFormat::PlainText);
    assert_eq!(doc.id, "doc1");
    assert_eq!(doc.content, "Hello world");
    assert_eq!(doc.format, DocumentFormat::PlainText);
    assert!(doc.metadata.is_empty());
}

#[test]
fn test_document_with_metadata() {
    let mut meta = HashMap::new();
    meta.insert("author".to_string(), "test".to_string());
    let doc = Document::with_metadata("doc1", "content", DocumentFormat::Markdown, meta);
    assert_eq!(doc.metadata.get("author").unwrap(), "test");
}

#[test]
fn test_chunk_plaintext() {
    let doc = Document::new(
        "doc1",
        "Hello world. This is a test.",
        DocumentFormat::PlainText,
    );
    let chunks = doc.chunk(None);
    assert!(!chunks.is_empty());
    assert_eq!(chunks[0].metadata.get("doc_id").unwrap(), "doc1");
}

#[test]
fn test_chunk_markdown() {
    let content = "# Heading\n\nFirst paragraph.\n\nSecond paragraph.";
    let doc = Document::new("md1", content, DocumentFormat::Markdown);
    let chunks = doc.chunk(None);
    assert!(chunks.len() >= 2, "got {} chunks", chunks.len());
}

#[test]
fn test_chunk_code() {
    let content = "fn main() {\n    println!(\"hello\");\n}";
    let doc = Document::new("code1", content, DocumentFormat::Code);
    let chunks = doc.chunk(None);
    assert!(!chunks.is_empty());
    assert_eq!(chunks[0].metadata.get("format").unwrap(), "Code");
}

#[test]
fn test_chunk_with_custom_strategy() {
    let doc = Document::new(
        "doc1",
        "abcdefghijklmnopqrstuvwxyz",
        DocumentFormat::PlainText,
    );
    let chunks = doc.chunk(Some(ChunkStrategy::Fixed {
        size: 10,
        overlap: 0,
    }));
    assert_eq!(chunks.len(), 3);
}

#[test]
fn test_markdown_strips_html_comments() {
    let content = "Hello <!-- this is a comment --> world";
    let doc = Document::new("md1", content, DocumentFormat::Markdown);
    let preprocessed = doc.preprocess();
    assert!(!preprocessed.contains("comment"));
    assert!(preprocessed.contains("Hello"));
    assert!(preprocessed.contains("world"));
}

#[test]
fn test_markdown_nested_comments() {
    let content = "Before <!-- outer <!-- inner --> after --> End";
    let doc = Document::new("md1", content, DocumentFormat::Markdown);
    let preprocessed = doc.preprocess();
    assert!(preprocessed.contains("Before"));
    assert!(preprocessed.contains("End"));
    assert!(!preprocessed.contains("outer"));
}

#[test]
fn test_markdown_no_comments() {
    let content = "# Heading\n\nParagraph text here.";
    let doc = Document::new("md1", content, DocumentFormat::Markdown);
    let preprocessed = doc.preprocess();
    assert_eq!(preprocessed, content);
}

#[test]
fn test_code_preprocess_preserves_content() {
    let content = "fn main() { // comment\n    let x = 42;\n}";
    let doc = Document::new("c1", content, DocumentFormat::Code);
    let preprocessed = doc.preprocess();
    assert_eq!(preprocessed, content);
}

#[test]
fn test_chunk_empty_document() {
    let doc = Document::new("empty", "", DocumentFormat::PlainText);
    let chunks = doc.chunk(None);
    assert!(chunks.is_empty());
}

#[test]
fn test_document_format_in_chunk_metadata() {
    let doc = Document::new("doc1", "Some text.", DocumentFormat::PlainText);
    let chunks = doc.chunk(None);
    assert_eq!(chunks[0].metadata.get("format").unwrap(), "PlainText");
}
