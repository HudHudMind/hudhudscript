//! Iterators and conversions for EvaluationContext.

use serde_json::Value;
use std::collections::HashMap;

use super::EvaluationContext;

impl EvaluationContext {
    /// Get an iterator over the field names
    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.fields.keys()
    }

    /// Get an iterator over the field values
    pub fn values(&self) -> impl Iterator<Item = &Value> {
        self.fields.values()
    }

    /// Get an iterator over the field entries (key-value pairs)
    pub fn iter(&self) -> impl Iterator<Item = (&String, &Value)> {
        self.fields.iter()
    }

    /// Convert the context to a HashMap reference
    pub fn to_map(&self) -> &HashMap<String, Value> {
        &self.fields
    }

    /// Convert the context into a HashMap, consuming self
    pub fn into_map(self) -> HashMap<String, Value> {
        self.fields
    }
}

impl From<HashMap<String, Value>> for EvaluationContext {
    fn from(fields: HashMap<String, Value>) -> Self {
        Self::from_map(fields)
    }
}

impl From<EvaluationContext> for HashMap<String, Value> {
    fn from(context: EvaluationContext) -> Self {
        context.into_map()
    }
}
