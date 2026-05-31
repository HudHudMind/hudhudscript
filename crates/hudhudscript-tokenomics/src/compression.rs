//! Prompt compression pipeline — reduce token count while preserving meaning

use std::collections::HashMap;

/// Compression level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionLevel {
    /// No compression
    None,
    /// Light: remove redundant whitespace, boilerplate
    Light,
    /// Medium: TF-IDF based selective pruning (~30-50% reduction)
    Medium,
    /// Aggressive: heavy pruning (~50-70% reduction)
    Aggressive,
}

/// Result of compression
#[derive(Debug, Clone)]
pub struct CompressionResult {
    pub compressed: String,
    pub original_tokens: usize,
    pub compressed_tokens: usize,
    pub reduction_pct: f64,
    pub method: String,
}

/// Prompt compressor with TF-IDF based selective pruning
pub struct PromptCompressor {
    level: CompressionLevel,
    /// Stop words to remove in medium/aggressive mode
    stop_words: Vec<&'static str>,
}

impl PromptCompressor {
    pub fn new(level: CompressionLevel) -> Self {
        Self {
            level,
            stop_words: vec![
                "the", "a", "an", "is", "are", "was", "were", "be", "been", "being", "have", "has",
                "had", "do", "does", "did", "will", "would", "could", "should", "may", "might",
                "shall", "can", "need", "dare", "ought", "used", "to", "of", "in", "for", "on",
                "with", "at", "by", "from", "as", "into", "through", "during", "before", "after",
                "above", "below", "between", "out", "off", "over", "under", "again", "further",
                "then", "once", "here", "there", "when", "where", "why", "how", "all", "each",
                "every", "both", "few", "more", "most", "other", "some", "such", "no", "nor",
                "not", "only", "own", "same", "so", "than", "too", "very", "just", "because",
                "but", "and", "or", "if", "while", "although",
            ],
        }
    }

    /// Compress a prompt
    pub fn compress(&self, text: &str) -> CompressionResult {
        let original_tokens = estimate_tokens(text);

        let compressed = match self.level {
            CompressionLevel::None => text.to_string(),
            CompressionLevel::Light => self.compress_light(text),
            CompressionLevel::Medium => self.compress_medium(text),
            CompressionLevel::Aggressive => self.compress_aggressive(text),
        };

        let compressed_tokens = estimate_tokens(&compressed);
        let reduction = if original_tokens > 0 {
            (1.0 - compressed_tokens as f64 / original_tokens as f64) * 100.0
        } else {
            0.0
        };

        CompressionResult {
            compressed,
            original_tokens,
            compressed_tokens,
            reduction_pct: reduction.max(0.0),
            method: format!("{:?}", self.level),
        }
    }

    /// Light compression: normalize whitespace, remove trailing spaces
    fn compress_light(&self, text: &str) -> String {
        let mut result = String::with_capacity(text.len());
        let mut prev_space = false;
        for ch in text.chars() {
            if ch.is_whitespace() {
                if !prev_space {
                    result.push(' ');
                }
                prev_space = true;
            } else {
                result.push(ch);
                prev_space = false;
            }
        }
        result.trim().to_string()
    }

    /// Medium compression: remove stop words + normalize whitespace
    fn compress_medium(&self, text: &str) -> String {
        // Preserve code blocks
        let mut parts = Vec::new();
        let mut in_code = false;
        for line in text.lines() {
            if line.trim_start().starts_with("```") {
                in_code = !in_code;
                parts.push(line.to_string());
            } else if in_code {
                parts.push(line.to_string());
            } else {
                let filtered: Vec<&str> = line
                    .split_whitespace()
                    .filter(|w| {
                        let lower = w.to_lowercase();
                        let clean = lower.trim_matches(|c: char| !c.is_alphanumeric());
                        !self.stop_words.contains(&clean)
                    })
                    .collect();
                if !filtered.is_empty() {
                    parts.push(filtered.join(" "));
                }
            }
        }
        parts.join("\n")
    }

    /// Aggressive compression: TF-IDF scoring, keep only important sentences
    fn compress_aggressive(&self, text: &str) -> String {
        let sentences: Vec<&str> = text
            .split(['.', '\n'])
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        if sentences.len() <= 3 {
            return self.compress_medium(text);
        }

        // Compute TF-IDF scores per sentence
        let tf_idf_scores = self.compute_sentence_scores(&sentences);

        // Keep top 50% of sentences by score
        let mut scored: Vec<(usize, f64)> = tf_idf_scores.into_iter().enumerate().collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        let keep_count = (sentences.len() as f64 * 0.5).ceil() as usize;
        let mut keep_indices: Vec<usize> =
            scored.iter().take(keep_count).map(|(i, _)| *i).collect();
        keep_indices.sort(); // Preserve original order

        let result: Vec<&str> = keep_indices.iter().map(|&i| sentences[i]).collect();
        result.join(". ")
    }

    fn compute_sentence_scores(&self, sentences: &[&str]) -> Vec<f64> {
        // Document frequency
        let mut df: HashMap<String, usize> = HashMap::new();
        let n = sentences.len();

        let tokenized: Vec<Vec<String>> = sentences
            .iter()
            .map(|s| {
                s.split_whitespace()
                    .map(|w| {
                        w.to_lowercase()
                            .trim_matches(|c: char| !c.is_alphanumeric())
                            .to_string()
                    })
                    .filter(|w| !w.is_empty() && !self.stop_words.contains(&w.as_str()))
                    .collect()
            })
            .collect();

        for tokens in &tokenized {
            let unique: std::collections::HashSet<&String> = tokens.iter().collect();
            for word in unique {
                *df.entry(word.clone()).or_insert(0) += 1;
            }
        }

        // TF-IDF score per sentence
        tokenized
            .iter()
            .map(|tokens| {
                if tokens.is_empty() {
                    return 0.0;
                }
                let mut score = 0.0;
                let mut tf: HashMap<&String, usize> = HashMap::new();
                for t in tokens {
                    *tf.entry(t).or_insert(0) += 1;
                }
                for (word, count) in &tf {
                    let tf_val = *count as f64 / tokens.len() as f64;
                    let idf_val = ((n as f64) / (*df.get(*word).unwrap_or(&1) as f64)).ln() + 1.0;
                    score += tf_val * idf_val;
                }
                score
            })
            .collect()
    }
}

/// Simple token estimation (4 chars per token)
// NOTE: canonical version in hudhudscript-runtime::provider::estimate_tokens
fn estimate_tokens(text: &str) -> usize {
    (text.len() / 4).max(1)
}
