//! Source code translator
//!
//! Translates natural language source code to canonical form.

use crate::{KeywordMap, Language, LanguageDetector};

/// Source code translator
pub struct Translator {
    keyword_map: KeywordMap,
}

impl Translator {
    /// Create a new translator
    pub fn new() -> Self {
        Self {
            keyword_map: KeywordMap::new(),
        }
    }

    /// Translate source code to canonical English form
    pub fn translate_to_canonical(&self, source: &str) -> String {
        let language = LanguageDetector::detect(source);

        if language == Language::English {
            // Already in canonical form
            return source.to_string();
        }

        // Token-aware translation:
        //   - Skips string literals ("...", '...') so embedded keywords aren't rewritten
        //   - Skips line and block comments
        //   - Treats Unicode identifier chars as word characters (not just ASCII)
        //   - Maps each foreign keyword to its canonical English form via the keyword map
        let mut result = String::new();
        let mut current_word = String::new();
        let chars: Vec<char> = source.chars().collect();
        let mut i = 0;

        let flush = |word: &mut String, out: &mut String, kmap: &KeywordMap, lang| {
            if !word.is_empty() {
                if let Some(keyword) = kmap.lookup(word, lang) {
                    out.push_str(kmap.to_english(keyword));
                } else {
                    out.push_str(word);
                }
                word.clear();
            }
        };

        while i < chars.len() {
            let ch = chars[i];

            // String literal: copy verbatim until closing quote
            if ch == '"' || ch == '\'' {
                flush(&mut current_word, &mut result, &self.keyword_map, language);
                let quote = ch;
                result.push(ch);
                i += 1;
                while i < chars.len() {
                    let c = chars[i];
                    result.push(c);
                    if c == '\\' && i + 1 < chars.len() {
                        // Copy escaped char as-is
                        result.push(chars[i + 1]);
                        i += 2;
                        continue;
                    }
                    if c == quote {
                        i += 1;
                        break;
                    }
                    i += 1;
                }
                continue;
            }

            // Line comment
            if ch == '/' && i + 1 < chars.len() && chars[i + 1] == '/' {
                flush(&mut current_word, &mut result, &self.keyword_map, language);
                while i < chars.len() && chars[i] != '\n' {
                    result.push(chars[i]);
                    i += 1;
                }
                continue;
            }

            // Block comment
            if ch == '/' && i + 1 < chars.len() && chars[i + 1] == '*' {
                flush(&mut current_word, &mut result, &self.keyword_map, language);
                result.push_str("/*");
                i += 2;
                while i + 1 < chars.len() && !(chars[i] == '*' && chars[i + 1] == '/') {
                    result.push(chars[i]);
                    i += 1;
                }
                if i + 1 < chars.len() {
                    result.push_str("*/");
                    i += 2;
                }
                continue;
            }

            // Word character (Unicode identifier — letters, digits, underscore, all scripts)
            if ch.is_alphanumeric() || ch == '_' {
                current_word.push(ch);
            } else {
                flush(&mut current_word, &mut result, &self.keyword_map, language);
                result.push(ch);
            }
            i += 1;
        }

        flush(&mut current_word, &mut result, &self.keyword_map, language);
        result
    }

    /// Get detected language
    pub fn detect_language(&self, source: &str) -> Language {
        LanguageDetector::detect(source)
    }
}

impl Default for Translator {
    fn default() -> Self {
        Self::new()
    }
}
