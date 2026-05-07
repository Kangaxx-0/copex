# Spool Validation Execution Paths Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the real validation execution layer for Spool, including DAX and warehouse validation paths, truth-hierarchy and freshness rules, validation-floor enforcement, read-only SQL policy, and canonical comparison records.

**Architecture:** This plan keeps validation rules and comparison semantics in `spool-core`, while `spool-fabric` owns the DAX and warehouse transport seams plus SQL read-only enforcement. The resulting validation records must persist into the Plan 1 canonical evidence and task-result contracts so later TUI and export work can render them without transport-specific knowledge.

**Tech Stack:** Rust 2024, Tokio, Reqwest, Serde, serde_json, thiserror, pretty_assertions, sqlparser

---

## Context For The Implementer

This plan introduces the first real query-backed validation seams for Spool.

It must implement both the transport paths and the policy layer around them:

- truth precedence across evidence sources
- freshness tracking
- validation floor enforcement
- minimum validation patterns by investigation class
- canonical comparison records
- read-only SQL protection

The governing references are:

- `docs/superpowers/specs/2026-04-06-spool-refined-spec.md`
- `docs/superpowers/specs/2026-04-06-spool-dev-planning-readiness-design.md`
- `docs/superpowers/specs/2026-04-07-spool-contradiction-handling-subspec.md`
- `docs/plans/codex/2026-04-06-spool-01-harness-semantics-foundation.md`
- `docs/plans/codex/2026-04-06-spool-02-fabric-adapter-foundations.md`
- `docs/plans/codex/2026-04-06-spool-plan-index.md`

## Out Of Scope

- knowledge bundle authoring
- planner recipe-selection behavior
- TUI raw-output drill-down
- Fabric-side mutation
- export formatting
- durable-memory reuse

## Dependencies

- `docs/plans/codex/2026-04-06-spool-01-harness-semantics-foundation.md`
- `docs/plans/codex/2026-04-06-spool-02-fabric-adapter-foundations.md`

## Contract Impact

This plan implements:

- direct validation evidence records
- truth-hierarchy and freshness policy records
- validation-floor enforcement
- read-only SQL policy enforcement
- normalized DAX and warehouse validation results
- cross-source comparison records

This plan should not weaken the canonical result-state or confidence-cap rules from Plan 1.

## Requirement Traceability

| Requirement source | Required behavior in this plan | Planned coverage |
|---|---|---|
| Refined spec section 2.1 | DAX execution, warehouse validation, and structured evidence capture are in scope for v1 | Tasks 4, 5, and 7 implement query-backed validation and canonical persistence |
| Refined spec sections 2.3 and 3.6 | Query outputs are analytical evidence, summarized and attached as evidence rather than dumped as raw chat output | Tasks 1, 6, and 7 define normalized validation records, comparison records, and evidence persistence rules |
| Refined spec evidence and result semantics | Freshness and evidence strength must constrain confidence and result states | Tasks 1-2 define truth hierarchy, freshness state, validation floors, and minimum pattern enforcement |
| Planning spec section 5 | This plan must include a real integration-validation path against the dev Fabric workspace when the seam supports it | Tasks 4, 5, and 7 each include ignored live integration gates over the shared fixture set |
| Plan index live-validation rule | Live validation must load shared config, exercise the shared fixture set, and assert canonical Spool contracts | Task 7’s integrated live flow is the final gate and must assert on canonical evidence and result policy, not only transport success |

## Execution Invariants

- Validation records are canonical analytical artifacts, not transport wrappers. They must carry source kind, freshness state, comparable values or summaries, and enough metadata for later TUI/export work.
- Truth ranking is claim-class-sensitive. Do not implement one global precedence table and assume it works for every investigation class.
- `confirmed` requires fresh observed evidence. Stale-only, metadata-only, or durable-memory-only support cannot justify the strongest terminal state.
- Warehouse execution is read-only by policy, and that policy is parser-backed. Prefix checks and string heuristics are explicitly out of bounds.
- Comparison records capture analytical agreement state between normalized validation outputs. They are not allowed to depend on transport-specific payload quirks.
- The integrated live flow must end in a `CanonicalTaskResult` that remains evaluator-owned and policy-checked.

## Live Fixture Inputs And Success Conditions

This plan consumes these shared config values from the plan index:

- workspace fixture: `workspace_name` or `workspace_id`
- semantic-model fixture: `semantic_model_name` or `semantic_model_id`
- warehouse fixture: `warehouse_name` and `warehouse_dsn` when warehouse validation is enabled
- token source: environment variable named by `access_token_env`

The live gates are complete only when the plan proves:

- DAX execution succeeds against the configured semantic-model fixture
- warehouse execution is blocked for non-read-only SQL and succeeds for an allowed read-only query when `warehouse_dsn` is configured
- both live outputs normalize into canonical validation records
- comparison logic produces a canonical comparison record
- persistence into `CanonicalTaskResult` preserves observed versus derived evidence classes
- a `confirmed` result remains impossible under stale-only evidence

## Handoff Artifacts For Later Plans

- validation domain types and enforcement helpers consumed by TUI, export, and policy work
- DAX and warehouse transport clients reused by live session flows
- parser-backed SQL policy reused by later operational paths
- an architecture note explaining why validation policy stays in `spool-core` while transport remains in `spool-fabric`

## Integration Validation

Real validation gate:

- against the dev Fabric workspace, execute one real DAX query against a known semantic model
- execute one real read-only warehouse validation query when warehouse access is configured
- normalize both results into canonical validation records
- compare them through `spool-core`
- persist the observed evidence and derived comparison record into the canonical task result
- prove stale-only evidence cannot justify a `confirmed` result

## Open Items / Deferred Decisions

### Owned By This Plan

- exact normalized result shape for DAX and warehouse outputs
- exact SQL parser and deny-list behavior for read-only enforcement
- exact freshness metadata captured on validation outputs
- exact minimum validation-pattern mapping for v1 investigation classes

### Deferred To Later Plans

- recipe-driven validation selection
- TUI raw-output drill-down
- export formatting
- broader aggregation and visual diff rendering

### Review Triggers

- if warehouse access cannot be made reliably read-only with parser-backed enforcement
- if DAX or warehouse outputs need richer grain or filter metadata than the current validation record shape allows
- if the truth hierarchy requires knowledge-layer inputs not available until Plan 4
- if normalized comparison records cannot fit into the Plan 1 evidence and result contracts cleanly

## File Structure

| Path | Responsibility |
|---|---|
| `spool/spool-core/src/validation.rs` | validation rules, truth hierarchy, freshness, floors, comparison logic |
| `spool/spool-core/src/ports.rs` | validation transport traits |
| `spool/spool-core/tests/validation_policy.rs` | truth hierarchy, freshness, and floor tests |
| `spool/spool-core/tests/validation_flow.rs` | comparison and persistence tests |
| `spool/spool-fabric/src/dax.rs` | DAX execution client |
| `spool/spool-fabric/src/warehouse.rs` | warehouse validation client |
| `spool/spool-fabric/src/sql_policy.rs` | parser-backed read-only SQL enforcement |
| `spool/spool-fabric/tests/live_dax_smoke.rs` | ignored live DAX smoke test |
| `spool/spool-fabric/tests/live_warehouse_smoke.rs` | ignored live warehouse smoke test |
| `spool/docs/architecture/validation-layer.md` | architecture note for validation rules and transport seams |

### Task 1: Define Validation Records, Truth Hierarchy, And Freshness Policy

**Files:**
- Create: `spool/spool-core/src/validation.rs`
- Modify: `spool/spool-core/src/lib.rs`
- Create: `spool/spool-core/tests/validation_policy.rs`

- [ ] **Step 1: Write the failing validation-policy tests**

Create `spool/spool-core/tests/validation_policy.rs`:

```rust
use spool_core::{
    compare_truth_rank,
    ClaimClass,
    EvidenceFreshnessState,
    TruthSource,
    ValidationRecord,
};

#[test]
fn current_implementation_prefers_observed_query_evidence_over_authored_business_notes() {
    let left = compare_truth_rank(
        ClaimClass::CurrentImplementation,
        TruthSource::WarehouseQuery,
        TruthSource::BusinessKnowledge,
    );

    assert!(left.is_gt());
}

#[test]
fn validation_record_tracks_stale_state_explicitly() {
    let record = ValidationRecord::scalar(
        "val_1",
        TruthSource::DaxQuery,
        12.4,
        EvidenceFreshnessState::Stale,
    );

    assert_eq!(record.freshness_state, EvidenceFreshnessState::Stale);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:
```bash
cd spool
cargo test -p spool-core validation_policy
```

Expected: FAIL because the validation policy types do not exist.

- [ ] **Step 3: Implement validation records and truth/freshness policy**

Create `spool/spool-core/src/validation.rs` with:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ClaimClass {
    BusinessMeaning,
    CurrentImplementation,
    HistoricalContext,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TruthSource {
    RuntimeObservation,
    DaxQuery,
    WarehouseQuery,
    SemanticModelMetadata,
    ReportMetadata,
    BusinessKnowledge,
    DurableMemory,
    UserAssertion,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EvidenceFreshnessState {
    Fresh,
    Stale,
    Unknown,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ValidationRecord {
    pub validation_id: String,
    pub source: TruthSource,
    pub scalar_value: Option<f64>,
    pub freshness_state: EvidenceFreshnessState,
}

impl ValidationRecord {
    pub fn scalar(
        validation_id: impl Into<String>,
        source: TruthSource,
        scalar_value: f64,
        freshness_state: EvidenceFreshnessState,
    ) -> Self {
        Self {
            validation_id: validation_id.into(),
            source,
            scalar_value: Some(scalar_value),
            freshness_state,
        }
    }
}
```

Also implement:

- `compare_truth_rank(claim_class, left, right) -> std::cmp::Ordering`
- truth precedence rules for the three claim classes

- [ ] **Step 4: Run test to verify it passes**

Run:
```bash
cd spool
cargo test -p spool-core validation_policy
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
cd spool
git add spool-core
git commit -m "feat: add spool validation policy contracts"
```

### Task 2: Add Validation Floor And Minimum Pattern Enforcement

**Files:**
- Modify: `spool/spool-core/src/validation.rs`
- Modify: `spool/spool-core/src/lib.rs`
- Append: `spool/spool-core/tests/validation_policy.rs`

- [ ] **Step 1: Write the failing floor-enforcement test**

Append to `spool/spool-core/tests/validation_policy.rs`:

```rust
use spool_core::{enforce_validation_floor, InvestigationClass};
use spool_model::ValidationFloor;

#[test]
fn direct_validation_required_rejects_metadata_only_evidence() {
    let result = enforce_validation_floor(
        ValidationFloor::DirectValidationRequired,
        InvestigationClass::ReportNumberMismatch,
        &[],
    );

    assert!(result.is_err());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:
```bash
cd spool
cargo test -p spool-core direct_validation_required_rejects_metadata_only_evidence
```

Expected: FAIL because the validation-floor enforcement does not exist.

- [ ] **Step 3: Implement the floor and pattern rules**

Extend `spool/spool-core/src/validation.rs` with:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InvestigationClass {
    ReportNumberMismatch,
    MeasureLogicReview,
    WarehouseDisagreement,
    MetadataInvestigation,
}
```

Implement:

- `enforce_validation_floor(...)`
- minimum validation-pattern checks for the four investigation classes

Rules must include:

- every recommendation requires at least one observed evidence item
- stale-only evidence must not justify `confirmed`
- metadata-only investigation may cap at medium confidence

- [ ] **Step 4: Run test to verify it passes**

Run:
```bash
cd spool
cargo test -p spool-core direct_validation_required_rejects_metadata_only_evidence
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
cd spool
git add spool-core
git commit -m "feat: add spool validation floor enforcement"
```

### Task 3: Add The Parser-Backed Read-Only SQL Policy

**Files:**
- Create: `spool/spool-fabric/src/sql_policy.rs`
- Modify: `spool/spool-fabric/src/lib.rs`
- Create: `spool/spool-fabric/tests/sql_policy.rs`

- [ ] **Step 1: Write the failing SQL-policy tests**

Create `spool/spool-fabric/tests/sql_policy.rs`:

```rust
use spool_fabric::assert_read_only_sql;

#[test]
fn select_query_is_allowed() {
    assert!(assert_read_only_sql("SELECT 1 AS ok").is_ok());
}

#[test]
fn delete_query_is_rejected() {
    assert!(assert_read_only_sql("DELETE FROM fact_sales").is_err());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:
```bash
cd spool
cargo test -p spool-fabric sql_policy
```

Expected: FAIL because the SQL policy helper does not exist.

- [ ] **Step 3: Implement parser-backed SQL enforcement**

Create `spool/spool-fabric/src/sql_policy.rs`:

```rust
use sqlparser::dialect::MsSqlDialect;
use sqlparser::parser::Parser;

pub fn assert_read_only_sql(query: &str) -> Result<(), String> {
    let dialect = MsSqlDialect {};
    let statements = Parser::parse_sql(&dialect, query).map_err(|err| err.to_string())?;

    for statement in statements {
        let is_select = matches!(statement, sqlparser::ast::Statement::Query(_));
        if !is_select {
            return Err("only read-only SELECT-style statements are allowed".into());
        }
    }

    Ok(())
}
```

- [ ] **Step 4: Run test to verify it passes**

Run:
```bash
cd spool
cargo test -p spool-fabric sql_policy
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
cd spool
git add spool-fabric
git commit -m "feat: add spool read-only sql policy"
```

### Task 4: Implement The DAX Validation Client

**Files:**
- Create: `spool/spool-fabric/src/dax.rs`
- Modify: `spool/spool-fabric/src/lib.rs`
- Create: `spool/spool-fabric/tests/live_dax_smoke.rs`

- [ ] **Step 1: Write the failing live DAX test**

Create `spool/spool-fabric/tests/live_dax_smoke.rs`:

```rust
use spool_fabric::{DaxClient, FabricAuthSession, FabricConfig, FabricHttpClient, TokenSourceKind};

#[tokio::test]
#[ignore = "requires dev Fabric dataset and access token"]
async fn executes_dax_query_against_dev_model() {
    let config_path = std::env::var("SPOOL_CONFIG_PATH").unwrap_or_else(|_| {
        format!(
            "{}/.config/spool/dev.toml",
            std::env::var("HOME").unwrap()
        )
    });
    let config = FabricConfig::load_from_path(std::path::Path::new(&config_path)).unwrap();
    let token_env = config
        .access_token_env
        .clone()
        .unwrap_or_else(|| "SPOOL_FABRIC_ACCESS_TOKEN".into());
    let session = FabricAuthSession {
        token_source: TokenSourceKind::Env,
        access_token: std::env::var(token_env).unwrap(),
    };
    let http = FabricHttpClient::new(session);
    let dax = DaxClient::new(http);

    let result = dax
        .execute_query_from_config("EVALUATE ROW(\"Revenue\", [Revenue])")
        .await
        .unwrap();

    assert!(!result.raw_json.is_empty());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:
```bash
cd spool
cargo test -p spool-fabric executes_dax_query_against_dev_model -- --ignored
```

Expected: FAIL because the DAX client does not exist.

- [ ] **Step 3: Implement the DAX client**

Create `spool/spool-fabric/src/dax.rs`:

```rust
#[derive(Clone, Debug)]
pub struct DaxResponse {
    pub raw_json: String,
}

pub struct DaxClient {
    http: crate::FabricHttpClient,
}
```

Implement:

- `DaxClient::new(http)`
- `execute_query(dataset_id, query) -> Result<DaxResponse, reqwest::Error>`

Use the shared config contract from Plan 2 to resolve the configured semantic model fixture, call the Power BI executeQueries endpoint, and return normalized raw JSON for later canonical projection in `spool-core`.

- [ ] **Step 4: Run the live DAX smoke test**

Run:
```bash
cd spool
SPOOL_CONFIG_PATH=~/.config/spool/dev.toml \
cargo test -p spool-fabric executes_dax_query_against_dev_model -- --ignored --nocapture
```

Expected: PASS using the configured dev semantic model from the shared live fixture set.

- [ ] **Step 5: Commit**

```bash
cd spool
git add spool-fabric
git commit -m "feat: add spool dax validation client"
```

### Task 5: Implement The Warehouse Validation Client

**Files:**
- Create: `spool/spool-fabric/src/warehouse.rs`
- Modify: `spool/spool-fabric/src/lib.rs`
- Create: `spool/spool-fabric/tests/live_warehouse_smoke.rs`

- [ ] **Step 1: Write the failing live warehouse test**

Create `spool/spool-fabric/tests/live_warehouse_smoke.rs`:

```rust
use spool_fabric::{FabricAuthSession, TokenSourceKind, WarehouseClient};

#[tokio::test]
#[ignore = "requires dev warehouse connection string and token"]
async fn executes_read_only_warehouse_query() {
    let config_path = std::env::var("SPOOL_CONFIG_PATH").unwrap_or_else(|_| {
        format!(
            "{}/.config/spool/dev.toml",
            std::env::var("HOME").unwrap()
        )
    });
    let config = spool_fabric::FabricConfig::load_from_path(std::path::Path::new(&config_path)).unwrap();
    let token_env = config
        .access_token_env
        .clone()
        .unwrap_or_else(|| "SPOOL_FABRIC_ACCESS_TOKEN".into());
    let session = FabricAuthSession {
        token_source: TokenSourceKind::Env,
        access_token: std::env::var(token_env).unwrap(),
    };
    let client = WarehouseClient::from_config_path(
        std::path::Path::new(&config_path),
        session,
    )
    .unwrap();

    let result = client.execute_read_only("SELECT 1 AS ok").await.unwrap();

    assert!(result.raw_rows_preview.contains("ok"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:
```bash
cd spool
cargo test -p spool-fabric executes_read_only_warehouse_query -- --ignored
```

Expected: FAIL because the warehouse client does not exist.

- [ ] **Step 3: Implement the warehouse client**

Create `spool/spool-fabric/src/warehouse.rs`:

```rust
#[derive(Clone, Debug)]
pub struct WarehouseResponse {
    pub raw_rows_preview: String,
}

pub struct WarehouseClient {
    dsn: String,
    session: crate::FabricAuthSession,
}
```

Implement:

- `WarehouseClient::new(dsn, session)`
- `WarehouseClient::from_config_path(path, session)`
- `execute_read_only(query) -> Result<WarehouseResponse, String>`

Always call `assert_read_only_sql(query)` before transport execution.

- [ ] **Step 4: Run the live warehouse smoke test**

Run:
```bash
cd spool
SPOOL_CONFIG_PATH=~/.config/spool/dev.toml \
cargo test -p spool-fabric executes_read_only_warehouse_query -- --ignored --nocapture
```

Expected: PASS using the configured dev warehouse connection from the shared config contract.

- [ ] **Step 5: Commit**

```bash
cd spool
git add spool-fabric
git commit -m "feat: add spool warehouse validation client"
```

### Task 6: Implement Canonical Comparison Records

**Files:**
- Modify: `spool/spool-core/src/validation.rs`
- Modify: `spool/spool-core/src/lib.rs`
- Create: `spool/spool-core/tests/validation_flow.rs`

- [ ] **Step 1: Write the failing comparison-flow tests**

Create `spool/spool-core/tests/validation_flow.rs`:

```rust
use spool_core::{compare_validation_records, ComparisonOutcome, EvidenceFreshnessState, TruthSource, ValidationRecord};

#[test]
fn comparison_marks_match_when_scalar_values_align() {
    let dax = ValidationRecord::scalar("dax", TruthSource::DaxQuery, 12.4, EvidenceFreshnessState::Fresh);
    let warehouse = ValidationRecord::scalar("wh", TruthSource::WarehouseQuery, 12.4, EvidenceFreshnessState::Fresh);

    let comparison = compare_validation_records(&dax, &warehouse);

    assert_eq!(comparison.outcome, ComparisonOutcome::Match);
}

#[test]
fn comparison_marks_mismatch_when_scalar_values_differ() {
    let dax = ValidationRecord::scalar("dax", TruthSource::DaxQuery, 12.4, EvidenceFreshnessState::Fresh);
    let warehouse = ValidationRecord::scalar("wh", TruthSource::WarehouseQuery, 11.8, EvidenceFreshnessState::Fresh);

    let comparison = compare_validation_records(&dax, &warehouse);

    assert_eq!(comparison.outcome, ComparisonOutcome::Mismatch);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:
```bash
cd spool
cargo test -p spool-core validation_flow
```

Expected: FAIL because the comparison types do not exist.

- [ ] **Step 3: Implement comparison records**

Extend `spool/spool-core/src/validation.rs` with:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ComparisonOutcome {
    Match,
    Mismatch,
    Inconclusive,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ComparisonRecord {
    pub outcome: ComparisonOutcome,
    pub left_id: String,
    pub right_id: String,
}
```

Implement `compare_validation_records(left, right) -> ComparisonRecord`.

- [ ] **Step 4: Run test to verify it passes**

Run:
```bash
cd spool
cargo test -p spool-core validation_flow
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
cd spool
git add spool-core
git commit -m "feat: add spool validation comparison records"
```

### Task 7: Persist Validation Evidence Into Canonical Results

**Files:**
- Modify: `spool/spool-core/src/validation.rs`
- Modify: `spool/spool-core/src/lib.rs`
- Append: `spool/spool-core/tests/validation_flow.rs`

- [ ] **Step 1: Write the failing persistence test**

Append to `spool/spool-core/tests/validation_flow.rs`:

```rust
use spool_core::persist_validation_evidence;
use spool_model::{
    CanonicalTaskResult,
    Confidence,
    EvidenceLedger,
    FinalAuthority,
    Finding,
    RecommendedAction,
    ResultState,
    ValidationResult,
};

#[test]
fn validation_persistence_adds_observed_and_derived_records() {
    let result = CanonicalTaskResult {
        task_id: "task_123".into(),
        state: ResultState::SupportedHypothesis,
        confidence: Confidence::Medium,
        summary: "Mismatch under investigation".into(),
        findings: vec![Finding::new("finding_1", "Mismatch remains under investigation", "Validation evidence still needs to be attached")],
        evidence_refs: vec![],
        validation_results: vec![ValidationResult::passed("val_1", "comparison_stub", "Pre-existing validation record")],
        recommended_actions: vec![RecommendedAction::new("action_1", "continue_validation", "Persist validation evidence into the canonical result")],
        blockers: vec![],
        open_questions: vec![],
        proposed_changes: vec![],
        evidence: EvidenceLedger { items: vec![] },
        contradiction_refs: vec![],
        proposed_state: None,
        final_authority: FinalAuthority::Evaluator,
    };

    let validation = ValidationRecord::scalar("dax", TruthSource::DaxQuery, 12.4, EvidenceFreshnessState::Fresh);
    let comparison = ComparisonRecord {
        outcome: ComparisonOutcome::Match,
        left_id: "dax".into(),
        right_id: "wh".into(),
    };

    let updated = persist_validation_evidence(result, &[validation], Some(comparison));

    assert_eq!(updated.evidence.items.len(), 2);
}

#[tokio::test]
#[ignore = "requires shared dev Fabric fixtures from the plan index"]
async fn live_validation_flow_normalizes_and_persists_real_validation_outputs() {
    let result = spool_core::run_live_validation_flow_from_config().await.unwrap();

    assert!(!result.evidence.items.is_empty());
    assert!(!result.validation_results.is_empty());
    assert!(
        !matches!(result.state, ResultState::Confirmed)
            || result
                .evidence
                .items
                .iter()
                .any(|item| matches!(item.kind, spool_model::EvidenceKind::Observed))
    );
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:
```bash
cd spool
cargo test -p spool-core validation_persistence_adds_observed_and_derived_records
```

Expected: FAIL because the persistence helper does not exist.

- [ ] **Step 3: Implement canonical persistence for validation records**

Add `persist_validation_evidence(result, validations, comparison)` and `run_live_validation_flow_from_config()` to `spool-core/src/validation.rs`.

Rules:

- direct DAX and warehouse outputs become observed evidence
- comparison becomes derived evidence
- the resulting canonical task result remains evaluator-owned
- the ignored live flow must load the shared config contract, run the configured DAX query, optionally run the configured warehouse query when warehouse validation is enabled, and persist the canonical evidence before asserting policy constraints

- [ ] **Step 4: Run test to verify it passes**

Run:
```bash
cd spool
cargo test -p spool-core validation_persistence_adds_observed_and_derived_records
```

Expected: PASS.

- [ ] **Step 5: Run the integrated live validation path**

Run:
```bash
cd spool
SPOOL_CONFIG_PATH=~/.config/spool/dev.toml \
cargo test -p spool-core validation_flow -- --ignored --nocapture
```

Expected: PASS, with one ignored integration test that loads the shared config, executes the configured DAX query, optionally executes the configured warehouse query when `warehouse_dsn` is present, normalizes both results into canonical validation records, persists observed and derived evidence into a `CanonicalTaskResult`, and proves stale-only evidence does not justify `confirmed`.

- [ ] **Step 6: Commit**

```bash
cd spool
git add spool-core
git commit -m "feat: persist spool validation evidence into results"
```

### Task 8: Write The Validation-Layer Architecture Note

**Files:**
- Create: `spool/docs/architecture/validation-layer.md`

- [ ] **Step 1: Write the architecture note**

Create `spool/docs/architecture/validation-layer.md` describing:

- why validation policy lives in `spool-core`
- why transport clients live in `spool-fabric`
- how truth hierarchy varies by claim class
- why stale-only evidence cannot justify `confirmed`
- why SQL enforcement is parser-backed rather than string-prefix-based

- [ ] **Step 2: Review the note for missing concepts**

Run:
```bash
cd spool
rg -n "truth hierarchy|claim class|stale|confirmed|parser" docs/architecture/validation-layer.md
```

Expected: the note explicitly mentions all five concepts.

- [ ] **Step 3: Commit**

```bash
cd spool
git add docs/architecture/validation-layer.md
git commit -m "docs: add spool validation layer architecture note"
```

## Self-Review Checklist

- Spec coverage: This plan covers DAX and warehouse validation execution, truth hierarchy, freshness, validation floors, minimum validation patterns, parser-backed SQL policy enforcement, comparison records, and canonical persistence.
- Placeholder scan: No step may reduce SQL policy to `starts_with("SELECT")`, reduce comparison to transport-specific strings, or ignore stale-evidence caps.
- Type consistency: Keep `ClaimClass`, `TruthSource`, `EvidenceFreshnessState`, `ValidationRecord`, `ValidationFloor`, `InvestigationClass`, `ComparisonOutcome`, `ComparisonRecord`, `DaxResponse`, and `WarehouseResponse` stable because Plans 5 and 6 consume them directly.
