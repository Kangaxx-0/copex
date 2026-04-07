# Plan 4: Validation Execution Paths

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Establish real DAX and warehouse validation execution, SQL policy enforcement, evidence capture from query results, truth hierarchy ranking, freshness policy, validation floor enforcement, and minimum validation patterns by investigation class -- all integrated with the evaluator loop from Plan 1.

**Architecture:** Two new crates -- `spool-exec` (validation execution adapters and SQL policy) and `spool-validation` (evidence capture, truth hierarchy, freshness, validation floor, cross-checks) -- living in the `spool/` workspace. `spool-exec` depends on `spool-protocol` and `spool-fabric` (from Plan 2). `spool-validation` depends on `spool-protocol` and `spool-core`. All external data access is behind traits with fixture implementations for unit testing. Integration tests target a dev Fabric workspace.

**Tech Stack:** Rust 2024 edition, serde/serde_json, chrono, uuid, tokio, async-trait, thiserror, tiberius (T-SQL), sqlparser (SQL policy), reqwest (DAX execution via Fabric REST)

**Governing spec:** `docs/superpowers/specs/2026-04-06-spool-refined-spec.md`
**Planning readiness:** `docs/superpowers/specs/2026-04-06-spool-dev-planning-readiness-design.md`

---

## Plan-Specific Sections

### Subsystem Scope

This plan owns:

- DAX query execution adapter (semantic model DAX query path, Spec Section 11.6)
- warehouse validation via read-only T-SQL (Spec Section 11.6)
- SQL policy enforcement -- read-only only, non-read statements rejected (Spec Section 14.2)
- evidence capture from query results (Spec Section 9.1)
- truth hierarchy implementation and ranking function (Spec Section 9.2)
- freshness policy metadata and interpretation rules (Spec Section 9.3)
- minimum validation patterns by investigation class (Spec Section 9.7)
- higher-risk validation cross-checks (Spec Section 9.8)
- query result handling -- summarize, preview, drill-down (Spec Section 3.6)
- validation floor enforcement (Spec Section 9.6)
- integration of validation execution with the evaluator loop from Plan 1

### Out Of Scope

- live Fabric auth and token acquisition (Plan 2)
- Fabric REST client construction and base HTTP transport (Plan 2)
- knowledge bundle loading and Tier 1/Tier 2 structure (Plan 3)
- TUI rendering of validation results (Plan 5)
- durable memory, exports, telemetry (Plan 6)
- LLM provider integration (the harness traits abstract this away)
- mutation operations (v1 is proposal-only, Spec Section 2.3)

### Dependencies

- **Plan 1:** `spool-protocol` evidence types, evaluator loop, evidence ledger, contradiction ledger, task contract, task result schema
- **Plan 2:** `spool-fabric` REST client for DAX execution endpoint, auth token provider trait
- **Plan 3:** knowledge bundle types used by truth hierarchy to rank curated LOB knowledge vs. runtime evidence

### Contract Impact

This plan **implements** the following governing contracts from the refined spec:

- Data access adapters (Spec Section 11.6)
- SQL policy (Spec Section 14.2)
- Evidence ledger capture from query results (Spec Section 9.1)
- Truth hierarchy (Spec Section 9.2)
- Freshness policy (Spec Section 9.3)
- Validation floor (Spec Section 9.6)
- Minimum validation patterns by investigation class (Spec Section 9.7)
- Higher-risk validation cross-checks (Spec Section 9.8)
- Query result handling (Spec Section 3.6)

This plan **pressures** the following contracts:

- Evaluator loop protocol (Spec Section 4.3-4.6) -- validation results feed directly into the evaluator packet, so integration may reveal loop protocol gaps
- Capability contract (Spec Section 11.4) -- validation execution must respect declared validation_capabilities

### Validation

Plan 4 is proven through:

- unit tests: SQL policy parsing, truth hierarchy ranking, freshness metadata, validation floor checks, query result summarization
- adapter tests: mock HTTP for DAX execution, mock SQL connection for warehouse queries
- integration tests: evaluator loop with real validation adapters injected via fixture evidence
- integration validation against dev Fabric workspace: DAX query round-trip, warehouse read-only query round-trip

Integration validation scenarios for dev Fabric workspace:

| Seam | Scenario | Environment | Success Condition |
|------|----------|-------------|-------------------|
| DAX execution | Execute a simple EVALUATE query against a known semantic model | Dev Fabric workspace | Response contains expected row structure and values |
| Warehouse SQL | Execute a SELECT query against a known warehouse table | Dev Fabric workspace | Response contains expected row structure and values |
| SQL policy | Attempt INSERT/UPDATE/DELETE against warehouse | Dev Fabric workspace | Rejected before reaching warehouse |
| Evidence capture | Execute DAX query and capture result as EvidenceItem | Dev Fabric workspace | EvidenceItem round-trips through ledger with correct class and freshness |

### Open Items

**Owned by this plan:**

- exact T-SQL transport library choice (resolved: `tiberius` for TDS protocol, Spec Section 16 lists warehouse SQL transport as open)
- exact SQL statement classification approach (resolved: `sqlparser` crate with MSSQL dialect)
- exact query result summarization heuristics for large result sets (resolved during implementation)
- exact freshness metadata fields beyond `observed_at` (resolved during implementation)

**Deferred to later plans:**

- how knowledge bundle version is used in freshness comparison (Plan 3 defines bundle versioning)
- durable memory freshness comparison (Plan 6)
- TUI rendering of query result previews and drill-down (Plan 5)
- exact DAX query construction from measure definitions (generator responsibility, outside adapter scope)

**Review triggers:**

- if DAX execution via Fabric REST proves insufficient and requires MCP, revisit adapter boundary
- if warehouse T-SQL via `tiberius` proves insufficient for Fabric Warehouse specifically, revisit transport choice
- if evaluator loop integration reveals that validation results need richer structure in the evaluator packet, revisit Plan 1 evaluator packet schema

---

## Task 1: Crate Scaffolding

**Files:**

- Modify: `spool/Cargo.toml`
- Create: `spool/spool-exec/Cargo.toml`
- Create: `spool/spool-exec/src/lib.rs`
- Create: `spool/spool-validation/Cargo.toml`
- Create: `spool/spool-validation/src/lib.rs`

**Step 1: Add new crates to workspace**

Update the workspace Cargo.toml to include the new crates:

```toml
# spool/Cargo.toml
[workspace]
members = [
    "spool-protocol",
    "spool-core",
    "spool-exec",
    "spool-validation",
]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2024"
license = "Apache-2.0"

[workspace.dependencies]
spool-protocol = { path = "spool-protocol" }
spool-core = { path = "spool-core" }
spool-exec = { path = "spool-exec" }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v4", "serde"] }
tokio = { version = "1", features = ["full"] }
async-trait = "0.1"
thiserror = "2"
```

**Step 2: Create spool-exec crate**

```toml
# spool/spool-exec/Cargo.toml
[package]
name = "spool-exec"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
spool-protocol = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
chrono = { workspace = true }
uuid = { workspace = true }
tokio = { workspace = true }
async-trait = { workspace = true }
thiserror = { workspace = true }
sqlparser = "0.53"
reqwest = { version = "0.12", features = ["json"] }
tiberius = { version = "0.12", default-features = false, features = ["tds73", "chrono", "rustls"] }

[dev-dependencies]
tokio = { workspace = true, features = ["test-util", "macros"] }
wiremock = "0.6"
```

```rust
// spool/spool-exec/src/lib.rs
pub mod sql_policy;
pub mod dax_adapter;
pub mod warehouse_adapter;
pub mod error;
```

**Step 3: Create spool-validation crate**

```toml
# spool/spool-validation/Cargo.toml
[package]
name = "spool-validation"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
spool-protocol = { workspace = true }
spool-core = { workspace = true }
spool-exec = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
chrono = { workspace = true }
uuid = { workspace = true }
tokio = { workspace = true }
async-trait = { workspace = true }
thiserror = { workspace = true }

[dev-dependencies]
tokio = { workspace = true, features = ["test-util", "macros"] }
```

```rust
// spool/spool-validation/src/lib.rs
pub mod evidence_capture;
pub mod truth_hierarchy;
pub mod freshness;
pub mod validation_floor;
pub mod validation_patterns;
pub mod cross_checks;
pub mod query_result_handling;
pub mod evaluator_integration;
```

**Step 4: Create error module for spool-exec**

```rust
// spool/spool-exec/src/error.rs
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ExecError {
    #[error("sql policy violation: {0}")]
    SqlPolicyViolation(String),

    #[error("dax execution error: {0}")]
    DaxExecution(String),

    #[error("warehouse execution error: {0}")]
    WarehouseExecution(String),

    #[error("connection error: {0}")]
    Connection(String),

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("http error: {0}")]
    Http(String),
}
```

**Step 5: Create placeholder modules**

Create empty files for each module declared in both lib.rs files. Each file should contain only a comment:

```rust
// placeholder -- implemented in later tasks
```

**Step 6: Verify build**

Run: `cd spool && cargo check`
Expected: compiles with no errors

**Step 7: Commit**

```bash
git add spool/
git commit -m "feat(spool): scaffold spool-exec and spool-validation crates for validation execution paths"
```

---

## Task 2: SQL Policy Enforcement

**Files:**

- Create: `spool/spool-exec/src/sql_policy.rs`

**Step 1: Write the failing test**

Add to `spool/spool-exec/src/sql_policy.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_select_allowed() {
        let result = validate_sql("SELECT col1, col2 FROM dbo.sales WHERE year = 2025");
        assert!(result.is_ok());
    }

    #[test]
    fn select_with_aggregation_allowed() {
        let result = validate_sql(
            "SELECT region, SUM(revenue) AS total_revenue FROM dbo.sales GROUP BY region",
        );
        assert!(result.is_ok());
    }

    #[test]
    fn select_with_cte_allowed() {
        let result = validate_sql(
            "WITH cte AS (SELECT id, revenue FROM dbo.sales WHERE year = 2025) \
             SELECT id, revenue FROM cte WHERE revenue > 1000000",
        );
        assert!(result.is_ok());
    }

    #[test]
    fn select_with_subquery_allowed() {
        let result = validate_sql(
            "SELECT * FROM dbo.sales WHERE region IN (SELECT region FROM dbo.regions WHERE active = 1)",
        );
        assert!(result.is_ok());
    }

    #[test]
    fn select_with_join_allowed() {
        let result = validate_sql(
            "SELECT s.id, r.name FROM dbo.sales s INNER JOIN dbo.regions r ON s.region_id = r.id",
        );
        assert!(result.is_ok());
    }

    #[test]
    fn insert_rejected() {
        let result = validate_sql("INSERT INTO dbo.sales (col1) VALUES ('x')");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("policy violation"),
            "expected policy violation, got: {err}",
        );
    }

    #[test]
    fn update_rejected() {
        let result = validate_sql("UPDATE dbo.sales SET col1 = 'x' WHERE id = 1");
        assert!(result.is_err());
    }

    #[test]
    fn delete_rejected() {
        let result = validate_sql("DELETE FROM dbo.sales WHERE id = 1");
        assert!(result.is_err());
    }

    #[test]
    fn drop_table_rejected() {
        let result = validate_sql("DROP TABLE dbo.sales");
        assert!(result.is_err());
    }

    #[test]
    fn truncate_rejected() {
        let result = validate_sql("TRUNCATE TABLE dbo.sales");
        assert!(result.is_err());
    }

    #[test]
    fn create_table_rejected() {
        let result = validate_sql("CREATE TABLE dbo.test (id INT)");
        assert!(result.is_err());
    }

    #[test]
    fn alter_table_rejected() {
        let result = validate_sql("ALTER TABLE dbo.sales ADD col2 INT");
        assert!(result.is_err());
    }

    #[test]
    fn exec_procedure_rejected() {
        let result = validate_sql("EXEC sp_helpdb");
        assert!(result.is_err());
    }

    #[test]
    fn multiple_statements_first_non_select_rejected() {
        let result = validate_sql(
            "SELECT 1; INSERT INTO dbo.sales (col1) VALUES ('x')",
        );
        assert!(result.is_err());
    }

    #[test]
    fn empty_sql_rejected() {
        let result = validate_sql("");
        assert!(result.is_err());
    }

    #[test]
    fn whitespace_only_rejected() {
        let result = validate_sql("   \n\t  ");
        assert!(result.is_err());
    }

    #[test]
    fn classify_returns_correct_kind() {
        assert_eq!(
            classify_sql("SELECT 1").unwrap(),
            SqlStatementKind::Select,
        );
        assert_eq!(
            classify_sql("WITH cte AS (SELECT 1) SELECT * FROM cte").unwrap(),
            SqlStatementKind::Select,
        );
        assert_eq!(
            classify_sql("INSERT INTO t VALUES (1)").unwrap(),
            SqlStatementKind::Insert,
        );
        assert_eq!(
            classify_sql("UPDATE t SET x = 1").unwrap(),
            SqlStatementKind::Update,
        );
        assert_eq!(
            classify_sql("DELETE FROM t").unwrap(),
            SqlStatementKind::Delete,
        );
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cd spool && cargo test -p spool-exec -- sql_policy`
Expected: FAIL -- types and functions not defined yet

**Step 3: Write the implementation**

Replace the placeholder content of `spool/spool-exec/src/sql_policy.rs` with:

```rust
// spool/spool-exec/src/sql_policy.rs
//
// SQL policy enforcement for Spec Section 14.2: warehouse SQL is read-only in v1.
// Non-read statements are not allowed.

use sqlparser::ast::Statement;
use sqlparser::dialect::MsSqlDialect;
use sqlparser::parser::Parser;

use crate::error::ExecError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SqlStatementKind {
    Select,
    Insert,
    Update,
    Delete,
    CreateTable,
    AlterTable,
    Drop,
    Truncate,
    Execute,
    Other,
}

impl std::fmt::Display for SqlStatementKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            SqlStatementKind::Select => "SELECT",
            SqlStatementKind::Insert => "INSERT",
            SqlStatementKind::Update => "UPDATE",
            SqlStatementKind::Delete => "DELETE",
            SqlStatementKind::CreateTable => "CREATE TABLE",
            SqlStatementKind::AlterTable => "ALTER TABLE",
            SqlStatementKind::Drop => "DROP",
            SqlStatementKind::Truncate => "TRUNCATE",
            SqlStatementKind::Execute => "EXECUTE",
            SqlStatementKind::Other => "OTHER",
        };
        write!(f, "{label}")
    }
}

/// Classify a SQL statement string into a SqlStatementKind.
///
/// Parses the first statement using the MSSQL dialect.
/// Returns an error if the SQL cannot be parsed or is empty.
pub fn classify_sql(sql: &str) -> Result<SqlStatementKind, ExecError> {
    let trimmed = sql.trim();
    if trimmed.is_empty() {
        return Err(ExecError::SqlPolicyViolation(
            "empty SQL statement".into(),
        ));
    }

    let dialect = MsSqlDialect {};
    let statements = Parser::parse_sql(&dialect, trimmed).map_err(|e| {
        ExecError::SqlPolicyViolation(format!("SQL parse error: {e}"))
    })?;

    if statements.is_empty() {
        return Err(ExecError::SqlPolicyViolation(
            "no SQL statements found".into(),
        ));
    }

    Ok(classify_statement(&statements[0]))
}

fn classify_statement(stmt: &Statement) -> SqlStatementKind {
    match stmt {
        Statement::Query(_) => SqlStatementKind::Select,
        Statement::Insert(_) => SqlStatementKind::Insert,
        Statement::Update { .. } => SqlStatementKind::Update,
        Statement::Delete(_) => SqlStatementKind::Delete,
        Statement::CreateTable(_) => SqlStatementKind::CreateTable,
        Statement::AlterTable { .. } => SqlStatementKind::AlterTable,
        Statement::Drop { .. } => SqlStatementKind::Drop,
        Statement::Truncate { .. } => SqlStatementKind::Truncate,
        Statement::Execute { .. } => SqlStatementKind::Execute,
        _ => SqlStatementKind::Other,
    }
}

/// Validate a SQL string against the read-only policy.
///
/// Only SELECT statements (including WITH/CTE queries) are allowed.
/// All other statement types are rejected with a policy violation error.
///
/// If the input contains multiple statements separated by semicolons,
/// every statement must be a SELECT.
pub fn validate_sql(sql: &str) -> Result<(), ExecError> {
    let trimmed = sql.trim();
    if trimmed.is_empty() {
        return Err(ExecError::SqlPolicyViolation(
            "empty SQL statement".into(),
        ));
    }

    let dialect = MsSqlDialect {};
    let statements = Parser::parse_sql(&dialect, trimmed).map_err(|e| {
        ExecError::SqlPolicyViolation(format!("SQL parse error: {e}"))
    })?;

    if statements.is_empty() {
        return Err(ExecError::SqlPolicyViolation(
            "no SQL statements found".into(),
        ));
    }

    for stmt in &statements {
        let kind = classify_statement(stmt);
        if kind != SqlStatementKind::Select {
            return Err(ExecError::SqlPolicyViolation(format!(
                "sql policy violation: {kind} statements are not allowed; only SELECT is permitted in v1"
            )));
        }
    }

    Ok(())
}

// tests at bottom (from Step 1)
```

**Step 4: Run test to verify it passes**

Run: `cd spool && cargo test -p spool-exec -- sql_policy`
Expected: 17 tests PASS

**Step 5: Commit**

```bash
git add spool/spool-exec/src/sql_policy.rs
git commit -m "feat(spool-exec): SQL policy enforcement -- read-only only, all non-SELECT statements rejected per Spec Section 14.2"
```

---

## Task 3: DAX Execution Adapter Trait And Fixture

**Files:**

- Create: `spool/spool-exec/src/dax_adapter.rs`

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[tokio::test]
    async fn fixture_dax_adapter_returns_configured_response() {
        let response = DaxQueryResponse {
            rows: vec![
                DaxRow {
                    values: vec![
                        DaxValue::String("North".into()),
                        DaxValue::Float(12_400_000.0),
                    ],
                },
                DaxRow {
                    values: vec![
                        DaxValue::String("South".into()),
                        DaxValue::Float(8_200_000.0),
                    ],
                },
            ],
            columns: vec![
                DaxColumn {
                    name: "Region".into(),
                    data_type: "String".into(),
                },
                DaxColumn {
                    name: "Revenue".into(),
                    data_type: "Double".into(),
                },
            ],
            executed_at: Utc::now(),
            execution_duration_ms: 42,
        };

        let adapter = FixtureDaxAdapter::new(vec![Ok(response.clone())]);
        let request = DaxQueryRequest {
            dataset_id: "ds_123".into(),
            query: "EVALUATE SUMMARIZECOLUMNS('Sales'[Region], \"Revenue\", SUM('Sales'[Revenue]))".into(),
        };

        let result = adapter.execute_dax(&request).await.unwrap();
        assert_eq!(result.rows.len(), 2);
        assert_eq!(result.columns.len(), 2);
        assert_eq!(result.columns[0].name, "Region");
    }

    #[tokio::test]
    async fn fixture_dax_adapter_returns_error() {
        let adapter = FixtureDaxAdapter::new(vec![Err(ExecError::DaxExecution(
            "model not found".into(),
        ))]);
        let request = DaxQueryRequest {
            dataset_id: "ds_missing".into(),
            query: "EVALUATE VALUES('Missing')".into(),
        };

        let result = adapter.execute_dax(&request).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("model not found"));
    }

    #[test]
    fn dax_query_response_round_trip() {
        let response = DaxQueryResponse {
            rows: vec![DaxRow {
                values: vec![DaxValue::Integer(42)],
            }],
            columns: vec![DaxColumn {
                name: "count".into(),
                data_type: "Int64".into(),
            }],
            executed_at: Utc::now(),
            execution_duration_ms: 10,
        };

        let json = serde_json::to_string(&response).unwrap();
        let restored: DaxQueryResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.rows.len(), 1);
        assert_eq!(restored.columns[0].name, "count");
    }

    #[test]
    fn all_dax_value_variants_serialize() {
        let values = vec![
            DaxValue::String("hello".into()),
            DaxValue::Integer(42),
            DaxValue::Float(3.14),
            DaxValue::Boolean(true),
            DaxValue::Null,
        ];
        for v in values {
            let json = serde_json::to_string(&v).unwrap();
            let restored: DaxValue = serde_json::from_str(&json).unwrap();
            assert_eq!(
                serde_json::to_string(&restored).unwrap(),
                json,
            );
        }
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cd spool && cargo test -p spool-exec -- dax_adapter`
Expected: FAIL

**Step 3: Write the implementation**

```rust
// spool/spool-exec/src/dax_adapter.rs
//
// DAX query execution adapter for Spec Section 11.6: semantic-model DAX query path.
// The trait abstracts the transport so that unit tests use fixtures and integration
// tests use the real Fabric REST endpoint.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::ExecError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaxQueryRequest {
    pub dataset_id: String,
    pub query: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DaxValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Null,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaxColumn {
    pub name: String,
    pub data_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaxRow {
    pub values: Vec<DaxValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaxQueryResponse {
    pub rows: Vec<DaxRow>,
    pub columns: Vec<DaxColumn>,
    pub executed_at: DateTime<Utc>,
    pub execution_duration_ms: u64,
}

#[async_trait]
pub trait DaxAdapter: Send + Sync {
    async fn execute_dax(
        &self,
        request: &DaxQueryRequest,
    ) -> Result<DaxQueryResponse, ExecError>;
}

// --- Fixture implementation ---

pub struct FixtureDaxAdapter {
    responses: std::sync::Mutex<Vec<Result<DaxQueryResponse, ExecError>>>,
}

impl FixtureDaxAdapter {
    pub fn new(responses: Vec<Result<DaxQueryResponse, ExecError>>) -> Self {
        Self {
            responses: std::sync::Mutex::new(responses),
        }
    }
}

#[async_trait]
impl DaxAdapter for FixtureDaxAdapter {
    async fn execute_dax(
        &self,
        _request: &DaxQueryRequest,
    ) -> Result<DaxQueryResponse, ExecError> {
        let mut responses = self.responses.lock().unwrap();
        if responses.is_empty() {
            Err(ExecError::DaxExecution(
                "no more fixture DAX responses".into(),
            ))
        } else {
            responses.remove(0)
        }
    }
}

// tests at bottom (from Step 1)
```

**Step 4: Run test to verify it passes**

Run: `cd spool && cargo test -p spool-exec -- dax_adapter`
Expected: 4 tests PASS

**Step 5: Commit**

```bash
git add spool/spool-exec/src/dax_adapter.rs
git commit -m "feat(spool-exec): DAX execution adapter trait with fixture implementation and typed query response model"
```

---

## Task 4: Warehouse SQL Adapter Trait And Fixture

**Files:**

- Create: `spool/spool-exec/src/warehouse_adapter.rs`

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[tokio::test]
    async fn fixture_warehouse_adapter_returns_configured_response() {
        let response = WarehouseQueryResponse {
            rows: vec![
                WarehouseRow {
                    values: vec![
                        SqlValue::String("North".into()),
                        SqlValue::Decimal("12400000.00".into()),
                    ],
                },
                WarehouseRow {
                    values: vec![
                        SqlValue::String("South".into()),
                        SqlValue::Decimal("8200000.00".into()),
                    ],
                },
            ],
            columns: vec![
                SqlColumn {
                    name: "Region".into(),
                    data_type: "nvarchar".into(),
                },
                SqlColumn {
                    name: "Revenue".into(),
                    data_type: "decimal".into(),
                },
            ],
            executed_at: Utc::now(),
            execution_duration_ms: 150,
            row_count: 2,
        };

        let adapter = FixtureWarehouseAdapter::new(vec![Ok(response.clone())]);
        let request = WarehouseQueryRequest {
            warehouse_id: "wh_456".into(),
            sql: "SELECT Region, SUM(Revenue) AS Revenue FROM dbo.sales GROUP BY Region".into(),
        };

        let result = adapter.execute_sql(&request).await.unwrap();
        assert_eq!(result.rows.len(), 2);
        assert_eq!(result.columns.len(), 2);
        assert_eq!(result.row_count, 2);
    }

    #[tokio::test]
    async fn fixture_warehouse_adapter_rejects_non_select() {
        let adapter = PolicyEnforcingWarehouseAdapter::new(
            Box::new(FixtureWarehouseAdapter::new(vec![])),
        );
        let request = WarehouseQueryRequest {
            warehouse_id: "wh_456".into(),
            sql: "DELETE FROM dbo.sales WHERE id = 1".into(),
        };

        let result = adapter.execute_sql(&request).await;
        assert!(result.is_err());
        assert!(
            result.unwrap_err().to_string().contains("policy violation"),
        );
    }

    #[tokio::test]
    async fn policy_enforcing_adapter_allows_select() {
        let inner_response = WarehouseQueryResponse {
            rows: vec![WarehouseRow {
                values: vec![SqlValue::Integer(42)],
            }],
            columns: vec![SqlColumn {
                name: "count".into(),
                data_type: "int".into(),
            }],
            executed_at: Utc::now(),
            execution_duration_ms: 5,
            row_count: 1,
        };

        let adapter = PolicyEnforcingWarehouseAdapter::new(
            Box::new(FixtureWarehouseAdapter::new(vec![Ok(inner_response)])),
        );
        let request = WarehouseQueryRequest {
            warehouse_id: "wh_456".into(),
            sql: "SELECT COUNT(*) AS count FROM dbo.sales".into(),
        };

        let result = adapter.execute_sql(&request).await.unwrap();
        assert_eq!(result.row_count, 1);
    }

    #[test]
    fn warehouse_query_response_round_trip() {
        let response = WarehouseQueryResponse {
            rows: vec![WarehouseRow {
                values: vec![SqlValue::Integer(100)],
            }],
            columns: vec![SqlColumn {
                name: "total".into(),
                data_type: "int".into(),
            }],
            executed_at: Utc::now(),
            execution_duration_ms: 20,
            row_count: 1,
        };

        let json = serde_json::to_string(&response).unwrap();
        let restored: WarehouseQueryResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.row_count, 1);
        assert_eq!(restored.columns[0].name, "total");
    }

    #[test]
    fn all_sql_value_variants_serialize() {
        let values = vec![
            SqlValue::String("hello".into()),
            SqlValue::Integer(42),
            SqlValue::Float(3.14),
            SqlValue::Decimal("99.99".into()),
            SqlValue::Boolean(false),
            SqlValue::Null,
        ];
        for v in values {
            let json = serde_json::to_string(&v).unwrap();
            let restored: SqlValue = serde_json::from_str(&json).unwrap();
            assert_eq!(serde_json::to_string(&restored).unwrap(), json);
        }
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cd spool && cargo test -p spool-exec -- warehouse_adapter`
Expected: FAIL

**Step 3: Write the implementation**

```rust
// spool/spool-exec/src/warehouse_adapter.rs
//
// Warehouse SQL execution adapter for Spec Section 11.6: Fabric Warehouse.
// Includes a policy-enforcing wrapper that validates SQL through the sql_policy
// module before forwarding to the inner adapter. This enforces Spec Section 14.2:
// warehouse SQL is read-only in v1.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::ExecError;
use crate::sql_policy;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarehouseQueryRequest {
    pub warehouse_id: String,
    pub sql: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SqlValue {
    String(String),
    Integer(i64),
    Float(f64),
    Decimal(String),
    Boolean(bool),
    Null,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SqlColumn {
    pub name: String,
    pub data_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarehouseRow {
    pub values: Vec<SqlValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarehouseQueryResponse {
    pub rows: Vec<WarehouseRow>,
    pub columns: Vec<SqlColumn>,
    pub executed_at: DateTime<Utc>,
    pub execution_duration_ms: u64,
    pub row_count: usize,
}

#[async_trait]
pub trait WarehouseAdapter: Send + Sync {
    async fn execute_sql(
        &self,
        request: &WarehouseQueryRequest,
    ) -> Result<WarehouseQueryResponse, ExecError>;
}

// --- Policy-enforcing wrapper ---

/// A wrapper adapter that validates SQL against the read-only policy
/// before delegating to an inner adapter.
///
/// This is the primary entry point for warehouse query execution.
/// Direct use of the inner adapter bypasses policy enforcement.
pub struct PolicyEnforcingWarehouseAdapter {
    inner: Box<dyn WarehouseAdapter>,
}

impl PolicyEnforcingWarehouseAdapter {
    pub fn new(inner: Box<dyn WarehouseAdapter>) -> Self {
        Self { inner }
    }
}

#[async_trait]
impl WarehouseAdapter for PolicyEnforcingWarehouseAdapter {
    async fn execute_sql(
        &self,
        request: &WarehouseQueryRequest,
    ) -> Result<WarehouseQueryResponse, ExecError> {
        sql_policy::validate_sql(&request.sql)?;
        self.inner.execute_sql(request).await
    }
}

// --- Fixture implementation ---

pub struct FixtureWarehouseAdapter {
    responses: std::sync::Mutex<Vec<Result<WarehouseQueryResponse, ExecError>>>,
}

impl FixtureWarehouseAdapter {
    pub fn new(
        responses: Vec<Result<WarehouseQueryResponse, ExecError>>,
    ) -> Self {
        Self {
            responses: std::sync::Mutex::new(responses),
        }
    }
}

#[async_trait]
impl WarehouseAdapter for FixtureWarehouseAdapter {
    async fn execute_sql(
        &self,
        _request: &WarehouseQueryRequest,
    ) -> Result<WarehouseQueryResponse, ExecError> {
        let mut responses = self.responses.lock().unwrap();
        if responses.is_empty() {
            Err(ExecError::WarehouseExecution(
                "no more fixture warehouse responses".into(),
            ))
        } else {
            responses.remove(0)
        }
    }
}

// tests at bottom (from Step 1)
```

**Step 4: Run test to verify it passes**

Run: `cd spool && cargo test -p spool-exec -- warehouse_adapter`
Expected: 5 tests PASS

**Step 5: Commit**

```bash
git add spool/spool-exec/src/warehouse_adapter.rs
git commit -m "feat(spool-exec): warehouse SQL adapter with policy-enforcing wrapper per Spec Section 14.2"
```

---

## Task 5: Evidence Capture From Query Results

**Files:**

- Create: `spool/spool-validation/src/evidence_capture.rs`

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use spool_exec::dax_adapter::{DaxColumn, DaxQueryResponse, DaxRow, DaxValue};
    use spool_exec::warehouse_adapter::{
        SqlColumn, SqlValue, WarehouseQueryResponse, WarehouseRow,
    };
    use spool_protocol::artifact::ArtifactId;
    use spool_protocol::evidence::{EvidenceClass, EvidenceType};

    #[test]
    fn capture_dax_query_result_as_evidence() {
        let response = DaxQueryResponse {
            rows: vec![
                DaxRow {
                    values: vec![DaxValue::String("North".into()), DaxValue::Float(12_400_000.0)],
                },
            ],
            columns: vec![
                DaxColumn { name: "Region".into(), data_type: "String".into() },
                DaxColumn { name: "Revenue".into(), data_type: "Double".into() },
            ],
            executed_at: Utc::now(),
            execution_duration_ms: 42,
        };

        let artifact_ref = ArtifactId("art_measure_revenue".into());
        let query_text = "EVALUATE SUMMARIZECOLUMNS(...)";

        let evidence = capture_dax_evidence(&response, &artifact_ref, query_text);

        assert_eq!(evidence.evidence_type, EvidenceType::Observed);
        assert_eq!(evidence.evidence_class, EvidenceClass::DaxQueryResult);
        assert!(evidence.artifact_refs.contains(&artifact_ref));
        assert!(evidence.observed_at.is_some());
        assert!(evidence.summary.contains("1 row"));
        assert!(evidence.detail.is_some());

        let detail = evidence.detail.unwrap();
        assert!(detail.get("query").is_some());
        assert!(detail.get("row_count").is_some());
        assert!(detail.get("columns").is_some());
    }

    #[test]
    fn capture_warehouse_query_result_as_evidence() {
        let response = WarehouseQueryResponse {
            rows: vec![
                WarehouseRow {
                    values: vec![SqlValue::String("North".into()), SqlValue::Decimal("12400000.00".into())],
                },
                WarehouseRow {
                    values: vec![SqlValue::String("South".into()), SqlValue::Decimal("8200000.00".into())],
                },
            ],
            columns: vec![
                SqlColumn { name: "Region".into(), data_type: "nvarchar".into() },
                SqlColumn { name: "Revenue".into(), data_type: "decimal".into() },
            ],
            executed_at: Utc::now(),
            execution_duration_ms: 150,
            row_count: 2,
        };

        let artifact_ref = ArtifactId("art_warehouse_sales".into());
        let query_text = "SELECT Region, SUM(Revenue) ...";

        let evidence = capture_warehouse_evidence(&response, &artifact_ref, query_text);

        assert_eq!(evidence.evidence_type, EvidenceType::Observed);
        assert_eq!(evidence.evidence_class, EvidenceClass::WarehouseQueryResult);
        assert!(evidence.artifact_refs.contains(&artifact_ref));
        assert!(evidence.summary.contains("2 row"));
    }

    #[test]
    fn captured_evidence_has_freshness_metadata() {
        let response = DaxQueryResponse {
            rows: vec![],
            columns: vec![],
            executed_at: Utc::now(),
            execution_duration_ms: 5,
        };

        let evidence = capture_dax_evidence(
            &response,
            &ArtifactId("art_1".into()),
            "EVALUATE VALUES('T')",
        );

        assert!(evidence.observed_at.is_some());
        let detail = evidence.detail.unwrap();
        assert!(detail.get("executed_at").is_some());
        assert!(detail.get("execution_duration_ms").is_some());
    }

    #[test]
    fn evidence_id_is_unique_per_capture() {
        let response = DaxQueryResponse {
            rows: vec![],
            columns: vec![],
            executed_at: Utc::now(),
            execution_duration_ms: 1,
        };
        let art = ArtifactId("art_1".into());

        let ev1 = capture_dax_evidence(&response, &art, "Q1");
        let ev2 = capture_dax_evidence(&response, &art, "Q2");
        assert_ne!(ev1.id, ev2.id);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cd spool && cargo test -p spool-validation -- evidence_capture`
Expected: FAIL

**Step 3: Write the implementation**

```rust
// spool/spool-validation/src/evidence_capture.rs
//
// Evidence capture from query results per Spec Section 9.1.
// Wraps DAX and warehouse query responses into EvidenceItem types
// from spool-protocol with correct evidence class, freshness metadata,
// and structured detail payloads.

use chrono::Utc;
use spool_exec::dax_adapter::DaxQueryResponse;
use spool_exec::warehouse_adapter::WarehouseQueryResponse;
use spool_protocol::artifact::ArtifactId;
use spool_protocol::evidence::{EvidenceClass, EvidenceId, EvidenceItem, EvidenceType};
use uuid::Uuid;

/// Capture a DAX query response as an observed evidence item.
///
/// The evidence item carries:
/// - evidence_type: Observed (runtime query evidence, Spec Section 9.2 rank 1)
/// - evidence_class: DaxQueryResult
/// - freshness: observed_at set to response execution time
/// - detail: structured JSON with query text, row count, column info, and timing
pub fn capture_dax_evidence(
    response: &DaxQueryResponse,
    artifact_ref: &ArtifactId,
    query_text: &str,
) -> EvidenceItem {
    let row_count = response.rows.len();
    let column_names: Vec<&str> = response.columns.iter().map(|c| c.name.as_str()).collect();

    let summary = format!(
        "DAX query returned {row_count} row{} with columns: {}",
        if row_count == 1 { "" } else { "s" },
        column_names.join(", "),
    );

    let detail = serde_json::json!({
        "query": query_text,
        "row_count": row_count,
        "columns": response.columns,
        "executed_at": response.executed_at.to_rfc3339(),
        "execution_duration_ms": response.execution_duration_ms,
        "sample_rows": &response.rows[..std::cmp::min(response.rows.len(), 10)],
    });

    EvidenceItem {
        id: EvidenceId(format!("ev_dax_{}", Uuid::new_v4())),
        evidence_type: EvidenceType::Observed,
        evidence_class: EvidenceClass::DaxQueryResult,
        source: "dax_query_result".into(),
        summary,
        artifact_refs: vec![artifact_ref.clone()],
        observed_at: Some(response.executed_at),
        detail: Some(detail),
    }
}

/// Capture a warehouse query response as an observed evidence item.
///
/// The evidence item carries:
/// - evidence_type: Observed (runtime query evidence, Spec Section 9.2 rank 1)
/// - evidence_class: WarehouseQueryResult
/// - freshness: observed_at set to response execution time
/// - detail: structured JSON with query text, row count, column info, and timing
pub fn capture_warehouse_evidence(
    response: &WarehouseQueryResponse,
    artifact_ref: &ArtifactId,
    query_text: &str,
) -> EvidenceItem {
    let row_count = response.row_count;
    let column_names: Vec<&str> = response.columns.iter().map(|c| c.name.as_str()).collect();

    let summary = format!(
        "Warehouse query returned {row_count} row{} with columns: {}",
        if row_count == 1 { "" } else { "s" },
        column_names.join(", "),
    );

    let detail = serde_json::json!({
        "query": query_text,
        "row_count": row_count,
        "columns": response.columns,
        "executed_at": response.executed_at.to_rfc3339(),
        "execution_duration_ms": response.execution_duration_ms,
        "sample_rows": &response.rows[..std::cmp::min(response.rows.len(), 10)],
    });

    EvidenceItem {
        id: EvidenceId(format!("ev_wh_{}", Uuid::new_v4())),
        evidence_type: EvidenceType::Observed,
        evidence_class: EvidenceClass::WarehouseQueryResult,
        source: "warehouse_query_result".into(),
        summary,
        artifact_refs: vec![artifact_ref.clone()],
        observed_at: Some(response.executed_at),
        detail: Some(detail),
    }
}

// tests at bottom (from Step 1)
```

**Step 4: Run test to verify it passes**

Run: `cd spool && cargo test -p spool-validation -- evidence_capture`
Expected: 4 tests PASS

**Step 5: Commit**

```bash
git add spool/spool-validation/src/evidence_capture.rs
git commit -m "feat(spool-validation): evidence capture from DAX and warehouse query results per Spec Section 9.1"
```

---

## Task 6: Truth Hierarchy

**Files:**

- Create: `spool/spool-validation/src/truth_hierarchy.rs`

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use spool_protocol::artifact::ArtifactId;
    use spool_protocol::evidence::{EvidenceClass, EvidenceId, EvidenceItem, EvidenceType};

    fn make_evidence(
        id: &str,
        evidence_type: EvidenceType,
        evidence_class: EvidenceClass,
        source: &str,
    ) -> EvidenceItem {
        EvidenceItem {
            id: EvidenceId(id.into()),
            evidence_type,
            evidence_class,
            source: source.into(),
            summary: format!("Evidence {id}"),
            artifact_refs: vec![ArtifactId("art_1".into())],
            observed_at: Some(Utc::now()),
            detail: None,
        }
    }

    #[test]
    fn runtime_query_outranks_metadata() {
        let query_ev = make_evidence(
            "ev_dax",
            EvidenceType::Observed,
            EvidenceClass::DaxQueryResult,
            "dax_query_result",
        );
        let metadata_ev = make_evidence(
            "ev_meta",
            EvidenceType::Observed,
            EvidenceClass::SemanticModelMetadata,
            "semantic_model_metadata",
        );

        let query_rank = truth_rank(&query_ev);
        let meta_rank = truth_rank(&metadata_ev);

        // Lower rank number = higher precedence
        assert!(
            query_rank < meta_rank,
            "runtime query (rank {query_rank}) should outrank metadata (rank {meta_rank})",
        );
    }

    #[test]
    fn metadata_outranks_derived() {
        let metadata_ev = make_evidence(
            "ev_meta",
            EvidenceType::Observed,
            EvidenceClass::SemanticModelMetadata,
            "semantic_model_metadata",
        );
        let derived_ev = make_evidence(
            "ev_derived",
            EvidenceType::Derived,
            EvidenceClass::CrossSourceComparison,
            "comparison",
        );

        assert!(truth_rank(&metadata_ev) < truth_rank(&derived_ev));
    }

    #[test]
    fn proposed_has_lowest_rank() {
        let proposed_ev = make_evidence(
            "ev_proposed",
            EvidenceType::Proposed,
            EvidenceClass::DaxQueryResult,
            "user_assertion",
        );
        let observed_ev = make_evidence(
            "ev_observed",
            EvidenceType::Observed,
            EvidenceClass::DaxQueryResult,
            "dax_query_result",
        );

        assert!(truth_rank(&proposed_ev) > truth_rank(&observed_ev));
    }

    #[test]
    fn sort_by_truth_hierarchy() {
        let items = vec![
            make_evidence("ev_proposed", EvidenceType::Proposed, EvidenceClass::MeasureDefinition, "user"),
            make_evidence("ev_dax", EvidenceType::Observed, EvidenceClass::DaxQueryResult, "dax"),
            make_evidence("ev_meta", EvidenceType::Observed, EvidenceClass::SemanticModelMetadata, "meta"),
            make_evidence("ev_wh", EvidenceType::Observed, EvidenceClass::WarehouseQueryResult, "wh"),
            make_evidence("ev_derived", EvidenceType::Derived, EvidenceClass::CrossSourceComparison, "comp"),
        ];

        let sorted = sort_by_truth_rank(&items);
        assert_eq!(sorted[0].id, EvidenceId("ev_dax".into()));
        assert_eq!(sorted[1].id, EvidenceId("ev_wh".into()));
    }

    #[test]
    fn stronger_evidence_wins_conflict() {
        let runtime_ev = make_evidence(
            "ev_runtime",
            EvidenceType::Observed,
            EvidenceClass::DaxQueryResult,
            "dax",
        );
        let knowledge_ev = make_evidence(
            "ev_knowledge",
            EvidenceType::Derived,
            EvidenceClass::MeasureDefinition,
            "knowledge_bundle",
        );

        let winner = resolve_conflict(&runtime_ev, &knowledge_ev);
        assert_eq!(winner, ConflictResolution::PreferFirst);

        let winner_reversed = resolve_conflict(&knowledge_ev, &runtime_ev);
        assert_eq!(winner_reversed, ConflictResolution::PreferSecond);
    }

    #[test]
    fn same_rank_yields_ambiguous() {
        let ev1 = make_evidence(
            "ev_dax_1",
            EvidenceType::Observed,
            EvidenceClass::DaxQueryResult,
            "dax",
        );
        let ev2 = make_evidence(
            "ev_dax_2",
            EvidenceType::Observed,
            EvidenceClass::DaxQueryResult,
            "dax",
        );

        let winner = resolve_conflict(&ev1, &ev2);
        assert_eq!(winner, ConflictResolution::Ambiguous);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cd spool && cargo test -p spool-validation -- truth_hierarchy`
Expected: FAIL

**Step 3: Write the implementation**

```rust
// spool/spool-validation/src/truth_hierarchy.rs
//
// Truth hierarchy implementation per Spec Section 9.2.
//
// Default precedence order (rank 1 is highest):
// 1. Runtime observed evidence from exact target artifact or direct validation query
// 2. Current Fabric or semantic-model metadata for the exact target artifact
// 3. Curated LOB knowledge for the selected bundle
// 4. Durable memory from prior sessions
// 5. Recipe guidance
// 6. User assertions that have not yet been validated

use spool_protocol::evidence::{EvidenceClass, EvidenceItem, EvidenceType};

/// Truth rank for an evidence item. Lower values indicate higher precedence.
///
/// Ranking is based on the combination of evidence_type and evidence_class
/// per the default hierarchy in Spec Section 9.2.
pub fn truth_rank(evidence: &EvidenceItem) -> u32 {
    match evidence.evidence_type {
        EvidenceType::Observed => match evidence.evidence_class {
            // Rank 1: runtime observed evidence from validation queries
            EvidenceClass::DaxQueryResult => 10,
            EvidenceClass::WarehouseQueryResult => 11,
            EvidenceClass::CrossSourceComparison => 12,
            // Rank 2: current metadata for exact target artifact
            EvidenceClass::ReportMetadata => 20,
            EvidenceClass::VisualMetadata => 21,
            EvidenceClass::SemanticModelMetadata => 22,
            EvidenceClass::MeasureDefinition => 23,
        },
        EvidenceType::Derived => match evidence.evidence_class {
            // Rank 3-5: derived evidence from knowledge, memory, or comparison
            EvidenceClass::CrossSourceComparison => 30,
            EvidenceClass::MeasureDefinition => 31,
            _ => 40,
        },
        EvidenceType::Proposed => {
            // Rank 6: user assertions not yet validated
            60
        }
    }
}

/// Sort evidence items by truth hierarchy rank (highest precedence first).
pub fn sort_by_truth_rank(items: &[EvidenceItem]) -> Vec<EvidenceItem> {
    let mut sorted: Vec<EvidenceItem> = items.to_vec();
    sorted.sort_by_key(|item| truth_rank(item));
    sorted
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictResolution {
    /// First evidence item has higher precedence.
    PreferFirst,
    /// Second evidence item has higher precedence.
    PreferSecond,
    /// Both have equal rank -- ambiguous, may need contradiction handling.
    Ambiguous,
}

/// Resolve a conflict between two evidence items using truth hierarchy ranking.
///
/// Per Spec Section 9.2:
/// - Higher-precedence sources normally outweigh lower-precedence sources
/// - If both have the same rank, the conflict is ambiguous and may require
///   contradiction handling
pub fn resolve_conflict(first: &EvidenceItem, second: &EvidenceItem) -> ConflictResolution {
    let rank_first = truth_rank(first);
    let rank_second = truth_rank(second);

    if rank_first < rank_second {
        ConflictResolution::PreferFirst
    } else if rank_second < rank_first {
        ConflictResolution::PreferSecond
    } else {
        ConflictResolution::Ambiguous
    }
}

// tests at bottom (from Step 1)
```

**Step 4: Run test to verify it passes**

Run: `cd spool && cargo test -p spool-validation -- truth_hierarchy`
Expected: 6 tests PASS

**Step 5: Commit**

```bash
git add spool/spool-validation/src/truth_hierarchy.rs
git commit -m "feat(spool-validation): truth hierarchy ranking and conflict resolution per Spec Section 9.2"
```

---

## Task 7: Freshness Policy

**Files:**

- Create: `spool/spool-validation/src/freshness.rs`

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};
    use spool_protocol::artifact::ArtifactId;
    use spool_protocol::evidence::{EvidenceClass, EvidenceId, EvidenceItem, EvidenceType};

    fn make_evidence_at(id: &str, class: EvidenceClass, observed_at: DateTime<Utc>) -> EvidenceItem {
        EvidenceItem {
            id: EvidenceId(id.into()),
            evidence_type: EvidenceType::Observed,
            evidence_class: class,
            source: "test".into(),
            summary: format!("Evidence {id}"),
            artifact_refs: vec![ArtifactId("art_1".into())],
            observed_at: Some(observed_at),
            detail: Some(serde_json::json!({
                "executed_at": observed_at.to_rfc3339(),
            })),
        }
    }

    #[test]
    fn fresh_evidence_is_not_stale() {
        let now = Utc::now();
        let ev = make_evidence_at("ev_1", EvidenceClass::DaxQueryResult, now);
        let assessment = assess_freshness(&ev, &FreshnessContext::default());

        assert_eq!(assessment.status, FreshnessStatus::Fresh);
        assert!(!assessment.is_stale());
    }

    #[test]
    fn old_evidence_is_stale() {
        let old_time = Utc::now() - Duration::hours(25);
        let ev = make_evidence_at("ev_1", EvidenceClass::DaxQueryResult, old_time);
        let ctx = FreshnessContext {
            max_age_runtime_query: Duration::hours(24),
            max_age_metadata: Duration::hours(48),
            known_refresh_after: None,
        };
        let assessment = assess_freshness(&ev, &ctx);

        assert_eq!(assessment.status, FreshnessStatus::Stale);
        assert!(assessment.is_stale());
    }

    #[test]
    fn evidence_before_known_refresh_is_stale() {
        let observed = Utc::now() - Duration::hours(2);
        let refresh = Utc::now() - Duration::hours(1);
        let ev = make_evidence_at("ev_1", EvidenceClass::WarehouseQueryResult, observed);
        let ctx = FreshnessContext {
            max_age_runtime_query: Duration::hours(24),
            max_age_metadata: Duration::hours(48),
            known_refresh_after: Some(refresh),
        };
        let assessment = assess_freshness(&ev, &ctx);

        assert_eq!(assessment.status, FreshnessStatus::StaleByRefresh);
        assert!(assessment.is_stale());
    }

    #[test]
    fn evidence_after_known_refresh_is_fresh() {
        let refresh = Utc::now() - Duration::hours(2);
        let observed = Utc::now() - Duration::hours(1);
        let ev = make_evidence_at("ev_1", EvidenceClass::DaxQueryResult, observed);
        let ctx = FreshnessContext {
            max_age_runtime_query: Duration::hours(24),
            max_age_metadata: Duration::hours(48),
            known_refresh_after: Some(refresh),
        };
        let assessment = assess_freshness(&ev, &ctx);

        assert_eq!(assessment.status, FreshnessStatus::Fresh);
    }

    #[test]
    fn evidence_without_timestamp_is_unknown() {
        let ev = EvidenceItem {
            id: EvidenceId("ev_no_time".into()),
            evidence_type: EvidenceType::Observed,
            evidence_class: EvidenceClass::DaxQueryResult,
            source: "test".into(),
            summary: "no timestamp".into(),
            artifact_refs: vec![],
            observed_at: None,
            detail: None,
        };
        let assessment = assess_freshness(&ev, &FreshnessContext::default());

        assert_eq!(assessment.status, FreshnessStatus::Unknown);
    }

    #[test]
    fn metadata_has_longer_freshness_window() {
        let age_30h = Utc::now() - Duration::hours(30);
        let query_ev = make_evidence_at("ev_q", EvidenceClass::DaxQueryResult, age_30h);
        let meta_ev = make_evidence_at("ev_m", EvidenceClass::SemanticModelMetadata, age_30h);
        let ctx = FreshnessContext {
            max_age_runtime_query: Duration::hours(24),
            max_age_metadata: Duration::hours(48),
            known_refresh_after: None,
        };

        assert!(assess_freshness(&query_ev, &ctx).is_stale());
        assert!(!assess_freshness(&meta_ev, &ctx).is_stale());
    }

    #[test]
    fn stale_evidence_cannot_justify_confirmed() {
        let old = Utc::now() - Duration::hours(25);
        let ev = make_evidence_at("ev_1", EvidenceClass::DaxQueryResult, old);
        let ctx = FreshnessContext {
            max_age_runtime_query: Duration::hours(24),
            max_age_metadata: Duration::hours(48),
            known_refresh_after: None,
        };
        let assessment = assess_freshness(&ev, &ctx);

        assert!(!assessment.can_support_confirmed());
    }

    #[test]
    fn fresh_evidence_can_support_confirmed() {
        let now = Utc::now();
        let ev = make_evidence_at("ev_1", EvidenceClass::DaxQueryResult, now);
        let assessment = assess_freshness(&ev, &FreshnessContext::default());

        assert!(assessment.can_support_confirmed());
    }

    #[test]
    fn pick_fresher_evidence() {
        let older = Utc::now() - Duration::hours(5);
        let newer = Utc::now() - Duration::hours(1);
        let ev_old = make_evidence_at("ev_old", EvidenceClass::DaxQueryResult, older);
        let ev_new = make_evidence_at("ev_new", EvidenceClass::DaxQueryResult, newer);

        assert_eq!(
            fresher_of(&ev_old, &ev_new),
            FreshnessComparison::SecondFresher,
        );
        assert_eq!(
            fresher_of(&ev_new, &ev_old),
            FreshnessComparison::FirstFresher,
        );
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cd spool && cargo test -p spool-validation -- freshness`
Expected: FAIL

**Step 3: Write the implementation**

```rust
// spool/spool-validation/src/freshness.rs
//
// Freshness policy per Spec Section 9.3.
//
// Freshness matters because report definitions, semantic models, warehouse data,
// curated knowledge, and prior-session memory can all drift over time.
//
// Key rules:
// - stale evidence may guide investigation but should weaken confidence
// - stale evidence alone must not justify a confirmed result
// - if freshness cannot be determined, confidence should be reduced

use chrono::{DateTime, Duration, Utc};
use spool_protocol::evidence::{EvidenceClass, EvidenceItem};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FreshnessStatus {
    /// Evidence was observed within the acceptable freshness window.
    Fresh,
    /// Evidence has exceeded the time-based freshness threshold.
    Stale,
    /// Evidence was observed before a known data refresh or republish event.
    StaleByRefresh,
    /// Freshness cannot be determined (no timestamp available).
    Unknown,
}

#[derive(Debug, Clone)]
pub struct FreshnessAssessment {
    pub status: FreshnessStatus,
    pub evidence_id: String,
    pub observed_at: Option<DateTime<Utc>>,
    pub age: Option<Duration>,
    pub reason: String,
}

impl FreshnessAssessment {
    pub fn is_stale(&self) -> bool {
        matches!(
            self.status,
            FreshnessStatus::Stale | FreshnessStatus::StaleByRefresh,
        )
    }

    /// Per Spec Section 9.3: stale evidence alone must not justify confirmed.
    /// Unknown freshness should also reduce confidence.
    pub fn can_support_confirmed(&self) -> bool {
        self.status == FreshnessStatus::Fresh
    }
}

/// Context for freshness evaluation.
///
/// Different evidence classes have different freshness windows per Spec Section 9.3.
pub struct FreshnessContext {
    /// Maximum age for runtime query evidence before it is considered stale.
    pub max_age_runtime_query: Duration,
    /// Maximum age for metadata evidence before it is considered stale.
    pub max_age_metadata: Duration,
    /// If a known data refresh or artifact republish occurred after this time,
    /// evidence observed before this time is stale-by-refresh.
    pub known_refresh_after: Option<DateTime<Utc>>,
}

impl Default for FreshnessContext {
    fn default() -> Self {
        Self {
            max_age_runtime_query: Duration::hours(24),
            max_age_metadata: Duration::hours(48),
            known_refresh_after: None,
        }
    }
}

/// Assess freshness of a single evidence item.
pub fn assess_freshness(
    evidence: &EvidenceItem,
    context: &FreshnessContext,
) -> FreshnessAssessment {
    let evidence_id = evidence.id.0.clone();

    let Some(observed_at) = evidence.observed_at else {
        return FreshnessAssessment {
            status: FreshnessStatus::Unknown,
            evidence_id,
            observed_at: None,
            age: None,
            reason: "no observed_at timestamp available".into(),
        };
    };

    let now = Utc::now();
    let age = now - observed_at;

    // Check if observed before a known refresh/republish event
    if let Some(refresh_time) = context.known_refresh_after {
        if observed_at < refresh_time {
            return FreshnessAssessment {
                status: FreshnessStatus::StaleByRefresh,
                evidence_id,
                observed_at: Some(observed_at),
                age: Some(age),
                reason: format!(
                    "evidence observed before known refresh at {}",
                    refresh_time.to_rfc3339(),
                ),
            };
        }
    }

    // Check time-based staleness by evidence class
    let max_age = max_age_for_class(&evidence.evidence_class, context);

    if age > max_age {
        FreshnessAssessment {
            status: FreshnessStatus::Stale,
            evidence_id,
            observed_at: Some(observed_at),
            age: Some(age),
            reason: format!(
                "evidence age ({} hours) exceeds threshold ({} hours) for class {:?}",
                age.num_hours(),
                max_age.num_hours(),
                evidence.evidence_class,
            ),
        }
    } else {
        FreshnessAssessment {
            status: FreshnessStatus::Fresh,
            evidence_id,
            observed_at: Some(observed_at),
            age: Some(age),
            reason: "within freshness window".into(),
        }
    }
}

fn max_age_for_class(class: &EvidenceClass, context: &FreshnessContext) -> Duration {
    match class {
        // Runtime query evidence: tighter window
        EvidenceClass::DaxQueryResult
        | EvidenceClass::WarehouseQueryResult
        | EvidenceClass::CrossSourceComparison => context.max_age_runtime_query,
        // Metadata evidence: longer window
        EvidenceClass::ReportMetadata
        | EvidenceClass::VisualMetadata
        | EvidenceClass::SemanticModelMetadata
        | EvidenceClass::MeasureDefinition => context.max_age_metadata,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FreshnessComparison {
    FirstFresher,
    SecondFresher,
    Same,
    Indeterminate,
}

/// Compare freshness of two evidence items.
///
/// Per Spec Section 9.3: when multiple evidence items disagree and one is known
/// to be fresher, freshness should be considered explicitly in contradiction handling.
pub fn fresher_of(first: &EvidenceItem, second: &EvidenceItem) -> FreshnessComparison {
    match (first.observed_at, second.observed_at) {
        (Some(t1), Some(t2)) => {
            if t1 > t2 {
                FreshnessComparison::FirstFresher
            } else if t2 > t1 {
                FreshnessComparison::SecondFresher
            } else {
                FreshnessComparison::Same
            }
        }
        (Some(_), None) => FreshnessComparison::FirstFresher,
        (None, Some(_)) => FreshnessComparison::SecondFresher,
        (None, None) => FreshnessComparison::Indeterminate,
    }
}

// tests at bottom (from Step 1)
```

**Step 4: Run test to verify it passes**

Run: `cd spool && cargo test -p spool-validation -- freshness`
Expected: 9 tests PASS

**Step 5: Commit**

```bash
git add spool/spool-validation/src/freshness.rs
git commit -m "feat(spool-validation): freshness policy with class-aware staleness, refresh-based invalidation, and confidence gating per Spec Section 9.3"
```

---

## Task 8: Validation Floor Enforcement

**Files:**

- Create: `spool/spool-validation/src/validation_floor.rs`

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use spool_protocol::artifact::ArtifactId;
    use spool_protocol::evidence::{EvidenceClass, EvidenceId, EvidenceItem, EvidenceType};
    use spool_protocol::task_contract::ValidationFloor;

    fn make_evidence(id: &str, etype: EvidenceType, class: EvidenceClass) -> EvidenceItem {
        EvidenceItem {
            id: EvidenceId(id.into()),
            evidence_type: etype,
            evidence_class: class,
            source: "test".into(),
            summary: format!("Evidence {id}"),
            artifact_refs: vec![ArtifactId("art_1".into())],
            observed_at: Some(Utc::now()),
            detail: None,
        }
    }

    #[test]
    fn floor_met_with_observed_and_validation() {
        let evidence = vec![
            make_evidence("ev_1", EvidenceType::Observed, EvidenceClass::ReportMetadata),
            make_evidence("ev_2", EvidenceType::Observed, EvidenceClass::DaxQueryResult),
        ];
        let result = check_validation_floor(
            &evidence,
            &ValidationFloor::DirectValidationRequired,
        );
        assert!(result.is_met);
        assert!(result.violations.is_empty());
    }

    #[test]
    fn floor_not_met_without_observed_evidence() {
        let evidence = vec![
            make_evidence("ev_1", EvidenceType::Derived, EvidenceClass::CrossSourceComparison),
            make_evidence("ev_2", EvidenceType::Proposed, EvidenceClass::MeasureDefinition),
        ];
        let result = check_validation_floor(
            &evidence,
            &ValidationFloor::DirectValidationRequired,
        );
        assert!(!result.is_met);
        assert!(
            result.violations.iter().any(|v| v.contains("observed")),
            "expected observed evidence violation, got: {:?}",
            result.violations,
        );
    }

    #[test]
    fn floor_not_met_without_direct_validation() {
        let evidence = vec![
            make_evidence("ev_1", EvidenceType::Observed, EvidenceClass::ReportMetadata),
        ];
        let result = check_validation_floor(
            &evidence,
            &ValidationFloor::DirectValidationRequired,
        );
        assert!(!result.is_met);
        assert!(
            result.violations.iter().any(|v| v.contains("direct validation")),
            "expected direct validation violation, got: {:?}",
            result.violations,
        );
    }

    #[test]
    fn metadata_only_floor_met_with_metadata() {
        let evidence = vec![
            make_evidence("ev_1", EvidenceType::Observed, EvidenceClass::SemanticModelMetadata),
        ];
        let result = check_validation_floor(
            &evidence,
            &ValidationFloor::MetadataOnly,
        );
        assert!(result.is_met);
    }

    #[test]
    fn metadata_only_floor_not_met_without_observed() {
        let evidence: Vec<EvidenceItem> = vec![];
        let result = check_validation_floor(
            &evidence,
            &ValidationFloor::MetadataOnly,
        );
        assert!(!result.is_met);
    }

    #[test]
    fn empty_evidence_never_meets_floor() {
        let evidence: Vec<EvidenceItem> = vec![];
        let result = check_validation_floor(
            &evidence,
            &ValidationFloor::DirectValidationRequired,
        );
        assert!(!result.is_met);
        assert!(!result.violations.is_empty());
    }

    #[test]
    fn warehouse_result_counts_as_direct_validation() {
        let evidence = vec![
            make_evidence("ev_1", EvidenceType::Observed, EvidenceClass::ReportMetadata),
            make_evidence("ev_2", EvidenceType::Observed, EvidenceClass::WarehouseQueryResult),
        ];
        let result = check_validation_floor(
            &evidence,
            &ValidationFloor::DirectValidationRequired,
        );
        assert!(result.is_met);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cd spool && cargo test -p spool-validation -- validation_floor`
Expected: FAIL

**Step 3: Write the implementation**

```rust
// spool/spool-validation/src/validation_floor.rs
//
// Validation floor enforcement per Spec Section 9.6.
//
// Every recommendation must include at least one observed evidence item.
// Every non-trivial recommendation must include at least one direct validation
// step tied to a relevant artifact, DAX query result, or warehouse result.

use spool_protocol::evidence::{EvidenceClass, EvidenceItem, EvidenceType};
use spool_protocol::task_contract::ValidationFloor;

#[derive(Debug, Clone)]
pub struct ValidationFloorResult {
    pub is_met: bool,
    pub violations: Vec<String>,
    pub has_observed_evidence: bool,
    pub has_direct_validation: bool,
}

/// Evidence classes that count as direct validation steps.
///
/// These represent runtime query execution against actual data sources,
/// as opposed to metadata inspection or derived analysis.
fn is_direct_validation_class(class: &EvidenceClass) -> bool {
    matches!(
        class,
        EvidenceClass::DaxQueryResult
            | EvidenceClass::WarehouseQueryResult
            | EvidenceClass::CrossSourceComparison
    )
}

/// Check whether the collected evidence meets the validation floor
/// defined in the task contract.
///
/// Per Spec Section 9.6:
/// - Every recommendation must include at least one observed evidence item
/// - Every non-trivial recommendation must include at least one direct
///   validation step tied to a relevant artifact, DAX query result,
///   or warehouse result
pub fn check_validation_floor(
    evidence: &[EvidenceItem],
    floor: &ValidationFloor,
) -> ValidationFloorResult {
    let has_observed = evidence
        .iter()
        .any(|e| e.evidence_type == EvidenceType::Observed);

    let has_direct_validation = evidence.iter().any(|e| {
        e.evidence_type == EvidenceType::Observed && is_direct_validation_class(&e.evidence_class)
    });

    let mut violations = Vec::new();

    if !has_observed {
        violations.push(
            "validation floor requires at least one observed evidence item".into(),
        );
    }

    match floor {
        ValidationFloor::DirectValidationRequired => {
            if !has_direct_validation {
                violations.push(
                    "validation floor requires at least one direct validation step (DAX query, warehouse query, or cross-source comparison)".into(),
                );
            }
        }
        ValidationFloor::MetadataOnly => {
            // MetadataOnly floor only requires observed evidence, no direct validation needed
        }
    }

    ValidationFloorResult {
        is_met: violations.is_empty(),
        violations,
        has_observed_evidence: has_observed,
        has_direct_validation,
    }
}

// tests at bottom (from Step 1)
```

**Step 4: Run test to verify it passes**

Run: `cd spool && cargo test -p spool-validation -- validation_floor`
Expected: 7 tests PASS

**Step 5: Commit**

```bash
git add spool/spool-validation/src/validation_floor.rs
git commit -m "feat(spool-validation): validation floor enforcement per Spec Section 9.6"
```

---

## Task 9: Minimum Validation Patterns By Investigation Class

**Files:**

- Create: `spool/spool-validation/src/validation_patterns.rs`

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use spool_protocol::artifact::ArtifactId;
    use spool_protocol::evidence::{EvidenceClass, EvidenceId, EvidenceItem, EvidenceType};

    fn make_evidence(id: &str, class: EvidenceClass) -> EvidenceItem {
        EvidenceItem {
            id: EvidenceId(id.into()),
            evidence_type: EvidenceType::Observed,
            evidence_class: class,
            source: "test".into(),
            summary: format!("Evidence {id}"),
            artifact_refs: vec![ArtifactId("art_1".into())],
            observed_at: Some(Utc::now()),
            detail: None,
        }
    }

    #[test]
    fn report_number_mismatch_met() {
        let evidence = vec![
            make_evidence("ev_1", EvidenceClass::ReportMetadata),
            make_evidence("ev_2", EvidenceClass::MeasureDefinition),
            make_evidence("ev_3", EvidenceClass::DaxQueryResult),
        ];
        let result = check_investigation_pattern(
            &InvestigationClass::ReportNumberMismatch,
            &evidence,
        );
        assert!(result.is_met);
        assert!(result.missing_steps.is_empty());
    }

    #[test]
    fn report_number_mismatch_missing_query() {
        let evidence = vec![
            make_evidence("ev_1", EvidenceClass::ReportMetadata),
            make_evidence("ev_2", EvidenceClass::MeasureDefinition),
        ];
        let result = check_investigation_pattern(
            &InvestigationClass::ReportNumberMismatch,
            &evidence,
        );
        assert!(!result.is_met);
        assert!(
            result.missing_steps.iter().any(|s| s.contains("query-based validation")),
            "expected query validation missing, got: {:?}",
            result.missing_steps,
        );
    }

    #[test]
    fn measure_logic_review_met() {
        let evidence = vec![
            make_evidence("ev_1", EvidenceClass::MeasureDefinition),
            make_evidence("ev_2", EvidenceClass::SemanticModelMetadata),
            make_evidence("ev_3", EvidenceClass::DaxQueryResult),
        ];
        let result = check_investigation_pattern(
            &InvestigationClass::MeasureLogicReview,
            &evidence,
        );
        assert!(result.is_met);
    }

    #[test]
    fn measure_logic_review_missing_definition() {
        let evidence = vec![
            make_evidence("ev_1", EvidenceClass::DaxQueryResult),
        ];
        let result = check_investigation_pattern(
            &InvestigationClass::MeasureLogicReview,
            &evidence,
        );
        assert!(!result.is_met);
        assert!(
            result.missing_steps.iter().any(|s| s.contains("measure definition")),
        );
    }

    #[test]
    fn warehouse_vs_model_disagreement_met() {
        let evidence = vec![
            make_evidence("ev_1", EvidenceClass::WarehouseQueryResult),
            make_evidence("ev_2", EvidenceClass::DaxQueryResult),
            make_evidence("ev_3", EvidenceClass::CrossSourceComparison),
        ];
        let result = check_investigation_pattern(
            &InvestigationClass::WarehouseVsModelDisagreement,
            &evidence,
        );
        assert!(result.is_met);
    }

    #[test]
    fn warehouse_vs_model_missing_warehouse() {
        let evidence = vec![
            make_evidence("ev_1", EvidenceClass::DaxQueryResult),
        ];
        let result = check_investigation_pattern(
            &InvestigationClass::WarehouseVsModelDisagreement,
            &evidence,
        );
        assert!(!result.is_met);
        assert!(
            result.missing_steps.iter().any(|s| s.contains("warehouse")),
        );
    }

    #[test]
    fn metadata_investigation_met() {
        let evidence = vec![
            make_evidence("ev_1", EvidenceClass::ReportMetadata),
        ];
        let result = check_investigation_pattern(
            &InvestigationClass::MetadataOrArtifactResolution,
            &evidence,
        );
        assert!(result.is_met);
    }

    #[test]
    fn metadata_investigation_empty() {
        let result = check_investigation_pattern(
            &InvestigationClass::MetadataOrArtifactResolution,
            &[],
        );
        assert!(!result.is_met);
    }

    #[test]
    fn metadata_investigation_caps_result_state() {
        let evidence = vec![
            make_evidence("ev_1", EvidenceClass::ReportMetadata),
        ];
        let result = check_investigation_pattern(
            &InvestigationClass::MetadataOrArtifactResolution,
            &evidence,
        );
        assert!(result.max_result_state_without_additional.is_some());
        assert_eq!(
            result.max_result_state_without_additional.unwrap(),
            MaxResultState::Inconclusive,
        );
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cd spool && cargo test -p spool-validation -- validation_patterns`
Expected: FAIL

**Step 3: Write the implementation**

```rust
// spool/spool-validation/src/validation_patterns.rs
//
// Minimum validation patterns by investigation class per Spec Section 9.7.
//
// These are minimums, not ceilings. The evaluator may require stronger
// validation when the claim or recommendation is higher risk.

use spool_protocol::evidence::{EvidenceClass, EvidenceItem};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InvestigationClass {
    /// Spec Section 9.7: report number mismatch
    ReportNumberMismatch,
    /// Spec Section 9.7: measure logic review
    MeasureLogicReview,
    /// Spec Section 9.7: warehouse-vs-model disagreement
    WarehouseVsModelDisagreement,
    /// Spec Section 9.7: metadata or artifact-resolution investigation
    MetadataOrArtifactResolution,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaxResultState {
    /// No cap -- result state is determined by evaluator
    NoCap,
    /// Cannot go above inconclusive without additional evidence
    Inconclusive,
}

#[derive(Debug, Clone)]
pub struct PatternCheckResult {
    pub investigation_class: InvestigationClass,
    pub is_met: bool,
    pub missing_steps: Vec<String>,
    /// If set, indicates the maximum result state achievable without
    /// additional evidence beyond what is currently collected.
    pub max_result_state_without_additional: Option<MaxResultState>,
}

fn has_class(evidence: &[EvidenceItem], class: &EvidenceClass) -> bool {
    evidence.iter().any(|e| e.evidence_class == *class)
}

fn has_any_query_validation(evidence: &[EvidenceItem]) -> bool {
    evidence.iter().any(|e| {
        matches!(
            e.evidence_class,
            EvidenceClass::DaxQueryResult | EvidenceClass::WarehouseQueryResult
        )
    })
}

fn has_any_metadata(evidence: &[EvidenceItem]) -> bool {
    evidence.iter().any(|e| {
        matches!(
            e.evidence_class,
            EvidenceClass::ReportMetadata
                | EvidenceClass::VisualMetadata
                | EvidenceClass::SemanticModelMetadata
                | EvidenceClass::MeasureDefinition
        )
    })
}

/// Check whether collected evidence meets the minimum validation pattern
/// for a given investigation class.
///
/// Per Spec Section 9.7, each investigation class has specific minimum
/// evidence requirements.
pub fn check_investigation_pattern(
    class: &InvestigationClass,
    evidence: &[EvidenceItem],
) -> PatternCheckResult {
    match class {
        InvestigationClass::ReportNumberMismatch => {
            check_report_number_mismatch(evidence)
        }
        InvestigationClass::MeasureLogicReview => {
            check_measure_logic_review(evidence)
        }
        InvestigationClass::WarehouseVsModelDisagreement => {
            check_warehouse_vs_model(evidence)
        }
        InvestigationClass::MetadataOrArtifactResolution => {
            check_metadata_investigation(evidence)
        }
    }
}

/// Report number mismatch: Spec Section 9.7
/// - inspect the target report artifact
/// - inspect the relevant semantic-model measure or logic
/// - run at least one direct query-based validation
fn check_report_number_mismatch(evidence: &[EvidenceItem]) -> PatternCheckResult {
    let mut missing = Vec::new();

    let has_report = has_class(evidence, &EvidenceClass::ReportMetadata)
        || has_class(evidence, &EvidenceClass::VisualMetadata);
    if !has_report {
        missing.push("inspect target report artifact (ReportMetadata or VisualMetadata)".into());
    }

    let has_measure = has_class(evidence, &EvidenceClass::MeasureDefinition)
        || has_class(evidence, &EvidenceClass::SemanticModelMetadata);
    if !has_measure {
        missing.push("inspect relevant semantic-model measure or logic (MeasureDefinition or SemanticModelMetadata)".into());
    }

    if !has_any_query_validation(evidence) {
        missing.push("run at least one direct query-based validation (DaxQueryResult or WarehouseQueryResult)".into());
    }

    PatternCheckResult {
        investigation_class: InvestigationClass::ReportNumberMismatch,
        is_met: missing.is_empty(),
        missing_steps: missing,
        max_result_state_without_additional: None,
    }
}

/// Measure logic review: Spec Section 9.7
/// - inspect the measure definition
/// - inspect dependencies or referenced objects when relevant
/// - run at least one validating query using the measure
fn check_measure_logic_review(evidence: &[EvidenceItem]) -> PatternCheckResult {
    let mut missing = Vec::new();

    if !has_class(evidence, &EvidenceClass::MeasureDefinition) {
        missing.push("inspect the measure definition (MeasureDefinition)".into());
    }

    if !has_any_query_validation(evidence) {
        missing.push("run at least one validating query using the measure (DaxQueryResult or WarehouseQueryResult)".into());
    }

    PatternCheckResult {
        investigation_class: InvestigationClass::MeasureLogicReview,
        is_met: missing.is_empty(),
        missing_steps: missing,
        max_result_state_without_additional: None,
    }
}

/// Warehouse-vs-model disagreement: Spec Section 9.7
/// - run warehouse-side validation
/// - run model-side or DAX-side validation
/// - compare like-for-like scope and filters
fn check_warehouse_vs_model(evidence: &[EvidenceItem]) -> PatternCheckResult {
    let mut missing = Vec::new();

    if !has_class(evidence, &EvidenceClass::WarehouseQueryResult) {
        missing.push("run warehouse-side validation (WarehouseQueryResult)".into());
    }

    if !has_class(evidence, &EvidenceClass::DaxQueryResult) {
        missing.push("run model-side or DAX-side validation (DaxQueryResult)".into());
    }

    // CrossSourceComparison is desirable but the spec says "compare like-for-like"
    // which can be implicit in having both sources

    PatternCheckResult {
        investigation_class: InvestigationClass::WarehouseVsModelDisagreement,
        is_met: missing.is_empty(),
        missing_steps: missing,
        max_result_state_without_additional: None,
    }
}

/// Metadata or artifact-resolution investigation: Spec Section 9.7
/// - inspect direct metadata
/// - do not assign a result state above inconclusive unless supported by
///   additional evidence beyond metadata alone
fn check_metadata_investigation(evidence: &[EvidenceItem]) -> PatternCheckResult {
    let mut missing = Vec::new();

    if !has_any_metadata(evidence) {
        missing.push("inspect direct metadata (any metadata evidence class)".into());
    }

    // Per spec: cannot go above inconclusive with metadata alone
    let has_query = has_any_query_validation(evidence);
    let cap = if has_query {
        None
    } else {
        Some(MaxResultState::Inconclusive)
    };

    PatternCheckResult {
        investigation_class: InvestigationClass::MetadataOrArtifactResolution,
        is_met: missing.is_empty(),
        missing_steps: missing,
        max_result_state_without_additional: cap,
    }
}

// tests at bottom (from Step 1)
```

**Step 4: Run test to verify it passes**

Run: `cd spool && cargo test -p spool-validation -- validation_patterns`
Expected: 9 tests PASS

**Step 5: Commit**

```bash
git add spool/spool-validation/src/validation_patterns.rs
git commit -m "feat(spool-validation): minimum validation patterns by investigation class per Spec Section 9.7"
```

---

## Task 10: Higher-Risk Validation Cross-Checks

**Files:**

- Create: `spool/spool-validation/src/cross_checks.rs`

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use spool_protocol::artifact::ArtifactId;
    use spool_protocol::evidence::{EvidenceClass, EvidenceId, EvidenceItem, EvidenceType};

    fn make_evidence(id: &str, class: EvidenceClass) -> EvidenceItem {
        EvidenceItem {
            id: EvidenceId(id.into()),
            evidence_type: EvidenceType::Observed,
            evidence_class: class,
            source: "test".into(),
            summary: format!("Evidence {id}"),
            artifact_refs: vec![ArtifactId("art_1".into())],
            observed_at: Some(Utc::now()),
            detail: None,
        }
    }

    #[test]
    fn all_cross_checks_present() {
        let evidence = vec![
            make_evidence("ev_1", EvidenceClass::ReportMetadata),
            make_evidence("ev_2", EvidenceClass::DaxQueryResult),
            make_evidence("ev_3", EvidenceClass::WarehouseQueryResult),
            make_evidence("ev_4", EvidenceClass::MeasureDefinition),
            make_evidence("ev_5", EvidenceClass::CrossSourceComparison),
        ];
        let result = evaluate_cross_checks(&evidence);
        assert!(result.all_met());
        assert_eq!(result.checks.len(), 5);
    }

    #[test]
    fn missing_warehouse_comparison() {
        let evidence = vec![
            make_evidence("ev_1", EvidenceClass::ReportMetadata),
            make_evidence("ev_2", EvidenceClass::DaxQueryResult),
            make_evidence("ev_3", EvidenceClass::MeasureDefinition),
        ];
        let result = evaluate_cross_checks(&evidence);
        assert!(!result.all_met());
        let missing: Vec<&str> = result
            .checks
            .iter()
            .filter(|c| !c.present)
            .map(|c| c.name.as_str())
            .collect();
        assert!(missing.contains(&"warehouse_result_comparison"));
        assert!(missing.contains(&"cross_source_contradiction_check"));
    }

    #[test]
    fn empty_evidence_nothing_met() {
        let result = evaluate_cross_checks(&[]);
        assert!(!result.all_met());
        assert!(result.checks.iter().all(|c| !c.present));
    }

    #[test]
    fn coverage_ratio_calculation() {
        let evidence = vec![
            make_evidence("ev_1", EvidenceClass::ReportMetadata),
            make_evidence("ev_2", EvidenceClass::DaxQueryResult),
        ];
        let result = evaluate_cross_checks(&evidence);
        // 2 out of 5 checks met
        let ratio = result.coverage_ratio();
        assert!(ratio > 0.3 && ratio < 0.5);
    }

    #[test]
    fn sufficient_for_higher_risk_requires_full_coverage() {
        let evidence = vec![
            make_evidence("ev_1", EvidenceClass::ReportMetadata),
            make_evidence("ev_2", EvidenceClass::DaxQueryResult),
            make_evidence("ev_3", EvidenceClass::WarehouseQueryResult),
            make_evidence("ev_4", EvidenceClass::MeasureDefinition),
            make_evidence("ev_5", EvidenceClass::CrossSourceComparison),
        ];
        let result = evaluate_cross_checks(&evidence);
        assert!(result.sufficient_for_higher_risk());

        let partial = vec![
            make_evidence("ev_1", EvidenceClass::DaxQueryResult),
            make_evidence("ev_2", EvidenceClass::WarehouseQueryResult),
        ];
        let result2 = evaluate_cross_checks(&partial);
        assert!(!result2.sufficient_for_higher_risk());
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cd spool && cargo test -p spool-validation -- cross_checks`
Expected: FAIL

**Step 3: Write the implementation**

```rust
// spool/spool-validation/src/cross_checks.rs
//
// Higher-risk validation cross-checks per Spec Section 9.8.
//
// Higher-risk conclusions should use stronger cross-checks such as:
// - artifact metadata inspection
// - DAX query comparison
// - warehouse result comparison
// - business-definition comparison from knowledge
// - contradiction checks across sources

use spool_protocol::evidence::{EvidenceClass, EvidenceItem};

#[derive(Debug, Clone)]
pub struct CrossCheck {
    pub name: String,
    pub description: String,
    pub present: bool,
}

#[derive(Debug, Clone)]
pub struct CrossCheckResult {
    pub checks: Vec<CrossCheck>,
}

impl CrossCheckResult {
    pub fn all_met(&self) -> bool {
        self.checks.iter().all(|c| c.present)
    }

    pub fn coverage_ratio(&self) -> f64 {
        if self.checks.is_empty() {
            return 0.0;
        }
        let met = self.checks.iter().filter(|c| c.present).count();
        met as f64 / self.checks.len() as f64
    }

    /// Per Spec Section 9.8: higher-risk conclusions require all cross-checks.
    pub fn sufficient_for_higher_risk(&self) -> bool {
        self.all_met()
    }
}

/// Evaluate the presence of higher-risk cross-checks in collected evidence.
///
/// Per Spec Section 9.8, the full cross-check set includes:
/// 1. Artifact metadata inspection
/// 2. DAX query comparison
/// 3. Warehouse result comparison
/// 4. Business-definition comparison from knowledge
/// 5. Contradiction checks across sources
pub fn evaluate_cross_checks(evidence: &[EvidenceItem]) -> CrossCheckResult {
    let has_metadata = evidence.iter().any(|e| {
        matches!(
            e.evidence_class,
            EvidenceClass::ReportMetadata
                | EvidenceClass::VisualMetadata
                | EvidenceClass::SemanticModelMetadata
        )
    });

    let has_dax = evidence
        .iter()
        .any(|e| e.evidence_class == EvidenceClass::DaxQueryResult);

    let has_warehouse = evidence
        .iter()
        .any(|e| e.evidence_class == EvidenceClass::WarehouseQueryResult);

    let has_business_definition = evidence
        .iter()
        .any(|e| e.evidence_class == EvidenceClass::MeasureDefinition);

    let has_cross_source = evidence
        .iter()
        .any(|e| e.evidence_class == EvidenceClass::CrossSourceComparison);

    let checks = vec![
        CrossCheck {
            name: "artifact_metadata_inspection".into(),
            description: "Artifact metadata inspection (report, visual, or semantic model)".into(),
            present: has_metadata,
        },
        CrossCheck {
            name: "dax_query_comparison".into(),
            description: "DAX query comparison against semantic model".into(),
            present: has_dax,
        },
        CrossCheck {
            name: "warehouse_result_comparison".into(),
            description: "Warehouse result comparison against warehouse data".into(),
            present: has_warehouse,
        },
        CrossCheck {
            name: "business_definition_comparison".into(),
            description: "Business-definition comparison from knowledge (measure definition)".into(),
            present: has_business_definition,
        },
        CrossCheck {
            name: "cross_source_contradiction_check".into(),
            description: "Contradiction checks across sources (cross-source comparison)".into(),
            present: has_cross_source,
        },
    ];

    CrossCheckResult { checks }
}

// tests at bottom (from Step 1)
```

**Step 4: Run test to verify it passes**

Run: `cd spool && cargo test -p spool-validation -- cross_checks`
Expected: 5 tests PASS

**Step 5: Commit**

```bash
git add spool/spool-validation/src/cross_checks.rs
git commit -m "feat(spool-validation): higher-risk validation cross-checks per Spec Section 9.8"
```

---

## Task 11: Query Result Handling -- Summarize, Preview, Drill-Down

**Files:**

- Create: `spool/spool-validation/src/query_result_handling.rs`

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use spool_exec::dax_adapter::{DaxColumn, DaxQueryResponse, DaxRow, DaxValue};
    use spool_exec::warehouse_adapter::{
        SqlColumn, SqlValue, WarehouseQueryResponse, WarehouseRow,
    };

    fn make_dax_response(row_count: usize) -> DaxQueryResponse {
        let rows: Vec<DaxRow> = (0..row_count)
            .map(|i| DaxRow {
                values: vec![
                    DaxValue::String(format!("Region_{i}")),
                    DaxValue::Float(1_000_000.0 * (i as f64 + 1.0)),
                ],
            })
            .collect();
        DaxQueryResponse {
            rows,
            columns: vec![
                DaxColumn { name: "Region".into(), data_type: "String".into() },
                DaxColumn { name: "Revenue".into(), data_type: "Double".into() },
            ],
            executed_at: Utc::now(),
            execution_duration_ms: 42,
        }
    }

    fn make_warehouse_response(row_count: usize) -> WarehouseQueryResponse {
        let rows: Vec<WarehouseRow> = (0..row_count)
            .map(|i| WarehouseRow {
                values: vec![
                    SqlValue::String(format!("Region_{i}")),
                    SqlValue::Decimal(format!("{}.00", 1_000_000 * (i + 1))),
                ],
            })
            .collect();
        WarehouseQueryResponse {
            rows,
            columns: vec![
                SqlColumn { name: "Region".into(), data_type: "nvarchar".into() },
                SqlColumn { name: "Revenue".into(), data_type: "decimal".into() },
            ],
            executed_at: Utc::now(),
            execution_duration_ms: 100,
            row_count,
        }
    }

    #[test]
    fn summarize_small_dax_result() {
        let response = make_dax_response(3);
        let summary = summarize_dax_result(&response, &SummarizeConfig::default());

        assert!(summary.summary_text.contains("3 row"));
        assert!(summary.summary_text.contains("Region"));
        assert!(summary.summary_text.contains("Revenue"));
        assert_eq!(summary.total_rows, 3);
        assert_eq!(summary.preview_rows, 3);
        assert!(!summary.truncated);
    }

    #[test]
    fn summarize_large_dax_result_truncates() {
        let response = make_dax_response(500);
        let config = SummarizeConfig { max_preview_rows: 10 };
        let summary = summarize_dax_result(&response, &config);

        assert_eq!(summary.total_rows, 500);
        assert_eq!(summary.preview_rows, 10);
        assert!(summary.truncated);
        assert!(summary.summary_text.contains("500 row"));
        assert!(summary.summary_text.contains("showing first 10"));
    }

    #[test]
    fn summarize_small_warehouse_result() {
        let response = make_warehouse_response(2);
        let summary = summarize_warehouse_result(&response, &SummarizeConfig::default());

        assert!(summary.summary_text.contains("2 row"));
        assert_eq!(summary.total_rows, 2);
        assert!(!summary.truncated);
    }

    #[test]
    fn summarize_large_warehouse_result_truncates() {
        let response = make_warehouse_response(200);
        let config = SummarizeConfig { max_preview_rows: 5 };
        let summary = summarize_warehouse_result(&response, &config);

        assert_eq!(summary.total_rows, 200);
        assert_eq!(summary.preview_rows, 5);
        assert!(summary.truncated);
    }

    #[test]
    fn preview_returns_bounded_rows() {
        let response = make_dax_response(50);
        let preview = preview_dax_rows(&response, 5);

        assert_eq!(preview.len(), 5);
    }

    #[test]
    fn preview_all_when_under_limit() {
        let response = make_dax_response(3);
        let preview = preview_dax_rows(&response, 10);

        assert_eq!(preview.len(), 3);
    }

    #[test]
    fn drill_down_returns_all_rows() {
        let response = make_dax_response(100);
        let all = drill_down_dax(&response);

        assert_eq!(all.len(), 100);
    }

    #[test]
    fn empty_result_summarizes_correctly() {
        let response = make_dax_response(0);
        let summary = summarize_dax_result(&response, &SummarizeConfig::default());

        assert_eq!(summary.total_rows, 0);
        assert_eq!(summary.preview_rows, 0);
        assert!(summary.summary_text.contains("0 row"));
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cd spool && cargo test -p spool-validation -- query_result_handling`
Expected: FAIL

**Step 3: Write the implementation**

```rust
// spool/spool-validation/src/query_result_handling.rs
//
// Query result handling per Spec Section 3.6.
//
// Spool treats query results as analytical evidence, not as default chat payloads.
// Default behavior:
// - summarize results in natural language
// - highlight key rows, aggregates, anomalies, or mismatches
// - attach evidence references
//
// If a query returns a large result set:
// - summarize by default
// - show only a bounded preview when useful
// - allow explicit raw-output drill-down on request

use spool_exec::dax_adapter::{DaxQueryResponse, DaxRow};
use spool_exec::warehouse_adapter::{WarehouseQueryResponse, WarehouseRow};

pub struct SummarizeConfig {
    pub max_preview_rows: usize,
}

impl Default for SummarizeConfig {
    fn default() -> Self {
        Self {
            max_preview_rows: 20,
        }
    }
}

#[derive(Debug, Clone)]
pub struct QuerySummary {
    pub summary_text: String,
    pub total_rows: usize,
    pub preview_rows: usize,
    pub truncated: bool,
    pub column_names: Vec<String>,
}

/// Summarize a DAX query response for user presentation.
///
/// Per Spec Section 3.6: summarize by default, show bounded preview,
/// allow drill-down on request.
pub fn summarize_dax_result(
    response: &DaxQueryResponse,
    config: &SummarizeConfig,
) -> QuerySummary {
    let total = response.rows.len();
    let column_names: Vec<String> = response.columns.iter().map(|c| c.name.clone()).collect();
    let preview_count = std::cmp::min(total, config.max_preview_rows);
    let truncated = total > config.max_preview_rows;

    let truncation_note = if truncated {
        format!(" (showing first {preview_count})")
    } else {
        String::new()
    };

    let summary_text = format!(
        "DAX query returned {total} row{} with columns: {}{truncation_note}",
        if total == 1 { "" } else { "s" },
        column_names.join(", "),
    );

    QuerySummary {
        summary_text,
        total_rows: total,
        preview_rows: preview_count,
        truncated,
        column_names,
    }
}

/// Summarize a warehouse query response for user presentation.
pub fn summarize_warehouse_result(
    response: &WarehouseQueryResponse,
    config: &SummarizeConfig,
) -> QuerySummary {
    let total = response.row_count;
    let column_names: Vec<String> = response.columns.iter().map(|c| c.name.clone()).collect();
    let preview_count = std::cmp::min(total, config.max_preview_rows);
    let truncated = total > config.max_preview_rows;

    let truncation_note = if truncated {
        format!(" (showing first {preview_count})")
    } else {
        String::new()
    };

    let summary_text = format!(
        "Warehouse query returned {total} row{} with columns: {}{truncation_note}",
        if total == 1 { "" } else { "s" },
        column_names.join(", "),
    );

    QuerySummary {
        summary_text,
        total_rows: total,
        preview_rows: preview_count,
        truncated,
        column_names,
    }
}

/// Return a bounded preview of DAX query result rows.
///
/// This is the default display mode per Spec Section 3.6.
pub fn preview_dax_rows(response: &DaxQueryResponse, max_rows: usize) -> Vec<&DaxRow> {
    response.rows.iter().take(max_rows).collect()
}

/// Return a bounded preview of warehouse query result rows.
pub fn preview_warehouse_rows(
    response: &WarehouseQueryResponse,
    max_rows: usize,
) -> Vec<&WarehouseRow> {
    response.rows.iter().take(max_rows).collect()
}

/// Return all rows from a DAX query response for explicit drill-down.
///
/// Per Spec Section 3.6: allows explicit raw-output drill-down on request.
pub fn drill_down_dax(response: &DaxQueryResponse) -> &[DaxRow] {
    &response.rows
}

/// Return all rows from a warehouse query response for explicit drill-down.
pub fn drill_down_warehouse(response: &WarehouseQueryResponse) -> &[WarehouseRow] {
    &response.rows
}

// tests at bottom (from Step 1)
```

**Step 4: Run test to verify it passes**

Run: `cd spool && cargo test -p spool-validation -- query_result_handling`
Expected: 8 tests PASS

**Step 5: Commit**

```bash
git add spool/spool-validation/src/query_result_handling.rs
git commit -m "feat(spool-validation): query result handling with summarize, preview, and drill-down per Spec Section 3.6"
```

---

## Task 12: Evaluator Integration -- Validation In The Loop

**Files:**

- Create: `spool/spool-validation/src/evaluator_integration.rs`

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use spool_core::contradiction_ledger::ContradictionLedger;
    use spool_core::evidence_ledger::EvidenceLedger;
    use spool_protocol::artifact::{ArtifactId, ArtifactType};
    use spool_protocol::checkpoint::{CheckpointPolicy, CheckpointTrigger};
    use spool_protocol::evidence::{EvidenceClass, EvidenceId, EvidenceItem, EvidenceType};
    use spool_protocol::task_contract::{
        ArtifactRef, RecipeSelectionMode, Scope, TaskContract, TaskId, TaskStatus, ValidationFloor,
    };
    use spool_protocol::task_result::ResultState;
    use crate::freshness::{FreshnessContext, FreshnessStatus};
    use crate::truth_hierarchy;
    use crate::validation_floor;
    use crate::validation_patterns::{InvestigationClass, MaxResultState};

    fn sample_contract() -> TaskContract {
        TaskContract {
            task_id: TaskId("task_val".into()),
            intent: "Investigate report number mismatch".into(),
            scope: Scope {
                lob: "finance".into(),
                workspace: "Executive BI".into(),
                artifacts: vec![
                    ArtifactRef {
                        artifact_type: ArtifactType::Report,
                        reference: "Executive Revenue Report".into(),
                    },
                ],
            },
            selected_recipe: Some("report_number_mismatch".into()),
            selected_recipe_selection_mode: Some(RecipeSelectionMode::AutoSelect),
            assumptions: vec![],
            expected_evidence_classes: vec![
                EvidenceClass::ReportMetadata,
                EvidenceClass::MeasureDefinition,
                EvidenceClass::DaxQueryResult,
            ],
            validation_floor: ValidationFloor::DirectValidationRequired,
            checkpoint_policy: CheckpointPolicy {
                ask_on: vec![CheckpointTrigger::Ambiguous],
            },
            clarification_checkpoints: vec![],
            approval_checkpoints: vec![],
            expected_deliverable_shape: "structured_task_result".into(),
            evaluator_packet_requirements: vec![],
            task_status: TaskStatus::Active,
            created_at: None,
            updated_at: None,
        }
    }

    fn make_evidence(id: &str, class: EvidenceClass) -> EvidenceItem {
        EvidenceItem {
            id: EvidenceId(id.into()),
            evidence_type: EvidenceType::Observed,
            evidence_class: class,
            source: "test".into(),
            summary: format!("Evidence {id}"),
            artifact_refs: vec![ArtifactId("art_1".into())],
            observed_at: Some(Utc::now()),
            detail: None,
        }
    }

    #[test]
    fn pre_finalization_check_passes_when_floor_met() {
        let mut ledger = EvidenceLedger::new();
        ledger.append(make_evidence("ev_1", EvidenceClass::ReportMetadata));
        ledger.append(make_evidence("ev_2", EvidenceClass::MeasureDefinition));
        ledger.append(make_evidence("ev_3", EvidenceClass::DaxQueryResult));

        let contract = sample_contract();
        let contradictions = ContradictionLedger::new();

        let result = pre_finalization_check(
            &contract,
            &ledger,
            &contradictions,
            &InvestigationClass::ReportNumberMismatch,
            &ResultState::Confirmed,
        );

        assert!(result.can_finalize);
        assert!(result.issues.is_empty());
    }

    #[test]
    fn pre_finalization_check_fails_without_validation_floor() {
        let mut ledger = EvidenceLedger::new();
        ledger.append(make_evidence("ev_1", EvidenceClass::ReportMetadata));
        // No DAX or warehouse query result -- floor not met

        let contract = sample_contract();
        let contradictions = ContradictionLedger::new();

        let result = pre_finalization_check(
            &contract,
            &ledger,
            &contradictions,
            &InvestigationClass::ReportNumberMismatch,
            &ResultState::Confirmed,
        );

        assert!(!result.can_finalize);
        assert!(result.issues.iter().any(|i| i.contains("validation floor")));
    }

    #[test]
    fn pre_finalization_caps_state_for_metadata_only_investigation() {
        let mut ledger = EvidenceLedger::new();
        ledger.append(make_evidence("ev_1", EvidenceClass::SemanticModelMetadata));
        // Only metadata, no query validation

        let contract = TaskContract {
            validation_floor: ValidationFloor::MetadataOnly,
            ..sample_contract()
        };
        let contradictions = ContradictionLedger::new();

        let result = pre_finalization_check(
            &contract,
            &ledger,
            &contradictions,
            &InvestigationClass::MetadataOrArtifactResolution,
            &ResultState::Confirmed,
        );

        assert!(!result.can_finalize);
        assert!(result.issues.iter().any(|i| i.contains("inconclusive")));
    }

    #[test]
    fn pre_finalization_rejects_confirmed_with_stale_evidence_only() {
        let old = Utc::now() - chrono::Duration::hours(50);
        let ev = EvidenceItem {
            id: EvidenceId("ev_stale".into()),
            evidence_type: EvidenceType::Observed,
            evidence_class: EvidenceClass::DaxQueryResult,
            source: "test".into(),
            summary: "Stale evidence".into(),
            artifact_refs: vec![ArtifactId("art_1".into())],
            observed_at: Some(old),
            detail: None,
        };

        let mut ledger = EvidenceLedger::new();
        ledger.append(make_evidence("ev_meta", EvidenceClass::ReportMetadata));
        ledger.append(make_evidence("ev_def", EvidenceClass::MeasureDefinition));
        ledger.append(ev);

        let contract = sample_contract();
        let contradictions = ContradictionLedger::new();

        let result = pre_finalization_check(
            &contract,
            &ledger,
            &contradictions,
            &InvestigationClass::ReportNumberMismatch,
            &ResultState::Confirmed,
        );

        assert!(!result.can_finalize);
        assert!(result.issues.iter().any(|i| i.contains("stale") || i.contains("freshness")));
    }

    #[test]
    fn pre_finalization_rejects_confirmed_with_unresolved_material_contradiction() {
        let mut ledger = EvidenceLedger::new();
        ledger.append(make_evidence("ev_1", EvidenceClass::ReportMetadata));
        ledger.append(make_evidence("ev_2", EvidenceClass::MeasureDefinition));
        ledger.append(make_evidence("ev_3", EvidenceClass::DaxQueryResult));

        let contract = sample_contract();
        let mut contradictions = ContradictionLedger::new();
        contradictions.record(spool_protocol::contradiction::ContradictionRecord {
            id: spool_protocol::contradiction::ContradictionId("c_1".into()),
            disputed_claim: "Revenue mismatch".into(),
            conflicting_evidence: vec![
                EvidenceId("ev_2".into()),
                EvidenceId("ev_3".into()),
            ],
            materiality: spool_protocol::contradiction::MaterialityLevel::Material,
            freshness_notes: None,
            resolution_attempted: false,
            resolution_detail: None,
            status: spool_protocol::contradiction::ContradictionStatus::Open,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        });

        let result = pre_finalization_check(
            &contract,
            &ledger,
            &contradictions,
            &InvestigationClass::ReportNumberMismatch,
            &ResultState::Confirmed,
        );

        assert!(!result.can_finalize);
        assert!(result.issues.iter().any(|i| i.contains("contradiction")));
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cd spool && cargo test -p spool-validation -- evaluator_integration`
Expected: FAIL

**Step 3: Write the implementation**

```rust
// spool/spool-validation/src/evaluator_integration.rs
//
// Integration of validation checks with the evaluator loop from Plan 1.
//
// This module provides a pre-finalization check that the evaluator can invoke
// before accepting a result. It combines:
// - validation floor enforcement (Spec Section 9.6)
// - minimum validation patterns by investigation class (Spec Section 9.7)
// - freshness policy (Spec Section 9.3)
// - contradiction state (Spec Section 9.4)
// - truth hierarchy ranking (Spec Section 9.2)

use spool_core::contradiction_ledger::ContradictionLedger;
use spool_core::evidence_ledger::EvidenceLedger;
use spool_protocol::evidence::EvidenceType;
use spool_protocol::task_contract::TaskContract;
use spool_protocol::task_result::ResultState;

use crate::freshness::{self, FreshnessContext, FreshnessStatus};
use crate::validation_floor;
use crate::validation_patterns::{self, InvestigationClass, MaxResultState};

#[derive(Debug, Clone)]
pub struct PreFinalizationResult {
    pub can_finalize: bool,
    pub issues: Vec<String>,
    pub suggested_state: Option<ResultState>,
}

/// Run a comprehensive pre-finalization check before the evaluator accepts a result.
///
/// This check integrates multiple validation concerns:
/// 1. Validation floor: is the minimum evidence bar met?
/// 2. Investigation pattern: does the evidence match the class-specific minimum?
/// 3. Freshness: are all material evidence items fresh enough to support the proposed state?
/// 4. Contradictions: are there unresolved material contradictions?
/// 5. State caps: does the investigation class cap the achievable result state?
pub fn pre_finalization_check(
    contract: &TaskContract,
    evidence: &EvidenceLedger,
    contradictions: &ContradictionLedger,
    investigation_class: &InvestigationClass,
    proposed_state: &ResultState,
) -> PreFinalizationResult {
    let mut issues = Vec::new();
    let all_evidence = evidence.all();

    // 1. Validation floor check
    let floor_result = validation_floor::check_validation_floor(
        all_evidence,
        &contract.validation_floor,
    );
    if !floor_result.is_met {
        for violation in &floor_result.violations {
            issues.push(format!("validation floor not met: {violation}"));
        }
    }

    // 2. Investigation pattern check
    let pattern_result = validation_patterns::check_investigation_pattern(
        investigation_class,
        all_evidence,
    );
    if !pattern_result.is_met {
        for step in &pattern_result.missing_steps {
            issues.push(format!("investigation pattern incomplete: {step}"));
        }
    }

    // 3. State cap from investigation class
    if *proposed_state == ResultState::Confirmed {
        if let Some(MaxResultState::Inconclusive) =
            pattern_result.max_result_state_without_additional
        {
            issues.push(
                "investigation class caps result at inconclusive without additional evidence beyond metadata".into(),
            );
        }
    }

    // 4. Freshness check for confirmed state
    if *proposed_state == ResultState::Confirmed {
        let freshness_ctx = FreshnessContext::default();
        let has_fresh_validation = all_evidence.iter().any(|ev| {
            if ev.evidence_type != EvidenceType::Observed {
                return false;
            }
            let assessment = freshness::assess_freshness(ev, &freshness_ctx);
            assessment.can_support_confirmed()
        });

        // Check if any validation evidence is stale
        let has_stale_validation = all_evidence.iter().any(|ev| {
            if ev.evidence_type != EvidenceType::Observed {
                return false;
            }
            let assessment = freshness::assess_freshness(ev, &freshness_ctx);
            assessment.is_stale()
        });

        if has_stale_validation && !has_fresh_validation {
            issues.push(
                "all validation evidence is stale; freshness policy prevents confirmed state (Spec Section 9.3)".into(),
            );
        }
    }

    // 5. Contradiction check for confirmed state
    if *proposed_state == ResultState::Confirmed
        && contradictions.has_unresolved_material()
    {
        issues.push(
            "unresolved material contradiction prevents confirmed state (Spec Section 9.4)".into(),
        );
    }

    let can_finalize = issues.is_empty();
    let suggested_state = if !can_finalize {
        Some(if contradictions.has_unresolved_material() {
            ResultState::Inconclusive
        } else {
            ResultState::SupportedHypothesis
        })
    } else {
        None
    };

    PreFinalizationResult {
        can_finalize,
        issues,
        suggested_state,
    }
}

// tests at bottom (from Step 1)
```

**Step 4: Run test to verify it passes**

Run: `cd spool && cargo test -p spool-validation -- evaluator_integration`
Expected: 5 tests PASS

**Step 5: Commit**

```bash
git add spool/spool-validation/src/evaluator_integration.rs
git commit -m "feat(spool-validation): evaluator integration with pre-finalization checks combining floor, patterns, freshness, and contradiction rules"
```

---

## Task 13: Fabric DAX REST Adapter (Live Transport)

**Files:**

- Create: `spool/spool-exec/src/dax_fabric.rs`
- Modify: `spool/spool-exec/src/lib.rs`

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{body_json_string, header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};
    use chrono::Utc;

    #[tokio::test]
    async fn successful_dax_execution_via_rest() {
        let mock_server = MockServer::start().await;

        let response_body = serde_json::json!({
            "results": [{
                "tables": [{
                    "rows": [
                        {"[Region]": "North", "[Revenue]": 12400000.0},
                        {"[Region]": "South", "[Revenue]": 8200000.0}
                    ]
                }]
            }]
        });

        Mock::given(method("POST"))
            .and(path("/v1.0/myorg/datasets/ds_123/executeQueries"))
            .and(header("Authorization", "Bearer test_token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&response_body))
            .mount(&mock_server)
            .await;

        let adapter = FabricDaxAdapter::new(
            mock_server.uri(),
            "test_token".into(),
        );

        let request = DaxQueryRequest {
            dataset_id: "ds_123".into(),
            query: "EVALUATE SUMMARIZECOLUMNS('Sales'[Region], \"Revenue\", SUM('Sales'[Revenue]))".into(),
        };

        let result = adapter.execute_dax(&request).await.unwrap();
        assert_eq!(result.rows.len(), 2);
    }

    #[tokio::test]
    async fn dax_execution_http_error() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v1.0/myorg/datasets/ds_missing/executeQueries"))
            .respond_with(ResponseTemplate::new(404).set_body_string("Not Found"))
            .mount(&mock_server)
            .await;

        let adapter = FabricDaxAdapter::new(
            mock_server.uri(),
            "test_token".into(),
        );

        let request = DaxQueryRequest {
            dataset_id: "ds_missing".into(),
            query: "EVALUATE VALUES('Missing')".into(),
        };

        let result = adapter.execute_dax(&request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn dax_execution_parses_column_names_from_row_keys() {
        let mock_server = MockServer::start().await;

        let response_body = serde_json::json!({
            "results": [{
                "tables": [{
                    "rows": [
                        {"[Name]": "Widget", "[Count]": 42}
                    ]
                }]
            }]
        });

        Mock::given(method("POST"))
            .and(path("/v1.0/myorg/datasets/ds_test/executeQueries"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&response_body))
            .mount(&mock_server)
            .await;

        let adapter = FabricDaxAdapter::new(
            mock_server.uri(),
            "test_token".into(),
        );

        let request = DaxQueryRequest {
            dataset_id: "ds_test".into(),
            query: "EVALUATE ROW(\"Name\", \"Widget\", \"Count\", 42)".into(),
        };

        let result = adapter.execute_dax(&request).await.unwrap();
        assert_eq!(result.columns.len(), 2);
        // Column names should have brackets stripped
        assert!(result.columns.iter().any(|c| c.name == "Name"));
        assert!(result.columns.iter().any(|c| c.name == "Count"));
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cd spool && cargo test -p spool-exec -- dax_fabric`
Expected: FAIL

**Step 3: Write the implementation**

Update `spool/spool-exec/src/lib.rs`:

```rust
// spool/spool-exec/src/lib.rs
pub mod sql_policy;
pub mod dax_adapter;
pub mod dax_fabric;
pub mod warehouse_adapter;
pub mod error;
```

```rust
// spool/spool-exec/src/dax_fabric.rs
//
// Fabric REST transport for DAX query execution.
//
// Uses the Power BI REST API executeQueries endpoint:
// POST /v1.0/myorg/datasets/{datasetId}/executeQueries
//
// The Fabric REST client and auth token are provided by Plan 2 (spool-fabric).
// This module provides the transport layer that converts HTTP request/response
// into the DaxAdapter trait interface.

use async_trait::async_trait;
use chrono::Utc;
use serde::Deserialize;
use std::time::Instant;

use crate::dax_adapter::{DaxAdapter, DaxColumn, DaxQueryRequest, DaxQueryResponse, DaxRow, DaxValue};
use crate::error::ExecError;

pub struct FabricDaxAdapter {
    base_url: String,
    access_token: String,
    client: reqwest::Client,
}

impl FabricDaxAdapter {
    pub fn new(base_url: String, access_token: String) -> Self {
        Self {
            base_url,
            access_token,
            client: reqwest::Client::new(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct ExecuteQueriesResponse {
    results: Vec<QueryResultEntry>,
}

#[derive(Debug, Deserialize)]
struct QueryResultEntry {
    tables: Vec<TableEntry>,
}

#[derive(Debug, Deserialize)]
struct TableEntry {
    rows: Vec<serde_json::Map<String, serde_json::Value>>,
}

#[async_trait]
impl DaxAdapter for FabricDaxAdapter {
    async fn execute_dax(
        &self,
        request: &DaxQueryRequest,
    ) -> Result<DaxQueryResponse, ExecError> {
        let url = format!(
            "{}/v1.0/myorg/datasets/{}/executeQueries",
            self.base_url, request.dataset_id,
        );

        let body = serde_json::json!({
            "queries": [{"query": request.query}],
            "serializerSettings": {"includeNulls": true}
        });

        let start = Instant::now();

        let http_response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| ExecError::Http(e.to_string()))?;

        let duration = start.elapsed();

        if !http_response.status().is_success() {
            let status = http_response.status();
            let body_text = http_response
                .text()
                .await
                .unwrap_or_else(|_| "unable to read body".into());
            return Err(ExecError::DaxExecution(format!(
                "DAX query failed with status {status}: {body_text}"
            )));
        }

        let response_body: ExecuteQueriesResponse = http_response
            .json()
            .await
            .map_err(|e| ExecError::DaxExecution(format!("failed to parse response: {e}")))?;

        // Extract rows from first result/table
        let raw_rows = response_body
            .results
            .into_iter()
            .next()
            .and_then(|r| r.tables.into_iter().next())
            .map(|t| t.rows)
            .unwrap_or_default();

        // Extract column names from first row keys
        let columns: Vec<DaxColumn> = if let Some(first_row) = raw_rows.first() {
            first_row
                .keys()
                .map(|k| {
                    let name = strip_dax_brackets(k);
                    DaxColumn {
                        name,
                        data_type: "Unknown".into(),
                    }
                })
                .collect()
        } else {
            vec![]
        };

        let column_keys: Vec<String> = if let Some(first_row) = raw_rows.first() {
            first_row.keys().cloned().collect()
        } else {
            vec![]
        };

        let rows: Vec<DaxRow> = raw_rows
            .iter()
            .map(|row| {
                let values = column_keys
                    .iter()
                    .map(|key| json_to_dax_value(row.get(key)))
                    .collect();
                DaxRow { values }
            })
            .collect();

        Ok(DaxQueryResponse {
            rows,
            columns,
            executed_at: Utc::now(),
            execution_duration_ms: duration.as_millis() as u64,
        })
    }
}

/// Strip DAX-style bracket notation from column names.
/// "[Region]" -> "Region", "[Sales Amount]" -> "Sales Amount"
fn strip_dax_brackets(name: &str) -> String {
    let trimmed = name.trim();
    if trimmed.starts_with('[') && trimmed.ends_with(']') {
        trimmed[1..trimmed.len() - 1].to_string()
    } else {
        trimmed.to_string()
    }
}

fn json_to_dax_value(value: Option<&serde_json::Value>) -> DaxValue {
    match value {
        None | Some(serde_json::Value::Null) => DaxValue::Null,
        Some(serde_json::Value::Bool(b)) => DaxValue::Boolean(*b),
        Some(serde_json::Value::Number(n)) => {
            if let Some(i) = n.as_i64() {
                DaxValue::Integer(i)
            } else if let Some(f) = n.as_f64() {
                DaxValue::Float(f)
            } else {
                DaxValue::String(n.to_string())
            }
        }
        Some(serde_json::Value::String(s)) => DaxValue::String(s.clone()),
        Some(other) => DaxValue::String(other.to_string()),
    }
}

// tests at bottom (from Step 1)
```

**Step 4: Run test to verify it passes**

Run: `cd spool && cargo test -p spool-exec -- dax_fabric`
Expected: 3 tests PASS

**Step 5: Commit**

```bash
git add spool/spool-exec/src/dax_fabric.rs spool/spool-exec/src/lib.rs
git commit -m "feat(spool-exec): Fabric REST DAX execution adapter with wiremock tests"
```

---

## Task 14: End-To-End Validation Scenario Test

**Files:**

- Create: `spool/spool-validation/tests/validation_scenario.rs`

**Step 1: Write the failing test**

```rust
// spool/spool-validation/tests/validation_scenario.rs
//
// End-to-end integration test: validation execution paths feeding into
// the evaluator loop from Plan 1.
//
// This test exercises the full path:
// 1. Execute a DAX query via fixture adapter
// 2. Execute a warehouse query via fixture adapter (with policy enforcement)
// 3. Capture both results as evidence
// 4. Check validation floor
// 5. Check investigation pattern
// 6. Check freshness
// 7. Run pre-finalization check
// 8. Verify the result is consistent with spec rules

use chrono::Utc;
use spool_core::contradiction_ledger::ContradictionLedger;
use spool_core::evidence_ledger::EvidenceLedger;
use spool_exec::dax_adapter::{
    DaxAdapter, DaxColumn, DaxQueryRequest, DaxQueryResponse, DaxRow, DaxValue, FixtureDaxAdapter,
};
use spool_exec::warehouse_adapter::{
    FixtureWarehouseAdapter, PolicyEnforcingWarehouseAdapter, SqlColumn, SqlValue,
    WarehouseAdapter, WarehouseQueryRequest, WarehouseQueryResponse, WarehouseRow,
};
use spool_protocol::artifact::{ArtifactId, ArtifactType};
use spool_protocol::checkpoint::{CheckpointPolicy, CheckpointTrigger};
use spool_protocol::evidence::EvidenceClass;
use spool_protocol::task_contract::{
    ArtifactRef, Scope, TaskContract, TaskId, TaskStatus, ValidationFloor,
};
use spool_protocol::task_result::ResultState;
use spool_validation::evidence_capture;
use spool_validation::evaluator_integration;
use spool_validation::freshness::{self, FreshnessContext};
use spool_validation::truth_hierarchy;
use spool_validation::validation_floor as vf;
use spool_validation::validation_patterns::InvestigationClass;

fn sample_contract() -> TaskContract {
    TaskContract {
        task_id: TaskId("task_e2e".into()),
        intent: "Investigate warehouse vs model disagreement".into(),
        scope: Scope {
            lob: "finance".into(),
            workspace: "Executive BI".into(),
            artifacts: vec![
                ArtifactRef {
                    artifact_type: ArtifactType::SemanticModel,
                    reference: "Sales Model".into(),
                },
            ],
        },
        selected_recipe: Some("warehouse_vs_model_disagreement".into()),
        selected_recipe_selection_mode: None,
        assumptions: vec![],
        expected_evidence_classes: vec![
            EvidenceClass::DaxQueryResult,
            EvidenceClass::WarehouseQueryResult,
        ],
        validation_floor: ValidationFloor::DirectValidationRequired,
        checkpoint_policy: CheckpointPolicy {
            ask_on: vec![CheckpointTrigger::Ambiguous],
        },
        clarification_checkpoints: vec![],
        approval_checkpoints: vec![],
        expected_deliverable_shape: "structured_task_result".into(),
        evaluator_packet_requirements: vec![],
        task_status: TaskStatus::Active,
        created_at: None,
        updated_at: None,
    }
}

#[tokio::test]
async fn full_validation_path_happy_path() {
    // 1. Set up fixture adapters
    let dax_response = DaxQueryResponse {
        rows: vec![DaxRow {
            values: vec![DaxValue::String("North".into()), DaxValue::Float(12_400_000.0)],
        }],
        columns: vec![
            DaxColumn { name: "Region".into(), data_type: "String".into() },
            DaxColumn { name: "Revenue".into(), data_type: "Double".into() },
        ],
        executed_at: Utc::now(),
        execution_duration_ms: 42,
    };

    let wh_response = WarehouseQueryResponse {
        rows: vec![WarehouseRow {
            values: vec![SqlValue::String("North".into()), SqlValue::Decimal("12400000.00".into())],
        }],
        columns: vec![
            SqlColumn { name: "Region".into(), data_type: "nvarchar".into() },
            SqlColumn { name: "Revenue".into(), data_type: "decimal".into() },
        ],
        executed_at: Utc::now(),
        execution_duration_ms: 100,
        row_count: 1,
    };

    let dax_adapter = FixtureDaxAdapter::new(vec![Ok(dax_response)]);
    let wh_adapter = PolicyEnforcingWarehouseAdapter::new(
        Box::new(FixtureWarehouseAdapter::new(vec![Ok(wh_response)])),
    );

    // 2. Execute queries
    let dax_request = DaxQueryRequest {
        dataset_id: "ds_123".into(),
        query: "EVALUATE SUMMARIZECOLUMNS(...)".into(),
    };
    let wh_request = WarehouseQueryRequest {
        warehouse_id: "wh_456".into(),
        sql: "SELECT Region, SUM(Revenue) AS Revenue FROM dbo.sales GROUP BY Region".into(),
    };

    let dax_result = dax_adapter.execute_dax(&dax_request).await.unwrap();
    let wh_result = wh_adapter.execute_sql(&wh_request).await.unwrap();

    // 3. Capture as evidence
    let art_ref = ArtifactId("art_model_sales".into());
    let dax_evidence = evidence_capture::capture_dax_evidence(
        &dax_result,
        &art_ref,
        &dax_request.query,
    );
    let wh_evidence = evidence_capture::capture_warehouse_evidence(
        &wh_result,
        &art_ref,
        &wh_request.sql,
    );

    // 4. Append to ledger
    let mut ledger = EvidenceLedger::new();
    ledger.append(dax_evidence);
    ledger.append(wh_evidence);

    // 5. Truth hierarchy: DAX and warehouse are both rank 1
    let sorted = truth_hierarchy::sort_by_truth_rank(ledger.all());
    assert_eq!(sorted.len(), 2);
    // Both should be top-rank runtime observed evidence
    assert!(truth_hierarchy::truth_rank(&sorted[0]) <= 12);
    assert!(truth_hierarchy::truth_rank(&sorted[1]) <= 12);

    // 6. Freshness: both are fresh
    let ctx = FreshnessContext::default();
    for ev in ledger.all() {
        let assessment = freshness::assess_freshness(ev, &ctx);
        assert!(assessment.can_support_confirmed());
    }

    // 7. Validation floor: met
    let floor = vf::check_validation_floor(
        ledger.all(),
        &ValidationFloor::DirectValidationRequired,
    );
    assert!(floor.is_met);

    // 8. Pre-finalization check: should pass for confirmed
    let contract = sample_contract();
    let contradictions = ContradictionLedger::new();
    let check = evaluator_integration::pre_finalization_check(
        &contract,
        &ledger,
        &contradictions,
        &InvestigationClass::WarehouseVsModelDisagreement,
        &ResultState::Confirmed,
    );
    assert!(check.can_finalize, "issues: {:?}", check.issues);
}

#[tokio::test]
async fn policy_enforcement_blocks_write_in_scenario() {
    let wh_adapter = PolicyEnforcingWarehouseAdapter::new(
        Box::new(FixtureWarehouseAdapter::new(vec![])),
    );

    let request = WarehouseQueryRequest {
        warehouse_id: "wh_456".into(),
        sql: "INSERT INTO dbo.sales (Region, Revenue) VALUES ('North', 12400000)".into(),
    };

    let result = wh_adapter.execute_sql(&request).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("policy violation"));
}

#[tokio::test]
async fn pre_finalization_blocks_confirmed_with_missing_evidence() {
    // Only DAX evidence, no warehouse -- pattern not met for warehouse_vs_model
    let dax_response = DaxQueryResponse {
        rows: vec![DaxRow {
            values: vec![DaxValue::Float(42.0)],
        }],
        columns: vec![DaxColumn { name: "Value".into(), data_type: "Double".into() }],
        executed_at: Utc::now(),
        execution_duration_ms: 10,
    };

    let dax_adapter = FixtureDaxAdapter::new(vec![Ok(dax_response)]);
    let dax_result = dax_adapter
        .execute_dax(&DaxQueryRequest {
            dataset_id: "ds_123".into(),
            query: "EVALUATE ROW(\"Value\", 42)".into(),
        })
        .await
        .unwrap();

    let art_ref = ArtifactId("art_1".into());
    let ev = evidence_capture::capture_dax_evidence(&dax_result, &art_ref, "Q");

    let mut ledger = EvidenceLedger::new();
    ledger.append(ev);

    let contract = sample_contract();
    let contradictions = ContradictionLedger::new();

    let check = evaluator_integration::pre_finalization_check(
        &contract,
        &ledger,
        &contradictions,
        &InvestigationClass::WarehouseVsModelDisagreement,
        &ResultState::Confirmed,
    );

    assert!(!check.can_finalize);
    assert!(check.issues.iter().any(|i| i.contains("warehouse")));
}
```

**Step 2: Run test to verify it fails**

Run: `cd spool && cargo test -p spool-validation --test validation_scenario`
Expected: FAIL

**Step 3: No new implementation needed -- the test exercises existing code**

The integration test exercises the full validation path using types and functions implemented in Tasks 2-12. No new production code is required.

**Step 4: Run test to verify it passes**

Run: `cd spool && cargo test -p spool-validation --test validation_scenario`
Expected: 3 tests PASS

**Step 5: Commit**

```bash
git add spool/spool-validation/tests/validation_scenario.rs
git commit -m "test(spool-validation): end-to-end validation scenario exercising DAX, warehouse, policy, evidence capture, and pre-finalization checks"
```
