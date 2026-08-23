#[cfg(feature = "db")]
use tokio::sync::OnceCell;

#[cfg(feature = "db")]
use super::DatabaseService;
use super::{ColumnInfo, DatabaseBackend, DatabaseConfig, DatabaseError, QueryResult};
#[cfg(feature = "db")]
use super::{DatabaseConnection, ExecuteOptions};

/// A configured agent/tool facade backed by one lazily opened, reusable pool.
pub struct DatabaseTool {
    config: DatabaseConfig,
    #[cfg(feature = "db")]
    connection: OnceCell<DatabaseConnection>,
}

impl DatabaseTool {
    pub fn new(config: DatabaseConfig) -> Self {
        Self {
            config,
            #[cfg(feature = "db")]
            connection: OnceCell::new(),
        }
    }

    pub fn backend(&self) -> &DatabaseBackend {
        &self.config.backend
    }

    pub async fn execute_query(
        &self,
        sql: &str,
        params: &[serde_json::Value],
    ) -> Result<QueryResult, DatabaseError> {
        #[cfg(feature = "db")]
        {
            let connection = self.connection().await?;
            DatabaseService
                .query(&connection.handle, sql, params, ExecuteOptions::default())
                .await
        }
        #[cfg(not(feature = "db"))]
        {
            let _ = (sql, params);
            Err(DatabaseError::FeatureNotEnabled)
        }
    }

    pub async fn execute(
        &self,
        sql: &str,
        params: &[serde_json::Value],
    ) -> Result<QueryResult, DatabaseError> {
        #[cfg(feature = "db")]
        {
            let connection = self.connection().await?;
            DatabaseService
                .execute(&connection.handle, sql, params, ExecuteOptions::default())
                .await
        }
        #[cfg(not(feature = "db"))]
        {
            let _ = (sql, params);
            Err(DatabaseError::FeatureNotEnabled)
        }
    }

    pub async fn list_tables(&self) -> Result<Vec<String>, DatabaseError> {
        #[cfg(feature = "db")]
        {
            let connection = self.connection().await?;
            DatabaseService.list_tables(&connection.handle, None).await
        }
        #[cfg(not(feature = "db"))]
        {
            Err(DatabaseError::FeatureNotEnabled)
        }
    }

    pub async fn describe_table(&self, table: &str) -> Result<Vec<ColumnInfo>, DatabaseError> {
        #[cfg(feature = "db")]
        {
            let connection = self.connection().await?;
            let result = DatabaseService
                .describe_table(&connection.handle, table, None)
                .await?;
            result.rows.into_iter().map(column_from_row).collect()
        }
        #[cfg(not(feature = "db"))]
        {
            let _ = table;
            Err(DatabaseError::FeatureNotEnabled)
        }
    }

    #[cfg(feature = "db")]
    pub async fn migrate(
        &self,
        migrations: Vec<super::Migration>,
    ) -> Result<super::MigrationReport, DatabaseError> {
        let connection = self.connection().await?;
        DatabaseService
            .migrate(&connection.handle, migrations)
            .await
    }

    #[cfg(feature = "db")]
    pub async fn close(&self) -> Result<(), DatabaseError> {
        if let Some(connection) = self.connection.get() {
            DatabaseService.close(&connection.handle).await
        } else {
            Ok(())
        }
    }

    #[cfg(feature = "db")]
    async fn connection(&self) -> Result<&DatabaseConnection, DatabaseError> {
        self.connection
            .get_or_try_init(|| DatabaseService.open(self.config.clone()))
            .await
    }
}

#[cfg(feature = "db")]
fn column_from_row(mut row: super::Row) -> Result<ColumnInfo, DatabaseError> {
    let name = take_string(&mut row, "name")?;
    let data_type = take_string(&mut row, "data_type")?;
    let nullable = row
        .remove("nullable")
        .as_ref()
        .and_then(boolish)
        .unwrap_or(true);
    let primary_key = row
        .remove("primary_key")
        .as_ref()
        .and_then(boolish)
        .unwrap_or(false);
    let default = row
        .remove("default")
        .and_then(|value| value.as_str().map(str::to_owned));
    Ok(ColumnInfo {
        name,
        data_type,
        nullable,
        primary_key,
        default,
    })
}

#[cfg(feature = "db")]
fn boolish(value: &serde_json::Value) -> Option<bool> {
    value
        .as_bool()
        .or_else(|| value.as_i64().map(|number| number != 0))
}

#[cfg(feature = "db")]
fn take_string(row: &mut super::Row, name: &str) -> Result<String, DatabaseError> {
    row.remove(name)
        .and_then(|value| value.as_str().map(str::to_owned))
        .ok_or_else(|| {
            DatabaseError::QueryFailed(format!(
                "metadata result did not contain string column '{name}'"
            ))
        })
}
