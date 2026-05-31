use crate::Value16;
use parking_lot::RwLock;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::sync::Arc;

pub fn serialize<S>(
    captures: &HashMap<String, Arc<RwLock<Value16>>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let flat: HashMap<String, Value16> = captures
        .iter()
        .map(|(k, cell)| (k.clone(), *cell.read()))
        .collect();
    flat.serialize(serializer)
}

pub fn deserialize<'de, D>(
    deserializer: D,
) -> Result<HashMap<String, Arc<RwLock<Value16>>>, D::Error>
where
    D: Deserializer<'de>,
{
    let flat: HashMap<String, Value16> = HashMap::deserialize(deserializer)?;
    Ok(flat
        .into_iter()
        .map(|(k, v)| (k, Arc::new(RwLock::new(v))))
        .collect())
}
