use super::{ColumnInfo, DatabaseBackend, DatabaseConfig, DatabaseError, QueryResult, Row};

/// Built-in database tool
///
/// When compiled **without** the `db` feature every method returns
/// `DatabaseError::FeatureNotEnabled`. Enable the feature and provide a real
/// sqlx pool to get full functionality.
pub struct DatabaseTool {
    config: DatabaseConfig,
}

impl DatabaseTool {
    /// Create a new DatabaseTool from the given configuration.
    pub fn new(config: DatabaseConfig) -> Self {
        Self { config }
    }

    /// Return the configured backend
    pub fn backend(&self) -> &DatabaseBackend {
        &self.config.backend
    }

    /// Execute a SQL query and return the result.
    pub async fn execute_query(
        &self,
        sql: &str,
        _params: &[serde_json::Value],
    ) -> Result<QueryResult, DatabaseError> {
        #[cfg(feature = "db")]
        {
            self.execute_query_impl(sql, _params).await
        }

        #[cfg(not(feature = "db"))]
        {
            let _ = sql;
            Err(DatabaseError::FeatureNotEnabled)
        }
    }

    /// List all tables in the connected database.
    pub async fn list_tables(&self) -> Result<Vec<String>, DatabaseError> {
        #[cfg(feature = "db")]
        {
            self.list_tables_impl().await
        }

        #[cfg(not(feature = "db"))]
        {
            Err(DatabaseError::FeatureNotEnabled)
        }
    }

    /// Describe columns for a given table.
    pub async fn describe_table(&self, table_name: &str) -> Result<Vec<ColumnInfo>, DatabaseError> {
        #[cfg(feature = "db")]
        {
            self.describe_table_impl(table_name).await
        }

        #[cfg(not(feature = "db"))]
        {
            let _ = table_name;
            Err(DatabaseError::FeatureNotEnabled)
        }
    }

    #[cfg(feature = "db")]
    async fn execute_query_impl(
        &self,
        sql: &str,
        params: &[serde_json::Value],
    ) -> Result<QueryResult, DatabaseError> {
        use sqlx::Row as SqlxRow;

        match self.config.backend {
            DatabaseBackend::Postgres => {
                let pool = sqlx::postgres::PgPoolOptions::new()
                    .max_connections(self.config.max_connections.unwrap_or(5))
                    .connect(&self.config.connection_string)
                    .await
                    .map_err(|e| DatabaseError::ConnectionFailed(e.to_string()))?;

                let mut query = sqlx::query(sql);
                for param in params {
                    query = query.bind(param.to_string());
                }

                let rows = query
                    .fetch_all(&pool)
                    .await
                    .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

                let columns: Vec<String> = if let Some(first) = rows.first() {
                    first
                        .columns()
                        .iter()
                        .map(|c| c.name().to_string())
                        .collect()
                } else {
                    Vec::new()
                };

                let result_rows: Vec<Row> = rows
                    .iter()
                    .map(|row| {
                        let mut map = Row::new();
                        for col in row.columns() {
                            let val: Option<String> = row.try_get(col.ordinal()).ok();
                            map.insert(
                                col.name().to_string(),
                                val.map(serde_json::Value::String)
                                    .unwrap_or(serde_json::Value::Null),
                            );
                        }
                        map
                    })
                    .collect();

                Ok(QueryResult {
                    rows: result_rows,
                    rows_affected: 0,
                    columns,
                })
            }

            DatabaseBackend::Mysql => Err(DatabaseError::ConnectionFailed(
                "MySQL: enable db feature".into(),
            )),
            DatabaseBackend::Sqlite => {
                let pool = sqlx::sqlite::SqlitePoolOptions::new()
                    .max_connections(self.config.max_connections.unwrap_or(1))
                    .connect(&self.config.connection_string)
                    .await
                    .map_err(|e| DatabaseError::ConnectionFailed(e.to_string()))?;

                let mut query = sqlx::query(sql);
                for param in params {
                    query = query.bind(param.to_string());
                }

                let rows = query
                    .fetch_all(&pool)
                    .await
                    .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

                let columns: Vec<String> = if let Some(first) = rows.first() {
                    first
                        .columns()
                        .iter()
                        .map(|c| c.name().to_string())
                        .collect()
                } else {
                    Vec::new()
                };

                let result_rows: Vec<Row> = rows
                    .iter()
                    .map(|row| {
                        let mut map = Row::new();
                        for col in row.columns() {
                            let val: Option<String> = row.try_get(col.ordinal()).ok();
                            map.insert(
                                col.name().to_string(),
                                val.map(serde_json::Value::String)
                                    .unwrap_or(serde_json::Value::Null),
                            );
                        }
                        map
                    })
                    .collect();

                Ok(QueryResult {
                    rows: result_rows,
                    rows_affected: 0,
                    columns,
                })
            }
        }
    }

    #[cfg(feature = "db")]
    async fn list_tables_impl(&self) -> Result<Vec<String>, DatabaseError> {
        let sql = match self.config.backend {
            DatabaseBackend::Postgres => {
                "SELECT tablename FROM pg_tables WHERE schemaname = 'public' ORDER BY tablename"
            }
            DatabaseBackend::Mysql => Err(DatabaseError::ConnectionFailed(
                "MySQL: enable db feature".into(),
            )),
            DatabaseBackend::Sqlite => {
                "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name"
            }
        };

        let result = self.execute_query_impl(sql, &[]).await?;
        let tables = result
            .rows
            .iter()
            .filter_map(|row| {
                row.values()
                    .next()
                    .and_then(|v| v.as_str().map(|s| s.to_string()))
            })
            .collect();

        Ok(tables)
    }

    #[cfg(feature = "db")]
    async fn describe_table_impl(
        &self,
        table_name: &str,
    ) -> Result<Vec<ColumnInfo>, DatabaseError> {
        let (sql, _) = match self.config.backend {
            DatabaseBackend::Postgres => (
                format!(
                    "SELECT column_name, data_type, is_nullable \
                     FROM information_schema.columns \
                     WHERE table_name = '{}' ORDER BY ordinal_position",
                    table_name
                ),
                (),
            ),
            DatabaseBackend::Sqlite => (format!("PRAGMA table_info({})", table_name), ()),
        };

        let result = self.execute_query_impl(&sql, &[]).await?;

        let columns = result
            .rows
            .iter()
            .map(|row| {
                let name = row
                    .get("column_name")
                    .or_else(|| row.get("name"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string();

                let data_type = row
                    .get("data_type")
                    .or_else(|| row.get("type"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string();

                let nullable = row
                    .get("is_nullable")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_uppercase() == "YES")
                    .unwrap_or(true);

                ColumnInfo {
                    name,
                    data_type,
                    nullable,
                }
            })
            .collect();

        Ok(columns)
    }
}
