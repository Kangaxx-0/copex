# Plan 1: Harness Semantics Foundation

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Prove Spool's core task/evidence/result/resume semantics in complete isolation from live Fabric systems.

**Architecture:** Two new crates — `spool-protocol` (shared domain types) and `spool-core` (harness orchestration) — living in a `spool/` workspace at the repo root. All external dependencies are behind traits with fixture implementations. The harness follows a planner → generator → evaluator loop with bounded iterations, append-only evidence, first-class contradictions, and evaluator-owned result finalization.

**Tech Stack:** Rust 2024 edition, serde/serde_json, chrono, uuid, tokio, async-trait, thiserror, tempfile (dev)

**Governing spec:** `docs/superpowers/specs/2026-04-06-spool-refined-spec.md`
**Planning readiness:** `docs/superpowers/specs/2026-04-06-spool-dev-planning-readiness-design.md`

---

## Plan-Specific Sections

### Subsystem Scope

This plan owns:

- canonical artifact identity model
- task contract schema and lifecycle
- evidence ledger (append-only)
- contradiction ledger (detect, record, resolve)
- evaluator loop protocol with bounded iterations
- generator obligation enforcement
- checkpoint policy types and enforcement
- canonical task result schema with confidence caps
- result-state authority rules (evaluator assigns final)
- persistence trait and JSONL implementation for structured state
- session state serialization for resume and compaction

### Out Of Scope

- live Fabric auth, artifact resolution, or API calls (Plan 2)
- knowledge bundle loading, Tier 1/Tier 2 structure (Plan 3)
- DAX or warehouse query execution (Plan 4)
- TUI rendering, progress surface, advanced view (Plan 5)
- durable memory, exports, telemetry (Plan 6)
- LLM provider integration (the harness traits abstract this away)

### Dependencies

- None. This is Plan 1. No prior plans or live runtime prerequisites.

### Contract Impact

This plan **implements** the following governing contracts from the refined spec:

- Artifact identity model (Spec Section 3.3-3.4)
- Task contract schema (Spec Section 5.2-5.3)
- Evaluator loop protocol (Spec Section 4.3-4.6)
- Evidence ledger semantics (Spec Section 9.1)
- Contradiction handling (Spec Section 9.4)
- Result state semantics (Spec Section 10.2-10.5)
- Confidence calibration and caps (Spec Section 10.6-10.7)
- Canonical task result schema (Spec Section 10.8)
- Checkpoint policy (Spec Section 6.2-6.4)
- Persisted state model (Spec Section 12.2)

### Validation

Plan 1 is proven through:

- contract tests: all protocol types serialize/deserialize round-trip
- state-machine tests: task lifecycle transitions, evaluator loop paths
- deterministic fixture scenarios: happy path, evidence request, exhaustion, contradiction, blocked, authority disagreement

No live systems. No network. All fixture-backed.

### Open Items

**Owned by this plan:**

- exact error type taxonomy for spool-core (resolved during implementation)
- exact field naming for JSON serialization (resolved by serde rename_all convention)

**Deferred to later plans:**

- exact persistence file location and naming (Plan 5: TUI/session)
- SQLite indexing layer (Plan 5 or Plan 6)
- knowledge bundle types beyond stubs (Plan 3)
- platform capability contract beyond stubs (Plan 2)

**Review triggers:**

- if evaluator loop semantics prove insufficient during Plan 4 (validation execution), revisit loop protocol
- if persistence format proves inadequate for resume during Plan 5, revisit structured state schema

---

## Task 1: Workspace Scaffolding

**Files:**

- Create: `spool/Cargo.toml`
- Create: `spool/spool-protocol/Cargo.toml`
- Create: `spool/spool-protocol/src/lib.rs`
- Create: `spool/spool-core/Cargo.toml`
- Create: `spool/spool-core/src/lib.rs`

**Step 1: Create workspace Cargo.toml**

```toml
# spool/Cargo.toml
[workspace]
members = [
    "spool-protocol",
    "spool-core",
]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2024"
license = "Apache-2.0"

[workspace.dependencies]
spool-protocol = { path = "spool-protocol" }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v4", "serde"] }
tokio = { version = "1", features = ["full"] }
async-trait = "0.1"
thiserror = "2"
```

**Step 2: Create spool-protocol crate**

```toml
# spool/spool-protocol/Cargo.toml
[package]
name = "spool-protocol"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
serde = { workspace = true }
serde_json = { workspace = true }
chrono = { workspace = true }
uuid = { workspace = true }
```

```rust
// spool/spool-protocol/src/lib.rs
pub mod artifact;
pub mod evidence;
pub mod contradiction;
pub mod evaluator;
pub mod checkpoint;
pub mod task_contract;
pub mod task_result;
```

**Step 3: Create spool-core crate**

```toml
# spool/spool-core/Cargo.toml
[package]
name = "spool-core"
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

[dev-dependencies]
tempfile = "3"
tokio = { workspace = true, features = ["test-util", "macros"] }
```

```rust
// spool/spool-core/src/lib.rs
pub mod error;
pub mod evidence_ledger;
pub mod contradiction_ledger;
pub mod harness;
pub mod evaluator_loop;
pub mod task_lifecycle;
pub mod persistence;
```

**Step 4: Create placeholder modules**

Create empty files for each module declared in both lib.rs files. Each file should contain only a comment:

```rust
// placeholder — implemented in later tasks
```

Also create the error module:

```rust
// spool/spool-core/src/error.rs
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SpoolError {
    #[error("planner error: {0}")]
    Planner(String),

    #[error("generator error: {0}")]
    Generator(String),

    #[error("evaluator error: {0}")]
    Evaluator(String),

    #[error("persistence error: {0}")]
    Persistence(String),

    #[error("invalid state transition from {from} to {to}")]
    InvalidStateTransition { from: String, to: String },

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}
```

**Step 5: Verify build**

Run: `cd spool && cargo check`
Expected: compiles with no errors

**Step 6: Commit**

```bash
git add spool/
git commit -m "feat(spool): scaffold workspace with spool-protocol and spool-core crates"
```

---

## Task 2: Artifact Identity Model

**Files:**

- Create: `spool/spool-protocol/src/artifact.rs`

**Step 1: Write the failing test**

Add to the bottom of `spool/spool-protocol/src/artifact.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn artifact_identity_round_trip() {
        let identity = ArtifactIdentity {
            artifact_id: ArtifactId("art_report_exec_rev".into()),
            artifact_type: ArtifactType::Report,
            workspace_id: Some("ws_123".into()),
            parent_artifact_id: None,
            canonical_locator: CanonicalLocator("fabric://workspace/ws_123/report/rpt_456".into()),
            display_name: "Executive Revenue Report".into(),
            resolution_basis: ResolutionBasis::ReportUrl,
        };

        let json = serde_json::to_string(&identity).unwrap();
        let restored: ArtifactIdentity = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.artifact_id, identity.artifact_id);
        assert_eq!(restored.artifact_type, identity.artifact_type);
        assert_eq!(restored.workspace_id, identity.workspace_id);
        assert_eq!(restored.parent_artifact_id, identity.parent_artifact_id);
        assert_eq!(restored.canonical_locator, identity.canonical_locator);
        assert_eq!(restored.display_name, identity.display_name);
        assert_eq!(restored.resolution_basis, identity.resolution_basis);
    }

    #[test]
    fn child_artifact_with_parent() {
        let identity = ArtifactIdentity {
            artifact_id: ArtifactId("art_measure_qoq".into()),
            artifact_type: ArtifactType::Measure,
            workspace_id: Some("ws_123".into()),
            parent_artifact_id: Some(ArtifactId("art_model_sales".into())),
            canonical_locator: CanonicalLocator(
                "fabric://workspace/ws_123/model/mod_789/measure/Sales[QoQ Revenue]".into(),
            ),
            display_name: "QoQ Revenue".into(),
            resolution_basis: ResolutionBasis::ExactApiMatch,
        };

        let json = serde_json::to_string_pretty(&identity).unwrap();
        assert!(json.contains("art_model_sales"));
        assert!(json.contains("measure"));
    }

    #[test]
    fn all_artifact_types_serialize() {
        let types = vec![
            ArtifactType::Report,
            ArtifactType::Page,
            ArtifactType::Visual,
            ArtifactType::SemanticModel,
            ArtifactType::Measure,
            ArtifactType::Table,
            ArtifactType::Column,
            ArtifactType::Relationship,
            ArtifactType::Warehouse,
            ArtifactType::QueryResult,
        ];

        for t in types {
            let json = serde_json::to_string(&t).unwrap();
            let restored: ArtifactType = serde_json::from_str(&json).unwrap();
            assert_eq!(restored, t);
        }
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cd spool && cargo test -p spool-protocol -- artifact`
Expected: FAIL — types not defined yet

**Step 3: Write the implementation**

Replace the placeholder content of `spool/spool-protocol/src/artifact.rs` with:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ArtifactId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactType {
    Report,
    Page,
    Visual,
    SemanticModel,
    Measure,
    Table,
    Column,
    Relationship,
    Warehouse,
    QueryResult,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResolutionBasis {
    ExplicitGuid,
    ReportUrl,
    ExactApiMatch,
    UniqueNameMatch,
    DerivedFromResolvedParent,
    RuntimeExecution,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CanonicalLocator(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactIdentity {
    pub artifact_id: ArtifactId,
    pub artifact_type: ArtifactType,
    pub workspace_id: Option<String>,
    pub parent_artifact_id: Option<ArtifactId>,
    pub canonical_locator: CanonicalLocator,
    pub display_name: String,
    pub resolution_basis: ResolutionBasis,
}

// tests at bottom of file (from Step 1)
```

**Step 4: Run test to verify it passes**

Run: `cd spool && cargo test -p spool-protocol -- artifact`
Expected: 3 tests PASS

**Step 5: Commit**

```bash
git add spool/spool-protocol/src/artifact.rs
git commit -m "feat(spool-protocol): artifact identity model with all v1 artifact types"
```

---

## Task 3: Evidence Types

**Files:**

- Create: `spool/spool-protocol/src/evidence.rs`

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::artifact::ArtifactId;
    use chrono::Utc;

    #[test]
    fn evidence_item_round_trip() {
        let item = EvidenceItem {
            id: EvidenceId("ev_12".into()),
            evidence_type: EvidenceType::Observed,
            evidence_class: EvidenceClass::DaxQueryResult,
            source: "dax_query_result".into(),
            summary: "Diagnostic DAX query returned 12.4M for Q1 revenue".into(),
            artifact_refs: vec![ArtifactId("art_measure_qoq".into())],
            observed_at: Some(Utc::now()),
            detail: Some(serde_json::json!({"value": 12_400_000})),
        };

        let json = serde_json::to_string(&item).unwrap();
        let restored: EvidenceItem = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.id, item.id);
        assert_eq!(restored.evidence_type, item.evidence_type);
        assert_eq!(restored.evidence_class, item.evidence_class);
    }

    #[test]
    fn all_evidence_types_serialize() {
        let types = vec![
            EvidenceType::Observed,
            EvidenceType::Derived,
            EvidenceType::Proposed,
        ];
        for t in types {
            let json = serde_json::to_string(&t).unwrap();
            let restored: EvidenceType = serde_json::from_str(&json).unwrap();
            assert_eq!(restored, t);
        }
    }

    #[test]
    fn all_evidence_classes_serialize() {
        let classes = vec![
            EvidenceClass::ReportMetadata,
            EvidenceClass::VisualMetadata,
            EvidenceClass::SemanticModelMetadata,
            EvidenceClass::MeasureDefinition,
            EvidenceClass::DaxQueryResult,
            EvidenceClass::WarehouseQueryResult,
            EvidenceClass::CrossSourceComparison,
        ];
        for c in classes {
            let json = serde_json::to_string(&c).unwrap();
            let restored: EvidenceClass = serde_json::from_str(&json).unwrap();
            assert_eq!(restored, c);
        }
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cd spool && cargo test -p spool-protocol -- evidence`
Expected: FAIL

**Step 3: Write the implementation**

```rust
// spool/spool-protocol/src/evidence.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::artifact::ArtifactId;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EvidenceId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceType {
    Observed,
    Derived,
    Proposed,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceClass {
    ReportMetadata,
    VisualMetadata,
    SemanticModelMetadata,
    MeasureDefinition,
    DaxQueryResult,
    WarehouseQueryResult,
    CrossSourceComparison,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceItem {
    pub id: EvidenceId,
    pub evidence_type: EvidenceType,
    pub evidence_class: EvidenceClass,
    pub source: String,
    pub summary: String,
    pub artifact_refs: Vec<ArtifactId>,
    pub observed_at: Option<DateTime<Utc>>,
    pub detail: Option<serde_json::Value>,
}

// tests at bottom (from Step 1)
```

**Step 4: Run test to verify it passes**

Run: `cd spool && cargo test -p spool-protocol -- evidence`
Expected: 3 tests PASS

**Step 5: Commit**

```bash
git add spool/spool-protocol/src/evidence.rs
git commit -m "feat(spool-protocol): evidence types with observed/derived/proposed and all v1 evidence classes"
```

---

## Task 4: Contradiction Types

**Files:**

- Create: `spool/spool-protocol/src/contradiction.rs`

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::evidence::EvidenceId;
    use chrono::Utc;

    #[test]
    fn contradiction_record_round_trip() {
        let now = Utc::now();
        let record = ContradictionRecord {
            id: ContradictionId("contra_1".into()),
            disputed_claim: "Q1 revenue does not match across sources".into(),
            conflicting_evidence: vec![
                EvidenceId("ev_12".into()),
                EvidenceId("ev_19".into()),
            ],
            materiality: MaterialityLevel::Material,
            freshness_notes: Some("Both observations from same session".into()),
            resolution_attempted: false,
            resolution_detail: None,
            status: ContradictionStatus::Open,
            created_at: now,
            updated_at: now,
        };

        let json = serde_json::to_string(&record).unwrap();
        let restored: ContradictionRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.id, record.id);
        assert_eq!(restored.status, ContradictionStatus::Open);
        assert_eq!(restored.materiality, MaterialityLevel::Material);
    }

    #[test]
    fn all_statuses_serialize() {
        let statuses = vec![
            ContradictionStatus::Open,
            ContradictionStatus::Resolved,
            ContradictionStatus::CarriedForward,
        ];
        for s in statuses {
            let json = serde_json::to_string(&s).unwrap();
            let restored: ContradictionStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(restored, s);
        }
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cd spool && cargo test -p spool-protocol -- contradiction`
Expected: FAIL

**Step 3: Write the implementation**

```rust
// spool/spool-protocol/src/contradiction.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::evidence::EvidenceId;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContradictionId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContradictionStatus {
    Open,
    Resolved,
    CarriedForward,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaterialityLevel {
    Material,
    NonMaterial,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContradictionRecord {
    pub id: ContradictionId,
    pub disputed_claim: String,
    pub conflicting_evidence: Vec<EvidenceId>,
    pub materiality: MaterialityLevel,
    pub freshness_notes: Option<String>,
    pub resolution_attempted: bool,
    pub resolution_detail: Option<String>,
    pub status: ContradictionStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// tests at bottom (from Step 1)
```

**Step 4: Run test to verify it passes**

Run: `cd spool && cargo test -p spool-protocol -- contradiction`
Expected: 2 tests PASS

**Step 5: Commit**

```bash
git add spool/spool-protocol/src/contradiction.rs
git commit -m "feat(spool-protocol): contradiction types with materiality and lifecycle status"
```

---

## Task 5: Evaluator And Checkpoint Types

**Files:**

- Create: `spool/spool-protocol/src/evaluator.rs`
- Create: `spool/spool-protocol/src/checkpoint.rs`

**Step 1: Write the failing tests**

In `evaluator.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evaluator_outcome_accept_round_trip() {
        let outcome = EvaluatorOutcome::Accept;
        let json = serde_json::to_string(&outcome).unwrap();
        let restored: EvaluatorOutcome = serde_json::from_str(&json).unwrap();
        assert!(matches!(restored, EvaluatorOutcome::Accept));
    }

    #[test]
    fn evaluator_outcome_request_more_evidence_round_trip() {
        let outcome = EvaluatorOutcome::RequestMoreEvidence {
            targets: vec![EvidenceTarget {
                description: "Run a DAX query scoped to the disputed measure".into(),
                target_artifact: Some("art_measure_qoq".into()),
                target_evidence_class: Some("dax_query_result".into()),
            }],
        };
        let json = serde_json::to_string(&outcome).unwrap();
        assert!(json.contains("request_more_evidence"));
        let restored: EvaluatorOutcome = serde_json::from_str(&json).unwrap();
        match restored {
            EvaluatorOutcome::RequestMoreEvidence { targets } => {
                assert_eq!(targets.len(), 1);
                assert!(targets[0].description.contains("DAX"));
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn all_inability_reasons_serialize() {
        let reasons = vec![
            InabilityReason::MissingAccess,
            InabilityReason::MissingUserClarification,
            InabilityReason::UnavailableArtifact,
            InabilityReason::OutOfScopeRequest,
            InabilityReason::PolicyBoundary,
        ];
        for r in reasons {
            let json = serde_json::to_string(&r).unwrap();
            let restored: InabilityReason = serde_json::from_str(&json).unwrap();
            assert_eq!(restored, r);
        }
    }
}
```

In `checkpoint.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn checkpoint_policy_round_trip() {
        let policy = CheckpointPolicy {
            ask_on: vec![
                CheckpointTrigger::Ambiguous,
                CheckpointTrigger::ScopeExpanding,
            ],
        };
        let json = serde_json::to_string(&policy).unwrap();
        let restored: CheckpointPolicy = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.ask_on.len(), 2);
    }

    #[test]
    fn ask_user_question_with_options() {
        let q = AskUserQuestion {
            id: "q_1".into(),
            checkpoint_class: CheckpointClass::Information,
            trigger: CheckpointTrigger::Ambiguous,
            question: "Which report do you mean?".into(),
            options: Some(vec![
                "Executive Revenue Report".into(),
                "Monthly Summary Report".into(),
            ]),
            allows_free_text: true,
            asked_at: Utc::now(),
            answer: None,
        };
        let json = serde_json::to_string(&q).unwrap();
        let restored: AskUserQuestion = serde_json::from_str(&json).unwrap();
        assert!(restored.answer.is_none());
        assert_eq!(restored.options.unwrap().len(), 2);
    }

    #[test]
    fn ask_user_question_with_answer() {
        let now = Utc::now();
        let q = AskUserQuestion {
            id: "q_1".into(),
            checkpoint_class: CheckpointClass::Information,
            trigger: CheckpointTrigger::Ambiguous,
            question: "Which report?".into(),
            options: Some(vec!["Report A".into(), "Report B".into()]),
            allows_free_text: true,
            asked_at: now,
            answer: Some(UserAnswer {
                selected_option: Some("Report A".into()),
                free_text: None,
                answered_at: now,
            }),
        };
        let json = serde_json::to_string(&q).unwrap();
        let restored: AskUserQuestion = serde_json::from_str(&json).unwrap();
        assert!(restored.answer.is_some());
    }
}
```

**Step 2: Run tests to verify they fail**

Run: `cd spool && cargo test -p spool-protocol -- evaluator checkpoint`
Expected: FAIL

**Step 3: Write the implementations**

```rust
// spool/spool-protocol/src/evaluator.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceTarget {
    pub description: String,
    pub target_artifact: Option<String>,
    pub target_evidence_class: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InabilityReason {
    MissingAccess,
    MissingUserClarification,
    UnavailableArtifact,
    OutOfScopeRequest,
    PolicyBoundary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InabilityResponse {
    pub target: EvidenceTarget,
    pub reason: InabilityReason,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum EvaluatorOutcome {
    Accept,
    RequestMoreEvidence {
        targets: Vec<EvidenceTarget>,
    },
    Downgrade {
        reason: String,
        suggested_state: Option<String>,
    },
    Blocked {
        reason: String,
    },
    Contradiction {
        description: String,
        conflicting_sources: Vec<String>,
    },
}

// tests at bottom (from Step 1)
```

```rust
// spool/spool-protocol/src/checkpoint.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckpointClass {
    Information,
    Investigation,
    Action,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckpointTrigger {
    Ambiguous,
    ScopeExpanding,
    ExpectationShaping,
    SideEffecting,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointPolicy {
    pub ask_on: Vec<CheckpointTrigger>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAnswer {
    pub selected_option: Option<String>,
    pub free_text: Option<String>,
    pub answered_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AskUserQuestion {
    pub id: String,
    pub checkpoint_class: CheckpointClass,
    pub trigger: CheckpointTrigger,
    pub question: String,
    pub options: Option<Vec<String>>,
    pub allows_free_text: bool,
    pub asked_at: DateTime<Utc>,
    pub answer: Option<UserAnswer>,
}

// tests at bottom (from Step 1)
```

**Step 4: Run tests to verify they pass**

Run: `cd spool && cargo test -p spool-protocol -- evaluator checkpoint`
Expected: 6 tests PASS

**Step 5: Commit**

```bash
git add spool/spool-protocol/src/evaluator.rs spool/spool-protocol/src/checkpoint.rs
git commit -m "feat(spool-protocol): evaluator outcomes, evidence targets, checkpoint policy, and ask-user-question types"
```

---

## Task 6: Task Contract Schema

**Files:**

- Create: `spool/spool-protocol/src/task_contract.rs`

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::artifact::ArtifactType;
    use crate::checkpoint::CheckpointTrigger;
    use crate::evidence::EvidenceClass;

    #[test]
    fn task_contract_round_trip() {
        let contract = TaskContract {
            task_id: TaskId("task_123".into()),
            intent: "Find why the report revenue number does not match expected quarter totals.".into(),
            scope: Scope {
                lob: "finance".into(),
                workspace: "Executive BI".into(),
                artifacts: vec![
                    ArtifactRef {
                        artifact_type: ArtifactType::Report,
                        reference: "Executive Revenue Report".into(),
                    },
                    ArtifactRef {
                        artifact_type: ArtifactType::Measure,
                        reference: "Sales Model.Revenue".into(),
                    },
                ],
            },
            selected_recipe: Some("report_number_mismatch".into()),
            selected_recipe_selection_mode: Some(RecipeSelectionMode::AutoSelect),
            assumptions: vec![
                "The user is referring to the published report in Executive BI.".into(),
            ],
            expected_evidence_classes: vec![
                EvidenceClass::ReportMetadata,
                EvidenceClass::MeasureDefinition,
                EvidenceClass::DaxQueryResult,
                EvidenceClass::WarehouseQueryResult,
            ],
            validation_floor: ValidationFloor::DirectValidationRequired,
            checkpoint_policy: crate::checkpoint::CheckpointPolicy {
                ask_on: vec![
                    CheckpointTrigger::Ambiguous,
                    CheckpointTrigger::ScopeExpanding,
                    CheckpointTrigger::ExpectationShaping,
                    CheckpointTrigger::SideEffecting,
                ],
            },
            clarification_checkpoints: vec![],
            approval_checkpoints: vec![],
            expected_deliverable_shape: "structured_task_result".into(),
            evaluator_packet_requirements: vec![
                "evidence_summary".into(),
                "contradiction_summary".into(),
            ],
            task_status: TaskStatus::Planning,
            created_at: Some(chrono::Utc::now()),
            updated_at: None,
        };

        let json = serde_json::to_string_pretty(&contract).unwrap();
        let restored: TaskContract = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.task_id, contract.task_id);
        assert_eq!(restored.scope.artifacts.len(), 2);
        assert_eq!(restored.expected_evidence_classes.len(), 4);
        assert_eq!(restored.task_status, TaskStatus::Planning);
    }

    #[test]
    fn all_task_statuses_serialize() {
        let statuses = vec![
            TaskStatus::Planning,
            TaskStatus::Active,
            TaskStatus::Evaluating,
            TaskStatus::Completed,
            TaskStatus::Blocked,
            TaskStatus::Interrupted,
        ];
        for s in statuses {
            let json = serde_json::to_string(&s).unwrap();
            let restored: TaskStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(restored, s);
        }
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cd spool && cargo test -p spool-protocol -- task_contract`
Expected: FAIL

**Step 3: Write the implementation**

```rust
// spool/spool-protocol/src/task_contract.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::artifact::ArtifactType;
use crate::checkpoint::CheckpointPolicy;
use crate::evidence::EvidenceClass;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Planning,
    Active,
    Evaluating,
    Completed,
    Blocked,
    Interrupted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactRef {
    pub artifact_type: ArtifactType,
    pub reference: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Scope {
    pub lob: String,
    pub workspace: String,
    pub artifacts: Vec<ArtifactRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationFloor {
    DirectValidationRequired,
    MetadataOnly,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecipeSelectionMode {
    AutoSelect,
    Suggest,
    DoNotUse,
    UserRequestedOverride,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskContract {
    pub task_id: TaskId,
    pub intent: String,
    pub scope: Scope,
    pub selected_recipe: Option<String>,
    pub selected_recipe_selection_mode: Option<RecipeSelectionMode>,
    pub assumptions: Vec<String>,
    pub expected_evidence_classes: Vec<EvidenceClass>,
    pub validation_floor: ValidationFloor,
    pub checkpoint_policy: CheckpointPolicy,
    pub clarification_checkpoints: Vec<String>,
    pub approval_checkpoints: Vec<String>,
    pub expected_deliverable_shape: String,
    pub evaluator_packet_requirements: Vec<String>,
    pub task_status: TaskStatus,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

// tests at bottom (from Step 1)
```

**Step 4: Run test to verify it passes**

Run: `cd spool && cargo test -p spool-protocol -- task_contract`
Expected: 2 tests PASS

**Step 5: Commit**

```bash
git add spool/spool-protocol/src/task_contract.rs
git commit -m "feat(spool-protocol): task contract schema with scope, validation floor, recipe selection, and checkpoint policy"
```

---

## Task 7: Task Result Schema With Confidence Caps

**Files:**

- Create: `spool/spool-protocol/src/task_result.rs`

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::artifact::ArtifactType;
    use crate::contradiction::ContradictionId;
    use crate::evidence::EvidenceId;
    use crate::task_contract::TaskId;

    fn sample_result(
        state: ResultState,
        confidence: Confidence,
        contradiction_refs: Vec<ContradictionId>,
    ) -> TaskResult {
        TaskResult {
            task_id: TaskId("task_123".into()),
            proposed_state: Some(ResultState::Confirmed),
            state,
            confidence,
            summary: "Test summary".into(),
            findings: vec![Finding {
                id: "f_1".into(),
                title: "Test finding".into(),
                detail: "Detail".into(),
            }],
            evidence_refs: vec![EvidenceId("ev_1".into())],
            validation_results: vec![],
            recommended_actions: vec![],
            blockers: vec![],
            open_questions: vec![],
            proposed_changes: vec![],
            contradiction_refs,
            result_generated_at: Some(chrono::Utc::now()),
            result_version: Some(1),
        }
    }

    #[test]
    fn task_result_round_trip() {
        let result = sample_result(ResultState::Confirmed, Confidence::High, vec![]);
        let json = serde_json::to_string_pretty(&result).unwrap();
        let restored: TaskResult = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.state, ResultState::Confirmed);
        assert_eq!(restored.confidence, Confidence::High);
        assert_eq!(restored.findings.len(), 1);
    }

    #[test]
    fn confidence_cap_no_high_with_contradictions() {
        let result = sample_result(
            ResultState::Confirmed,
            Confidence::High,
            vec![ContradictionId("c_1".into())],
        );
        let violations = result.validate_confidence_caps();
        assert!(!violations.is_empty());
        assert!(violations[0].contains("contradiction"));
    }

    #[test]
    fn confidence_cap_no_high_with_inconclusive() {
        let result = sample_result(ResultState::Inconclusive, Confidence::High, vec![]);
        let violations = result.validate_confidence_caps();
        assert!(!violations.is_empty());
        assert!(violations[0].contains("inconclusive"));
    }

    #[test]
    fn confidence_cap_blocked_should_be_low() {
        let result = sample_result(ResultState::Blocked, Confidence::Medium, vec![]);
        let violations = result.validate_confidence_caps();
        assert!(!violations.is_empty());
        assert!(violations[0].contains("blocked"));
    }

    #[test]
    fn valid_result_no_violations() {
        let result = sample_result(ResultState::Confirmed, Confidence::High, vec![]);
        let violations = result.validate_confidence_caps();
        assert!(violations.is_empty());
    }

    #[test]
    fn all_result_states_serialize() {
        let states = vec![
            ResultState::Confirmed,
            ResultState::SupportedHypothesis,
            ResultState::Inconclusive,
            ResultState::Blocked,
        ];
        for s in states {
            let json = serde_json::to_string(&s).unwrap();
            let restored: ResultState = serde_json::from_str(&json).unwrap();
            assert_eq!(restored, s);
        }
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cd spool && cargo test -p spool-protocol -- task_result`
Expected: FAIL

**Step 3: Write the implementation**

```rust
// spool/spool-protocol/src/task_result.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::artifact::ArtifactType;
use crate::contradiction::ContradictionId;
use crate::evidence::EvidenceId;
use crate::task_contract::TaskId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResultState {
    Confirmed,
    SupportedHypothesis,
    Inconclusive,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Confidence {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub id: String,
    pub title: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub id: String,
    pub validation_type: String,
    pub status: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendedAction {
    pub id: String,
    pub action_type: String,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Blocker {
    pub id: String,
    pub blocker_type: String,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenQuestion {
    pub id: String,
    pub question: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposedChange {
    pub id: String,
    pub artifact_type: ArtifactType,
    pub artifact_ref: String,
    pub change_summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub task_id: TaskId,
    pub proposed_state: Option<ResultState>,
    pub state: ResultState,
    pub confidence: Confidence,
    pub summary: String,
    pub findings: Vec<Finding>,
    pub evidence_refs: Vec<EvidenceId>,
    pub validation_results: Vec<ValidationResult>,
    pub recommended_actions: Vec<RecommendedAction>,
    pub blockers: Vec<Blocker>,
    pub open_questions: Vec<OpenQuestion>,
    pub proposed_changes: Vec<ProposedChange>,
    pub contradiction_refs: Vec<ContradictionId>,
    pub result_generated_at: Option<DateTime<Utc>>,
    pub result_version: Option<u32>,
}

impl TaskResult {
    /// Validate confidence caps per spec Section 10.7.
    pub fn validate_confidence_caps(&self) -> Vec<String> {
        let mut violations = Vec::new();

        if self.confidence == Confidence::High && !self.contradiction_refs.is_empty() {
            violations.push(
                "high confidence not allowed with unresolved contradiction references".into(),
            );
        }

        if self.state == ResultState::Inconclusive && self.confidence == Confidence::High {
            violations.push(
                "high confidence not allowed with inconclusive result state".into(),
            );
        }

        if self.state == ResultState::Blocked && self.confidence != Confidence::Low {
            violations.push(
                "blocked result state should normally carry low confidence".into(),
            );
        }

        violations
    }
}

// tests at bottom (from Step 1)
```

**Step 4: Run test to verify it passes**

Run: `cd spool && cargo test -p spool-protocol -- task_result`
Expected: 6 tests PASS

**Step 5: Commit**

```bash
git add spool/spool-protocol/src/task_result.rs
git commit -m "feat(spool-protocol): task result schema with confidence caps validation per spec Section 10.7"
```

---

## Task 8: Evidence Ledger

**Files:**

- Modify: `spool/spool-core/src/evidence_ledger.rs`

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use spool_protocol::artifact::ArtifactId;
    use spool_protocol::evidence::{EvidenceClass, EvidenceId, EvidenceItem, EvidenceType};

    fn make_evidence(id: &str, class: EvidenceClass, artifact: &str) -> EvidenceItem {
        EvidenceItem {
            id: EvidenceId(id.into()),
            evidence_type: EvidenceType::Observed,
            evidence_class: class,
            source: "test".into(),
            summary: format!("Evidence {id}"),
            artifact_refs: vec![ArtifactId(artifact.into())],
            observed_at: Some(Utc::now()),
            detail: None,
        }
    }

    #[test]
    fn append_and_retrieve() {
        let mut ledger = EvidenceLedger::new();
        assert!(ledger.is_empty());

        ledger.append(make_evidence("ev_1", EvidenceClass::DaxQueryResult, "art_1"));
        assert_eq!(ledger.len(), 1);

        let item = ledger.get(&EvidenceId("ev_1".into())).unwrap();
        assert_eq!(item.summary, "Evidence ev_1");
    }

    #[test]
    fn query_by_class() {
        let mut ledger = EvidenceLedger::new();
        ledger.append(make_evidence("ev_1", EvidenceClass::DaxQueryResult, "art_1"));
        ledger.append(make_evidence("ev_2", EvidenceClass::ReportMetadata, "art_1"));
        ledger.append(make_evidence("ev_3", EvidenceClass::DaxQueryResult, "art_2"));

        let dax = ledger.query_by_class(&EvidenceClass::DaxQueryResult);
        assert_eq!(dax.len(), 2);

        let report = ledger.query_by_class(&EvidenceClass::ReportMetadata);
        assert_eq!(report.len(), 1);
    }

    #[test]
    fn query_by_artifact() {
        let mut ledger = EvidenceLedger::new();
        ledger.append(make_evidence("ev_1", EvidenceClass::DaxQueryResult, "art_1"));
        ledger.append(make_evidence("ev_2", EvidenceClass::ReportMetadata, "art_1"));
        ledger.append(make_evidence("ev_3", EvidenceClass::DaxQueryResult, "art_2"));

        let art1 = ledger.query_by_artifact(&ArtifactId("art_1".into()));
        assert_eq!(art1.len(), 2);

        let art2 = ledger.query_by_artifact(&ArtifactId("art_2".into()));
        assert_eq!(art2.len(), 1);
    }

    #[test]
    fn ledger_is_append_only() {
        let mut ledger = EvidenceLedger::new();
        ledger.append(make_evidence("ev_1", EvidenceClass::DaxQueryResult, "art_1"));
        ledger.append(make_evidence("ev_2", EvidenceClass::ReportMetadata, "art_1"));

        // No mutation or deletion methods exist — this is enforced by the API surface.
        // Verify entries are preserved in order.
        let all = ledger.all();
        assert_eq!(all.len(), 2);
        assert_eq!(all[0].id, EvidenceId("ev_1".into()));
        assert_eq!(all[1].id, EvidenceId("ev_2".into()));
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cd spool && cargo test -p spool-core -- evidence_ledger`
Expected: FAIL

**Step 3: Write the implementation**

```rust
// spool/spool-core/src/evidence_ledger.rs
use spool_protocol::artifact::ArtifactId;
use spool_protocol::evidence::{EvidenceClass, EvidenceId, EvidenceItem};

/// Append-only evidence ledger for a single task.
///
/// Evidence can be added and queried but never mutated or deleted.
/// This is the authoritative evidence source for a task (Spec Section 9.1).
pub struct EvidenceLedger {
    entries: Vec<EvidenceItem>,
}

impl EvidenceLedger {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn append(&mut self, item: EvidenceItem) {
        self.entries.push(item);
    }

    pub fn get(&self, id: &EvidenceId) -> Option<&EvidenceItem> {
        self.entries.iter().find(|e| e.id == *id)
    }

    pub fn query_by_class(&self, class: &EvidenceClass) -> Vec<&EvidenceItem> {
        self.entries
            .iter()
            .filter(|e| e.evidence_class == *class)
            .collect()
    }

    pub fn query_by_artifact(&self, artifact_id: &ArtifactId) -> Vec<&EvidenceItem> {
        self.entries
            .iter()
            .filter(|e| e.artifact_refs.contains(artifact_id))
            .collect()
    }

    pub fn all(&self) -> &[EvidenceItem] {
        &self.entries
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Default for EvidenceLedger {
    fn default() -> Self {
        Self::new()
    }
}

// tests at bottom (from Step 1)
```

**Step 4: Run test to verify it passes**

Run: `cd spool && cargo test -p spool-core -- evidence_ledger`
Expected: 4 tests PASS

**Step 5: Commit**

```bash
git add spool/spool-core/src/evidence_ledger.rs
git commit -m "feat(spool-core): append-only evidence ledger with class and artifact queries"
```

---

## Task 9: Contradiction Ledger

**Files:**

- Modify: `spool/spool-core/src/contradiction_ledger.rs`

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use spool_protocol::contradiction::{
        ContradictionId, ContradictionRecord, ContradictionStatus, MaterialityLevel,
    };
    use spool_protocol::evidence::EvidenceId;

    fn make_contradiction(id: &str, material: bool) -> ContradictionRecord {
        let now = Utc::now();
        ContradictionRecord {
            id: ContradictionId(id.into()),
            disputed_claim: format!("Disputed claim {id}"),
            conflicting_evidence: vec![
                EvidenceId("ev_a".into()),
                EvidenceId("ev_b".into()),
            ],
            materiality: if material {
                MaterialityLevel::Material
            } else {
                MaterialityLevel::NonMaterial
            },
            freshness_notes: None,
            resolution_attempted: false,
            resolution_detail: None,
            status: ContradictionStatus::Open,
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn record_and_query() {
        let mut ledger = ContradictionLedger::new();
        ledger.record(make_contradiction("c_1", true));
        ledger.record(make_contradiction("c_2", false));

        assert_eq!(ledger.all().len(), 2);
        assert_eq!(ledger.unresolved().len(), 2);
    }

    #[test]
    fn resolve_contradiction() {
        let mut ledger = ContradictionLedger::new();
        ledger.record(make_contradiction("c_1", true));

        let resolved = ledger.resolve(
            &ContradictionId("c_1".into()),
            "Freshness resolved the conflict".into(),
        );
        assert!(resolved);

        assert_eq!(ledger.unresolved().len(), 0);
        assert_eq!(ledger.all()[0].status, ContradictionStatus::Resolved);
        assert!(ledger.all()[0].resolution_attempted);
    }

    #[test]
    fn carry_forward() {
        let mut ledger = ContradictionLedger::new();
        ledger.record(make_contradiction("c_1", true));

        let carried = ledger.carry_forward(&ContradictionId("c_1".into()));
        assert!(carried);
        assert_eq!(ledger.all()[0].status, ContradictionStatus::CarriedForward);
    }

    #[test]
    fn has_unresolved_material() {
        let mut ledger = ContradictionLedger::new();
        assert!(!ledger.has_unresolved_material());

        ledger.record(make_contradiction("c_1", false));
        assert!(!ledger.has_unresolved_material());

        ledger.record(make_contradiction("c_2", true));
        assert!(ledger.has_unresolved_material());

        ledger.resolve(&ContradictionId("c_2".into()), "resolved".into());
        assert!(!ledger.has_unresolved_material());
    }

    #[test]
    fn resolve_nonexistent_returns_false() {
        let mut ledger = ContradictionLedger::new();
        assert!(!ledger.resolve(&ContradictionId("nope".into()), "x".into()));
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cd spool && cargo test -p spool-core -- contradiction_ledger`
Expected: FAIL

**Step 3: Write the implementation**

```rust
// spool/spool-core/src/contradiction_ledger.rs
use chrono::Utc;
use spool_protocol::contradiction::{
    ContradictionId, ContradictionRecord, ContradictionStatus, MaterialityLevel,
};

/// Contradiction ledger for a single task.
///
/// The contradiction ledger is the authoritative source of truth for
/// contradiction state (Spec Section 9.4). Other surfaces (task results,
/// compaction summaries) are projections.
pub struct ContradictionLedger {
    entries: Vec<ContradictionRecord>,
}

impl ContradictionLedger {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn record(&mut self, record: ContradictionRecord) {
        self.entries.push(record);
    }

    pub fn resolve(&mut self, id: &ContradictionId, detail: String) -> bool {
        if let Some(record) = self.entries.iter_mut().find(|r| r.id == *id) {
            record.status = ContradictionStatus::Resolved;
            record.resolution_attempted = true;
            record.resolution_detail = Some(detail);
            record.updated_at = Utc::now();
            true
        } else {
            false
        }
    }

    pub fn carry_forward(&mut self, id: &ContradictionId) -> bool {
        if let Some(record) = self.entries.iter_mut().find(|r| r.id == *id) {
            record.status = ContradictionStatus::CarriedForward;
            record.updated_at = Utc::now();
            true
        } else {
            false
        }
    }

    pub fn has_unresolved_material(&self) -> bool {
        self.entries.iter().any(|r| {
            r.materiality == MaterialityLevel::Material
                && r.status == ContradictionStatus::Open
        })
    }

    pub fn unresolved(&self) -> Vec<&ContradictionRecord> {
        self.entries
            .iter()
            .filter(|r| r.status == ContradictionStatus::Open)
            .collect()
    }

    pub fn all(&self) -> &[ContradictionRecord] {
        &self.entries
    }
}

impl Default for ContradictionLedger {
    fn default() -> Self {
        Self::new()
    }
}

// tests at bottom (from Step 1)
```

**Step 4: Run test to verify it passes**

Run: `cd spool && cargo test -p spool-core -- contradiction_ledger`
Expected: 5 tests PASS

**Step 5: Commit**

```bash
git add spool/spool-core/src/contradiction_ledger.rs
git commit -m "feat(spool-core): contradiction ledger with resolve, carry-forward, and materiality queries"
```

---

## Task 10: Harness Traits And Fixture Implementations

**Files:**

- Modify: `spool/spool-core/src/harness.rs`

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use super::fixtures::*;
    use crate::evidence_ledger::EvidenceLedger;
    use crate::contradiction_ledger::ContradictionLedger;
    use spool_protocol::checkpoint::{CheckpointPolicy, CheckpointTrigger};
    use spool_protocol::evidence::EvidenceClass;
    use spool_protocol::task_contract::{
        ArtifactRef, Scope, TaskContract, TaskId, TaskStatus, ValidationFloor,
    };
    use spool_protocol::artifact::ArtifactType;
    use spool_protocol::evaluator::EvaluatorOutcome;

    fn sample_contract() -> TaskContract {
        TaskContract {
            task_id: TaskId("task_test".into()),
            intent: "Test investigation".into(),
            scope: Scope {
                lob: "test".into(),
                workspace: "test_ws".into(),
                artifacts: vec![ArtifactRef {
                    artifact_type: ArtifactType::Report,
                    reference: "Test Report".into(),
                }],
            },
            selected_recipe: None,
            selected_recipe_selection_mode: None,
            assumptions: vec![],
            expected_evidence_classes: vec![EvidenceClass::ReportMetadata],
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
    async fn fixture_planner_creates_contract() {
        let planner = FixturePlanner::new(sample_contract());
        let input = UserRequest {
            raw_input: "test".into(),
            context: None,
        };
        let ctx = PlannerContext {
            available_recipes: vec![],
            available_lobs: vec!["test".into()],
            workspace_scope: Some("test_ws".into()),
        };
        let contract = planner.create_task_contract(&input, &ctx).await.unwrap();
        assert_eq!(contract.task_id, TaskId("task_test".into()));
    }

    #[tokio::test]
    async fn fixture_evaluator_returns_configured_outcome() {
        let evaluator = FixtureEvaluator::new(vec![EvaluatorOutcome::Accept]);
        let packet = EvaluatorPacket {
            contract: sample_contract(),
            evidence_summary: vec![],
            contradiction_summary: vec![],
            generator_output: GeneratorOutput {
                proposed_state: None,
                summary: "test".into(),
                findings: vec![],
                evidence_collected: vec![],
                recommended_actions: vec![],
                proposed_changes: vec![],
            },
        };
        let outcome = evaluator.evaluate(&packet).await.unwrap();
        assert!(matches!(outcome, EvaluatorOutcome::Accept));
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cd spool && cargo test -p spool-core -- harness::tests`
Expected: FAIL

**Step 3: Write the implementation**

```rust
// spool/spool-core/src/harness.rs
use async_trait::async_trait;
use spool_protocol::contradiction::ContradictionRecord;
use spool_protocol::evaluator::{EvaluatorOutcome, EvidenceTarget, InabilityResponse};
use spool_protocol::evidence::EvidenceItem;
use spool_protocol::task_contract::TaskContract;
use spool_protocol::task_result::{
    Finding, ProposedChange, RecommendedAction, ResultState,
};

use crate::error::SpoolError;

// --- Input and context types ---

pub struct UserRequest {
    pub raw_input: String,
    pub context: Option<String>,
}

pub struct PlannerContext {
    pub available_recipes: Vec<String>,
    pub available_lobs: Vec<String>,
    pub workspace_scope: Option<String>,
}

#[derive(Debug, Clone)]
pub struct GeneratorOutput {
    pub proposed_state: Option<ResultState>,
    pub summary: String,
    pub findings: Vec<Finding>,
    pub evidence_collected: Vec<EvidenceItem>,
    pub recommended_actions: Vec<RecommendedAction>,
    pub proposed_changes: Vec<ProposedChange>,
}

pub enum EvidenceCollectionResult {
    Collected(EvidenceItem),
    Unable(InabilityResponse),
}

pub struct EvaluatorPacket {
    pub contract: TaskContract,
    pub evidence_summary: Vec<EvidenceItem>,
    pub contradiction_summary: Vec<ContradictionRecord>,
    pub generator_output: GeneratorOutput,
}

// --- Traits ---

#[async_trait]
pub trait Planner: Send + Sync {
    async fn create_task_contract(
        &self,
        input: &UserRequest,
        context: &PlannerContext,
    ) -> Result<TaskContract, SpoolError>;
}

#[async_trait]
pub trait Generator: Send + Sync {
    async fn investigate(
        &self,
        contract: &TaskContract,
        evidence: &[EvidenceItem],
        contradictions: &[ContradictionRecord],
    ) -> Result<GeneratorOutput, SpoolError>;

    async fn collect_evidence(
        &self,
        target: &EvidenceTarget,
        contract: &TaskContract,
    ) -> Result<EvidenceCollectionResult, SpoolError>;
}

#[async_trait]
pub trait Evaluator: Send + Sync {
    async fn evaluate(
        &self,
        packet: &EvaluatorPacket,
    ) -> Result<EvaluatorOutcome, SpoolError>;
}

// --- Fixture implementations ---

pub mod fixtures {
    use super::*;
    use std::sync::Mutex;

    pub struct FixturePlanner {
        contract: TaskContract,
    }

    impl FixturePlanner {
        pub fn new(contract: TaskContract) -> Self {
            Self { contract }
        }
    }

    #[async_trait]
    impl Planner for FixturePlanner {
        async fn create_task_contract(
            &self,
            _input: &UserRequest,
            _context: &PlannerContext,
        ) -> Result<TaskContract, SpoolError> {
            Ok(self.contract.clone())
        }
    }

    pub struct FixtureGenerator {
        output: GeneratorOutput,
        evidence_responses: Mutex<Vec<EvidenceCollectionResult>>,
    }

    impl FixtureGenerator {
        pub fn new(output: GeneratorOutput) -> Self {
            Self {
                output,
                evidence_responses: Mutex::new(Vec::new()),
            }
        }

        pub fn with_evidence_responses(
            mut self,
            responses: Vec<EvidenceCollectionResult>,
        ) -> Self {
            self.evidence_responses = Mutex::new(responses);
            self
        }
    }

    #[async_trait]
    impl Generator for FixtureGenerator {
        async fn investigate(
            &self,
            _contract: &TaskContract,
            _evidence: &[EvidenceItem],
            _contradictions: &[ContradictionRecord],
        ) -> Result<GeneratorOutput, SpoolError> {
            Ok(self.output.clone())
        }

        async fn collect_evidence(
            &self,
            _target: &EvidenceTarget,
            _contract: &TaskContract,
        ) -> Result<EvidenceCollectionResult, SpoolError> {
            let mut responses = self.evidence_responses.lock().unwrap();
            if responses.is_empty() {
                Err(SpoolError::Generator(
                    "no more fixture evidence responses".into(),
                ))
            } else {
                Ok(responses.remove(0))
            }
        }
    }

    pub struct FixtureEvaluator {
        outcomes: Mutex<Vec<EvaluatorOutcome>>,
    }

    impl FixtureEvaluator {
        pub fn new(outcomes: Vec<EvaluatorOutcome>) -> Self {
            Self {
                outcomes: Mutex::new(outcomes),
            }
        }
    }

    #[async_trait]
    impl Evaluator for FixtureEvaluator {
        async fn evaluate(
            &self,
            _packet: &EvaluatorPacket,
        ) -> Result<EvaluatorOutcome, SpoolError> {
            let mut outcomes = self.outcomes.lock().unwrap();
            if outcomes.is_empty() {
                Err(SpoolError::Evaluator(
                    "no more fixture evaluator outcomes".into(),
                ))
            } else {
                Ok(outcomes.remove(0))
            }
        }
    }
}

// tests at bottom (from Step 1)
```

**Step 4: Run test to verify it passes**

Run: `cd spool && cargo test -p spool-core -- harness::tests`
Expected: 2 tests PASS

**Step 5: Commit**

```bash
git add spool/spool-core/src/harness.rs
git commit -m "feat(spool-core): planner/generator/evaluator traits with fixture implementations"
```

---

## Task 11: Evaluator Loop Protocol

**Files:**

- Modify: `spool/spool-core/src/evaluator_loop.rs`

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::contradiction_ledger::ContradictionLedger;
    use crate::evidence_ledger::EvidenceLedger;
    use crate::harness::fixtures::*;
    use crate::harness::{EvidenceCollectionResult, GeneratorOutput};
    use chrono::Utc;
    use spool_protocol::artifact::ArtifactId;
    use spool_protocol::checkpoint::{CheckpointPolicy, CheckpointTrigger};
    use spool_protocol::evidence::{EvidenceClass, EvidenceId, EvidenceItem, EvidenceType};
    use spool_protocol::evaluator::{EvaluatorOutcome, EvidenceTarget};
    use spool_protocol::task_contract::*;
    use spool_protocol::task_result::ResultState;
    use spool_protocol::artifact::ArtifactType;

    fn sample_contract() -> TaskContract {
        TaskContract {
            task_id: TaskId("task_loop".into()),
            intent: "Test".into(),
            scope: Scope {
                lob: "test".into(),
                workspace: "ws".into(),
                artifacts: vec![ArtifactRef {
                    artifact_type: ArtifactType::Report,
                    reference: "Report".into(),
                }],
            },
            selected_recipe: None,
            selected_recipe_selection_mode: None,
            assumptions: vec![],
            expected_evidence_classes: vec![],
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

    fn sample_output() -> GeneratorOutput {
        GeneratorOutput {
            proposed_state: Some(ResultState::Confirmed),
            summary: "test finding".into(),
            findings: vec![],
            evidence_collected: vec![],
            recommended_actions: vec![],
            proposed_changes: vec![],
        }
    }

    fn make_evidence_item(id: &str) -> EvidenceItem {
        EvidenceItem {
            id: EvidenceId(id.into()),
            evidence_type: EvidenceType::Observed,
            evidence_class: EvidenceClass::DaxQueryResult,
            source: "test".into(),
            summary: format!("Evidence {id}"),
            artifact_refs: vec![ArtifactId("art_1".into())],
            observed_at: Some(Utc::now()),
            detail: None,
        }
    }

    #[tokio::test]
    async fn accept_on_first_iteration() {
        let generator = FixtureGenerator::new(sample_output());
        let evaluator = FixtureEvaluator::new(vec![EvaluatorOutcome::Accept]);
        let mut evidence = EvidenceLedger::new();
        let mut contradictions = ContradictionLedger::new();
        let config = EvaluatorLoopConfig { max_iterations: 3 };

        let result = run_evaluator_loop(
            &generator,
            &evaluator,
            &sample_contract(),
            &mut evidence,
            &mut contradictions,
            sample_output(),
            &config,
        )
        .await
        .unwrap();

        assert_eq!(result.iterations_used, 1);
        assert!(!result.exhausted);
        assert!(matches!(result.evaluator_outcome, EvaluatorOutcome::Accept));
    }

    #[tokio::test]
    async fn request_more_evidence_then_accept() {
        let generator = FixtureGenerator::new(sample_output())
            .with_evidence_responses(vec![
                EvidenceCollectionResult::Collected(make_evidence_item("ev_new")),
            ]);
        let evaluator = FixtureEvaluator::new(vec![
            EvaluatorOutcome::RequestMoreEvidence {
                targets: vec![EvidenceTarget {
                    description: "Run DAX query".into(),
                    target_artifact: Some("art_1".into()),
                    target_evidence_class: Some("dax_query_result".into()),
                }],
            },
            EvaluatorOutcome::Accept,
        ]);
        let mut evidence = EvidenceLedger::new();
        let mut contradictions = ContradictionLedger::new();
        let config = EvaluatorLoopConfig { max_iterations: 5 };

        let result = run_evaluator_loop(
            &generator,
            &evaluator,
            &sample_contract(),
            &mut evidence,
            &mut contradictions,
            sample_output(),
            &config,
        )
        .await
        .unwrap();

        assert_eq!(result.iterations_used, 2);
        assert!(!result.exhausted);
        assert_eq!(evidence.len(), 1); // new evidence was collected
    }

    #[tokio::test]
    async fn loop_exhaustion_prevents_confirmed() {
        let generator = FixtureGenerator::new(sample_output())
            .with_evidence_responses(vec![
                EvidenceCollectionResult::Collected(make_evidence_item("ev_1")),
                EvidenceCollectionResult::Collected(make_evidence_item("ev_2")),
            ]);
        let evaluator = FixtureEvaluator::new(vec![
            EvaluatorOutcome::RequestMoreEvidence {
                targets: vec![EvidenceTarget {
                    description: "need more".into(),
                    target_artifact: None,
                    target_evidence_class: None,
                }],
            },
            EvaluatorOutcome::RequestMoreEvidence {
                targets: vec![EvidenceTarget {
                    description: "still need more".into(),
                    target_artifact: None,
                    target_evidence_class: None,
                }],
            },
        ]);
        let mut evidence = EvidenceLedger::new();
        let mut contradictions = ContradictionLedger::new();
        let config = EvaluatorLoopConfig { max_iterations: 2 };

        let result = run_evaluator_loop(
            &generator,
            &evaluator,
            &sample_contract(),
            &mut evidence,
            &mut contradictions,
            sample_output(),
            &config,
        )
        .await
        .unwrap();

        assert!(result.exhausted);
        // Spec Section 4.6: after loop exhaustion, final state must not be confirmed
        assert_ne!(result.final_state, ResultState::Confirmed);
    }

    #[tokio::test]
    async fn downgrade_reduces_state() {
        let generator = FixtureGenerator::new(sample_output());
        let evaluator = FixtureEvaluator::new(vec![EvaluatorOutcome::Downgrade {
            reason: "evidence too weak".into(),
            suggested_state: Some("supported_hypothesis".into()),
        }]);
        let mut evidence = EvidenceLedger::new();
        let mut contradictions = ContradictionLedger::new();
        let config = EvaluatorLoopConfig { max_iterations: 3 };

        let result = run_evaluator_loop(
            &generator,
            &evaluator,
            &sample_contract(),
            &mut evidence,
            &mut contradictions,
            sample_output(),
            &config,
        )
        .await
        .unwrap();

        assert_eq!(result.final_state, ResultState::SupportedHypothesis);
        assert!(!result.exhausted);
    }

    #[tokio::test]
    async fn blocked_returns_blocked_state() {
        let generator = FixtureGenerator::new(sample_output());
        let evaluator = FixtureEvaluator::new(vec![EvaluatorOutcome::Blocked {
            reason: "missing access".into(),
        }]);
        let mut evidence = EvidenceLedger::new();
        let mut contradictions = ContradictionLedger::new();
        let config = EvaluatorLoopConfig { max_iterations: 3 };

        let result = run_evaluator_loop(
            &generator,
            &evaluator,
            &sample_contract(),
            &mut evidence,
            &mut contradictions,
            sample_output(),
            &config,
        )
        .await
        .unwrap();

        assert_eq!(result.final_state, ResultState::Blocked);
    }

    #[tokio::test]
    async fn contradiction_records_to_ledger() {
        let generator = FixtureGenerator::new(sample_output());
        let evaluator = FixtureEvaluator::new(vec![EvaluatorOutcome::Contradiction {
            description: "DAX and warehouse disagree".into(),
            conflicting_sources: vec!["ev_1".into(), "ev_2".into()],
        }]);
        let mut evidence = EvidenceLedger::new();
        let mut contradictions = ContradictionLedger::new();
        let config = EvaluatorLoopConfig { max_iterations: 1 };

        let result = run_evaluator_loop(
            &generator,
            &evaluator,
            &sample_contract(),
            &mut evidence,
            &mut contradictions,
            sample_output(),
            &config,
        )
        .await
        .unwrap();

        assert_eq!(contradictions.all().len(), 1);
        assert!(contradictions.has_unresolved_material());
        // With unresolved material contradiction and exhaustion, cannot be confirmed
        assert_ne!(result.final_state, ResultState::Confirmed);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cd spool && cargo test -p spool-core -- evaluator_loop`
Expected: FAIL

**Step 3: Write the implementation**

```rust
// spool/spool-core/src/evaluator_loop.rs
use chrono::Utc;
use spool_protocol::contradiction::{
    ContradictionId, ContradictionRecord, ContradictionStatus, MaterialityLevel,
};
use spool_protocol::evaluator::EvaluatorOutcome;
use spool_protocol::task_contract::TaskContract;
use spool_protocol::task_result::{Confidence, ResultState};

use crate::contradiction_ledger::ContradictionLedger;
use crate::error::SpoolError;
use crate::evidence_ledger::EvidenceLedger;
use crate::harness::{
    EvaluatorPacket, EvidenceCollectionResult, Generator, GeneratorOutput, Evaluator,
};

pub struct EvaluatorLoopConfig {
    pub max_iterations: usize,
}

pub struct EvaluatorLoopResult {
    pub final_state: ResultState,
    pub final_confidence: Confidence,
    pub evaluator_outcome: EvaluatorOutcome,
    pub iterations_used: usize,
    pub exhausted: bool,
    pub generator_proposed_state: Option<ResultState>,
}

/// Run the bounded evaluator loop per Spec Section 4.3-4.6.
///
/// The loop runs at most `config.max_iterations` rounds. On each round,
/// the evaluator reviews the current evidence and generator output and
/// returns one of five outcome classes. The loop routes each outcome
/// accordingly and terminates when the evaluator accepts, blocks,
/// downgrades, or when iterations are exhausted.
pub async fn run_evaluator_loop(
    generator: &dyn Generator,
    evaluator: &dyn Evaluator,
    contract: &TaskContract,
    evidence: &mut EvidenceLedger,
    contradictions: &mut ContradictionLedger,
    initial_output: GeneratorOutput,
    config: &EvaluatorLoopConfig,
) -> Result<EvaluatorLoopResult, SpoolError> {
    let mut current_output = initial_output;
    let mut iterations = 0;

    loop {
        iterations += 1;

        let packet = EvaluatorPacket {
            contract: contract.clone(),
            evidence_summary: evidence.all().to_vec(),
            contradiction_summary: contradictions.all().to_vec(),
            generator_output: current_output.clone(),
        };

        let outcome = evaluator.evaluate(&packet).await?;

        match &outcome {
            EvaluatorOutcome::Accept => {
                let confidence = determine_confidence(contradictions);
                return Ok(EvaluatorLoopResult {
                    final_state: current_output
                        .proposed_state
                        .clone()
                        .unwrap_or(ResultState::Confirmed),
                    final_confidence: confidence,
                    evaluator_outcome: outcome,
                    iterations_used: iterations,
                    exhausted: false,
                    generator_proposed_state: current_output.proposed_state,
                });
            }

            EvaluatorOutcome::RequestMoreEvidence { targets } => {
                if iterations >= config.max_iterations {
                    // Spec Section 4.6: after exhaustion, state must not be confirmed
                    return Ok(EvaluatorLoopResult {
                        final_state: ResultState::SupportedHypothesis,
                        final_confidence: Confidence::Medium,
                        evaluator_outcome: outcome,
                        iterations_used: iterations,
                        exhausted: true,
                        generator_proposed_state: current_output.proposed_state,
                    });
                }

                // Spec Section 4.4: generator must collect or return inability reason
                for target in targets {
                    match generator.collect_evidence(target, contract).await? {
                        EvidenceCollectionResult::Collected(item) => {
                            evidence.append(item);
                        }
                        EvidenceCollectionResult::Unable(_inability) => {
                            // Recorded but loop continues
                        }
                    }
                }

                current_output = generator
                    .investigate(contract, evidence.all(), contradictions.all())
                    .await?;
            }

            EvaluatorOutcome::Downgrade {
                suggested_state, ..
            } => {
                let state = suggested_state
                    .as_deref()
                    .and_then(|s| match s {
                        "supported_hypothesis" => Some(ResultState::SupportedHypothesis),
                        "inconclusive" => Some(ResultState::Inconclusive),
                        "blocked" => Some(ResultState::Blocked),
                        _ => None,
                    })
                    .unwrap_or(ResultState::SupportedHypothesis);

                return Ok(EvaluatorLoopResult {
                    final_state: state,
                    final_confidence: Confidence::Medium,
                    evaluator_outcome: outcome,
                    iterations_used: iterations,
                    exhausted: false,
                    generator_proposed_state: current_output.proposed_state,
                });
            }

            EvaluatorOutcome::Blocked { .. } => {
                return Ok(EvaluatorLoopResult {
                    final_state: ResultState::Blocked,
                    final_confidence: Confidence::Low,
                    evaluator_outcome: outcome,
                    iterations_used: iterations,
                    exhausted: false,
                    generator_proposed_state: current_output.proposed_state,
                });
            }

            EvaluatorOutcome::Contradiction {
                description,
                conflicting_sources,
            } => {
                let contradiction = ContradictionRecord {
                    id: ContradictionId(format!("contra_{iterations}")),
                    disputed_claim: description.clone(),
                    conflicting_evidence: vec![],
                    materiality: MaterialityLevel::Material,
                    freshness_notes: None,
                    resolution_attempted: false,
                    resolution_detail: None,
                    status: ContradictionStatus::Open,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                };
                contradictions.record(contradiction);

                if iterations >= config.max_iterations {
                    return Ok(EvaluatorLoopResult {
                        final_state: ResultState::Inconclusive,
                        final_confidence: Confidence::Low,
                        evaluator_outcome: outcome,
                        iterations_used: iterations,
                        exhausted: true,
                        generator_proposed_state: current_output.proposed_state,
                    });
                }

                current_output = generator
                    .investigate(contract, evidence.all(), contradictions.all())
                    .await?;
            }
        }
    }
}

fn determine_confidence(contradictions: &ContradictionLedger) -> Confidence {
    if contradictions.has_unresolved_material() {
        // Spec Section 10.7: high confidence not allowed with unresolved material contradiction
        Confidence::Medium
    } else {
        Confidence::High
    }
}

// tests at bottom (from Step 1)
```

**Step 4: Run test to verify it passes**

Run: `cd spool && cargo test -p spool-core -- evaluator_loop`
Expected: 6 tests PASS

**Step 5: Commit**

```bash
git add spool/spool-core/src/evaluator_loop.rs
git commit -m "feat(spool-core): bounded evaluator loop with all five outcome paths and loop-exhaustion semantics"
```

---

## Task 12: Task Lifecycle State Machine

**Files:**

- Modify: `spool/spool-core/src/task_lifecycle.rs`

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use spool_protocol::task_contract::TaskStatus;

    #[test]
    fn valid_transitions() {
        assert!(is_valid_transition(&TaskStatus::Planning, &TaskStatus::Active));
        assert!(is_valid_transition(&TaskStatus::Active, &TaskStatus::Evaluating));
        assert!(is_valid_transition(&TaskStatus::Evaluating, &TaskStatus::Completed));
        assert!(is_valid_transition(&TaskStatus::Active, &TaskStatus::Blocked));
        assert!(is_valid_transition(&TaskStatus::Active, &TaskStatus::Interrupted));
        assert!(is_valid_transition(&TaskStatus::Evaluating, &TaskStatus::Active));
    }

    #[test]
    fn invalid_transitions() {
        assert!(!is_valid_transition(&TaskStatus::Completed, &TaskStatus::Active));
        assert!(!is_valid_transition(&TaskStatus::Planning, &TaskStatus::Completed));
        assert!(!is_valid_transition(&TaskStatus::Planning, &TaskStatus::Evaluating));
    }

    #[test]
    fn lifecycle_tracker_happy_path() {
        let mut tracker = TaskLifecycle::new();
        assert_eq!(tracker.status(), &TaskStatus::Planning);

        assert!(tracker.transition(TaskStatus::Active).is_ok());
        assert_eq!(tracker.status(), &TaskStatus::Active);

        assert!(tracker.transition(TaskStatus::Evaluating).is_ok());
        assert_eq!(tracker.status(), &TaskStatus::Evaluating);

        assert!(tracker.transition(TaskStatus::Completed).is_ok());
        assert_eq!(tracker.status(), &TaskStatus::Completed);
    }

    #[test]
    fn lifecycle_tracker_rejects_invalid() {
        let mut tracker = TaskLifecycle::new();
        let result = tracker.transition(TaskStatus::Completed);
        assert!(result.is_err());
        assert_eq!(tracker.status(), &TaskStatus::Planning);
    }

    #[test]
    fn lifecycle_tracker_interrupted_and_resume() {
        let mut tracker = TaskLifecycle::new();
        tracker.transition(TaskStatus::Active).unwrap();
        tracker.transition(TaskStatus::Interrupted).unwrap();

        assert_eq!(tracker.status(), &TaskStatus::Interrupted);
        assert!(tracker.was_interrupted());

        // Can resume from interrupted
        tracker.transition(TaskStatus::Active).unwrap();
        assert_eq!(tracker.status(), &TaskStatus::Active);
    }

    #[test]
    fn lifecycle_tracks_phase_history() {
        let mut tracker = TaskLifecycle::new();
        tracker.transition(TaskStatus::Active).unwrap();
        tracker.transition(TaskStatus::Evaluating).unwrap();
        tracker.transition(TaskStatus::Active).unwrap();

        let history = tracker.history();
        assert_eq!(history.len(), 4); // Planning + 3 transitions
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cd spool && cargo test -p spool-core -- task_lifecycle`
Expected: FAIL

**Step 3: Write the implementation**

```rust
// spool/spool-core/src/task_lifecycle.rs
use chrono::{DateTime, Utc};
use spool_protocol::task_contract::TaskStatus;

use crate::error::SpoolError;

/// Valid state transitions for a task lifecycle.
///
/// Planning → Active → Evaluating → Completed
///                   → Blocked
///                   → Interrupted → Active (resume)
///          Evaluating → Active (evaluator requests more evidence)
pub fn is_valid_transition(from: &TaskStatus, to: &TaskStatus) -> bool {
    matches!(
        (from, to),
        (TaskStatus::Planning, TaskStatus::Active)
            | (TaskStatus::Active, TaskStatus::Evaluating)
            | (TaskStatus::Active, TaskStatus::Blocked)
            | (TaskStatus::Active, TaskStatus::Interrupted)
            | (TaskStatus::Evaluating, TaskStatus::Completed)
            | (TaskStatus::Evaluating, TaskStatus::Active)
            | (TaskStatus::Evaluating, TaskStatus::Blocked)
            | (TaskStatus::Evaluating, TaskStatus::Interrupted)
            | (TaskStatus::Interrupted, TaskStatus::Active)
            | (TaskStatus::Blocked, TaskStatus::Active)
    )
}

#[derive(Debug, Clone)]
pub struct PhaseEntry {
    pub status: TaskStatus,
    pub entered_at: DateTime<Utc>,
}

pub struct TaskLifecycle {
    current: TaskStatus,
    history: Vec<PhaseEntry>,
    interrupted: bool,
}

impl TaskLifecycle {
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            current: TaskStatus::Planning,
            history: vec![PhaseEntry {
                status: TaskStatus::Planning,
                entered_at: now,
            }],
            interrupted: false,
        }
    }

    pub fn status(&self) -> &TaskStatus {
        &self.current
    }

    pub fn was_interrupted(&self) -> bool {
        self.interrupted
    }

    pub fn history(&self) -> &[PhaseEntry] {
        &self.history
    }

    pub fn transition(&mut self, to: TaskStatus) -> Result<(), SpoolError> {
        if !is_valid_transition(&self.current, &to) {
            return Err(SpoolError::InvalidStateTransition {
                from: format!("{:?}", self.current),
                to: format!("{to:?}"),
            });
        }

        if to == TaskStatus::Interrupted {
            self.interrupted = true;
        }

        self.current = to.clone();
        self.history.push(PhaseEntry {
            status: to,
            entered_at: Utc::now(),
        });

        Ok(())
    }
}

impl Default for TaskLifecycle {
    fn default() -> Self {
        Self::new()
    }
}

// tests at bottom (from Step 1)
```

**Step 4: Run test to verify it passes**

Run: `cd spool && cargo test -p spool-core -- task_lifecycle`
Expected: 6 tests PASS

**Step 5: Commit**

```bash
git add spool/spool-core/src/task_lifecycle.rs
git commit -m "feat(spool-core): task lifecycle state machine with valid transitions and interruption tracking"
```

---

## Task 13: Persistence Layer

**Files:**

- Modify: `spool/spool-core/src/persistence.rs`

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use spool_protocol::artifact::ArtifactType;
    use spool_protocol::checkpoint::{CheckpointPolicy, CheckpointTrigger};
    use spool_protocol::contradiction::{
        ContradictionId, ContradictionRecord, ContradictionStatus, MaterialityLevel,
    };
    use spool_protocol::evidence::{EvidenceClass, EvidenceId, EvidenceItem, EvidenceType};
    use spool_protocol::task_contract::*;
    use spool_protocol::task_result::*;
    use tempfile::TempDir;

    fn sample_session_state() -> SessionState {
        let now = Utc::now();
        SessionState {
            session_id: "session_1".into(),
            created_at: now,
            selected_lob: "finance".into(),
            workspace_scope: "Executive BI".into(),
            task_contracts: vec![TaskContract {
                task_id: TaskId("task_1".into()),
                intent: "test".into(),
                scope: Scope {
                    lob: "finance".into(),
                    workspace: "Executive BI".into(),
                    artifacts: vec![ArtifactRef {
                        artifact_type: ArtifactType::Report,
                        reference: "Report".into(),
                    }],
                },
                selected_recipe: None,
                selected_recipe_selection_mode: None,
                assumptions: vec![],
                expected_evidence_classes: vec![EvidenceClass::ReportMetadata],
                validation_floor: ValidationFloor::DirectValidationRequired,
                checkpoint_policy: CheckpointPolicy {
                    ask_on: vec![CheckpointTrigger::Ambiguous],
                },
                clarification_checkpoints: vec![],
                approval_checkpoints: vec![],
                expected_deliverable_shape: "structured_task_result".into(),
                evaluator_packet_requirements: vec![],
                task_status: TaskStatus::Completed,
                created_at: Some(now),
                updated_at: None,
            }],
            task_results: vec![],
            evidence_items: vec![EvidenceItem {
                id: EvidenceId("ev_1".into()),
                evidence_type: EvidenceType::Observed,
                evidence_class: EvidenceClass::DaxQueryResult,
                source: "test".into(),
                summary: "test evidence".into(),
                artifact_refs: vec![],
                observed_at: Some(now),
                detail: None,
            }],
            contradiction_records: vec![],
            checkpoint_history: vec![],
        }
    }

    #[test]
    fn jsonl_round_trip() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("session.jsonl");
        let persistence = JsonlPersistence::new(path.clone());

        let state = sample_session_state();
        persistence.save(&state).unwrap();

        let restored = persistence.load().unwrap();
        assert_eq!(restored.session_id, state.session_id);
        assert_eq!(restored.task_contracts.len(), 1);
        assert_eq!(restored.evidence_items.len(), 1);
    }

    #[test]
    fn load_nonexistent_returns_error() {
        let persistence = JsonlPersistence::new("/nonexistent/path.jsonl".into());
        assert!(persistence.load().is_err());
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cd spool && cargo test -p spool-core -- persistence`
Expected: FAIL

**Step 3: Write the implementation**

```rust
// spool/spool-core/src/persistence.rs
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use spool_protocol::checkpoint::AskUserQuestion;
use spool_protocol::contradiction::ContradictionRecord;
use spool_protocol::evidence::EvidenceItem;
use spool_protocol::task_contract::TaskContract;
use spool_protocol::task_result::TaskResult;

use crate::error::SpoolError;

/// Session state for persistence and resume.
///
/// This is the structured state that enables resume and compaction
/// (Spec Section 12.2). Raw transcript history is separate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    pub session_id: String,
    pub created_at: DateTime<Utc>,
    pub selected_lob: String,
    pub workspace_scope: String,
    pub task_contracts: Vec<TaskContract>,
    pub task_results: Vec<TaskResult>,
    pub evidence_items: Vec<EvidenceItem>,
    pub contradiction_records: Vec<ContradictionRecord>,
    pub checkpoint_history: Vec<AskUserQuestion>,
}

/// Persistence provider trait.
pub trait PersistenceProvider {
    fn save(&self, state: &SessionState) -> Result<(), SpoolError>;
    fn load(&self) -> Result<SessionState, SpoolError>;
}

/// JSONL-based persistence implementation.
///
/// Writes the full session state as a single JSON document per save.
/// Future iterations may use append-only JSONL with indexed events.
pub struct JsonlPersistence {
    path: PathBuf,
}

impl JsonlPersistence {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl PersistenceProvider for JsonlPersistence {
    fn save(&self, state: &SessionState) -> Result<(), SpoolError> {
        let json = serde_json::to_string_pretty(state)?;
        std::fs::write(&self.path, json)?;
        Ok(())
    }

    fn load(&self) -> Result<SessionState, SpoolError> {
        let content = std::fs::read_to_string(&self.path)?;
        let state: SessionState = serde_json::from_str(&content)?;
        Ok(state)
    }
}

// tests at bottom (from Step 1)
```

**Step 4: Run test to verify it passes**

Run: `cd spool && cargo test -p spool-core -- persistence`
Expected: 2 tests PASS

**Step 5: Commit**

```bash
git add spool/spool-core/src/persistence.rs
git commit -m "feat(spool-core): persistence trait with JSONL implementation and session state round-trip"
```

---

## Task 14: Integration Scenarios

**Files:**

- Create: `spool/spool-core/tests/integration_scenarios.rs`

This task validates the full harness through end-to-end scenarios using fixtures only.

**Step 1: Write the integration tests**

```rust
// spool/spool-core/tests/integration_scenarios.rs
//!
//! End-to-end integration scenarios for Plan 1: Harness Semantics Foundation.
//!
//! These scenarios prove the core task/evidence/result semantics in isolation
//! from any live system, per the Plan 1 rule in the dev planning readiness doc.

use chrono::Utc;
use spool_core::contradiction_ledger::ContradictionLedger;
use spool_core::error::SpoolError;
use spool_core::evaluator_loop::{run_evaluator_loop, EvaluatorLoopConfig};
use spool_core::evidence_ledger::EvidenceLedger;
use spool_core::harness::fixtures::*;
use spool_core::harness::*;
use spool_core::persistence::{JsonlPersistence, PersistenceProvider, SessionState};
use spool_core::task_lifecycle::TaskLifecycle;
use spool_protocol::artifact::{ArtifactId, ArtifactType};
use spool_protocol::checkpoint::{CheckpointPolicy, CheckpointTrigger};
use spool_protocol::contradiction::{
    ContradictionId, ContradictionRecord, ContradictionStatus, MaterialityLevel,
};
use spool_protocol::evaluator::{
    EvaluatorOutcome, EvidenceTarget, InabilityReason, InabilityResponse,
};
use spool_protocol::evidence::{EvidenceClass, EvidenceId, EvidenceItem, EvidenceType};
use spool_protocol::task_contract::*;
use spool_protocol::task_result::*;

fn sample_contract() -> TaskContract {
    TaskContract {
        task_id: TaskId("task_scenario".into()),
        intent: "Find why report revenue does not match expected quarter totals".into(),
        scope: Scope {
            lob: "finance".into(),
            workspace: "Executive BI".into(),
            artifacts: vec![
                ArtifactRef {
                    artifact_type: ArtifactType::Report,
                    reference: "Executive Revenue Report".into(),
                },
                ArtifactRef {
                    artifact_type: ArtifactType::Measure,
                    reference: "Sales Model.Revenue".into(),
                },
            ],
        },
        selected_recipe: Some("report_number_mismatch".into()),
        selected_recipe_selection_mode: Some(RecipeSelectionMode::AutoSelect),
        assumptions: vec!["User refers to the published report in Executive BI".into()],
        expected_evidence_classes: vec![
            EvidenceClass::ReportMetadata,
            EvidenceClass::MeasureDefinition,
            EvidenceClass::DaxQueryResult,
            EvidenceClass::WarehouseQueryResult,
        ],
        validation_floor: ValidationFloor::DirectValidationRequired,
        checkpoint_policy: CheckpointPolicy {
            ask_on: vec![
                CheckpointTrigger::Ambiguous,
                CheckpointTrigger::ScopeExpanding,
            ],
        },
        clarification_checkpoints: vec![],
        approval_checkpoints: vec![],
        expected_deliverable_shape: "structured_task_result".into(),
        evaluator_packet_requirements: vec!["evidence_summary".into()],
        task_status: TaskStatus::Active,
        created_at: Some(Utc::now()),
        updated_at: None,
    }
}

fn make_evidence(id: &str, class: EvidenceClass, summary: &str) -> EvidenceItem {
    EvidenceItem {
        id: EvidenceId(id.into()),
        evidence_type: EvidenceType::Observed,
        evidence_class: class,
        source: "fixture".into(),
        summary: summary.into(),
        artifact_refs: vec![ArtifactId("art_1".into())],
        observed_at: Some(Utc::now()),
        detail: None,
    }
}

fn confirmed_output() -> GeneratorOutput {
    GeneratorOutput {
        proposed_state: Some(ResultState::Confirmed),
        summary: "Revenue mismatch traced to semantic-model measure logic".into(),
        findings: vec![Finding {
            id: "f_1".into(),
            title: "Revenue variance from measure logic".into(),
            detail: "The QoQ measure uses stale quarter offset logic".into(),
        }],
        evidence_collected: vec![],
        recommended_actions: vec![RecommendedAction {
            id: "a_1".into(),
            action_type: "proposed_model_change".into(),
            summary: "Update quarter-over-quarter revenue measure logic".into(),
        }],
        proposed_changes: vec![ProposedChange {
            id: "c_1".into(),
            artifact_type: ArtifactType::Measure,
            artifact_ref: "Sales Model.Sales[QoQ Revenue]".into(),
            change_summary: "Replace quarter offset logic".into(),
        }],
    }
}

// --- Scenario 1: Happy path → confirmed with high confidence ---

#[tokio::test]
async fn scenario_happy_path_confirmed() {
    let contract = sample_contract();

    // Lifecycle: planning → active → evaluating → completed
    let mut lifecycle = TaskLifecycle::new();
    lifecycle.transition(TaskStatus::Active).unwrap();

    // Seed evidence
    let mut evidence = EvidenceLedger::new();
    evidence.append(make_evidence(
        "ev_1",
        EvidenceClass::DaxQueryResult,
        "DAX returned 12.4M for Q1",
    ));
    evidence.append(make_evidence(
        "ev_2",
        EvidenceClass::WarehouseQueryResult,
        "Warehouse returned 12.4M for Q1",
    ));

    let mut contradictions = ContradictionLedger::new();

    // Generator produces output, evaluator accepts immediately
    let generator = FixtureGenerator::new(confirmed_output());
    let evaluator = FixtureEvaluator::new(vec![EvaluatorOutcome::Accept]);
    let config = EvaluatorLoopConfig { max_iterations: 3 };

    lifecycle.transition(TaskStatus::Evaluating).unwrap();

    let loop_result = run_evaluator_loop(
        &generator,
        &evaluator,
        &contract,
        &mut evidence,
        &mut contradictions,
        confirmed_output(),
        &config,
    )
    .await
    .unwrap();

    assert_eq!(loop_result.final_state, ResultState::Confirmed);
    assert_eq!(loop_result.final_confidence, Confidence::High);
    assert!(!loop_result.exhausted);
    assert_eq!(loop_result.iterations_used, 1);

    lifecycle.transition(TaskStatus::Completed).unwrap();

    // Build canonical task result
    let result = TaskResult {
        task_id: contract.task_id.clone(),
        proposed_state: loop_result.generator_proposed_state,
        state: loop_result.final_state,
        confidence: loop_result.final_confidence,
        summary: "Revenue mismatch traced to semantic-model measure logic".into(),
        findings: vec![Finding {
            id: "f_1".into(),
            title: "Revenue variance from measure logic".into(),
            detail: "The QoQ measure uses stale quarter offset logic".into(),
        }],
        evidence_refs: vec![EvidenceId("ev_1".into()), EvidenceId("ev_2".into())],
        validation_results: vec![ValidationResult {
            id: "val_1".into(),
            validation_type: "dax_and_warehouse_comparison".into(),
            status: "passed".into(),
            detail: "DAX and warehouse aligned".into(),
        }],
        recommended_actions: vec![RecommendedAction {
            id: "a_1".into(),
            action_type: "proposed_model_change".into(),
            summary: "Update QoQ measure logic".into(),
        }],
        blockers: vec![],
        open_questions: vec![],
        proposed_changes: vec![],
        contradiction_refs: vec![],
        result_generated_at: Some(Utc::now()),
        result_version: Some(1),
    };

    // Confidence caps should pass
    assert!(result.validate_confidence_caps().is_empty());
}

// --- Scenario 2: Evaluator requests evidence → generator collects → accept ---

#[tokio::test]
async fn scenario_evidence_request_then_accept() {
    let contract = sample_contract();
    let mut evidence = EvidenceLedger::new();
    let mut contradictions = ContradictionLedger::new();

    let new_evidence = make_evidence(
        "ev_new",
        EvidenceClass::MeasureDefinition,
        "Measure uses PREVIOUSQUARTER instead of DATEADD",
    );

    let generator = FixtureGenerator::new(confirmed_output()).with_evidence_responses(vec![
        EvidenceCollectionResult::Collected(new_evidence),
    ]);
    let evaluator = FixtureEvaluator::new(vec![
        EvaluatorOutcome::RequestMoreEvidence {
            targets: vec![EvidenceTarget {
                description: "Retrieve the measure definition for QoQ Revenue".into(),
                target_artifact: Some("art_measure_qoq".into()),
                target_evidence_class: Some("measure_definition".into()),
            }],
        },
        EvaluatorOutcome::Accept,
    ]);
    let config = EvaluatorLoopConfig { max_iterations: 5 };

    let result = run_evaluator_loop(
        &generator,
        &evaluator,
        &contract,
        &mut evidence,
        &mut contradictions,
        confirmed_output(),
        &config,
    )
    .await
    .unwrap();

    assert_eq!(result.final_state, ResultState::Confirmed);
    assert_eq!(result.iterations_used, 2);
    assert!(!result.exhausted);
    assert_eq!(evidence.len(), 1); // new evidence was appended
}

// --- Scenario 3: Loop exhaustion → supported_hypothesis ---

#[tokio::test]
async fn scenario_loop_exhaustion_downgrades() {
    let contract = sample_contract();
    let mut evidence = EvidenceLedger::new();
    let mut contradictions = ContradictionLedger::new();

    let generator = FixtureGenerator::new(confirmed_output()).with_evidence_responses(vec![
        EvidenceCollectionResult::Collected(make_evidence(
            "ev_1",
            EvidenceClass::DaxQueryResult,
            "partial result",
        )),
        EvidenceCollectionResult::Collected(make_evidence(
            "ev_2",
            EvidenceClass::DaxQueryResult,
            "another partial",
        )),
        EvidenceCollectionResult::Collected(make_evidence(
            "ev_3",
            EvidenceClass::DaxQueryResult,
            "yet another",
        )),
    ]);
    let evaluator = FixtureEvaluator::new(vec![
        EvaluatorOutcome::RequestMoreEvidence {
            targets: vec![EvidenceTarget {
                description: "need more".into(),
                target_artifact: None,
                target_evidence_class: None,
            }],
        },
        EvaluatorOutcome::RequestMoreEvidence {
            targets: vec![EvidenceTarget {
                description: "still not enough".into(),
                target_artifact: None,
                target_evidence_class: None,
            }],
        },
        EvaluatorOutcome::RequestMoreEvidence {
            targets: vec![EvidenceTarget {
                description: "keep going".into(),
                target_artifact: None,
                target_evidence_class: None,
            }],
        },
    ]);
    let config = EvaluatorLoopConfig { max_iterations: 3 };

    let result = run_evaluator_loop(
        &generator,
        &evaluator,
        &contract,
        &mut evidence,
        &mut contradictions,
        confirmed_output(),
        &config,
    )
    .await
    .unwrap();

    assert!(result.exhausted);
    // Spec Section 4.6: after loop exhaustion, final state must not be confirmed
    assert_ne!(result.final_state, ResultState::Confirmed);
    assert_eq!(result.final_state, ResultState::SupportedHypothesis);
}

// --- Scenario 4: Contradiction detected → prevents confirmed ---

#[tokio::test]
async fn scenario_contradiction_prevents_confirmed() {
    let contract = sample_contract();
    let mut evidence = EvidenceLedger::new();
    let mut contradictions = ContradictionLedger::new();

    let generator = FixtureGenerator::new(confirmed_output());
    let evaluator = FixtureEvaluator::new(vec![EvaluatorOutcome::Contradiction {
        description: "DAX says 12.4M but warehouse says 11.8M".into(),
        conflicting_sources: vec!["ev_dax".into(), "ev_warehouse".into()],
    }]);
    let config = EvaluatorLoopConfig { max_iterations: 1 };

    let result = run_evaluator_loop(
        &generator,
        &evaluator,
        &contract,
        &mut evidence,
        &mut contradictions,
        confirmed_output(),
        &config,
    )
    .await
    .unwrap();

    // Contradiction recorded in ledger
    assert_eq!(contradictions.all().len(), 1);
    assert!(contradictions.has_unresolved_material());

    // Cannot be confirmed with unresolved material contradiction
    assert_ne!(result.final_state, ResultState::Confirmed);

    // Build result and verify confidence caps catch this
    let task_result = TaskResult {
        task_id: contract.task_id.clone(),
        proposed_state: Some(ResultState::Confirmed),
        state: result.final_state,
        confidence: Confidence::High, // intentionally wrong — should be caught
        summary: "test".into(),
        findings: vec![],
        evidence_refs: vec![],
        validation_results: vec![],
        recommended_actions: vec![],
        blockers: vec![],
        open_questions: vec![],
        proposed_changes: vec![],
        contradiction_refs: vec![ContradictionId("contra_1".into())],
        result_generated_at: None,
        result_version: None,
    };

    let violations = task_result.validate_confidence_caps();
    assert!(!violations.is_empty());
}

// --- Scenario 5: Blocked task ---

#[tokio::test]
async fn scenario_blocked_task() {
    let contract = sample_contract();
    let mut evidence = EvidenceLedger::new();
    let mut contradictions = ContradictionLedger::new();

    let generator = FixtureGenerator::new(confirmed_output());
    let evaluator = FixtureEvaluator::new(vec![EvaluatorOutcome::Blocked {
        reason: "Cannot access the semantic model — insufficient permissions".into(),
    }]);
    let config = EvaluatorLoopConfig { max_iterations: 3 };

    let result = run_evaluator_loop(
        &generator,
        &evaluator,
        &contract,
        &mut evidence,
        &mut contradictions,
        confirmed_output(),
        &config,
    )
    .await
    .unwrap();

    assert_eq!(result.final_state, ResultState::Blocked);
    assert_eq!(result.final_confidence, Confidence::Low);
    assert!(!result.exhausted);
}

// --- Scenario 6: Generator/evaluator disagreement → evaluator wins ---

#[tokio::test]
async fn scenario_evaluator_authority_overrides_generator() {
    let contract = sample_contract();
    let mut evidence = EvidenceLedger::new();
    let mut contradictions = ContradictionLedger::new();

    // Generator proposes confirmed, evaluator downgrades
    let generator = FixtureGenerator::new(confirmed_output());
    let evaluator = FixtureEvaluator::new(vec![EvaluatorOutcome::Downgrade {
        reason: "Evidence is indirect — no like-for-like validation was performed".into(),
        suggested_state: Some("supported_hypothesis".into()),
    }]);
    let config = EvaluatorLoopConfig { max_iterations: 3 };

    let result = run_evaluator_loop(
        &generator,
        &evaluator,
        &contract,
        &mut evidence,
        &mut contradictions,
        confirmed_output(),
        &config,
    )
    .await
    .unwrap();

    // Evaluator wins per Spec Section 10.5
    assert_eq!(result.final_state, ResultState::SupportedHypothesis);
    // Generator's proposed state is still visible
    assert_eq!(
        result.generator_proposed_state,
        Some(ResultState::Confirmed)
    );
}

// --- Scenario 7: Full lifecycle with persistence round-trip ---

#[tokio::test]
async fn scenario_persist_and_restore_session() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join("session.jsonl");

    // Run a scenario
    let contract = sample_contract();
    let mut evidence = EvidenceLedger::new();
    evidence.append(make_evidence(
        "ev_1",
        EvidenceClass::DaxQueryResult,
        "DAX result",
    ));
    let contradictions = ContradictionLedger::new();

    // Persist
    let state = SessionState {
        session_id: "session_scenario_7".into(),
        created_at: Utc::now(),
        selected_lob: "finance".into(),
        workspace_scope: "Executive BI".into(),
        task_contracts: vec![contract],
        task_results: vec![],
        evidence_items: evidence.all().to_vec(),
        contradiction_records: contradictions.all().to_vec(),
        checkpoint_history: vec![],
    };

    let persistence = JsonlPersistence::new(path);
    persistence.save(&state).unwrap();

    // Restore and verify
    let restored = persistence.load().unwrap();
    assert_eq!(restored.session_id, "session_scenario_7");
    assert_eq!(restored.task_contracts.len(), 1);
    assert_eq!(restored.evidence_items.len(), 1);
    assert_eq!(
        restored.task_contracts[0].task_id,
        TaskId("task_scenario".into())
    );
}
```

**Step 2: Run tests to verify they fail**

Run: `cd spool && cargo test --test integration_scenarios`
Expected: FAIL (won't compile until all modules are properly exported)

**Step 3: Ensure all modules are properly exported in lib.rs**

Verify `spool/spool-core/src/lib.rs` exports all modules:

```rust
pub mod error;
pub mod evidence_ledger;
pub mod contradiction_ledger;
pub mod harness;
pub mod evaluator_loop;
pub mod task_lifecycle;
pub mod persistence;
```

**Step 4: Run tests to verify they pass**

Run: `cd spool && cargo test --test integration_scenarios`
Expected: 7 tests PASS

Then run the full test suite:

Run: `cd spool && cargo test`
Expected: all tests PASS (protocol + core + integration)

**Step 5: Commit**

```bash
git add spool/spool-core/tests/integration_scenarios.rs spool/spool-core/src/lib.rs
git commit -m "feat(spool-core): integration scenarios proving all harness semantics — happy path, evidence request, exhaustion, contradiction, blocked, authority, persistence"
```

---

## Summary

| Task | What it proves | Test count |
|------|---------------|------------|
| 1 | Workspace builds | 0 (build check) |
| 2 | Artifact identity model | 3 |
| 3 | Evidence types | 3 |
| 4 | Contradiction types | 2 |
| 5 | Evaluator + checkpoint types | 6 |
| 6 | Task contract schema | 2 |
| 7 | Task result + confidence caps | 6 |
| 8 | Evidence ledger | 4 |
| 9 | Contradiction ledger | 5 |
| 10 | Harness traits + fixtures | 2 |
| 11 | Evaluator loop protocol | 6 |
| 12 | Task lifecycle state machine | 6 |
| 13 | Persistence layer | 2 |
| 14 | Integration scenarios | 7 |
| **Total** | | **54** |
