//! Document management

pub type DocumentUri = String;

/// Document
#[derive(Debug, Clone)]
pub struct Document {
    uri: DocumentUri,
    text: String,
    version: i32,
}

impl Document {
    pub fn new(uri: DocumentUri, text: String) -> Self {
        Self {
            uri,
            text,
            version: 0,
        }
    }

    pub fn uri(&self) -> &str {
        &self.uri
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn version(&self) -> i32 {
        self.version
    }

    pub fn update(&mut self, text: String) {
        self.text = text;
        self.version += 1;
    }
}
