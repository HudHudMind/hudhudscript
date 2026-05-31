//! Tests extracted from hudhudscript-rag/src/chunking.rs

use hudhudscript_rag::chunking::{ChunkStrategy, Chunker};

#[test]
fn test_fixed_chunking_basic() {
    let text = "abcdefghij";
    let chunks = Chunker::chunk(
        text,
        ChunkStrategy::Fixed {
            size: 4,
            overlap: 0,
        },
    );
    assert_eq!(chunks.len(), 3);
    assert_eq!(chunks[0].text, "abcd");
    assert_eq!(chunks[1].text, "efgh");
    assert_eq!(chunks[2].text, "ij");
}

#[test]
fn test_fixed_chunking_with_overlap() {
    let text = "abcdefghij";
    let chunks = Chunker::chunk(
        text,
        ChunkStrategy::Fixed {
            size: 4,
            overlap: 2,
        },
    );
    assert_eq!(chunks[0].text, "abcd");
    assert_eq!(chunks[1].text, "cdef");
    assert!(chunks.len() >= 3);
}

#[test]
fn test_fixed_empty() {
    let chunks = Chunker::chunk(
        "",
        ChunkStrategy::Fixed {
            size: 4,
            overlap: 0,
        },
    );
    assert!(chunks.is_empty());
}

#[test]
fn test_semantic_chunking() {
    let text = "First paragraph.\n\nSecond paragraph.\n\nThird paragraph.";
    let chunks = Chunker::chunk(text, ChunkStrategy::Semantic);
    assert_eq!(chunks.len(), 3);
    assert_eq!(chunks[0].text, "First paragraph.");
    assert_eq!(chunks[1].text, "Second paragraph.");
    assert_eq!(chunks[2].text, "Third paragraph.");
}

#[test]
fn test_semantic_empty() {
    let chunks = Chunker::chunk("", ChunkStrategy::Semantic);
    assert!(chunks.is_empty());
}

#[test]
fn test_semantic_single_paragraph() {
    let text = "Just one paragraph here.";
    let chunks = Chunker::chunk(text, ChunkStrategy::Semantic);
    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0].text, "Just one paragraph here.");
}

#[test]
fn test_recursive_chunking_small() {
    let text = "Short text.";
    let chunks = Chunker::chunk(text, ChunkStrategy::Recursive { max_size: 100 });
    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0].text, "Short text.");
}

#[test]
fn test_recursive_chunking_splits_paragraphs() {
    let text = "First para.\n\nSecond para.";
    let chunks = Chunker::chunk(text, ChunkStrategy::Recursive { max_size: 20 });
    assert!(chunks.len() >= 2);
}

#[test]
fn test_chunk_offsets() {
    let text = "Hello world";
    let chunks = Chunker::chunk(
        text,
        ChunkStrategy::Fixed {
            size: 5,
            overlap: 0,
        },
    );
    assert_eq!(chunks[0].start, 0);
    assert_eq!(chunks[0].end, 5);
}

#[test]
fn test_fixed_zero_size() {
    let chunks = Chunker::chunk(
        "hello",
        ChunkStrategy::Fixed {
            size: 0,
            overlap: 0,
        },
    );
    assert!(chunks.is_empty());
}

#[test]
fn test_fixed_overlap_equals_size() {
    // overlap >= size-1 means effective overlap = size-1, step = 1
    let text = "abcdef";
    let chunks = Chunker::chunk(
        text,
        ChunkStrategy::Fixed {
            size: 3,
            overlap: 3,
        },
    );
    // step = 1 (3 - min(3, 2) = 1), so we get many small chunks
    assert!(chunks.len() >= 2);
    assert_eq!(chunks[0].text, "abc");
}

#[test]
fn test_recursive_empty_text() {
    let chunks = Chunker::chunk("", ChunkStrategy::Recursive { max_size: 100 });
    assert!(chunks.is_empty());
}

#[test]
fn test_recursive_zero_max_size() {
    let chunks = Chunker::chunk("hello world", ChunkStrategy::Recursive { max_size: 0 });
    assert!(chunks.is_empty());
}

#[test]
fn test_recursive_long_sentence_fallback_to_fixed() {
    // A single paragraph with one long sentence exceeding max_size
    let long_sentence = "a".repeat(200);
    let chunks = Chunker::chunk(&long_sentence, ChunkStrategy::Recursive { max_size: 50 });
    assert!(
        chunks.len() >= 4,
        "expected at least 4 fixed-split chunks, got {}",
        chunks.len()
    );
    for chunk in &chunks {
        assert!(chunk.text.len() <= 50);
    }
}

#[test]
fn test_semantic_with_blank_paragraphs() {
    let text = "First.\n\n\n\nSecond.";
    let chunks = Chunker::chunk(text, ChunkStrategy::Semantic);
    assert_eq!(chunks.len(), 2);
    assert_eq!(chunks[0].text, "First.");
    assert_eq!(chunks[1].text, "Second.");
}

#[test]
fn test_recursive_sentence_splitting() {
    // Two paragraphs, second is long enough to need sentence splitting
    let text = "Short paragraph.\n\nFirst sentence here. Second sentence here. Third sentence here. Fourth sentence here.";
    let chunks = Chunker::chunk(text, ChunkStrategy::Recursive { max_size: 50 });
    assert!(chunks.len() >= 2);
}

#[test]
fn test_split_sentences() {
    let sentences = Chunker::split_sentences("Hello world. How are you? Fine! Thanks.");
    assert_eq!(sentences.len(), 4);
}
