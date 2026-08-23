use hudhudscript_bytecode::error::{compile_codes, CompileError, CompileResult};
use hudhudscript_bytecode::Value16;
use hudhudscript_tools::database::{
    runtime, DatabaseConfig, DatabaseConnection, DatabaseService, DatabaseTransaction,
    ExecuteOptions, Migration, TransactionOptions,
};

use crate::vm::VM;

impl VM {
    pub(crate) fn call_database_method(
        &self,
        receiver: &Value16,
        method: &str,
        args: Vec<Value16>,
    ) -> CompileResult<Value16> {
        self.host_access_policy.ensure_module_allowed("database")?;
        let module = receiver
            .as_object()
            .and_then(|object| object.get("__module"))
            .and_then(Value16::as_string)
            .ok_or_else(|| error("invalid database receiver"))?;
        match module.as_str() {
            "database" => call_module(method, &args),
            "database.connection" => call_connection(receiver, method, &args),
            "database.transaction" => call_transaction(receiver, method, &args),
            _ => Err(error("invalid database object")),
        }
    }
}

fn call_module(method: &str, args: &[Value16]) -> CompileResult<Value16> {
    match method {
        "open" | "connect" => {
            let config_value = args
                .first()
                .ok_or_else(|| error("database.open(config) requires a config object"))?;
            let config: DatabaseConfig = serde_json::from_value(to_json(config_value))
                .map_err(|cause| error(format!("invalid database config: {cause}")))?;
            let connection = run(async move { DatabaseService.open(config).await })?;
            Ok(connection_value(&connection))
        }
        _ => Err(error(format!("unknown database method '{method}'"))),
    }
}

fn call_connection(receiver: &Value16, method: &str, args: &[Value16]) -> CompileResult<Value16> {
    let handle = marker(receiver, "__database_handle")?;
    match method {
        "query" => {
            let (sql, params, options) = query_arguments(args)?;
            json_result(run(async move {
                DatabaseService.query(&handle, &sql, &params, options).await
            })?)
        }
        "execute" => {
            let (sql, params, options) = query_arguments(args)?;
            json_result(run(async move {
                DatabaseService
                    .execute(&handle, &sql, &params, options)
                    .await
            })?)
        }
        "health" => json_result(run(async move { DatabaseService.health(&handle).await })?),
        "status" => json_result(DatabaseService.status(&handle).map_err(db_error)?),
        "listTables" | "list_tables" => {
            let schema = optional_string(args.first())?;
            json_result(run(async move {
                DatabaseService
                    .list_tables(&handle, schema.as_deref())
                    .await
            })?)
        }
        "describe" | "describeTable" | "describe_table" => {
            let table = required_string(args.first(), "describe(table) requires a table name")?;
            let schema = optional_string(args.get(1))?;
            json_result(run(async move {
                DatabaseService
                    .describe_table(&handle, &table, schema.as_deref())
                    .await
            })?)
        }
        "begin" => {
            let options: TransactionOptions = optional_object(args.first())?;
            let transaction = run(async move { DatabaseService.begin(&handle, options).await })?;
            Ok(transaction_value(&transaction))
        }
        "migrate" => {
            let value = args
                .first()
                .ok_or_else(|| error("migrate(migrations) requires an array"))?;
            let migrations: Vec<Migration> = serde_json::from_value(to_json(value))
                .map_err(|cause| error(format!("invalid migrations: {cause}")))?;
            json_result(run(async move {
                DatabaseService.migrate(&handle, migrations).await
            })?)
        }
        "close" => {
            run(async move { DatabaseService.close(&handle).await })?;
            Ok(Value16::null())
        }
        _ => Err(error(format!(
            "unknown database connection method '{method}'"
        ))),
    }
}

fn call_transaction(receiver: &Value16, method: &str, args: &[Value16]) -> CompileResult<Value16> {
    let transaction = marker(receiver, "__database_transaction")?;
    match method {
        "query" => {
            let (sql, params, options) = query_arguments(args)?;
            json_result(run(async move {
                DatabaseService
                    .transaction_query(&transaction, &sql, &params, options)
                    .await
            })?)
        }
        "execute" => {
            let (sql, params, options) = query_arguments(args)?;
            json_result(run(async move {
                DatabaseService
                    .transaction_execute(&transaction, &sql, &params, options)
                    .await
            })?)
        }
        "commit" => {
            run(async move { DatabaseService.commit(&transaction).await })?;
            Ok(Value16::null())
        }
        "rollback" => {
            run(async move { DatabaseService.rollback(&transaction).await })?;
            Ok(Value16::null())
        }
        _ => Err(error(format!(
            "unknown database transaction method '{method}'"
        ))),
    }
}

fn query_arguments(
    args: &[Value16],
) -> CompileResult<(String, Vec<serde_json::Value>, ExecuteOptions)> {
    let sql = required_string(args.first(), "query(sql, params?, options?) requires SQL")?;
    let params = match args.get(1) {
        None => Vec::new(),
        Some(value) if value.is_null() => Vec::new(),
        Some(value) => to_json(value)
            .as_array()
            .cloned()
            .ok_or_else(|| error("database query params must be an array"))?,
    };
    let options = optional_object(args.get(2))?;
    Ok((sql, params, options))
}

fn optional_object<T>(value: Option<&Value16>) -> CompileResult<T>
where
    T: serde::de::DeserializeOwned + Default,
{
    match value {
        None => Ok(T::default()),
        Some(value) if value.is_null() => Ok(T::default()),
        Some(value) => serde_json::from_value(to_json(value))
            .map_err(|cause| error(format!("invalid database options: {cause}"))),
    }
}

fn required_string(value: Option<&Value16>, message: &str) -> CompileResult<String> {
    value
        .and_then(Value16::as_string)
        .ok_or_else(|| error(message))
}

fn optional_string(value: Option<&Value16>) -> CompileResult<Option<String>> {
    match value {
        None => Ok(None),
        Some(value) if value.is_null() => Ok(None),
        Some(value) => value
            .as_string()
            .map(Some)
            .ok_or_else(|| error("expected a string or null")),
    }
}

fn marker(receiver: &Value16, name: &str) -> CompileResult<String> {
    receiver
        .as_object()
        .and_then(|object| object.get(name))
        .and_then(Value16::as_string)
        .ok_or_else(|| error("invalid or closed database handle"))
}

fn connection_value(connection: &DatabaseConnection) -> Value16 {
    handle_value(
        "database.connection",
        "__database_handle",
        &connection.handle,
        &connection.backend.to_string(),
    )
}

fn transaction_value(transaction: &DatabaseTransaction) -> Value16 {
    handle_value(
        "database.transaction",
        "__database_transaction",
        &transaction.transaction,
        &transaction.backend.to_string(),
    )
}

fn handle_value(module: &str, marker: &str, id: &str, backend: &str) -> Value16 {
    let mut object = hudhudscript_bytecode::ObjMap::default();
    object.insert("__module".to_string(), Value16::string(module.to_string()));
    object.insert(marker.to_string(), Value16::string(id.to_string()));
    object.insert("backend".to_string(), Value16::string(backend.to_string()));
    Value16::object(object)
}

fn to_json(value: &Value16) -> serde_json::Value {
    crate::vm::governance_ops::value_to_serde_json(value)
}

fn json_result(value: impl serde::Serialize) -> CompileResult<Value16> {
    serde_json::to_value(value)
        .map(|value| crate::vm::json::serde_to_value(&value))
        .map_err(|cause| error(format!("database result serialization failed: {cause}")))
}

fn run<F, T>(future: F) -> CompileResult<T>
where
    F: std::future::Future<Output = Result<T, hudhudscript_tools::database::DatabaseError>> + Send,
    T: Send,
{
    runtime::block_on(future).map_err(db_error)
}

fn db_error(cause: hudhudscript_tools::database::DatabaseError) -> CompileError {
    error(cause.to_string())
}

fn error(message: impl Into<String>) -> CompileError {
    compile_codes::runtime_error(message.into())
}
