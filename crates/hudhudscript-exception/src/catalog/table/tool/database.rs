use crate::catalog::category::ExceptionCategory;
use crate::catalog::codes::ExceptionCode;
use crate::catalog::entry::ExceptionEntry;

pub const DATABASE_CONNECTION_FAILED: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(65),
        long_code: "HHS_E_DATABASE_CONNECTION_FAILED",
        short_code: "E0065",
        title: "Database connection could not be established",
        short_description: "The database driver failed to open a connection to the configured server, often due to network, auth, or DSN issues.",
        long_description: "This error wraps the underlying driver failure when opening a connection. Causes include unreachable host, wrong port, TLS handshake failure, expired credentials, exhausted connection pool, or a database that is not yet accepting connections during startup.

Fix it by validating the connection string with a CLI client (`psql`, `mysql`, `sqlite3`) using the same DSN, then checking firewall and DNS resolution. For pooled connections, confirm the pool size and timeout settings allow new connections under load.

In containerized environments this often appears at startup when the script races the database. Add a retry-with-backoff loop or a readiness probe before issuing the first query.",
        hints: &["Verify the DSN with a native CLI client first", "Check host, port, TLS mode, username, and password individually", "Add retry-with-backoff for startup races against the database", "Inspect server logs for refused or rate-limited connections"],
        example_bad: Some("let db = database::connect(\"postgres://user@unreachable/db\");"),
        example_good: Some("let db = retry(3, || database::connect(env(\"DATABASE_URL\")));"),
        see_also: &["DatabaseQueryFailed", "DatabaseFeatureNotEnabled", "DatabaseUnsupportedBackend"],
        since_version: "0.4.0",
        category: ExceptionCategory::Tool,
    };

pub const DATABASE_FEATURE_NOT_ENABLED: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(66),
        long_code: "HHS_E_DATABASE_FEATURE_NOT_ENABLED",
        short_code: "E0066",
        title: "Database feature flag is disabled at build time",
        short_description: "The runtime was compiled without the `db` feature, so full sqlx-backed database support is unavailable.",
        long_description: "HudHudScript can be built in a slim configuration that omits database drivers. When a script calls `database::*` against such a build, this error is returned to make the missing capability explicit instead of silently failing later.

Fix it by rebuilding the runtime with `--features db` (or the equivalent meta-feature for your distribution), or by switching to a build that already enables it. CI images and minimal Docker variants are the most common offenders.

If you cannot enable the feature, restructure the script to use an external query tool over a process boundary, or move database work into a service that exposes an HTTP/JSON API.",
        hints: &["Rebuild the runtime with `cargo build --features db`", "Check `hudhud --version` for the enabled feature list", "Use a full image instead of the slim/minimal variant", "Wrap database code in `if has_feature(\"db\")` for portability"],
        example_bad: Some("// runtime built without `db` feature
let rows = database::query(\"SELECT 1\");"),
        example_good: Some("// build: cargo build --release --features db
let rows = database::query(\"SELECT 1\");"),
        see_also: &["DatabaseConnectionFailed", "DatabaseUnsupportedBackend", "ToolExecutionFailed"],
        since_version: "0.4.0",
        category: ExceptionCategory::Tool,
    };

pub const DATABASE_INVALID_ARGUMENTS: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(67),
        long_code: "HHS_E_DATABASE_INVALID_ARGUMENTS",
        short_code: "E0067",
        title: "Database call received invalid arguments",
        short_description: "Arguments passed to a database function failed validation before any query was issued.",
        long_description: "This error fires before the query reaches the driver. Typical causes are missing required fields (no DSN, no SQL string), wrong types in the parameter binding list, mismatched placeholder counts, or unsupported option keys in the call options object.

Fix it by reading the function signature for the database call you are using and matching every required argument with the right type. For parameterized queries make sure the number of `?`/`$1` placeholders matches the length of the bind list.

This often shows up when refactoring from string interpolation to bind parameters, or when migrating between Postgres-style and MySQL-style placeholders.",
        hints: &["Match placeholder count to bind list length exactly", "Use the right placeholder syntax for your backend ($1 vs ?)", "Pass scalars/arrays — not objects — as bind parameters", "Re-read the database::* signature for required vs optional args"],
        example_bad: Some("database::query(\"SELECT * FROM t WHERE a=$1 AND b=$2\", [42]);"),
        example_good: Some("database::query(\"SELECT * FROM t WHERE a=$1 AND b=$2\", [42, \"x\"]);"),
        see_also: &["DatabaseQueryFailed", "ToolInvalidArguments", "ToolValidation"],
        since_version: "0.4.0",
        category: ExceptionCategory::Tool,
    };

pub const DATABASE_QUERY_FAILED: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(68),
        long_code: "HHS_E_DATABASE_QUERY_FAILED",
        short_code: "E0068",
        title: "SQL query execution failed",
        short_description: "The database server rejected or aborted the query — typical causes are syntax errors, missing tables, or constraint violations.",
        long_description: "The driver successfully sent the query but the server returned an error. The wrapped message comes straight from the backend and usually identifies the offending column, constraint, or syntax position. Common cases include unknown table/column, NOT NULL violation, foreign-key violation, deadlock, or permission denied.

Fix it by reading the wrapped backend message carefully — it is far more specific than the HudHudScript wrapper. Reproduce the failing query in a SQL client with the exact same parameters to isolate whether the issue is data, schema, or permissions.

For transient errors like deadlocks or serialization failures, retry the whole transaction. For schema drift, run migrations before the script.",
        hints: &["Read the wrapped backend message — it names the column/constraint", "Reproduce the failing query in a native SQL client", "Retry deadlocks and serialization failures from the top of the txn", "Run migrations before scripts that depend on schema changes"],
        example_bad: Some("database::query(\"SELECT * FROM users WHERE eemail = $1\", [e]);"),
        example_good: Some("database::query(\"SELECT * FROM users WHERE email = $1\", [e]);"),
        see_also: &["DatabaseConnectionFailed", "DatabaseInvalidArguments", "DatabaseUnsupportedBackend"],
        since_version: "0.4.0",
        category: ExceptionCategory::Tool,
    };

pub const DATABASE_UNSUPPORTED_BACKEND: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(69),
        long_code: "HHS_E_DATABASE_UNSUPPORTED_BACKEND",
        short_code: "E0069",
        title: "Database backend is not supported",
        short_description: "The DSN scheme refers to a backend that this runtime build cannot drive (for example oracle:// or mssql://).",
        long_description: "HudHudScript currently ships drivers for a known set of backends — typically Postgres, MySQL/MariaDB, and SQLite. A DSN whose scheme is outside that set produces this error so the script fails fast instead of hanging on a half-implemented driver.

Fix it by switching to a supported backend, or by routing through a compatibility proxy (FDW, ODBC bridge, or a small service) that exposes one of the supported wire protocols.

If you control the DSN, double-check the URL scheme — typos like `postgress://` or `mysql2://` also reach this branch.",
        hints: &["Use a supported scheme: postgres, mysql, sqlite", "Check for typos in the DSN scheme prefix", "Front the unsupported backend with a Postgres FDW or proxy", "Move backend-specific work into an HTTP microservice"],
        example_bad: Some("database::connect(\"oracle://...\");"),
        example_good: Some("database::connect(\"postgres://user:pass@host/db\");"),
        see_also: &["DatabaseConnectionFailed", "DatabaseFeatureNotEnabled", "DatabaseInvalidArguments"],
        since_version: "0.4.0",
        category: ExceptionCategory::Tool,
    };
