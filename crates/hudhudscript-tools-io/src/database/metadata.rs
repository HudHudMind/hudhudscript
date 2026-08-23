use base64::Engine;
use serde_json::{json, Value};

use super::{DatabaseBackend, DatabaseError, DatabaseService, QueryResult, Row};

pub(crate) async fn list_tables(
    service: &DatabaseService,
    handle: &str,
    schema: Option<&str>,
) -> Result<Vec<String>, DatabaseError> {
    let backend = service.backend(handle)?;
    let (sql, params) = match backend {
        DatabaseBackend::Postgres => (
            "SELECT tablename AS hudhud_table_name FROM pg_catalog.pg_tables WHERE schemaname = $1 ORDER BY tablename",
            vec![json!(schema.unwrap_or("public"))],
        ),
        DatabaseBackend::Mysql => (
            "SELECT table_name AS hudhud_table_name FROM information_schema.tables WHERE table_schema = COALESCE(?, DATABASE()) AND table_type = 'BASE TABLE' ORDER BY table_name",
            vec![schema.map(Value::from).unwrap_or(Value::Null)],
        ),
        DatabaseBackend::Sqlite => (
            "SELECT name AS hudhud_table_name FROM sqlite_schema WHERE type = 'table' AND name NOT LIKE 'sqlite_%' ORDER BY name",
            Vec::new(),
        ),
    };
    let result = service.query_metadata(handle, sql, &params).await?;
    Ok(result.rows.into_iter().filter_map(table_name).collect())
}

fn table_name(row: Row) -> Option<String> {
    row.into_iter()
        .find(|(name, _)| name.eq_ignore_ascii_case("hudhud_table_name"))
        .and_then(|(_, value)| metadata_text(&value))
}

pub(crate) async fn describe_table(
    service: &DatabaseService,
    handle: &str,
    table: &str,
    schema: Option<&str>,
) -> Result<QueryResult, DatabaseError> {
    if table.is_empty() {
        return Err(DatabaseError::InvalidArguments(
            "table name is empty".into(),
        ));
    }
    let backend = service.backend(handle)?;
    let (sql, params) = match backend {
        DatabaseBackend::Postgres => (
            "SELECT c.column_name AS name, c.data_type, c.is_nullable = 'YES' AS nullable, c.column_default AS \"default\", EXISTS (SELECT 1 FROM information_schema.table_constraints tc JOIN information_schema.key_column_usage kcu ON tc.constraint_name = kcu.constraint_name AND tc.table_schema = kcu.table_schema WHERE tc.constraint_type = 'PRIMARY KEY' AND tc.table_schema = c.table_schema AND tc.table_name = c.table_name AND kcu.column_name = c.column_name) AS primary_key FROM information_schema.columns c WHERE c.table_schema = $1 AND c.table_name = $2 ORDER BY c.ordinal_position",
            vec![json!(schema.unwrap_or("public")), json!(table)],
        ),
        DatabaseBackend::Mysql => (
            "SELECT column_name AS name, column_type AS data_type, is_nullable = 'YES' AS nullable, column_default AS `default`, column_key = 'PRI' AS primary_key FROM information_schema.columns WHERE table_schema = COALESCE(?, DATABASE()) AND table_name = ? ORDER BY ordinal_position",
            vec![schema.map(Value::from).unwrap_or(Value::Null), json!(table)],
        ),
        DatabaseBackend::Sqlite => (
            "SELECT name, type AS data_type, `notnull` = 0 AS nullable, dflt_value AS `default`, pk > 0 AS primary_key FROM pragma_table_info(?) ORDER BY cid",
            vec![json!(table)],
        ),
    };
    let result = service.query_metadata(handle, sql, &params).await?;
    if backend == DatabaseBackend::Mysql {
        normalize_mysql_description(result)
    } else {
        Ok(result)
    }
}

fn normalize_mysql_description(mut result: QueryResult) -> Result<QueryResult, DatabaseError> {
    const TEXT_FIELDS: [&str; 3] = ["name", "data_type", "default"];
    const BOOL_FIELDS: [&str; 2] = ["nullable", "primary_key"];
    for row in &mut result.rows {
        for field in TEXT_FIELDS {
            let value = take_column(row, field).ok_or_else(|| invalid_metadata(field))?;
            let normalized = if value.is_null() {
                Value::Null
            } else {
                Value::String(metadata_text(&value).ok_or_else(|| invalid_metadata(field))?)
            };
            row.insert(field.into(), normalized);
        }
        for field in BOOL_FIELDS {
            let value = take_column(row, field).ok_or_else(|| invalid_metadata(field))?;
            let normalized = metadata_bool(&value).ok_or_else(|| invalid_metadata(field))?;
            row.insert(field.into(), Value::Bool(normalized));
        }
    }
    result.columns = vec![
        "name".into(),
        "data_type".into(),
        "nullable".into(),
        "default".into(),
        "primary_key".into(),
    ];
    Ok(result)
}

fn take_column(row: &mut Row, expected: &str) -> Option<Value> {
    let key = row
        .keys()
        .find(|name| name.eq_ignore_ascii_case(expected))?
        .clone();
    row.remove(&key)
}

fn metadata_text(value: &Value) -> Option<String> {
    if let Some(text) = value.as_str() {
        return Some(text.to_owned());
    }
    let object = value.as_object()?;
    if object.get("$type")?.as_str()? != "bytes" {
        return None;
    }
    let encoded = object.get("base64")?.as_str()?;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .ok()?;
    String::from_utf8(bytes).ok()
}

fn metadata_bool(value: &Value) -> Option<bool> {
    value
        .as_bool()
        .or_else(|| value.as_i64().map(|number| number != 0))
        .or_else(
            || match metadata_text(value)?.to_ascii_lowercase().as_str() {
                "1" | "true" | "yes" => Some(true),
                "0" | "false" | "no" => Some(false),
                _ => None,
            },
        )
}

fn invalid_metadata(field: &str) -> DatabaseError {
    DatabaseError::QueryFailed(format!("invalid MySQL metadata field '{field}'"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_name_accepts_driver_specific_column_case() {
        for key in ["hudhud_table_name", "HUDHUD_TABLE_NAME"] {
            let row = Row::from([(key.into(), json!("users"))]);
            assert_eq!(table_name(row).as_deref(), Some("users"));
        }
    }

    #[test]
    fn metadata_text_accepts_mysql_information_schema_bytes() {
        let value = json!({"$type": "bytes", "base64": "dXNlcnM="});
        assert_eq!(metadata_text(&value).as_deref(), Some("users"));
        assert_eq!(metadata_bool(&json!(1)), Some(true));
        assert_eq!(metadata_bool(&json!(0)), Some(false));
    }
}
