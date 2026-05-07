# Spool TUI And Session UX Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the terminal-native Spool operator experience, including plan mode, checkpoint Q&A, live progress rendering, advanced evidence and disagreement views, structured compaction, and resume restoration.

**Architecture:** This plan introduces a dedicated `spool-tui` crate with a real Ratatui and Crossterm application loop, explicit app state, widget-focused render modules, and a projection layer that derives render state from canonical harness data. The TUI remains a consumer of canonical task, evidence, contradiction, validation, and knowledge contracts, including the selected-LOB knowledge projection from Plan 4, rather than becoming a parallel source of truth.

**Tech Stack:** Rust 2024, Ratatui, Crossterm, Tokio, insta, pretty_assertions

---

## Context For The Implementer

This plan turns the contract and validation work from Plans 1-4 into an operator-facing terminal application.

The TUI must support:

- plan mode for clarifying task scope
- checkpoint questions
- progress and active artifact focus
- advanced rendering for evidence, contradictions, evaluator disagreement, and durable-memory influence provenance
- structured compaction and resume
- real session walkthroughs against the dev Fabric workspace

The governing references are:

- `docs/superpowers/specs/2026-04-06-spool-refined-spec.md`
- `docs/superpowers/specs/2026-04-06-spool-dev-planning-readiness-design.md`
- `docs/superpowers/specs/2026-04-07-spool-contradiction-handling-subspec.md`
- `docs/plans/codex/2026-04-06-spool-01-harness-semantics-foundation.md`
- `docs/plans/codex/2026-04-06-spool-03-validation-execution-paths.md`
- `docs/plans/codex/2026-04-06-spool-04-knowledge-and-indexing.md`
- `docs/plans/codex/2026-04-06-spool-plan-index.md`

## Out Of Scope

- export adapters
- durable-memory authoring UX
- remote or headless mode
- Fabric-side mutation workflows
- non-terminal render surfaces

## Dependencies

- `docs/plans/codex/2026-04-06-spool-01-harness-semantics-foundation.md`
- `docs/plans/codex/2026-04-06-spool-02-fabric-adapter-foundations.md`
- `docs/plans/codex/2026-04-06-spool-03-validation-execution-paths.md`
- `docs/plans/codex/2026-04-06-spool-04-knowledge-and-indexing.md`

## Contract Impact

This plan consumes existing task, evidence, contradiction, validation, knowledge-projection, and durable-memory provenance contracts without changing ownership rules.

It may introduce view-state and projection types, but those must derive from canonical state rather than becoming new authorities. When recipe-selection state is rendered, the TUI should present a user-facing investigation approach summary rather than exposing raw internal recipe IDs as the primary UX.

## Requirement Traceability

| Requirement source | Required behavior in this plan | Planned coverage |
|---|---|---|
| Refined spec section 2.1 | Chat-first terminal UX, plan mode, and resumable sessions are in scope for v1 | Tasks 1, 3, 5, 7, and 8 define app state, plan mode, resume views, event flow, and the real session walkthrough |
| Refined spec evidence and contradiction behavior | Operators need inspectable evidence, contradictions, and evaluator disagreement without hidden reasoning leakage | Tasks 4 and 6 define advanced views and projection helpers from canonical contradiction, validation, and provenance records |
| Planning spec plan intent for Plan 5 | This plan owns plan mode, progress, advanced view, compaction, and resume rendering with user-facing approach summaries | Tasks 2-8 cover each of those render surfaces and the live interaction path |
| Plan index contradiction and recipe rules | User-facing UX must describe approaches in task language and stay aligned with contradiction-handling semantics | Tasks 3, 4, 5, and 8 render approach summaries, contradiction resolution details, and waiting states from canonical fields |

## Execution Invariants

- `spool-tui` is a projection layer over canonical state. It must not become a second workflow engine with its own hidden interpretation of task progress.
- Plan mode presents the chosen investigation approach in operator language. Raw recipe IDs may exist internally, but they are not acceptable as the primary UI label.
- Advanced view may expose evidence counts, contradiction details, disagreement summaries, and durable-memory provenance only from canonical records. It must not mine hidden prompt text or chain-of-thought.
- Resume behavior is exact restoration of canonical pending state. A banner that says "session can be resumed" is insufficient unless the TUI can restore pending requests, active focus, and waiting flags.
- Snapshot tests must cover every user-visible surface added by this plan, because the TUI is one of the highest-regression areas in the product.

## Live Walkthrough Inputs And Success Conditions

Task 8 uses the shared config and fixture contract from the plan index plus outputs from Plans 1-4. The live walkthrough is complete only when one real run demonstrates:

- plan mode can render a user-facing scope and approach summary for the configured workspace/report fixture
- a live task transitions into a checkpoint state that requires user input
- an interrupted session resumes with the same active artifact focus and waiting flags
- advanced view renders contradiction detail and memory provenance from canonical records, not placeholders
- the real app shell remains consistent with the snapshot-covered projection logic

## Handoff Artifacts For Later Plans

- stable view structs and projection helpers that Plan 6 can reuse for export, telemetry, and operational polish
- snapshot suites that lock the intended rendering behavior
- an architecture note explaining the TUI ownership boundary and why canonical state remains the source of truth

## Integration Validation

Primary integration gate:

- run a real session against the dev Fabric workspace from plan mode through at least one completed task, one waiting-on-user-input state, and one interrupted task
- prove the progress surface updates phase and artifact focus correctly
- prove advanced view exposes evidence, contradictions, evaluator disagreement, and durable-memory influence provenance without leaking hidden reasoning
- prove resume restores pending evaluator requests, pending user-input or approval interactions, and active artifact focus from persisted state

Secondary validation:

- snapshot coverage for plan mode, progress view, advanced view, compaction summary, and interrupted resume states

## Open Items / Deferred Decisions

### Owned By This Plan

- exact component tree
- exact keybindings for advanced view, resume affordances, and plan-mode navigation
- exact layout split between progress, transcript, and advanced panes
- exact compaction summary presentation

### Deferred To Later Plans

- export workflow UI
- team or shared storage UX
- remote or headless operation

### Review Triggers

- if the event model cannot surface evaluator disagreement without reaching into hidden internal transcript state
- if advanced view requires non-canonical transport payloads instead of canonical projections
- if interrupted-session resume cannot restore pending evaluator requests, pending user-input or approval interactions, or active context from Plan 1 persisted state
- if the selected-LOB knowledge projection is too large for the default terminal layout without another compaction layer
- if durable-memory provenance cannot be rendered from canonical trace or projection data

## File Structure

| Path | Responsibility |
|---|---|
| `spool/spool-tui/Cargo.toml` | TUI crate manifest |
| `spool/spool-tui/src/lib.rs` | crate exports |
| `spool/spool-tui/src/app.rs` | app state, event loop, and top-level orchestration |
| `spool/spool-tui/src/events.rs` | keyboard and app event definitions |
| `spool/spool-tui/src/layout.rs` | top-level layout calculations |
| `spool/spool-tui/src/projections.rs` | canonical-state to view-state projections |
| `spool/spool-tui/src/plan_mode.rs` | plan-mode state and question widgets |
| `spool/spool-tui/src/progress.rs` | progress widget rendering |
| `spool/spool-tui/src/advanced_view.rs` | evidence, contradictions, and disagreement widgets |
| `spool/spool-tui/src/resume.rs` | compaction and resume widgets |
| `spool/spool-tui/tests/render_snapshots.rs` | snapshot tests |
| `spool/spool-tui/tests/app_flow.rs` | event-loop flow tests |
| `spool/docs/architecture/tui-session-ux.md` | architecture note for the TUI state model |

### Task 1: Create The TUI Crate And App-State Skeleton

**Files:**
- Modify: `spool/Cargo.toml`
- Create: `spool/spool-tui/Cargo.toml`
- Create: `spool/spool-tui/src/lib.rs`
- Create: `spool/spool-tui/src/app.rs`
- Create: `spool/spool-tui/src/events.rs`
- Create: `spool/spool-tui/tests/app_flow.rs`

- [ ] **Step 1: Write the failing app-state test**

Create `spool/spool-tui/tests/app_flow.rs`:

```rust
use spool_tui::{AppMode, AppState};

#[test]
fn app_starts_in_plan_mode() {
    let state = AppState::default();

    assert_eq!(state.mode, AppMode::Plan);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:
```bash
cd spool
cargo test -p spool-tui app_starts_in_plan_mode
```

Expected: FAIL because the TUI crate does not exist.

- [ ] **Step 3: Implement the crate and app-state skeleton**

Create `spool/spool-tui/src/app.rs`:

```rust
use crate::LayoutState;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AppMode {
    Plan,
    Session,
    Resume,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppState {
    pub mode: AppMode,
    pub layout: LayoutState,
    pub resume_available: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            mode: AppMode::Plan,
            layout: LayoutState::default(),
            resume_available: false,
        }
    }
}
```

Create `spool/spool-tui/src/events.rs`:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AppEvent {
    Tick,
    Input(char),
    Escape,
}
```

Create `spool/spool-tui/src/lib.rs`:

```rust
mod app;
mod events;
mod layout;

pub use app::{AppMode, AppState};
pub use events::AppEvent;
pub use layout::LayoutState;
```

- [ ] **Step 4: Run test to verify it passes**

Run:
```bash
cd spool
cargo test -p spool-tui app_starts_in_plan_mode
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
cd spool
git add Cargo.toml spool-tui
git commit -m "feat: add spool tui app state skeleton"
```

### Task 2: Add Layout And Progress Widgets

**Files:**
- Create: `spool/spool-tui/src/layout.rs`
- Create: `spool/spool-tui/src/progress.rs`
- Modify: `spool/spool-tui/src/lib.rs`
- Create: `spool/spool-tui/tests/render_snapshots.rs`

- [ ] **Step 1: Write the failing progress snapshot test**

Create `spool/spool-tui/tests/render_snapshots.rs`:

```rust
use insta::assert_snapshot;
use spool_model::{TaskLiveStatus, WorkPhase};
use spool_tui::ProgressView;

#[test]
fn progress_view_renders_phase_artifact_and_latest_finding() {
    let view = ProgressView {
        task_status: TaskLiveStatus::Active { waiting_flags: vec![] },
        phase: WorkPhase::Evaluating,
        artifact_focus: "Executive Revenue Report".into(),
        latest_finding: "Running warehouse cross-check".into(),
    };

    assert_snapshot!(view.render_text(), @r"
    Progress
    Status: active
    Phase: evaluating
    Artifact: Executive Revenue Report
    Latest: Running warehouse cross-check
    ");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:
```bash
cd spool
cargo test -p spool-tui progress_view_renders_phase_artifact_and_latest_finding
```

Expected: FAIL because the progress widget does not exist.

- [ ] **Step 3: Implement layout and progress widgets**

Create `spool/spool-tui/src/layout.rs`:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LayoutState {
    pub show_advanced: bool,
}

impl Default for LayoutState {
    fn default() -> Self {
        Self { show_advanced: false }
    }
}
```

Create `spool/spool-tui/src/progress.rs`:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProgressView {
    pub task_status: spool_model::TaskLiveStatus,
    pub phase: spool_model::WorkPhase,
    pub artifact_focus: String,
    pub latest_finding: String,
}
```

Implement `ProgressView::render_text()` returning the snapshot shape from the test, including:

- `Status: active` or the matching canonical task-status label
- `Phase: evaluating` or the matching canonical phase label
- the current artifact focus
- the latest finding or action summary

- [ ] **Step 4: Run test to verify it passes**

Run:
```bash
cd spool
cargo test -p spool-tui progress_view_renders_phase_artifact_and_latest_finding
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
cd spool
git add spool-tui
git commit -m "feat: add spool progress widgets"
```

### Task 3: Add Plan Mode And Checkpoint Widgets

**Files:**
- Create: `spool/spool-tui/src/plan_mode.rs`
- Modify: `spool/spool-tui/src/lib.rs`
- Append: `spool/spool-tui/tests/render_snapshots.rs`

- [ ] **Step 1: Write the failing plan-mode snapshot**

Append to `spool/spool-tui/tests/render_snapshots.rs`:

```rust
use spool_tui::PlanModeView;

#[test]
fn plan_mode_renders_scope_approach_and_start_actions() {
    let view = PlanModeView {
        request_summary: "Investigate a revenue mismatch".into(),
        selected_scope: "Executive BI / Executive Revenue Report".into(),
        selected_approach: Some("Report Number Mismatch".into()),
        approach_source: "planner auto-selected".into(),
        expected_evidence: vec![
            "report_metadata".into(),
            "measure_definition".into(),
            "warehouse_query_result".into(),
        ],
    };

    assert_snapshot!(view.render_text(), @r"
    Plan Mode
    Request: Investigate a revenue mismatch
    Scope: Executive BI / Executive Revenue Report
    Approach: Report Number Mismatch
    Source: planner auto-selected
    Evidence: report_metadata, measure_definition, warehouse_query_result
    Actions: start now | keep refining
    ");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:
```bash
cd spool
cargo test -p spool-tui plan_mode_renders_scope_approach_and_start_actions
```

Expected: FAIL because the plan-mode view does not exist.

- [ ] **Step 3: Implement plan mode and checkpoint widgets**

Create `spool/spool-tui/src/plan_mode.rs`:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlanModeView {
    pub request_summary: String,
    pub selected_scope: String,
    pub selected_approach: Option<String>,
    pub approach_source: String,
    pub expected_evidence: Vec<String>,
}
```

Implement `PlanModeView::render_text()` with:

- request summary
- selected scope
- selected approach or `none`
- approach source in task language, such as `planner auto-selected` or `user preference honored`
- expected evidence classes
- `start now | keep refining` action row

- [ ] **Step 4: Run test to verify it passes**

Run:
```bash
cd spool
cargo test -p spool-tui plan_mode_renders_scope_approach_and_start_actions
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
cd spool
git add spool-tui
git commit -m "feat: add spool plan mode widgets"
```

### Task 4: Add Advanced View Widgets For Evidence, Contradictions, Disagreement, And Memory Provenance

**Files:**
- Create: `spool/spool-tui/src/advanced_view.rs`
- Modify: `spool/spool-tui/src/lib.rs`
- Append: `spool/spool-tui/tests/render_snapshots.rs`

- [ ] **Step 1: Write the failing advanced-view snapshot**

Append to `spool/spool-tui/tests/render_snapshots.rs`:

```rust
use spool_core::{MemoryInfluenceRecord, MemoryScope};
use spool_tui::AdvancedView;
use spool_model::WorkPhase;

#[test]
fn advanced_view_shows_evidence_contradictions_and_evaluator_note() {
    let view = AdvancedView {
        evidence_count: 3,
        contradiction_count: 1,
        evaluator_note: "Requested more evidence before confirmation".into(),
        validation_summary: "warehouse and dax disagree".into(),
        contradiction_resolution_note: Some("Reopened after evaluator found the prior resolution lacked fresh observed evidence".into()),
        memory_influence: Some(MemoryInfluenceRecord {
            memory_id: "mem_1".into(),
            scope: MemoryScope::Workspace("Executive BI".into()),
            source_basis: "team_file".into(),
            reason: "Applied report naming convention during artifact resolution".into(),
            phase: WorkPhase::Planning,
        }),
    };

    assert_snapshot!(view.render_text(), @r"
    Advanced View
    Evidence: 3
    Contradictions: 1
    Evaluator: Requested more evidence before confirmation
    Validation: warehouse and dax disagree
    Contradiction detail: Reopened after evaluator found the prior resolution lacked fresh observed evidence
    Memory: Used workspace memory: Executive BI report naming convention
    ");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:
```bash
cd spool
cargo test -p spool-tui advanced_view_shows_evidence_contradictions_and_evaluator_note
```

Expected: FAIL because the advanced view does not exist.

- [ ] **Step 3: Implement the advanced view**

Create `spool/spool-tui/src/advanced_view.rs`:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdvancedView {
    pub evidence_count: usize,
    pub contradiction_count: usize,
    pub evaluator_note: String,
    pub validation_summary: String,
    pub contradiction_resolution_note: Option<String>,
    pub memory_influence: Option<spool_core::MemoryInfluenceRecord>,
}
```

Implement `AdvancedView::render_text()` using the snapshot shape from the test.

If no durable-memory influence exists for the current task, omit the memory line. Render the line from the structured provenance record rather than from ad hoc prose.
If contradiction detail exists, render it from canonical contradiction fields such as `resolution_attempted`, `resolution_note`, and evaluator-driven reopen reasons rather than from hidden transcript state.

- [ ] **Step 4: Run test to verify it passes**

Run:
```bash
cd spool
cargo test -p spool-tui advanced_view_shows_evidence_contradictions_and_evaluator_note
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
cd spool
git add spool-tui
git commit -m "feat: add spool advanced view widgets"
```

### Task 5: Add Compaction And Resume Widgets

**Files:**
- Create: `spool/spool-tui/src/resume.rs`
- Modify: `spool/spool-tui/src/lib.rs`
- Append: `spool/spool-tui/tests/render_snapshots.rs`

- [ ] **Step 1: Write the failing resume snapshot**

Append to `spool/spool-tui/tests/render_snapshots.rs`:

```rust
use spool_core::{MemoryInfluenceRecord, MemoryScope};
use spool_tui::ResumeView;
use spool_model::{TaskLiveStatus, TaskWaitingFlag, WorkPhase};

#[test]
fn resume_view_shows_pending_request_and_active_focus() {
    let view = ResumeView {
        task_id: "task_123".into(),
        task_status: TaskLiveStatus::Active {
            waiting_flags: vec![TaskWaitingFlag::WaitingOnUserInput],
        },
        phase: WorkPhase::Generating,
        pending_request: Some("Select the revenue visual to inspect next".into()),
        active_focus: vec!["Executive Revenue Report".into(), "Revenue Card".into()],
        relevant_memory: vec![MemoryInfluenceRecord {
            memory_id: "mem_1".into(),
            scope: MemoryScope::Workspace("Executive BI".into()),
            source_basis: "team_file".into(),
            reason: "Applied report naming convention during artifact resolution".into(),
            phase: WorkPhase::Planning,
        }],
    };

    assert_snapshot!(view.render_text(), @r"
    Resume
    Task: task_123
    Status: active
    Interrupted during: generating
    Waiting on: waiting_on_user_input
    Pending request: Select the revenue visual to inspect next
    Active focus: Executive Revenue Report, Revenue Card
    Memory: Workspace memory: Executive BI report naming convention
    ");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:
```bash
cd spool
cargo test -p spool-tui resume_view_shows_pending_request_and_active_focus
```

Expected: FAIL because the resume view does not exist.

- [ ] **Step 3: Implement compaction and resume widgets**

Create `spool/spool-tui/src/resume.rs`:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResumeView {
    pub task_id: String,
    pub task_status: spool_model::TaskLiveStatus,
    pub phase: spool_model::WorkPhase,
    pub pending_request: Option<String>,
    pub active_focus: Vec<String>,
    pub relevant_memory: Vec<spool_core::MemoryInfluenceRecord>,
}
```

Implement `ResumeView::render_text()` with:

- task ID
- interrupted phase
- pending evaluator request
- active artifact focus
- relevant durable memory when it influenced restored context

Render relevant memory from structured provenance records rather than from preformatted strings.

- [ ] **Step 4: Run test to verify it passes**

Run:
```bash
cd spool
cargo test -p spool-tui resume_view_shows_pending_request_and_active_focus
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
cd spool
git add spool-tui
git commit -m "feat: add spool resume widgets"
```

### Task 6: Add Canonical-State Projection Layer

**Files:**
- Create: `spool/spool-tui/src/projections.rs`
- Modify: `spool/spool-tui/src/lib.rs`
- Create: `spool/spool-tui/tests/projections.rs`

- [ ] **Step 1: Write the failing projection test**

Create `spool/spool-tui/tests/projections.rs`:

```rust
use spool_model::TaskWaitingFlag;
use spool_tui::project_waiting_flag_label;

#[test]
fn projection_maps_waiting_user_input_to_human_label() {
    assert_eq!(
        project_waiting_flag_label(&TaskWaitingFlag::WaitingOnUserInput),
        "Waiting On User Input"
    );
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:
```bash
cd spool
cargo test -p spool-tui projection_maps_waiting_user_input_to_human_label
```

Expected: FAIL because the projection helper does not exist.

- [ ] **Step 3: Implement the projection layer**

Create `spool/spool-tui/src/projections.rs`:

```rust
pub fn project_phase_label(phase: &spool_model::WorkPhase) -> &'static str {
    match phase {
        spool_model::WorkPhase::Planning => "Planning",
        spool_model::WorkPhase::Generating => "Generating",
        spool_model::WorkPhase::Evaluating => "Evaluating",
        spool_model::WorkPhase::ResultFinalization => "Finalizing Result",
    }
}

pub fn project_waiting_flag_label(flag: &spool_model::TaskWaitingFlag) -> &'static str {
    match flag {
        spool_model::TaskWaitingFlag::WaitingOnUserInput => "Waiting On User Input",
        spool_model::TaskWaitingFlag::WaitingOnApproval => "Waiting On Approval",
    }
}
```

Add additional helpers that derive:

- human-readable phase labels
- progress summaries
- advanced-view summaries
- resume summaries
- durable-memory provenance summaries

from canonical task, evidence, contradiction, validation, and session-state inputs.

- [ ] **Step 4: Run test to verify it passes**

Run:
```bash
cd spool
cargo test -p spool-tui projection_maps_waiting_user_input_to_human_label
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
cd spool
git add spool-tui
git commit -m "feat: add spool tui projection layer"
```

### Task 7: Implement The Event Loop And Key Handling

**Files:**
- Modify: `spool/spool-tui/src/app.rs`
- Modify: `spool/spool-tui/src/events.rs`
- Modify: `spool/spool-tui/tests/app_flow.rs`

- [ ] **Step 1: Write the failing event-loop test**

Append to `spool/spool-tui/tests/app_flow.rs`:

```rust
use spool_tui::{AppEvent, AppMode, AppState};

#[test]
fn escape_from_session_opens_resume_mode_when_resume_state_exists() {
    let mut state = AppState::default();
    state.mode = AppMode::Session;
    state.resume_available = true;

    state.apply(AppEvent::Escape);

    assert_eq!(state.mode, AppMode::Resume);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:
```bash
cd spool
cargo test -p spool-tui escape_from_session_opens_resume_mode_when_resume_state_exists
```

Expected: FAIL because the event handler does not exist.

- [ ] **Step 3: Implement the event loop state transitions**

Extend `spool/spool-tui/src/app.rs` with:

- `AppState::apply(event)`
- mode transitions for plan, session, and resume
- toggling advanced view
- simple key handling for `Escape`, `a`, and `Enter`

Behavior rules:

- `Escape` from `Session` enters `Resume` only when `resume_available` is true
- `Escape` from `Resume` returns to `Session`
- `a` toggles `layout.show_advanced`
- `Enter` in `Plan` starts the session flow

- [ ] **Step 4: Run test to verify it passes**

Run:
```bash
cd spool
cargo test -p spool-tui escape_from_session_opens_resume_mode_when_resume_state_exists
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
cd spool
git add spool-tui
git commit -m "feat: add spool tui event loop state transitions"
```

### Task 8: Run The Real Session UX Integration Path

**Files:**
- Modify: `spool/spool-tui/src/app.rs`

- [ ] **Step 1: Wire the app shell to the real session path**

Update `spool/spool-tui/src/app.rs` so the app shell can:

- enter plan mode
- render progress during a live task
- surface advanced view when requested
- restore resume state from persisted session data
- surface both waiting-on-user-input and waiting-on-approval states from canonical session data

- [ ] **Step 2: Run the real integration walkthrough**

Run:
```bash
cd spool
cargo test -p spool-tui
cargo run -p spool -- --config ~/.config/spool/dev.toml
```

Expected: tests PASS, and the real app shell can exercise this exact walkthrough against the dev Fabric environment:

- start in plan mode with a user-facing approach summary rather than a raw recipe ID
- continue into one waiting-on-user-input checkpoint
- resume an interrupted session that is waiting on approval
- open advanced view and verify contradiction detail renders from canonical contradiction fields
- restore the active artifact focus shown before the interruption

- [ ] **Step 3: Commit**

```bash
cd spool
git add spool-tui
git commit -m "feat: wire spool tui app shell to session flow"
```

### Task 9: Write The TUI Session-UX Architecture Note

**Files:**
- Create: `spool/docs/architecture/tui-session-ux.md`

- [ ] **Step 1: Write the architecture note**

Create `spool/docs/architecture/tui-session-ux.md` describing:

- why the TUI projects from canonical state instead of owning workflow state
- how plan mode and checkpoint UI relate to the task contract
- how advanced view exposes disagreement without hidden reasoning leakage
- how compaction and resume rely on Plan 1 persisted-state contracts
- what later plans may add without redefining widget responsibilities

- [ ] **Step 2: Review the note for missing concepts**

Run:
```bash
cd spool
rg -n "canonical state|plan mode|disagreement|resume|widget" docs/architecture/tui-session-ux.md
```

Expected: the note explicitly mentions all five concepts.

- [ ] **Step 3: Commit**

```bash
cd spool
git add docs/architecture/tui-session-ux.md
git commit -m "docs: add spool tui session ux architecture note"
```

## Self-Review Checklist

- Spec coverage: This plan covers plan mode, checkpoints, progress rendering, advanced evidence and disagreement rendering, compaction display, resume restoration, and real-session event handling.
- Placeholder scan: No step may stop at string-only demos, omit the event loop, or treat resume as a banner without structured restoration.
- Type consistency: Keep `AppMode`, `AppState`, `AppEvent`, `ProgressView`, `PlanModeView`, `AdvancedView`, `ResumeView`, and projection helpers stable because Plan 6 and later polish work consume them directly.
