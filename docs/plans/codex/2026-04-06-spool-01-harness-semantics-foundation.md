# Spool Harness Semantics Foundation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the greenfield Spool workspace backbone and prove the canonical harness semantics, including task contracts, evidence, contradictions, evaluator-loop behavior, checkpoints, and persisted resume state, without any live Fabric dependency.

**Architecture:** This plan creates a new Rust workspace with a narrow `spool-model` crate for stable product contracts and a `spool-core` crate for the bounded planner/generator/evaluator loop, fake adapters, and persistence seams. The plan keeps all validation local and deterministic so the core product semantics are proven before any real platform adapter, DAX, warehouse, or TUI work starts.

**Tech Stack:** Rust 2024, Tokio, Serde, serde_json, thiserror, chrono, uuid, pretty_assertions, insta

---

## Context For The Implementer

This is Plan 1. It owns the stable contract backbone for all later plans.

The governing references are:

- `docs/superpowers/specs/2026-04-06-spool-refined-spec.md`
- `docs/superpowers/specs/2026-04-06-spool-dev-planning-readiness-design.md`
- `docs/superpowers/specs/2026-04-07-spool-contradiction-handling-subspec.md`
- `docs/plans/codex/2026-04-06-spool-plan-index.md`

This plan creates the base workspace shape for the future standalone project:

```text
spool/
  Cargo.toml
  rust-toolchain.toml
  README.md
  docs/
    architecture/
      harness-foundation.md
  spool-model/
  spool-core/
```

## Out Of Scope

- live auth
- live Fabric artifact resolution
- live DAX execution
- live warehouse validation
- knowledge bundle loading
- planner recipe selection logic beyond the stored contract surface
- TUI rendering
- export adapters beyond persisted structured session state
- durable-memory managed storage and explicit authoring flows

## Dependencies

- none

## Contract Impact

This plan implements the governing contracts for:

- canonical artifact identity
- canonical task contract
- checkpoint policy and persisted Q&A history
- evidence ledger
- contradiction ledger
- canonical task result
- evaluator packet
- bounded evaluator-loop protocol
- confidence-cap enforcement
- persisted session, compaction, and resume state
- contradiction resolution tracking fields required by the April 7 contradiction subspec
- durable-memory and user-assertion evidence classes for contradiction participation
- task-record recipe-deviation tracking so later execution can explain departures from the selected approach

This plan implements those contracts directly. Later plans may consume them, but they should not weaken or silently redefine them.

## Requirement Traceability

| Requirement source | Required behavior in this plan | Planned coverage |
|---|---|---|
| Refined spec sections 3.2-3.5 | User inputs must normalize into canonical artifact identities and weak silent matches are not allowed | Task 2 defines artifact identity, locator, parentage, and resolution-basis rules |
| Refined spec harness semantics | Planner, generator, and evaluator must exchange canonical task, evidence, contradiction, and result contracts | Tasks 3-6 define the task contract, ledgers, evaluator packet, and bounded harness loop |
| Refined spec resume and compaction behavior | Persistence must restore exact task context, waiting state, and active evidence without replaying raw chat text | Task 8 defines persisted session state, compaction summaries, and resume markers |
| Planning spec section 4 | Plan 1 must prove harness semantics in isolation with deterministic or fixture-backed tests only | Tasks 1 and 6-8 use local contract tests, state-machine tests, and snapshot-backed persistence tests; no live Fabric gate belongs here |
| April 7 contradiction subspec | Contradiction records must support reopen, failed resolution, and explicit resolution-attempt metadata | Tasks 4, 5, and 7 carry contradiction fields, evaluator outcomes, and deterministic scenario coverage |

## Execution Invariants

- `ArtifactIdentity` is the only persisted authority for resolved artifacts. Display names and URLs may help derive identity, but they never replace the canonical locator and resolution basis after resolution.
- The harness loop is bounded and explicit. Planner, generator, and evaluator transitions must be represented as named phases and outcomes, not ad hoc flags.
- Evidence and contradiction ledgers are first-class structured state. They must remain inspectable without parsing generator prose.
- Waiting states are product behavior, not error cases. Pending user-input and pending-approval states must survive persistence and resume intact.
- Confidence and result-state caps are evaluator-governed rules over canonical evidence, not over hidden prompt text or transient transcript fragments.
- Recipe-deviation tracking belongs in the task/session record even before live recipe selection exists, because later plans need a durable explanation surface for "planned approach" versus "executed approach."

## Handoff Artifacts For Later Plans

By the end of this plan, later plans must be able to rely on these outputs without reinterpretation:

- `spool-model` exports for artifact identity, task contract, checkpoint records, evidence, contradictions, result states, evaluator outcomes, and session persistence
- `spool-core` ports and harness orchestration traits that later adapter, validation, and TUI crates can plug into
- deterministic fake implementations and fixture scenarios that later plans can reuse when they need harness behavior without live Fabric
- an architecture note that explains contract ownership, evaluator-loop boundaries, and resume/compaction assumptions

## Validation Strategy

This plan validates only with local tests and deterministic fixtures:

- domain serialization and roundtrip tests for canonical contracts
- rule-enforcement tests for result-state and confidence caps
- harness state-machine tests for planner/generator/evaluator transitions
- deterministic fixture scenarios for `accept`, `request_more_evidence`, `downgrade`, `blocked`, and `contradiction`
- persistence and compaction tests for interrupted and exhausted tasks
- JSON snapshot tests for persisted session payloads

No live external integration gate belongs in this plan.

## Open Items / Deferred Decisions

### Owned By This Plan

- exact Rust type boundaries between `spool-model` and `spool-core`
- exact evaluator-packet shape needed for the bounded evaluator pass
- exact persisted-state record boundaries between task history, live task status, pending interactions, evidence ledger, and compaction summary
- exact checkpoint and Q&A record shape for resume safety
- exact task-record shape for recipe-deviation capture

### Deferred To Later Plans

- Fabric auth and runtime adapter behavior
- live artifact resolution implementation
- live DAX and warehouse transports
- knowledge bundle loading and recipe selection policy
- TUI component tree and event loop
- export rendering
- durable-memory loading, explicit authoring, and review policy

### Review Triggers

- if the canonical task contract cannot express the spec-required evidence floor, checkpoint policy, or evaluator-packet requirements cleanly
- if the evaluator loop cannot represent all five evaluator outcomes without ad hoc flags
- if confidence caps require data that the canonical result object does not retain
- if persisted resume state cannot restore an interrupted evaluator request, a pending user-input or approval interaction, or a loop-exhausted unresolved result without replaying raw transcript history
- if contradiction records cannot retain `resolution_attempted` and `resolution_note` without parallel hidden state
- if task execution can deviate from a selected recipe but the persisted task record cannot explain what changed, why, and whether confidence moved

## File Structure

### Workspace bootstrap

| Path | Responsibility |
|---|---|
| `spool/Cargo.toml` | workspace manifest and shared dependencies |
| `spool/rust-toolchain.toml` | toolchain pin |
| `spool/README.md` | workspace bootstrap instructions |
| `spool/spool-model/Cargo.toml` | contract crate manifest |
| `spool/spool-model/src/lib.rs` | contract exports |
| `spool/spool-core/Cargo.toml` | harness crate manifest |
| `spool/spool-core/src/lib.rs` | harness exports |

### Contract modules

| Path | Responsibility |
|---|---|
| `spool/spool-model/src/artifact.rs` | artifact identity, kinds, locator strength, resolution basis |
| `spool/spool-model/src/task_contract.rs` | canonical task contract, scope, artifacts, validation floor |
| `spool/spool-model/src/checkpoint.rs` | checkpoint policy, checkpoint classes, AskUserQuestion records |
| `spool/spool-model/src/evidence.rs` | evidence item types, evidence classes, freshness metadata |
| `spool/spool-model/src/contradiction.rs` | structured contradiction records and resolution lifecycle |
| `spool/spool-model/src/result.rs` | result state, confidence, cap validation, evaluator-owned classification |
| `spool/spool-model/src/evaluator.rs` | evaluator outcomes, requested evidence targets, inability reasons, packet metadata |
| `spool/spool-model/src/session.rs` | persisted task/session state, compaction summary, resume markers |
| `spool/spool-model/tests/*.rs` | contract roundtrip and contract-rule tests |

### Harness modules

| Path | Responsibility |
|---|---|
| `spool/spool-core/src/ports.rs` | planner, generator, evaluator, and persistence traits |
| `spool/spool-core/src/harness.rs` | bounded harness orchestration and phase machine |
| `spool/spool-core/src/fakes.rs` | deterministic fake planner, generator, evaluator, persistence seams |
| `spool/spool-core/src/persistence.rs` | JSON persistence and compaction helpers |
| `spool/spool-core/tests/harness_flow.rs` | state-machine tests for bounded loop behavior |
| `spool/spool-core/tests/fixture_scenarios.rs` | deterministic end-to-end fixture scenarios |
| `spool/spool-core/tests/resume_flow.rs` | interrupted-session and loop-exhaustion restore tests |
| `spool/spool-core/tests/snapshots/*.snap` | persisted-state snapshots |

### Supporting docs

| Path | Responsibility |
|---|---|
| `spool/docs/architecture/harness-foundation.md` | short architecture note for the contract and harness seams |

### Task 1: Bootstrap The Greenfield Workspace

**Files:**
- Create: `spool/Cargo.toml`
- Create: `spool/rust-toolchain.toml`
- Create: `spool/README.md`
- Create: `spool/spool-model/Cargo.toml`
- Create: `spool/spool-model/src/lib.rs`
- Create: `spool/spool-core/Cargo.toml`
- Create: `spool/spool-core/src/lib.rs`
- Create: `spool/spool-core/tests/workspace_boot.rs`

- [ ] **Step 1: Write the failing workspace smoke test**

Create `spool/spool-core/tests/workspace_boot.rs`:

```rust
use spool_model::TaskId;

#[test]
fn workspace_boots_domain_crates() {
    let task_id = TaskId::new("task_boot");
    assert_eq!(task_id.as_str(), "task_boot");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:
```bash
cd spool
cargo test -p spool-core workspace_boots_domain_crates
```

Expected: FAIL because the workspace and crates do not exist yet.

- [ ] **Step 3: Write the minimal workspace bootstrap**

Create `spool/Cargo.toml`:

```toml
[workspace]
members = ["spool-model", "spool-core"]
resolver = "2"

[workspace.package]
edition = "2024"
version = "0.1.0"
license = "Apache-2.0"

[workspace.dependencies]
chrono = { version = "0.4", features = ["serde"] }
insta = "1"
pretty_assertions = "1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
tokio = { version = "1", features = ["macros", "rt-multi-thread", "sync"] }
uuid = { version = "1", features = ["serde", "v4"] }
```

Create `spool/rust-toolchain.toml`:

```toml
[toolchain]
channel = "stable"
components = ["clippy", "rustfmt"]
```

Create `spool/spool-model/Cargo.toml`:

```toml
[package]
name = "spool-model"
edition.workspace = true
version.workspace = true
license.workspace = true

[dependencies]
chrono.workspace = true
serde.workspace = true
uuid.workspace = true
```

Create `spool/spool-model/src/lib.rs`:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskId(String);

impl TaskId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
```

Create `spool/spool-core/Cargo.toml`:

```toml
[package]
name = "spool-core"
edition.workspace = true
version.workspace = true
license.workspace = true

[dependencies]
serde.workspace = true
serde_json.workspace = true
spool-model = { path = "../spool-model" }
thiserror.workspace = true
tokio.workspace = true

[dev-dependencies]
insta.workspace = true
pretty_assertions.workspace = true
```

Create `spool/spool-core/src/lib.rs`:

```rust
pub fn crate_ready() -> bool {
    true
}
```

Create `spool/README.md`:

```markdown
# Spool

Spool is a terminal-native analytics investigation agent for Microsoft Fabric and Power BI.

This workspace is greenfield and intentionally separate from `copex`.
```

- [ ] **Step 4: Run test to verify it passes**

Run:
```bash
cd spool
cargo test -p spool-core workspace_boots_domain_crates
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
cd spool
git add Cargo.toml rust-toolchain.toml README.md spool-model spool-core
git commit -m "feat: bootstrap spool workspace"
```

### Task 2: Define Canonical Artifact Identity

**Files:**
- Create: `spool/spool-model/src/artifact.rs`
- Modify: `spool/spool-model/src/lib.rs`
- Create: `spool/spool-model/tests/artifact_roundtrip.rs`

- [ ] **Step 1: Write the failing artifact roundtrip test**

Create `spool/spool-model/tests/artifact_roundtrip.rs`:

```rust
use spool_model::{ArtifactIdentity, ArtifactKind, ResolutionBasis};

#[test]
fn artifact_identity_roundtrips_with_resolution_context() {
    let identity = ArtifactIdentity::new(
        "art_report_exec_rev",
        ArtifactKind::Report,
        Some("ws_123"),
        None,
        "fabric://workspace/ws_123/report/rpt_456",
        "Executive Revenue Report",
        ResolutionBasis::ReportUrl,
    );

    let json = serde_json::to_string_pretty(&identity).unwrap();
    let restored: ArtifactIdentity = serde_json::from_str(&json).unwrap();

    assert_eq!(restored, identity);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:
```bash
cd spool
cargo test -p spool-model artifact_identity_roundtrips_with_resolution_context
```

Expected: FAIL because the artifact contract types do not exist.

- [ ] **Step 3: Implement the artifact identity contract**

Create `spool/spool-model/src/artifact.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactKind {
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

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResolutionBasis {
    ExplicitGuid,
    ReportUrl,
    ExactApiMatch,
    UniqueScopedName,
    DerivedChildLocator,
    RuntimeExecution,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ArtifactIdentity {
    pub artifact_id: String,
    pub artifact_type: ArtifactKind,
    pub workspace_id: Option<String>,
    pub parent_artifact_id: Option<String>,
    pub canonical_locator: String,
    pub display_name: String,
    pub resolution_basis: ResolutionBasis,
}

impl ArtifactIdentity {
    pub fn new(
        artifact_id: impl Into<String>,
        artifact_type: ArtifactKind,
        workspace_id: Option<&str>,
        parent_artifact_id: Option<&str>,
        canonical_locator: impl Into<String>,
        display_name: impl Into<String>,
        resolution_basis: ResolutionBasis,
    ) -> Self {
        Self {
            artifact_id: artifact_id.into(),
            artifact_type,
            workspace_id: workspace_id.map(str::to_string),
            parent_artifact_id: parent_artifact_id.map(str::to_string),
            canonical_locator: canonical_locator.into(),
            display_name: display_name.into(),
            resolution_basis,
        }
    }
}
```

Update `spool/spool-model/src/lib.rs`:

```rust
mod artifact;

pub use artifact::{ArtifactIdentity, ArtifactKind, ResolutionBasis};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskId(String);

impl TaskId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run:
```bash
cd spool
cargo test -p spool-model artifact_identity_roundtrips_with_resolution_context
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
cd spool
git add spool-model
git commit -m "feat: add spool canonical artifact identity"
```

### Task 3: Define Canonical Task Contract And Checkpoint Policy

**Files:**
- Create: `spool/spool-model/src/task_contract.rs`
- Create: `spool/spool-model/src/checkpoint.rs`
- Modify: `spool/spool-model/src/lib.rs`
- Create: `spool/spool-model/tests/task_contract_roundtrip.rs`

- [ ] **Step 1: Write the failing task-contract test**

Create `spool/spool-model/tests/task_contract_roundtrip.rs`:

```rust
use spool_model::{
    AskOn,
    ArtifactSelector,
    CheckpointPolicy,
    TaskContract,
    TaskScope,
    ValidationFloor,
};

#[test]
fn task_contract_roundtrips_with_validation_floor_and_checkpoints() {
    let contract = TaskContract {
        task_id: "task_123".into(),
        intent: "Find why the report revenue number does not match expected quarter totals.".into(),
        scope: TaskScope {
            lob: "finance".into(),
            workspace: "Executive BI".into(),
            artifacts: vec![
                ArtifactSelector {
                    artifact_type: "report".into(),
                    reference: "Executive Revenue Report".into(),
                },
            ],
        },
        selected_recipe: Some("report_number_mismatch".into()),
        selected_recipe_selection_mode: Some(RecipeSelectionMode::AutoSelect),
        selected_recipe_rationale: Some(
            "Strongest fit for a report-versus-model mismatch investigation.".into(),
        ),
        selected_recipe_user_preference: None,
        assumptions: vec!["The user means the published report.".into()],
        expected_evidence_classes: vec!["report_metadata".into(), "measure_definition".into()],
        validation_floor: ValidationFloor::DirectValidationRequired,
        checkpoint_policy: CheckpointPolicy {
            ask_on: vec![AskOn::Ambiguous, AskOn::ScopeExpanding],
        },
        clarification_checkpoints: vec![],
        approval_checkpoints: vec![],
        expected_deliverable_shape: "structured_result".into(),
        evaluator_packet_requirements: vec!["result_summary".into(), "evidence_ledger".into()],
    };

    let json = serde_json::to_string_pretty(&contract).unwrap();
    let restored: TaskContract = serde_json::from_str(&json).unwrap();

    assert_eq!(restored, contract);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:
```bash
cd spool
cargo test -p spool-model task_contract_roundtrips_with_validation_floor_and_checkpoints
```

Expected: FAIL because the task-contract and checkpoint types do not exist.

- [ ] **Step 3: Implement the task-contract and checkpoint types**

Create `spool/spool-model/src/task_contract.rs` and `spool/spool-model/src/checkpoint.rs` with:

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ArtifactSelector {
    pub artifact_type: String,
    pub reference: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TaskScope {
    pub lob: String,
    pub workspace: String,
    pub artifacts: Vec<ArtifactSelector>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationFloor {
    MetadataOnlyAllowed,
    DirectValidationPreferred,
    DirectValidationRequired,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecipeSelectionMode {
    AutoSelect,
    Suggest,
    UserRequestedOverride,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TaskContract {
    pub task_id: String,
    pub intent: String,
    pub scope: TaskScope,
    pub selected_recipe: Option<String>,
    pub selected_recipe_selection_mode: Option<RecipeSelectionMode>,
    pub selected_recipe_rationale: Option<String>,
    pub selected_recipe_user_preference: Option<String>,
    pub assumptions: Vec<String>,
    pub expected_evidence_classes: Vec<String>,
    pub validation_floor: ValidationFloor,
    pub checkpoint_policy: CheckpointPolicy,
    pub clarification_checkpoints: Vec<AskUserQuestion>,
    pub approval_checkpoints: Vec<AskUserQuestion>,
    pub expected_deliverable_shape: String,
    pub evaluator_packet_requirements: Vec<String>,
}
```

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AskOn {
    Ambiguous,
    ScopeExpanding,
    ExpectationShaping,
    SideEffecting,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CheckpointPolicy {
    pub ask_on: Vec<AskOn>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AskUserQuestion {
    pub question_id: String,
    pub prompt: String,
    pub options: Vec<String>,
    pub answer: Option<String>,
}
```

- [ ] **Step 4: Run test to verify it passes**

Run:
```bash
cd spool
cargo test -p spool-model task_contract_roundtrips_with_validation_floor_and_checkpoints
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
cd spool
git add spool-model
git commit -m "feat: add spool task contract and checkpoint policy"
```

### Task 4: Define Structured Evidence And Contradiction Ledgers

**Files:**
- Create: `spool/spool-model/src/evidence.rs`
- Create: `spool/spool-model/src/contradiction.rs`
- Modify: `spool/spool-model/src/lib.rs`
- Create: `spool/spool-model/tests/ledger_rules.rs`

- [ ] **Step 1: Write the failing ledger tests**

Create `spool/spool-model/tests/ledger_rules.rs`:

```rust
use spool_model::{
    ContradictionLedger,
    ContradictionMateriality,
    ContradictionStatus,
    EvidenceClass,
    EvidenceItem,
    EvidenceKind,
    EvidenceLedger,
    EvidenceFreshness,
};

#[test]
fn evidence_item_roundtrips_with_class_kind_and_freshness() {
    let item = EvidenceItem {
        evidence_id: "ev_1".into(),
        kind: EvidenceKind::Observed,
        class: EvidenceClass::ReportMetadata,
        summary: "Report metadata inspected".into(),
        artifact_refs: vec!["art_report_exec_rev".into()],
        freshness: EvidenceFreshness {
            observed_at: "2026-04-06T10:00:00Z".into(),
            freshness_note: "fresh".into(),
        },
    };

    let json = serde_json::to_string_pretty(&item).unwrap();
    let restored: EvidenceItem = serde_json::from_str(&json).unwrap();
    assert_eq!(restored, item);
}

#[test]
fn contradiction_ledger_retains_materiality_and_status() {
    let ledger = ContradictionLedger::single(
        "ctr_1",
        "Report total differs from warehouse total",
        vec!["ev_report".into(), "ev_wh".into()],
        ContradictionMateriality::Material,
        ContradictionStatus::Open,
    );

    assert_eq!(ledger.items[0].materiality, ContradictionMateriality::Material);
    assert_eq!(ledger.items[0].status, ContradictionStatus::Open);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:
```bash
cd spool
cargo test -p spool-model ledger_rules
```

Expected: FAIL because the evidence and contradiction ledgers do not exist.

- [ ] **Step 3: Implement the evidence and contradiction contracts**

Create `spool/spool-model/src/evidence.rs` and `spool/spool-model/src/contradiction.rs` with:

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceKind {
    Observed,
    Derived,
    Proposed,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceClass {
    ReportMetadata,
    VisualConfiguration,
    SemanticModelMetadata,
    MeasureDefinition,
    DaxQueryResult,
    WarehouseQueryResult,
    BusinessKnowledge,
    DurableMemory,
    UserAssertion,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct EvidenceFreshness {
    pub observed_at: String,
    pub freshness_note: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct EvidenceItem {
    pub evidence_id: String,
    pub kind: EvidenceKind,
    pub class: EvidenceClass,
    pub summary: String,
    pub artifact_refs: Vec<String>,
    pub freshness: EvidenceFreshness,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct EvidenceLedger {
    pub items: Vec<EvidenceItem>,
}
```

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContradictionMateriality {
    Material,
    NonMaterial,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContradictionStatus {
    Open,
    Resolved,
    CarriedForward,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ContradictionRecord {
    pub contradiction_id: String,
    pub disputed_claim: String,
    pub conflicting_evidence_refs: Vec<String>,
    pub materiality: ContradictionMateriality,
    pub freshness_note: String,
    pub resolution_attempted: bool,
    pub resolution_note: Option<String>,
    pub status: ContradictionStatus,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ContradictionLedger {
    pub items: Vec<ContradictionRecord>,
}

impl ContradictionLedger {
    pub fn single(
        contradiction_id: impl Into<String>,
        disputed_claim: impl Into<String>,
        conflicting_evidence_refs: Vec<String>,
        materiality: ContradictionMateriality,
        status: ContradictionStatus,
    ) -> Self {
        Self {
            items: vec![ContradictionRecord {
                contradiction_id: contradiction_id.into(),
                disputed_claim: disputed_claim.into(),
                conflicting_evidence_refs,
                materiality,
                freshness_note: "freshness unknown".into(),
                resolution_attempted: false,
                resolution_note: None,
                status,
            }],
        }
    }

    pub fn single_material_open() -> Self {
        Self::single(
            "ctr_open",
            "material contradiction remains",
            vec![],
            ContradictionMateriality::Material,
            ContradictionStatus::Open,
        )
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run:
```bash
cd spool
cargo test -p spool-model ledger_rules
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
cd spool
git add spool-model
git commit -m "feat: add spool evidence and contradiction ledgers"
```

### Task 5: Define Canonical Result Semantics And Evaluator Protocol

**Files:**
- Create: `spool/spool-model/src/result.rs`
- Create: `spool/spool-model/src/evaluator.rs`
- Modify: `spool/spool-model/src/lib.rs`
- Create: `spool/spool-model/tests/result_rules.rs`

- [ ] **Step 1: Write the failing result-rule tests**

Create `spool/spool-model/tests/result_rules.rs`:

```rust
use spool_model::{
    Blocker,
    CanonicalTaskResult,
    Confidence,
    ContradictionLedger,
    EvaluatorOutcome,
    EvidenceLedger,
    FinalAuthority,
    Finding,
    OpenQuestion,
    ProposedChange,
    RecommendedAction,
    ResultState,
    ValidationResult,
};

#[test]
fn evaluator_outcome_includes_all_five_protocol_variants() {
    let outcomes = vec![
        EvaluatorOutcome::Accept,
        EvaluatorOutcome::RequestMoreEvidence { requested_targets: vec!["inspect_visual_filters".into()] },
        EvaluatorOutcome::Downgrade { reason: "evidence insufficient".into() },
        EvaluatorOutcome::Blocked { reason: "missing access".into() },
        EvaluatorOutcome::Contradiction { reason: "material conflict remains".into() },
    ];

    assert_eq!(outcomes.len(), 5);
}

#[test]
fn confirmed_high_is_rejected_when_material_contradictions_remain() {
    let result = CanonicalTaskResult {
        task_id: "task_123".into(),
        state: ResultState::Confirmed,
        confidence: Confidence::High,
        summary: "Confirmed".into(),
        findings: vec![Finding::new("finding_1", "Confirmed finding", "Direct evidence supports the main claim.")],
        evidence_refs: vec![],
        validation_results: vec![ValidationResult::passed("val_1", "direct_validation", "Direct validation passed")],
        recommended_actions: vec![],
        blockers: vec![],
        open_questions: vec![],
        proposed_changes: vec![],
        evidence: EvidenceLedger { items: vec![] },
        contradiction_refs: vec!["contradiction_1".into()],
        proposed_state: None,
        final_authority: FinalAuthority::Evaluator,
    };

    let contradictions = ContradictionLedger::single_material_open();

    assert!(result.validate(&contradictions).is_err());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:
```bash
cd spool
cargo test -p spool-model result_rules
```

Expected: FAIL because the canonical result and evaluator types do not exist.

- [ ] **Step 3: Implement the result and evaluator contracts**

Create `spool/spool-model/src/result.rs` and `spool/spool-model/src/evaluator.rs` with:

```rust
use serde::{Deserialize, Serialize};

use crate::contradiction::ContradictionLedger;
use crate::evidence::EvidenceLedger;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResultState {
    Confirmed,
    SupportedHypothesis,
    Inconclusive,
    Blocked,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Confidence {
    High,
    Medium,
    Low,
}

impl Confidence {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::High => "high",
            Self::Medium => "medium",
            Self::Low => "low",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FinalAuthority {
    Evaluator,
}

impl FinalAuthority {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Evaluator => "evaluator",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Finding {
    pub id: String,
    pub title: String,
    pub detail: String,
}

impl Finding {
    pub fn new(id: impl Into<String>, title: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            detail: detail.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ValidationResult {
    pub id: String,
    pub kind: String,
    pub status: String,
    pub detail: String,
}

impl ValidationResult {
    pub fn passed(id: impl Into<String>, kind: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            kind: kind.into(),
            status: "passed".into(),
            detail: detail.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RecommendedAction {
    pub id: String,
    pub kind: String,
    pub summary: String,
}

impl RecommendedAction {
    pub fn new(id: impl Into<String>, kind: impl Into<String>, summary: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            kind: kind.into(),
            summary: summary.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Blocker {
    pub id: String,
    pub kind: String,
    pub summary: String,
}

impl Blocker {
    pub fn new(id: impl Into<String>, kind: impl Into<String>, summary: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            kind: kind.into(),
            summary: summary.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OpenQuestion {
    pub id: String,
    pub question: String,
}

impl OpenQuestion {
    pub fn new(id: impl Into<String>, question: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            question: question.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProposedChange {
    pub id: String,
    pub artifact_type: String,
    pub artifact_ref: String,
    pub change_summary: String,
}

impl ProposedChange {
    pub fn new(
        id: impl Into<String>,
        artifact_type: impl Into<String>,
        artifact_ref: impl Into<String>,
        change_summary: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            artifact_type: artifact_type.into(),
            artifact_ref: artifact_ref.into(),
            change_summary: change_summary.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CanonicalTaskResult {
    pub task_id: String,
    pub state: ResultState,
    pub confidence: Confidence,
    pub summary: String,
    pub findings: Vec<Finding>,
    pub evidence_refs: Vec<String>,
    pub validation_results: Vec<ValidationResult>,
    pub recommended_actions: Vec<RecommendedAction>,
    pub blockers: Vec<Blocker>,
    pub open_questions: Vec<OpenQuestion>,
    pub proposed_changes: Vec<ProposedChange>,
    pub evidence: EvidenceLedger,
    pub contradiction_refs: Vec<String>,
    pub proposed_state: Option<ResultState>,
    pub final_authority: FinalAuthority,
}

impl ResultState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Confirmed => "confirmed",
            Self::SupportedHypothesis => "supported_hypothesis",
            Self::Inconclusive => "inconclusive",
            Self::Blocked => "blocked",
        }
    }
}

impl CanonicalTaskResult {
    pub fn validate(&self, contradiction_ledger: &ContradictionLedger) -> Result<(), String> {
        let has_material_open_contradiction = contradiction_ledger.items.iter().any(|item| {
            matches!(item.materiality, crate::contradiction::ContradictionMateriality::Material)
                && matches!(item.status, crate::contradiction::ContradictionStatus::Open)
        });

        if matches!(self.state, ResultState::Confirmed)
            && matches!(self.confidence, Confidence::High)
            && has_material_open_contradiction
        {
            return Err("confirmed high cannot coexist with material open contradictions".into());
        }

        if !matches!(self.state, ResultState::Confirmed)
            && self.open_questions.is_empty()
            && self.blockers.is_empty()
            && self.recommended_actions.is_empty()
        {
            return Err("non-confirmed results must explain next steps, blockers, or unresolved questions".into());
        }

        Ok(())
    }
}
```

```rust
use serde::{Deserialize, Serialize};

use crate::checkpoint::AskUserQuestion;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InabilityReason {
    MissingAccess,
    MissingUserClarification,
    UnavailableArtifact,
    OutOfScope,
    PolicyBoundary,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvaluatorOutcome {
    Accept,
    RequestMoreEvidence { requested_targets: Vec<String> },
    Downgrade { reason: String },
    Blocked { reason: String },
    Contradiction { reason: String },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PendingInteractionKind {
    Approval,
    UserInput,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PendingInteraction {
    pub request_id: String,
    pub kind: PendingInteractionKind,
    pub summary: String,
    pub question: Option<AskUserQuestion>,
}
```

- [ ] **Step 4: Run test to verify it passes**

Run:
```bash
cd spool
cargo test -p spool-model result_rules
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
cd spool
git add spool-model
git commit -m "feat: add spool result semantics and evaluator protocol"
```

### Task 6: Implement The Bounded Harness Loop

**Files:**
- Create: `spool/spool-core/src/ports.rs`
- Create: `spool/spool-core/src/harness.rs`
- Modify: `spool/spool-core/src/lib.rs`
- Create: `spool/spool-core/tests/harness_flow.rs`

- [ ] **Step 1: Write the failing bounded-loop tests**

Create `spool/spool-core/tests/harness_flow.rs`:

```rust
use spool_core::{
    Harness,
    HarnessPhase,
    ScriptedEvaluator,
    ScriptedGenerator,
    StaticPlanner,
};

#[tokio::test]
async fn harness_completes_when_evaluator_accepts() {
    let mut harness = Harness::new(
        StaticPlanner::default(),
        ScriptedGenerator::accept_ready(),
        ScriptedEvaluator::accept(),
    );

    let result = harness.run("Why is revenue wrong?").await.unwrap();

    assert_eq!(harness.phase(), HarnessPhase::Completed);
    assert_eq!(result.final_authority.as_str(), "evaluator");
}

#[tokio::test]
async fn harness_waits_when_evaluator_requests_more_evidence() {
    let mut harness = Harness::new(
        StaticPlanner::default(),
        ScriptedGenerator::missing_evidence(),
        ScriptedEvaluator::request_more_evidence("inspect_visual_filters"),
    );

    harness.run("Investigate mismatch").await.unwrap();

    assert_eq!(harness.phase(), HarnessPhase::Generating);
    assert_eq!(harness.pending_evaluator_request(), Some("inspect_visual_filters"));
}

#[tokio::test]
async fn harness_waits_for_user_input_without_marking_task_complete() {
    let mut harness = Harness::new(
        StaticPlanner::default(),
        ScriptedGenerator::requires_user_input("Which revenue page should I inspect?"),
        ScriptedEvaluator::accept(),
    );

    harness.run("Investigate mismatch").await.unwrap();

    assert_eq!(harness.phase(), HarnessPhase::WaitingOnUserInput);
    assert_eq!(harness.latest_result(), None);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:
```bash
cd spool
cargo test -p spool-core harness_flow
```

Expected: FAIL because the harness and fake adapters do not exist.

- [ ] **Step 3: Implement the harness ports and phase machine**

Create `spool/spool-core/src/ports.rs` and `spool/spool-core/src/harness.rs` with:

```rust
use spool_model::{CanonicalTaskResult, EvaluatorOutcome, PendingInteraction, TaskContract};

#[derive(Debug, thiserror::Error)]
pub enum HarnessError {
    #[error("planner failed: {0}")]
    Planner(String),
    #[error("generator failed: {0}")]
    Generator(String),
    #[error("evaluator failed: {0}")]
    Evaluator(String),
}

pub trait PlannerPort {
    fn plan(&self, user_input: &str) -> Result<TaskContract, HarnessError>;
}

pub trait GeneratorPort {
    fn generate(&self, contract: &TaskContract) -> Result<GeneratorAdvance, HarnessError>;
}

pub trait EvaluatorPort {
    fn evaluate(&self, generated: &CanonicalTaskResult) -> Result<EvaluatorOutcome, HarnessError>;
}

pub enum GeneratorAdvance {
    Candidate(CanonicalTaskResult),
    PendingInteraction(PendingInteraction),
}
```

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HarnessPhase {
    Planning,
    Generating,
    Evaluating,
    WaitingOnApproval,
    WaitingOnUserInput,
    Completed,
    Interrupted,
}

pub struct Harness<P, G, E> {
    planner: P,
    generator: G,
    evaluator: E,
    phase: HarnessPhase,
    max_iterations: usize,
    pending_evaluator_request: Option<String>,
    latest_result: Option<CanonicalTaskResult>,
}
```

Implement accessors:

- `phase() -> HarnessPhase`
- `pending_evaluator_request() -> Option<&str>`
- `latest_result() -> Option<&CanonicalTaskResult>`

- [ ] **Step 4: Run test to verify it passes**

Run:
```bash
cd spool
cargo test -p spool-core harness_flow
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
cd spool
git add spool-core
git commit -m "feat: add spool bounded harness loop"
```

### Task 7: Add Deterministic Fixture Scenarios For All Evaluator Outcomes

**Files:**
- Create: `spool/spool-core/src/fakes.rs`
- Modify: `spool/spool-core/src/lib.rs`
- Create: `spool/spool-core/tests/fixture_scenarios.rs`

- [ ] **Step 1: Write the failing fixture-scenario tests**

Create `spool/spool-core/tests/fixture_scenarios.rs`:

```rust
use spool_core::{
    Harness,
    ScriptedEvaluator,
    ScriptedGenerator,
    StaticPlanner,
};
use spool_model::ResultState;

#[tokio::test]
async fn downgrade_scenario_ends_supported_hypothesis() {
    let mut harness = Harness::new(
        StaticPlanner::default(),
        ScriptedGenerator::leading_confirmed_claim(),
        ScriptedEvaluator::downgrade("claim too strong"),
    );

    let result = harness.run("Investigate mismatch").await.unwrap();

    assert_eq!(result.state, ResultState::SupportedHypothesis);
}

#[tokio::test]
async fn blocked_scenario_ends_blocked_with_low_confidence() {
    let mut harness = Harness::new(
        StaticPlanner::default(),
        ScriptedGenerator::blocked_by_missing_access(),
        ScriptedEvaluator::blocked("warehouse access missing"),
    );

    let result = harness.run("Cross-check warehouse").await.unwrap();

    assert_eq!(result.state, ResultState::Blocked);
    assert_eq!(result.confidence.as_str(), "low");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:
```bash
cd spool
cargo test -p spool-core fixture_scenarios
```

Expected: FAIL because the scripted fixture scenarios do not exist.

- [ ] **Step 3: Implement the scripted fakes and outcome fixtures**

Create `spool/spool-core/src/fakes.rs` with deterministic planner, generator, and evaluator scripts covering:

- `accept`
- `request_more_evidence`
- `downgrade`
- `blocked`
- `contradiction`
- loop exhaustion after repeated evidence requests
- contradiction scenario where the evaluator catches a missed contradiction and instructs the generator to record it on the next pass
- contradiction scenario where the evaluator reopens an improperly resolved contradiction through the `reason` string and the generator applies that instruction to the ledger

Expose those fixtures from `spool-core/src/lib.rs`.

- [ ] **Step 4: Run test to verify it passes**

Run:
```bash
cd spool
cargo test -p spool-core fixture_scenarios
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
cd spool
git add spool-core
git commit -m "feat: add spool evaluator outcome fixture scenarios"
```

### Task 8: Persist Structured Session State, Compaction, And Resume Markers

**Files:**
- Create: `spool/spool-model/src/session.rs`
- Create: `spool/spool-core/src/persistence.rs`
- Modify: `spool/spool-model/src/lib.rs`
- Modify: `spool/spool-core/src/lib.rs`
- Create: `spool/spool-core/tests/resume_flow.rs`
- Create: `spool/spool-core/tests/snapshots/persisted_session.snap`

- [ ] **Step 1: Write the failing resume and compaction tests**

Create `spool/spool-core/tests/resume_flow.rs`:

```rust
use spool_core::{CompactionStore, PersistedHarnessState};

#[test]
fn persisted_state_restores_active_generation_phase() {
    let state = PersistedHarnessState::active_generation(
        "task_123",
        "inspect_visual_filters",
    );

    let json = serde_json::to_string_pretty(&state).unwrap();
    let restored: PersistedHarnessState = serde_json::from_str(&json).unwrap();

    assert_eq!(restored, state);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:
```bash
cd spool
cargo test -p spool-core persisted_state_restores_active_generation_phase
```

Expected: FAIL because the persistence types do not exist.

- [ ] **Step 3: Implement the persisted-state and compaction contracts**

Create `spool/spool-model/src/session.rs` and `spool/spool-core/src/persistence.rs` with:

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkPhase {
    Planning,
    Generating,
    Evaluating,
    ResultFinalization,
}

impl WorkPhase {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planning => "planning",
            Self::Generating => "generating",
            Self::Evaluating => "evaluating",
            Self::ResultFinalization => "result_finalization",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskWaitingFlag {
    WaitingOnApproval,
    WaitingOnUserInput,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TaskLiveStatus {
    Active { waiting_flags: Vec<TaskWaitingFlag> },
    Interrupted,
    Completed,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CompactionSummary {
    pub active_task_id: Option<String>,
    pub current_phase: WorkPhase,
    pub evidence_summary: Vec<String>,
    pub contradiction_summary: Vec<String>,
    pub unresolved_questions: Vec<String>,
    pub active_artifact_focus: Vec<String>,
    pub recipe_deviation_summary: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RecipeDeviationRecord {
    pub recipe_id: String,
    pub recipe_label: String,
    pub changed_step: String,
    pub reason: String,
    pub supporting_evidence_refs: Vec<String>,
    pub confidence_changed: bool,
}
```

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PersistedHarnessState {
    pub task_id: String,
    pub task_status: TaskLiveStatus,
    pub phase: WorkPhase,
    pub pending_interactions: Vec<spool_model::PendingInteraction>,
    pub pending_evaluator_request: Option<String>,
    pub recipe_deviations: Vec<spool_model::RecipeDeviationRecord>,
    pub compaction_summary: spool_model::CompactionSummary,
}

pub struct CompactionStore;

impl PersistedHarnessState {
    pub fn active_generation(
        task_id: impl Into<String>,
        pending_request: impl Into<String>,
    ) -> Self {
        Self {
            task_id: task_id.into(),
            task_status: TaskLiveStatus::Active {
                waiting_flags: vec![],
            },
            phase: WorkPhase::Generating,
            pending_interactions: vec![],
            pending_evaluator_request: Some(pending_request.into()),
            recipe_deviations: vec![],
            compaction_summary: spool_model::CompactionSummary {
                active_task_id: Some("task_123".into()),
                current_phase: WorkPhase::Generating,
                evidence_summary: vec![],
                contradiction_summary: vec![],
                unresolved_questions: vec![],
                active_artifact_focus: vec![],
                recipe_deviation_summary: vec![],
            },
        }
    }
}
```

- [ ] **Step 4: Run the resume-flow test and snapshot**

Run:
```bash
cd spool
cargo test -p spool-core persisted_state_restores_active_generation_phase
cargo test -p spool-core resume_flow -- --nocapture
```

Expected: PASS, with a reviewed JSON snapshot for the persisted-state payload.

- [ ] **Step 5: Commit**

```bash
cd spool
git add spool-model spool-core
git commit -m "feat: add spool persisted session and compaction state"
```

### Task 9: Write The Harness Foundation Architecture Note

**Files:**
- Create: `spool/docs/architecture/harness-foundation.md`

- [ ] **Step 1: Write the architecture note**

Create `spool/docs/architecture/harness-foundation.md` describing:

- why `spool-model` owns stable product contracts
- why `spool-core` owns the bounded loop and fake adapters
- evaluator authority and why the generator does not finalize classification
- loop exhaustion behavior and why `confirmed` is forbidden after exhaustion
- what later plans may extend and what they must not redefine

- [ ] **Step 2: Review the note for contract drift**

Run:
```bash
cd spool
rg -n "confirmed|evaluator|loop exhaustion|checkpoint|compaction" docs/architecture/harness-foundation.md
```

Expected: the note explicitly mentions all five concepts.

- [ ] **Step 3: Commit**

```bash
cd spool
git add docs/architecture/harness-foundation.md
git commit -m "docs: add spool harness foundation architecture note"
```

## Self-Review Checklist

- Spec coverage: This plan covers artifact identity, task contract, checkpoint policy, evidence, contradiction, canonical result semantics, evaluator outcomes, bounded loop behavior, and persisted resume state.
- Placeholder scan: No step may collapse evaluator outcomes into booleans, contradiction records into strings, or compaction into transcript-tail only storage.
- Type consistency: Keep `ArtifactIdentity`, `TaskContract`, `CheckpointPolicy`, `EvidenceLedger`, `ContradictionLedger`, `CanonicalTaskResult`, `EvaluatorOutcome`, `PersistedHarnessState`, and `CompactionSummary` stable because Plans 2-6 consume them directly.
