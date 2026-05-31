//! Tests extracted from hudhudscript-rag/src/index.rs

use hudhudscript_rag::chunking::ChunkStrategy;
use hudhudscript_rag::document::{Document, DocumentFormat};
use hudhudscript_rag::provider::MockProvider;
use hudhudscript_rag::{Bm25Params, DocumentIndex, HybridWeights, IndexError};

fn make_index() -> DocumentIndex {
    DocumentIndex::new(Box::new(MockProvider::new(32)))
}

#[test]
fn test_add_document() {
    let mut idx = make_index();
    let doc = Document::new(
        "doc1",
        "Hello world. This is a test document.",
        DocumentFormat::PlainText,
    );
    let count = idx.add_document(doc).unwrap();
    assert!(count > 0);
    assert_eq!(idx.document_count(), 1);
    assert!(idx.contains("doc1"));
}

#[test]
fn test_add_duplicate_fails() {
    let mut idx = make_index();
    let doc1 = Document::new("doc1", "Content one.", DocumentFormat::PlainText);
    let doc2 = Document::new("doc1", "Content two.", DocumentFormat::PlainText);
    idx.add_document(doc1).unwrap();
    let result = idx.add_document(doc2);
    assert!(result.is_err());
}

#[test]
fn test_remove_document() {
    let mut idx = make_index();
    let doc = Document::new("doc1", "Hello world.", DocumentFormat::PlainText);
    idx.add_document(doc).unwrap();
    assert_eq!(idx.document_count(), 1);

    idx.remove_document("doc1").unwrap();
    assert_eq!(idx.document_count(), 0);
    assert!(!idx.contains("doc1"));
}

#[test]
fn test_remove_nonexistent() {
    let mut idx = make_index();
    let result = idx.remove_document("nonexistent");
    assert!(result.is_err());
}

#[test]
fn test_update_document() {
    let mut idx = make_index();
    let doc = Document::new("doc1", "Original content.", DocumentFormat::PlainText);
    idx.add_document(doc).unwrap();

    let updated = Document::new(
        "doc1",
        "Updated content with new information.",
        DocumentFormat::PlainText,
    );
    let count = idx.update_document(updated).unwrap();
    assert!(count > 0);
    assert_eq!(idx.document_count(), 1);

    let stored = idx.get_document("doc1").unwrap();
    assert!(stored.content.contains("Updated"));
}

#[test]
fn test_update_nonexistent() {
    let mut idx = make_index();
    let doc = Document::new("nonexistent", "Content.", DocumentFormat::PlainText);
    let result = idx.update_document(doc);
    assert!(result.is_err());
}

#[test]
fn test_search() {
    let mut idx = make_index();
    idx.add_document(Document::new(
        "doc1",
        "Rust programming language systems programming.",
        DocumentFormat::PlainText,
    ))
    .unwrap();
    idx.add_document(Document::new(
        "doc2",
        "Python scripting language data science.",
        DocumentFormat::PlainText,
    ))
    .unwrap();
    idx.add_document(Document::new(
        "doc3",
        "Cooking recipes pasta Italian food.",
        DocumentFormat::PlainText,
    ))
    .unwrap();

    let results = idx.search("programming language", 3).unwrap();
    assert!(!results.is_empty());
}

#[test]
fn test_search_empty_query() {
    let mut idx = make_index();
    idx.add_document(Document::new(
        "doc1",
        "Some content.",
        DocumentFormat::PlainText,
    ))
    .unwrap();

    let results = idx.search("", 5).unwrap();
    assert!(results.is_empty());
}

#[test]
fn test_chunk_count() {
    let mut idx = make_index();
    let doc = Document::new(
        "doc1",
        "First paragraph.\n\nSecond paragraph.\n\nThird paragraph.",
        DocumentFormat::Markdown,
    );
    let count = idx.add_document(doc).unwrap();
    assert_eq!(idx.chunk_count(), count);
}

#[test]
fn test_with_config() {
    let idx = DocumentIndex::with_config(
        Box::new(MockProvider::new(32)),
        HybridWeights {
            vector_weight: 0.5,
            keyword_weight: 0.5,
        },
        Bm25Params { k1: 2.0, b: 0.8 },
        Some(ChunkStrategy::Fixed {
            size: 100,
            overlap: 0,
        }),
    );
    assert_eq!(idx.document_count(), 0);
    assert_eq!(idx.chunk_count(), 0);
}

#[test]
fn test_search_after_remove() {
    let mut idx = make_index();
    idx.add_document(Document::new(
        "doc1",
        "Rust programming language.",
        DocumentFormat::PlainText,
    ))
    .unwrap();
    idx.remove_document("doc1").unwrap();
    let results = idx.search("programming", 5).unwrap();
    assert!(results.is_empty());
}

#[test]
fn test_update_then_search() {
    let mut idx = make_index();
    idx.add_document(Document::new(
        "doc1",
        "Original content about cats.",
        DocumentFormat::PlainText,
    ))
    .unwrap();
    idx.update_document(Document::new(
        "doc1",
        "Updated content about dogs and puppies.",
        DocumentFormat::PlainText,
    ))
    .unwrap();
    let doc = idx.get_document("doc1").unwrap();
    assert!(doc.content.contains("dogs"));
    assert!(!doc.content.contains("cats"));
}

#[test]
fn test_contains_after_operations() {
    let mut idx = make_index();
    assert!(!idx.contains("doc1"));
    idx.add_document(Document::new("doc1", "Hello.", DocumentFormat::PlainText))
        .unwrap();
    assert!(idx.contains("doc1"));
    idx.remove_document("doc1").unwrap();
    assert!(!idx.contains("doc1"));
}

#[test]
fn test_index_error_display() {
    let e = IndexError::NotFound("doc1".to_string());
    assert!(format!("{}", e).contains("document not found: doc1"));

    let e = IndexError::DuplicateId("doc1".to_string());
    assert!(format!("{}", e).contains("duplicate document id: doc1"));
}

#[test]
fn test_get_document() {
    let mut idx = make_index();
    let doc = Document::new("doc1", "Content here.", DocumentFormat::PlainText);
    idx.add_document(doc).unwrap();

    let retrieved = idx.get_document("doc1");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().content, "Content here.");

    assert!(idx.get_document("nonexistent").is_none());
}
