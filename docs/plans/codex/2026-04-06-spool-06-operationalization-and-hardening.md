# Spool Operationalization And Hardening Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Finish the Spool v1 hardening layer, including canonical result exports, explicit durable-memory contracts and managed storage scaffolding, policy enforcement, telemetry, and end-to-end operator validation.

**Architecture:** This plan extends the existing contract backbone instead of redefining it. `spool-core` owns export and durable-memory contracts, storage, scoped lookup primitives, and memory-influence provenance records, `spool-otel` owns telemetry contracts and initialization, and the `spool` app crate owns operator-facing entrypoints that exercise the full workflow from plan mode to final result, export, telemetry, and resume.

**Tech Stack:** Rust 2024, Serde, serde_json, Markdown string rendering, OpenTelemetry, tracing, tracing-subscriber, Tokio, pretty_assertions

---

## Context For The Implementer

This plan hardens the system around the already-defined harness, adapter, validation, knowledge, and TUI layers.

It must add:

- canonical export projections
- durable-memory source, scope, storage, lookup, and review scaffolding
- canonical memory-influence provenance records for trace and TUI rendering
- policy-sensitive hardening around result claims
- telemetry surfaces that remain inspectable without leaking hidden reasoning
- a concrete end-to-end operational walkthrough

The governing references are:

- `docs/superpowers/specs/2026-04-06-spool-refined-spec.md`
- `docs/superpowers/specs/2026-04-06-spool-dev-planning-readiness-design.md`
- `docs/superpowers/specs/2026-04-07-spool-contradiction-handling-subspec.md`
- `docs/plans/codex/2026-04-06-spool-01-harness-semantics-foundation.md`
- `docs/plans/codex/2026-04-06-spool-03-validation-execution-paths.md`
- `docs/plans/codex/2026-04-06-spool-05-tui-and-session-ux.md`
- `docs/plans/codex/2026-04-06-spool-plan-index.md`

## Out Of Scope

- Fabric-side mutation
- remote persistence
- team-shared memory stores
- wiki or ticketing integrations
- non-terminal render surfaces

## Dependencies

- `docs/plans/codex/2026-04-06-spool-01-harness-semantics-foundation.md`
- `docs/plans/codex/2026-04-06-spool-02-fabric-adapter-foundations.md`
- `docs/plans/codex/2026-04-06-spool-03-validation-execution-paths.md`
- `docs/plans/codex/2026-04-06-spool-04-knowledge-and-indexing.md`
- `docs/plans/codex/2026-04-06-spool-05-tui-and-session-ux.md`

## Contract Impact

This plan extends the system around the existing contracts. It should not make durable memory outrank fresh runtime evidence, let export formats become new canonical authorities, or let telemetry leak hidden reasoning content. It also must not introduce silent auto-learning from task results in v1.
This plan also owns the operational reliability validation for evaluator-to-generator contradiction correction described in the April 7 contradiction subspec.

## Requirement Traceability

| Requirement source | Required behavior in this plan | Planned coverage |
|---|---|---|
| Refined spec section 2.1 | Structured evidence capture and auditable results must remain exportable and inspectable | Tasks 1 and 7 define canonical export projection and end-to-end operational validation |
| Refined spec proposal-first stance | Runtime policy must prevent overclaiming when evidence is weak, stale, or contradicted | Tasks 4 and 4B implement result-policy enforcement and contradiction-correction reliability checks |
| Refined spec durable-context direction | Durable memory may inform work, but it must remain explicit, inspectable, and subordinate to runtime evidence | Tasks 2 and 3 define memory contracts, filtering, and provenance records |
| Planning spec Plan 6 intent | This plan owns exports, durable memory, telemetry, and product hardening as one bounded subsystem | Tasks 1-8 cover export, memory, policy, telemetry, CLI entrypoints, operational validation, and documentation |
| Plan index live-validation rule | The final validation must run through the shared config-backed environment and assert canonical state, not just command success | Task 7’s walkthrough verifies export, telemetry, durable memory, and resume against the dev Fabric workspace |

## Execution Invariants

- Export formats are projections from canonical results. They must never become the place where missing canonical fields are invented or repaired.
- Durable memory is explicit operator-managed input. Task execution must never silently author, mutate, or promote memory entries.
- Memory influence is inspectable provenance, not a hidden heuristic. If a memory entry affected work, later TUI/export/telemetry paths must be able to point to the record directly.
- Policy checks run before the product may claim operational success. A successful export or telemetry write does not legitimize an overclaimed result state.
- Telemetry must describe inspectable session state and flow milestones without leaking hidden reasoning content.
- Contradiction-correction reliability is measured on ledger state and reopen behavior, not on nicer final prose.

## Live Walkthrough Inputs And Success Conditions

Task 7 uses the shared config contract from the plan index plus the outputs of Plans 1-5. The final walkthrough is complete only when one run proves:

- canonical results can be exported to Markdown and JSON without losing state, evidence, or recommended-action meaning
- explicit durable-memory entries load from the configured or overridden memory file and remain subordinate to runtime evidence
- result-policy checks reject invalid high-confidence or contradiction-ignoring states
- telemetry records session milestones without relying on hidden reasoning text
- resume after export and telemetry still aligns with the persisted canonical session state

## Handoff Artifacts For Later Plans

- export helpers and memory/policy APIs that future operational work can extend without changing authority boundaries
- telemetry primitives for local initialization and session event emission
- contradiction reliability tests that protect the April 7 subspec behavior from regression
- an architecture note capturing exports, memory, policy, and telemetry boundaries

## Integration Validation

Real validation gate:

- run one end-to-end task through the dev Fabric workspace from plan mode to final result
- export the result to Markdown and JSON
- record telemetry for the run
- load at least one explicit durable-memory entry from managed storage without letting it change the current result state
- resume the session and verify the exported result and telemetry still align with canonical state

## Open Items / Deferred Decisions

### Owned By This Plan

- exact Markdown and JSON export adapter shapes
- exact telemetry event and span set for v1
- exact durable-memory storage file shape and managed-entry metadata
- exact local path and config precedence for the v1 durable-memory source-of-truth file
- exact policy checks that should hard-fail versus downgrade confidence

### Deferred To Later Plans

- team-shared memory
- interactive durable-memory authoring UX
- wiki or ticketing export targets
- advanced governance and RBAC
- remote telemetry backends beyond local initialization

### Review Triggers

- if export adapters need to mutate canonical result types instead of projecting from them
- if telemetry requires leaking hidden reasoning to remain useful
- if durable memory cannot remain inspectable and subordinate to runtime evidence
- if v1 memory loading requires silent write-back from task execution
- if result hardening requires contract fields missing from Plans 1 and 3

## File Structure

| Path | Responsibility |
|---|---|
| `spool/spool-core/src/export.rs` | export adapters for canonical results |
| `spool/spool-core/src/memory.rs` | durable-memory contracts, storage helpers, scope filtering, and memory-influence records |
| `spool/spool-core/src/policy.rs` | confirmation and hardening rules |
| `spool/spool-core/tests/contradiction_roundtrip.rs` | realistic contradiction correction reliability tests |
| `spool/spool-core/tests/export_flow.rs` | export tests |
| `spool/spool-core/tests/memory_flow.rs` | durable-memory tests |
| `spool/spool-core/tests/policy_flow.rs` | policy enforcement tests |
| `spool/spool-otel/Cargo.toml` | telemetry crate manifest |
| `spool/spool-otel/src/lib.rs` | telemetry exports |
| `spool/spool-otel/src/session.rs` | session telemetry events and spans |
| `spool/spool-otel/src/init.rs` | telemetry initialization |
| `spool/spool-otel/tests/session_telemetry.rs` | telemetry tests |
| `spool/spool/src/main.rs` | app operational entrypoints |
| `spool/docs/architecture/operational-hardening.md` | architecture note for exports, memory, policy, and telemetry |

In v1, the source of truth for durable memory should be an explicit local operator-managed file loaded from config or a CLI override. Normal task execution must not write to that file.

### Task 1: Add Canonical Export Adapters

**Files:**
- Create: `spool/spool-core/src/export.rs`
- Modify: `spool/spool-core/src/lib.rs`
- Create: `spool/spool-core/tests/export_flow.rs`

- [ ] **Step 1: Write the failing export tests**

Create `spool/spool-core/tests/export_flow.rs`:

```rust
use spool_core::{export_result_json, export_result_markdown};
use spool_model::{
    Blocker,
    CanonicalTaskResult,
    Confidence,
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
fn markdown_export_includes_summary_state_and_evidence_count() {
    let result = CanonicalTaskResult {
        task_id: "task_demo".into(),
        state: ResultState::SupportedHypothesis,
        confidence: Confidence::Medium,
        summary: "Model logic likely explains the mismatch".into(),
        findings: vec![Finding::new("finding_1", "Likely semantic-model issue", "Warehouse and report disagree in a way consistent with model logic drift")],
        evidence_refs: vec!["ev_1".into()],
        validation_results: vec![ValidationResult::passed("val_1", "report_vs_warehouse", "Warehouse validation aligned with DAX review")],
        recommended_actions: vec![RecommendedAction::new("action_1", "review_measure", "Review the affected revenue measure definition")],
        blockers: vec![],
        open_questions: vec![OpenQuestion::new("question_1", "Does the report visual apply a hidden page-level filter?")],
        proposed_changes: vec![ProposedChange::new("change_1", "measure", "Sales[Revenue]", "Align measure logic with the warehouse definition")],
        evidence: EvidenceLedger { items: vec![] },
        contradiction_refs: vec![],
        proposed_state: Some(ResultState::Confirmed),
        final_authority: FinalAuthority::Evaluator,
    };

    let markdown = export_result_markdown(&result);

    assert!(markdown.contains("Model logic likely explains the mismatch"));
    assert!(markdown.contains("supported_hypothesis"));
}

#[test]
fn json_export_roundtrips_from_canonical_result() {
    let result = CanonicalTaskResult {
        task_id: "task_demo".into(),
        state: ResultState::Blocked,
        confidence: Confidence::Low,
        summary: "Warehouse access missing".into(),
        findings: vec![Finding::new("finding_1", "Access block", "Warehouse access is required before validation can proceed")],
        evidence_refs: vec![],
        validation_results: vec![],
        recommended_actions: vec![RecommendedAction::new("action_1", "request_access", "Grant warehouse read access for the validation step")],
        blockers: vec![Blocker::new("blocker_1", "missing_access", "Warehouse read access is missing")],
        open_questions: vec![],
        proposed_changes: vec![],
        evidence: EvidenceLedger { items: vec![] },
        contradiction_refs: vec![],
        proposed_state: None,
        final_authority: FinalAuthority::Evaluator,
    };

    let json = export_result_json(&result);
    let restored: CanonicalTaskResult = serde_json::from_str(&json).unwrap();

    assert_eq!(restored, result);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:
```bash
cd spool
cargo test -p spool-core export_flow
```

Expected: FAIL because the export helpers do not exist.

- [ ] **Step 3: Implement the export adapters**

Create `spool/spool-core/src/export.rs`:

```rust
use spool_model::CanonicalTaskResult;

pub fn export_result_markdown(result: &CanonicalTaskResult) -> String {
    format!(
        "# Spool Result\n\n## Summary\n{}\n\n## State\n{}\n\n## Evidence\n{}\n\n## Findings\n{}\n\n## Recommended Actions\n{}\n",
        result.summary,
        result.state.as_str(),
        result.evidence.items.len(),
        result.findings.len(),
        result.recommended_actions.len(),
    )
}

pub fn export_result_json(result: &CanonicalTaskResult) -> String {
    serde_json::to_string_pretty(result).unwrap()
}
```

- [ ] **Step 4: Run test to verify it passes**

Run:
```bash
cd spool
cargo test -p spool-core export_flow
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
cd spool
git add spool-core
git commit -m "feat: add spool result export adapters"
```

### Task 2: Add Durable-Memory Contracts And Scope Metadata

**Files:**
- Create: `spool/spool-core/src/memory.rs`
- Modify: `spool/spool-core/src/lib.rs`
- Create: `spool/spool-core/tests/memory_flow.rs`

- [ ] **Step 1: Write the failing durable-memory contract test**

Create `spool/spool-core/tests/memory_flow.rs`:

```rust
use spool_core::{DurableMemoryEntry, MemoryScope, MemoryStatus};

#[test]
fn memory_entry_tracks_scope_and_active_status() {
    let entry = DurableMemoryEntry::new(
        "mem_1",
        "recurring_issue_pattern",
        MemoryScope::Workspace("Executive BI".into()),
        "user_file",
        "2026-04-06T10:00:00Z",
    );

    assert_eq!(entry.status, MemoryStatus::Active);
    assert_eq!(entry.scope, MemoryScope::Workspace("Executive BI".into()));
    assert_eq!(entry.source_basis, "user_file");
    assert_eq!(entry.created_at, "2026-04-06T10:00:00Z");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:
```bash
cd spool
cargo test -p spool-core memory_entry_tracks_scope_and_active_status
```

Expected: FAIL because the durable-memory types do not exist.

- [ ] **Step 3: Implement the durable-memory contracts**

Create `spool/spool-core/src/memory.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum MemoryScope {
    Lob(String),
    Workspace(String),
    Team(String),
    Global,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum MemoryStatus {
    Active,
    Stale,
    Disabled,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DurableMemoryEntry {
    pub memory_id: String,
    pub memory_type: String,
    pub scope: MemoryScope,
    pub created_at: String,
    pub updated_at: Option<String>,
    pub last_validated_at: Option<String>,
    pub source_basis: String,
    pub status: MemoryStatus,
}
```

Implement `DurableMemoryEntry::new(...)`.

- [ ] **Step 4: Run test to verify it passes**

Run:
```bash
cd spool
cargo test -p spool-core memory_entry_tracks_scope_and_active_status
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
cd spool
git add spool-core
git commit -m "feat: add spool durable memory contracts"
```

### Task 3: Add Durable-Memory Loading, Storage, And Scope-Filtering Helpers

**Files:**
- Modify: `spool/spool-core/src/memory.rs`
- Append: `spool/spool-core/tests/memory_flow.rs`

- [ ] **Step 1: Write the failing storage and filtering tests**

Append to `spool/spool-core/tests/memory_flow.rs`:

```rust
use spool_core::{
    filter_memory_for_scope,
    load_memory_entries_json,
    mark_memory_stale,
    record_memory_influence,
};
use spool_model::WorkPhase;

#[test]
fn workspace_memory_filters_and_can_be_marked_stale() {
    let loaded = load_memory_entries_json(
        r#"[{
            "memory_id":"mem_1",
            "memory_type":"reference_source",
            "scope":{"Workspace":"Executive BI"},
            "created_at":"2026-04-06T10:00:00Z",
            "updated_at":null,
            "last_validated_at":null,
            "source_basis":"team_file",
            "status":"Active"
        }]"#,
    )
    .unwrap();
    let entry = DurableMemoryEntry::new(
        "mem_1",
        "reference_source",
        MemoryScope::Workspace("Executive BI".into()),
        "team_file",
        "2026-04-06T10:00:00Z",
    );

    let visible = filter_memory_for_scope(
        &loaded,
        &MemoryScope::Workspace("Executive BI".into()),
    );
    let stale = mark_memory_stale(entry);
    let influence = record_memory_influence(
        &visible[0],
        "Applied workspace naming convention during artifact resolution",
        WorkPhase::Planning,
    );

    assert_eq!(loaded.len(), 1);
    assert_eq!(visible.len(), 1);
    assert_eq!(stale.status, MemoryStatus::Stale);
    assert_eq!(influence.memory_id, "mem_1");
    assert_eq!(influence.phase, WorkPhase::Planning);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:
```bash
cd spool
cargo test -p spool-core workspace_memory_filters_and_can_be_marked_stale
```

Expected: FAIL because the storage and filtering helpers do not exist.

- [ ] **Step 3: Implement the storage and filtering helpers**

Extend `spool/spool-core/src/memory.rs` with:

- `load_memory_entries_json(input) -> Result<Vec<DurableMemoryEntry>, _>`
- `filter_memory_for_scope(entries, scope) -> Vec<DurableMemoryEntry>`
- `MemoryInfluenceRecord`
- `record_memory_influence(entry, reason, phase) -> MemoryInfluenceRecord`
- `mark_memory_stale(entry) -> DurableMemoryEntry`
- `disable_memory(entry) -> DurableMemoryEntry`

Rules:

- loading must parse explicit managed memory entries only and must not infer new ones from task history
- filtering must respect scope rather than treating all memory as global
- memory influence records must be canonical and inspectable so later TUI work can render provenance without reconstructing it from prose
- `MemoryInfluenceRecord` should define at least `memory_id`, `scope`, `source_basis`, `reason`, and `phase`
- stale and disabled entries remain inspectable
- these helpers must not silently create new durable-memory entries from task execution

- [ ] **Step 4: Run test to verify it passes**

Run:
```bash
cd spool
cargo test -p spool-core workspace_memory_filters_and_can_be_marked_stale
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
cd spool
git add spool-core
git commit -m "feat: add spool durable memory storage helpers"
```

### Task 4: Add Policy Hardening For Result Claims

**Files:**
- Create: `spool/spool-core/src/policy.rs`
- Modify: `spool/spool-core/src/lib.rs`
- Create: `spool/spool-core/tests/policy_flow.rs`

- [ ] **Step 1: Write the failing policy test**

Create `spool/spool-core/tests/policy_flow.rs`:

```rust
use spool_core::assert_result_policy;
use spool_model::{
    Blocker,
    CanonicalTaskResult,
    Confidence,
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
fn policy_rejects_confirmed_without_observed_evidence() {
    let result = CanonicalTaskResult {
        task_id: "task_123".into(),
        state: ResultState::Confirmed,
        confidence: Confidence::High,
        summary: "Confirmed".into(),
        findings: vec![Finding::new("finding_1", "Confirmed finding", "Observed evidence supports the conclusion")],
        evidence_refs: vec![],
        validation_results: vec![ValidationResult::passed("val_1", "direct_validation", "Observed validation passed")],
        recommended_actions: vec![RecommendedAction::new("action_1", "document", "Document the confirmed result")],
        blockers: vec![],
        open_questions: vec![],
        proposed_changes: vec![ProposedChange::new("change_1", "measure", "Sales[Revenue]", "No change required")],
        evidence: EvidenceLedger { items: vec![] },
        contradiction_refs: vec![],
        proposed_state: None,
        final_authority: FinalAuthority::Evaluator,
    };

    assert!(assert_result_policy(&result).is_err());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:
```bash
cd spool
cargo test -p spool-core policy_flow
```

Expected: FAIL because the policy helper does not exist.

- [ ] **Step 3: Implement the policy helper**

Create `spool/spool-core/src/policy.rs`:

```rust
use spool_model::CanonicalTaskResult;

pub fn assert_result_policy(result: &CanonicalTaskResult) -> Result<(), String> {
    if matches!(result.state, spool_model::ResultState::Confirmed)
        && result.evidence.items.is_empty()
    {
        return Err("confirmed results require observed evidence".into());
    }

    Ok(())
}
```

The helper must also:

- reject `confirmed` when any open material contradiction remains in the contradiction ledger
- reject `high` confidence when only stale evidence or durable-memory evidence supports the leading claim
- preserve downgrade-safe states such as `supported_hypothesis` when contradictions remain unresolved but the result is still useful

- [ ] **Step 4: Run test to verify it passes**

Run:
```bash
cd spool
cargo test -p spool-core policy_flow
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
cd spool
git add spool-core
git commit -m "feat: add spool result policy hardening"
```

### Task 4B: Validate Contradiction Correction Reliability

**Files:**
- Create: `spool/spool-core/tests/contradiction_roundtrip.rs`

- [ ] **Step 1: Write the failing contradiction-roundtrip tests**

Create `spool/spool-core/tests/contradiction_roundtrip.rs`:

```rust
use spool_core::run_realistic_contradiction_roundtrip;

#[tokio::test]
#[ignore = "requires realistic mock or real LLM-backed evaluator/generator pair"]
async fn evaluator_can_force_recording_of_a_missed_contradiction() {
    let outcome = run_realistic_contradiction_roundtrip("missed_contradiction").await.unwrap();

    assert!(outcome.final_ledger.items.iter().any(|item| item.contradiction_id == "ctr_missed_1"));
}

#[tokio::test]
#[ignore = "requires realistic mock or real LLM-backed evaluator/generator pair"]
async fn evaluator_can_reopen_an_improperly_resolved_contradiction() {
    let outcome = run_realistic_contradiction_roundtrip("reopen_invalid_resolution").await.unwrap();

    let contradiction = outcome
        .final_ledger
        .items
        .iter()
        .find(|item| item.contradiction_id == "ctr_reopen_1")
        .unwrap();

    assert!(contradiction.resolution_attempted);
    assert!(matches!(contradiction.status, spool_model::ContradictionStatus::Open));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:
```bash
cd spool
cargo test -p spool-core contradiction_roundtrip -- --ignored
```

Expected: FAIL because the realistic contradiction-roundtrip harness does not exist.

- [ ] **Step 3: Implement the realistic contradiction-roundtrip harness**

Implement `run_realistic_contradiction_roundtrip(scenario)` in `spool-core` test support so the ignored tests execute scenarios 7.4 and 7.5 from the contradiction subspec with either a real LLM pair or a realistic mock LLM pair.

The harness must assert on ledger state, not just final result text:

- scenario `missed_contradiction` must end with a newly recorded contradiction
- scenario `reopen_invalid_resolution` must end with the contradiction reopened and annotated through canonical contradiction fields

- [ ] **Step 4: Run test to verify it passes**

Run:
```bash
cd spool
cargo test -p spool-core contradiction_roundtrip -- --ignored --nocapture
```

Expected: PASS, with the evaluator `reason` string causing the generator to update the correct contradiction record.

- [ ] **Step 5: Commit**

```bash
cd spool
git add spool-core
git commit -m "feat: validate spool contradiction correction roundtrip"
```

### Task 5: Add The Telemetry Crate And Session Events

**Files:**
- Modify: `spool/Cargo.toml`
- Create: `spool/spool-otel/Cargo.toml`
- Create: `spool/spool-otel/src/lib.rs`
- Create: `spool/spool-otel/src/session.rs`
- Create: `spool/spool-otel/src/init.rs`
- Create: `spool/spool-otel/tests/session_telemetry.rs`

- [ ] **Step 1: Write the failing telemetry test**

Create `spool/spool-otel/tests/session_telemetry.rs`:

```rust
use spool_otel::SessionTelemetryEvent;

#[test]
fn telemetry_event_tracks_phase_workspace_and_task() {
    let event = SessionTelemetryEvent::new("evaluating", "Executive BI", "task_123");

    assert_eq!(event.phase, "evaluating");
    assert_eq!(event.workspace, "Executive BI");
    assert_eq!(event.task_id, "task_123");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:
```bash
cd spool
cargo test -p spool-otel telemetry_event_tracks_phase_workspace_and_task
```

Expected: FAIL because the telemetry crate does not exist.

- [ ] **Step 3: Implement the telemetry crate**

Create `spool/spool-otel/src/session.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SessionTelemetryEvent {
    pub phase: String,
    pub workspace: String,
    pub task_id: String,
}

impl SessionTelemetryEvent {
    pub fn new(
        phase: impl Into<String>,
        workspace: impl Into<String>,
        task_id: impl Into<String>,
    ) -> Self {
        Self {
            phase: phase.into(),
            workspace: workspace.into(),
            task_id: task_id.into(),
        }
    }
}
```

Create `spool/spool-otel/src/init.rs`:

```rust
pub fn init_telemetry() {
    tracing_subscriber::fmt().with_target(false).init();
}
```

Expose both from `spool/spool-otel/src/lib.rs`.

- [ ] **Step 4: Run test to verify it passes**

Run:
```bash
cd spool
cargo test -p spool-otel telemetry_event_tracks_phase_workspace_and_task
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
cd spool
git add Cargo.toml spool-otel
git commit -m "feat: add spool telemetry crate"
```

### Task 6: Add CLI Operational Entry Points

**Files:**
- Modify: `spool/spool/src/main.rs`
- Create: `spool/spool/tests/cli_export.rs`

- [ ] **Step 1: Write the failing CLI export test**

Create `spool/spool/tests/cli_export.rs`:

```rust
#[test]
fn cli_accepts_export_flag_and_memory_override() {
    let args = clap::Parser::try_parse_from([
        "spool",
        "--config",
        "dev.toml",
        "--export-json",
        "--memory-file",
        "memory.json",
    ])
    .unwrap();
    let rendered = format!("{args:?}");

    assert!(rendered.contains("export_json"));
    assert!(rendered.contains("memory_file"));
}

#[test]
fn cli_memory_override_wins_over_config_default() {
    let cfg = load_config("dev.toml").unwrap();
    let resolved = resolve_memory_file(&cfg, Some("override.json".into()));

    assert_eq!(resolved.as_deref(), Some("override.json"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:
```bash
cd spool
cargo test -p spool cli_accepts_export_flag_and_memory_override
cargo test -p spool cli_memory_override_wins_over_config_default
```

Expected: FAIL because the CLI args and config-precedence helpers do not exist.

- [ ] **Step 3: Implement the CLI operational flags**

Extend `spool/spool/src/main.rs` so the app supports:

- `--config <path>`
- `--export-json`
- `--export-markdown`
- `--memory-file <path>`
- `--resume`

Keep the behavior simple in this plan: wire the flags into the operational flow without inventing new runtime contracts. The loaded config must support a default `memory_file` path, and `--memory-file` should override that configured durable-memory file path.

Implement:

- config loading that can read a default `memory_file`
- `resolve_memory_file(config, cli_override)` precedence helper

- [ ] **Step 4: Run test to verify it passes**

Run:
```bash
cd spool
cargo test -p spool cli_accepts_export_flag_and_memory_override
cargo test -p spool cli_memory_override_wins_over_config_default
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
cd spool
git add spool
git commit -m "feat: add spool operational cli flags"
```

### Task 7: Run The End-To-End Operational Validation

**Files:**
- No new code files required

- [ ] **Step 1: Run the staged validation sequence**

Run:
```bash
cd spool
cargo test -p spool-core
cargo test -p spool-core contradiction_roundtrip -- --ignored --nocapture
cargo test -p spool-fabric -- --ignored
cargo test -p spool-knowledge
cargo test -p spool-index -- --ignored
cargo test -p spool-tui
cargo test -p spool-otel
```

Expected: PASS, with ignored tests used for the real dev Fabric integration gates and the contradiction round-trip reliability scenarios.

- [ ] **Step 2: Run the end-to-end operator walkthrough**

Run:
```bash
cd spool
cargo run -p spool -- --config ~/.config/spool/dev.toml --export-json --export-markdown --resume
```

Expected: the operator can load config, connect to the dev Fabric workspace, emit a canonical result, export JSON and Markdown, initialize telemetry, load explicit durable-memory entries, and resume the session without contract drift.

- [ ] **Step 3: Commit**

```bash
cd spool
git add .
git commit -m "feat: harden spool operational surfaces"
```

### Task 8: Write The Operational Hardening Architecture Note

**Files:**
- Create: `spool/docs/architecture/operational-hardening.md`

- [ ] **Step 1: Write the architecture note**

Create `spool/docs/architecture/operational-hardening.md` describing:

- why export adapters project from canonical results
- why durable memory remains explicit and subordinate to runtime evidence
- why policy checks run before operational success is reported
- why telemetry tracks inspectable session state rather than hidden reasoning
- what future plans may extend without redefining these operational boundaries

- [ ] **Step 2: Review the note for missing concepts**

Run:
```bash
cd spool
rg -n "canonical results|runtime evidence|policy|telemetry|boundaries" docs/architecture/operational-hardening.md
```

Expected: the note explicitly mentions all five concepts.

- [ ] **Step 3: Commit**

```bash
cd spool
git add docs/architecture/operational-hardening.md
git commit -m "docs: add spool operational hardening architecture note"
```

## Self-Review Checklist

- Spec coverage: This plan covers canonical exports, durable-memory scope and managed-storage scaffolding, policy hardening, telemetry, operational CLI flags, and end-to-end validation.
- Placeholder scan: No step may reduce exports to summary-only text, reduce durable memory to status-only storage, introduce silent auto-learning, or treat telemetry as an unstructured debug blob.
- Type consistency: Keep `DurableMemoryEntry`, `MemoryScope`, `MemoryStatus`, `MemoryInfluenceRecord`, `SessionTelemetryEvent`, `assert_result_policy`, memory filtering helper names, and export helper names stable because later polish and integration work consume them directly.
