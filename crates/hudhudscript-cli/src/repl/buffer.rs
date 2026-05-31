//! Line editing support structures.

/// A minimal line buffer for basic editing support.
#[derive(Debug, Clone)]
pub struct LineBuffer {
    /// The current text content.
    pub content: String,
    /// Cursor position (byte offset).
    pub cursor: usize,
}

impl LineBuffer {
    /// Create an empty line buffer.
    pub fn new() -> Self {
        Self {
            content: String::new(),
            cursor: 0,
        }
    }

    /// Insert a character at the cursor position.
    pub fn insert(&mut self, ch: char) {
        self.content.insert(self.cursor, ch);
        self.cursor += ch.len_utf8();
    }

    /// Delete the character before the cursor (backspace).
    pub fn backspace(&mut self) -> bool {
        if self.cursor > 0 {
            let prev = self.content[..self.cursor]
                .chars()
                .last()
                .map(|c| c.len_utf8())
                .unwrap_or(0);
            self.cursor -= prev;
            self.content.remove(self.cursor);
            true
        } else {
            false
        }
    }

    /// Move cursor left by one character.
    pub fn move_left(&mut self) -> bool {
        if self.cursor > 0 {
            let prev = self.content[..self.cursor]
                .chars()
                .last()
                .map(|c| c.len_utf8())
                .unwrap_or(0);
            self.cursor -= prev;
            true
        } else {
            false
        }
    }

    /// Move cursor right by one character.
    pub fn move_right(&mut self) -> bool {
        if self.cursor < self.content.len() {
            let next = self.content[self.cursor..]
                .chars()
                .next()
                .map(|c| c.len_utf8())
                .unwrap_or(0);
            self.cursor += next;
            true
        } else {
            false
        }
    }

    /// Clear the line buffer.
    pub fn clear(&mut self) {
        self.content.clear();
        self.cursor = 0;
    }

    /// Return the current content as a string slice.
    pub fn as_str(&self) -> &str {
        &self.content
    }
}

impl Default for LineBuffer {
    fn default() -> Self {
        Self::new()
    }
}
