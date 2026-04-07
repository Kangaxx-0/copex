# Plan 5: TUI And Session UX

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Establish the `spool-tui` crate with terminal application shell, plan mode UX, progress surface, advanced view, compaction, resume, and session lifecycle rendering. This plan creates the user-facing terminal experience for Spool, borrowing event-driven architecture patterns from the codex-rs TUI while building Spool-specific analytics investigation surfaces.

**Architecture:** One new crate -- `spool-tui` -- living in the `spool/` workspace alongside `spool-protocol` and `spool-core` from Plan 1. The TUI uses a crossterm event loop driving ratatui rendering through an application state machine. Compaction produces structured working state from evidence/contradiction/contract rather than naive truncation. Resume restores from persisted `SessionState` through Plan 1's persistence layer.

**Tech Stack:** Rust 2024 edition, ratatui 0.29, crossterm 0.28, tokio, serde/serde_json, chrono, uuid, unicode-width

**Governing spec:** `docs/superpowers/specs/2026-04-06-spool-refined-spec.md`
**Planning readiness:** `docs/superpowers/specs/2026-04-06-spool-dev-planning-readiness-design.md`
**Plan 1 reference:** `docs/plans/claude/2026-04-06-plan-1-harness-semantics-foundation.md`
**Codex-rs TUI reference:** `codex-rs/tui/src/`

---

## Plan-Specific Sections

### Subsystem Scope

This plan owns:

- terminal application shell with crossterm/ratatui event loop
- application state machine (`AppState`) driving all view transitions
- plan mode UX: refine request, select scope, define artifacts, select recipes, finalize task contract (Spec Section 13.1)
- progress surface: current phase, artifact under investigation, latest finding, waiting state (Spec Section 13.2)
- advanced view: structured transcript, planner/generator/evaluator activity, evidence ledger detail (Spec Section 13.3)
- chat input widget with LLM streaming placeholder
- structured compaction: produce working state from evidence/contradiction/contract (Spec Section 12.3)
- active context composition from task contract, evidence, working state, recent tail, knowledge, durable memory (Spec Section 12.4)
- resume semantics: session-level restore from structured `SessionState` (Spec Section 12.5-12.8)
- interrupted task handling with explicit surfacing of interrupted state (Spec Section 12.8)
- session startup lifecycle rendering (Spec Section 15.1)
- task execution lifecycle rendering (Spec Section 15.2)

### Out Of Scope

- live Fabric auth or API calls (Plan 2)
- DAX or warehouse query execution (Plan 4)
- knowledge bundle loading and indexing (Plan 3)
- LLM provider integration and streaming (deferred to integration plan)
- durable memory persistence and lifecycle (Plan 6)
- exports, telemetry, policy hardening (Plan 6)
- exact audio/voice input handling
- multi-session management UI (v1 uses single active session)

### Dependencies

- **Plan 1:** `spool-protocol` types (task contract, evidence, contradiction, evaluator outcomes, task result, checkpoint, session state), `spool-core` (evidence ledger, contradiction ledger, task lifecycle, persistence layer, evaluator loop)
- **Plans 2-4:** adapter traits, knowledge loading, validation execution paths -- the TUI renders state produced by these systems but does not implement them. Fixture implementations from Plan 1 are sufficient for Plan 5 testing.

### Contract Impact

This plan **implements** the following governing contracts from the refined spec:

- Plan mode UX contract (Spec Section 13.1)
- Progress surface contract (Spec Section 13.2)
- Advanced view contract (Spec Section 13.3)
- Compaction rule (Spec Section 12.3)
- Active context composition rule (Spec Section 12.4)
- Resume semantics (Spec Section 12.5-12.8)
- Interrupted task handling (Spec Section 12.8)
- Session startup lifecycle (Spec Section 15.1)
- Task execution lifecycle rendering (Spec Section 15.2)

This plan **pressures** the following contracts from Plan 1:

- `SessionState` schema -- Plan 5 may need additional fields for view state persistence
- `PersistenceProvider` trait -- Plan 5 exercises it for real session save/load cycles

### Validation

Plan 5 is proven through:

- state-machine tests: all `AppState` transitions, plan mode flow, view transitions
- compaction logic tests: structured compaction output correctness, field preservation guarantees
- resume logic tests: round-trip through persistence, interrupted task surfacing, resume resolution rules
- active context composition tests: correct field selection, tail truncation, knowledge placeholder inclusion
- widget rendering tests: ratatui `TestBackend` snapshot verification for plan mode, progress surface, advanced view
- integration tests: full session lifecycle from startup through plan mode, execution, compaction, persist, resume

No live systems. No network. All fixture-backed.

**Integration validation justification (per planning readiness addendum Section 5):** Plan 5 renders state produced by other subsystems (evidence, contradictions, task results). It does not own any live external seam. Fixture-backed validation is acceptable here because all live Fabric behavior is owned by Plans 2 and 4.

### Open Items

**Owned by this plan:**

- exact widget layout proportions and resize behavior (resolved during implementation)
- exact key binding assignments for view switching and plan mode navigation (resolved during implementation)
- exact compaction summary field format (resolved by compaction tests)
- exact active context token budget and truncation strategy (resolved during implementation)

**Deferred to later plans:**

- LLM streaming integration for chat input (integration plan or Plan 6)
- real knowledge bundle injection into active context (Plan 3 integration)
- durable memory injection into active context (Plan 6)
- exact session file location and directory layout (Plan 6: operationalization)

**Review triggers:**

- if `SessionState` schema from Plan 1 proves insufficient for resume rendering, revisit persistence schema
- if compaction output format proves inadequate for LLM context composition during integration, revisit compaction contract
- if advanced view rendering needs data not captured in current evidence/contradiction ledger APIs, revisit Plan 1 ledger surface

---

## Task 1: Crate Scaffolding

**Files:**

- Modify: `spool/Cargo.toml`
- Create: `spool/spool-tui/Cargo.toml`
- Create: `spool/spool-tui/src/lib.rs`
- Create: `spool/spool-tui/src/main.rs`

**Step 1: Add spool-tui to workspace**

Modify `spool/Cargo.toml` to add `spool-tui` to the workspace members and add TUI dependencies:

```toml
# spool/Cargo.toml
[workspace]
members = [
    "spool-protocol",
    "spool-core",
    "spool-tui",
]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2024"
license = "Apache-2.0"

[workspace.dependencies]
spool-protocol = { path = "spool-protocol" }
spool-core = { path = "spool-core" }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v4", "serde"] }
tokio = { version = "1", features = ["full"] }
async-trait = "0.1"
thiserror = "2"
ratatui = "0.29"
crossterm = "0.28"
unicode-width = "0.2"
```

**Step 2: Create spool-tui crate**

```toml
# spool/spool-tui/Cargo.toml
[package]
name = "spool-tui"
version.workspace = true
edition.workspace = true
license.workspace = true

[[bin]]
name = "spool"
path = "src/main.rs"

[dependencies]
spool-protocol = { workspace = true }
spool-core = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
chrono = { workspace = true }
uuid = { workspace = true }
tokio = { workspace = true }
ratatui = { workspace = true }
crossterm = { workspace = true }
unicode-width = { workspace = true }
thiserror = { workspace = true }

[dev-dependencies]
tempfile = "3"
tokio = { workspace = true, features = ["test-util", "macros"] }
```

```rust
// spool/spool-tui/src/lib.rs
pub mod app_state;
pub mod compaction;
pub mod context;
pub mod event;
pub mod resume;
pub mod widgets;
```

```rust
// spool/spool-tui/src/main.rs
fn main() {
    println!("spool TUI — not yet implemented");
}
```

**Step 3: Create placeholder modules**

Create empty files for each module declared in lib.rs. Each file should contain only a comment:

```rust
// placeholder -- implemented in later tasks
```

Create these files:

- `spool/spool-tui/src/app_state.rs`
- `spool/spool-tui/src/compaction.rs`
- `spool/spool-tui/src/context.rs`
- `spool/spool-tui/src/event.rs`
- `spool/spool-tui/src/resume.rs`
- `spool/spool-tui/src/widgets/mod.rs`

For `widgets/mod.rs`:

```rust
// spool/spool-tui/src/widgets/mod.rs
pub mod plan_mode;
pub mod progress;
pub mod advanced_view;
pub mod chat_input;
pub mod status_bar;
```

Create placeholder files for each widget module:

- `spool/spool-tui/src/widgets/plan_mode.rs`
- `spool/spool-tui/src/widgets/progress.rs`
- `spool/spool-tui/src/widgets/advanced_view.rs`
- `spool/spool-tui/src/widgets/chat_input.rs`
- `spool/spool-tui/src/widgets/status_bar.rs`

**Step 4: Verify build**

Run: `cd spool && cargo check -p spool-tui`
Expected: compiles with no errors

**Step 5: Commit**

```bash
git add spool/
git commit -m "feat(spool-tui): scaffold spool-tui crate with module structure for TUI, compaction, resume, and widgets"
```

---

## Task 2: Application State Machine

**Files:**

- Modify: `spool/spool-tui/src/app_state.rs`

**Step 1: Write the failing test**

Add to `spool/spool-tui/src/app_state.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use spool_protocol::task_contract::TaskStatus;

    #[test]
    fn initial_state_is_startup() {
        let state = AppState::new();
        assert!(matches!(state.view(), ViewMode::Startup));
        assert!(state.session_id().is_none());
    }

    #[test]
    fn startup_to_plan_mode() {
        let mut state = AppState::new();
        state.initialize_session("session_1".into(), "finance".into(), "Executive BI".into());
        assert!(matches!(state.view(), ViewMode::PlanMode));
        assert_eq!(state.session_id(), Some("session_1"));
        assert_eq!(state.selected_lob(), "finance");
        assert_eq!(state.workspace_scope(), "Executive BI");
    }

    #[test]
    fn plan_mode_to_progress() {
        let mut state = AppState::new();
        state.initialize_session("s1".into(), "fin".into(), "ws".into());
        let result = state.transition_view(ViewMode::Progress);
        assert!(result.is_ok());
        assert!(matches!(state.view(), ViewMode::Progress));
    }

    #[test]
    fn progress_to_advanced_toggle() {
        let mut state = AppState::new();
        state.initialize_session("s1".into(), "fin".into(), "ws".into());
        state.transition_view(ViewMode::Progress).unwrap();

        let result = state.transition_view(ViewMode::Advanced);
        assert!(result.is_ok());
        assert!(matches!(state.view(), ViewMode::Advanced));

        // Toggle back
        let result = state.transition_view(ViewMode::Progress);
        assert!(result.is_ok());
        assert!(matches!(state.view(), ViewMode::Progress));
    }

    #[test]
    fn invalid_transition_from_startup() {
        let mut state = AppState::new();
        let result = state.transition_view(ViewMode::Progress);
        assert!(result.is_err());
    }

    #[test]
    fn task_phase_tracking() {
        let mut state = AppState::new();
        state.initialize_session("s1".into(), "fin".into(), "ws".into());
        assert_eq!(state.task_phase(), None);

        state.set_task_phase(TaskPhase::Planning);
        assert_eq!(state.task_phase(), Some(&TaskPhase::Planning));

        state.set_task_phase(TaskPhase::Investigating);
        assert_eq!(state.task_phase(), Some(&TaskPhase::Investigating));

        state.set_task_phase(TaskPhase::Evaluating);
        assert_eq!(state.task_phase(), Some(&TaskPhase::Evaluating));
    }

    #[test]
    fn artifact_focus_tracking() {
        let mut state = AppState::new();
        state.initialize_session("s1".into(), "fin".into(), "ws".into());

        assert!(state.artifact_focus().is_none());
        state.set_artifact_focus("Executive Revenue Report".into());
        assert_eq!(state.artifact_focus(), Some("Executive Revenue Report"));
        state.clear_artifact_focus();
        assert!(state.artifact_focus().is_none());
    }

    #[test]
    fn latest_finding_tracking() {
        let mut state = AppState::new();
        state.initialize_session("s1".into(), "fin".into(), "ws".into());

        assert!(state.latest_finding().is_none());
        state.set_latest_finding("Revenue mismatch traced to measure logic".into());
        assert_eq!(
            state.latest_finding(),
            Some("Revenue mismatch traced to measure logic")
        );
    }

    #[test]
    fn waiting_state_tracking() {
        let mut state = AppState::new();
        state.initialize_session("s1".into(), "fin".into(), "ws".into());

        assert!(state.waiting_state().is_none());
        state.set_waiting(WaitingState::UserInput {
            question: "Which report?".into(),
        });
        assert!(matches!(
            state.waiting_state(),
            Some(WaitingState::UserInput { .. })
        ));
        state.clear_waiting();
        assert!(state.waiting_state().is_none());
    }

    #[test]
    fn all_view_modes_serialize() {
        let modes = vec![
            ViewMode::Startup,
            ViewMode::PlanMode,
            ViewMode::Progress,
            ViewMode::Advanced,
            ViewMode::Chat,
        ];
        for m in modes {
            let json = serde_json::to_string(&m).unwrap();
            let restored: ViewMode = serde_json::from_str(&json).unwrap();
            assert_eq!(restored, m);
        }
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cd spool && cargo test -p spool-tui -- app_state`
Expected: FAIL -- types not defined yet

**Step 3: Write the implementation**

Replace the placeholder content of `spool/spool-tui/src/app_state.rs`:

```rust
// spool/spool-tui/src/app_state.rs
use serde::{Deserialize, Serialize};

/// The current view mode of the terminal application.
///
/// View modes correspond to the UX model in Spec Section 13.
/// Startup is the initial boot state. PlanMode is the analytics-native
/// planning surface. Progress is the default execution surface. Advanced
/// is the detailed transcript and ledger overlay. Chat is the free-form
/// input surface.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ViewMode {
    Startup,
    PlanMode,
    Progress,
    Advanced,
    Chat,
}

/// Task execution phase for progress surface rendering.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskPhase {
    Planning,
    Investigating,
    Evaluating,
    Finalizing,
}

/// Waiting state for the progress surface.
///
/// When Spool is waiting for user input or an external response,
/// the progress surface should indicate what is being waited on.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum WaitingState {
    UserInput { question: String },
    ExternalResponse { description: String },
    EvaluatorReview,
}

/// Application state machine for the Spool TUI.
///
/// Drives all view transitions and tracks the current task phase,
/// artifact focus, latest finding, and waiting state for progress
/// surface rendering (Spec Section 13.2).
pub struct AppState {
    view: ViewMode,
    session_id: Option<String>,
    selected_lob: String,
    workspace_scope: String,
    task_phase: Option<TaskPhase>,
    artifact_focus: Option<String>,
    latest_finding: Option<String>,
    waiting_state: Option<WaitingState>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            view: ViewMode::Startup,
            session_id: None,
            selected_lob: String::new(),
            workspace_scope: String::new(),
            task_phase: None,
            artifact_focus: None,
            latest_finding: None,
            waiting_state: None,
        }
    }

    pub fn view(&self) -> &ViewMode {
        &self.view
    }

    pub fn session_id(&self) -> Option<&str> {
        self.session_id.as_deref()
    }

    pub fn selected_lob(&self) -> &str {
        &self.selected_lob
    }

    pub fn workspace_scope(&self) -> &str {
        &self.workspace_scope
    }

    pub fn task_phase(&self) -> Option<&TaskPhase> {
        self.task_phase.as_ref()
    }

    pub fn artifact_focus(&self) -> Option<&str> {
        self.artifact_focus.as_deref()
    }

    pub fn latest_finding(&self) -> Option<&str> {
        self.latest_finding.as_deref()
    }

    pub fn waiting_state(&self) -> Option<&WaitingState> {
        self.waiting_state.as_ref()
    }

    /// Initialize a new session and transition from Startup to PlanMode.
    ///
    /// This implements the session startup lifecycle (Spec Section 15.1):
    /// after auth, LOB selection, and workspace scope are established,
    /// the TUI enters plan mode.
    pub fn initialize_session(
        &mut self,
        session_id: String,
        selected_lob: String,
        workspace_scope: String,
    ) {
        self.session_id = Some(session_id);
        self.selected_lob = selected_lob;
        self.workspace_scope = workspace_scope;
        self.view = ViewMode::PlanMode;
    }

    /// Transition between view modes with validation.
    ///
    /// Valid transitions:
    /// - PlanMode -> Progress (task started)
    /// - PlanMode -> Chat (free-form input during planning)
    /// - Progress -> Advanced (toggle detail view)
    /// - Progress -> Chat (user wants to send a message)
    /// - Advanced -> Progress (toggle back)
    /// - Advanced -> Chat (user input from advanced view)
    /// - Chat -> Progress (return to progress after input)
    /// - Chat -> PlanMode (return to plan mode)
    /// - Chat -> Advanced (return to advanced view)
    pub fn transition_view(&mut self, to: ViewMode) -> Result<(), AppStateError> {
        if !is_valid_view_transition(&self.view, &to) {
            return Err(AppStateError::InvalidViewTransition {
                from: format!("{:?}", self.view),
                to: format!("{to:?}"),
            });
        }
        self.view = to;
        Ok(())
    }

    pub fn set_task_phase(&mut self, phase: TaskPhase) {
        self.task_phase = Some(phase);
    }

    pub fn set_artifact_focus(&mut self, artifact: String) {
        self.artifact_focus = Some(artifact);
    }

    pub fn clear_artifact_focus(&mut self) {
        self.artifact_focus = None;
    }

    pub fn set_latest_finding(&mut self, finding: String) {
        self.latest_finding = Some(finding);
    }

    pub fn set_waiting(&mut self, state: WaitingState) {
        self.waiting_state = Some(state);
    }

    pub fn clear_waiting(&mut self) {
        self.waiting_state = None;
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

fn is_valid_view_transition(from: &ViewMode, to: &ViewMode) -> bool {
    matches!(
        (from, to),
        (ViewMode::PlanMode, ViewMode::Progress)
            | (ViewMode::PlanMode, ViewMode::Chat)
            | (ViewMode::Progress, ViewMode::Advanced)
            | (ViewMode::Progress, ViewMode::Chat)
            | (ViewMode::Advanced, ViewMode::Progress)
            | (ViewMode::Advanced, ViewMode::Chat)
            | (ViewMode::Chat, ViewMode::Progress)
            | (ViewMode::Chat, ViewMode::PlanMode)
            | (ViewMode::Chat, ViewMode::Advanced)
    )
}

#[derive(Debug, Clone)]
pub enum AppStateError {
    InvalidViewTransition { from: String, to: String },
}

impl std::fmt::Display for AppStateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppStateError::InvalidViewTransition { from, to } => {
                write!(f, "invalid view transition from {from} to {to}")
            }
        }
    }
}

impl std::error::Error for AppStateError {}

// tests at bottom of file (from Step 1)
```

**Step 4: Run test to verify it passes**

Run: `cd spool && cargo test -p spool-tui -- app_state`
Expected: 10 tests PASS

**Step 5: Commit**

```bash
git add spool/spool-tui/src/app_state.rs
git commit -m "feat(spool-tui): application state machine with view modes, task phase, artifact focus, and waiting state"
```

---

## Task 3: TUI Event Types

**Files:**

- Modify: `spool/spool-tui/src/event.rs`

**Step 1: Write the failing test**

Add to `spool/spool-tui/src/event.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_event_variants_exist() {
        let events: Vec<AppEvent> = vec![
            AppEvent::TerminalKey(KeyCode::Char('q')),
            AppEvent::TerminalResize { width: 80, height: 24 },
            AppEvent::Tick,
            AppEvent::UserSubmitInput("test input".into()),
            AppEvent::PlanModeAction(PlanModeAction::SelectLob("finance".into())),
            AppEvent::PlanModeAction(PlanModeAction::SetScope {
                workspace: "ws".into(),
            }),
            AppEvent::PlanModeAction(PlanModeAction::AddArtifact {
                artifact_type: "report".into(),
                reference: "Report A".into(),
            }),
            AppEvent::PlanModeAction(PlanModeAction::SelectRecipe("recipe_1".into())),
            AppEvent::PlanModeAction(PlanModeAction::FinalizeContract),
            AppEvent::PlanModeAction(PlanModeAction::KeepRefining),
            AppEvent::ToggleAdvancedView,
            AppEvent::TaskPhaseChanged(TaskPhaseEvent::Investigating {
                artifact: "Report A".into(),
            }),
            AppEvent::TaskPhaseChanged(TaskPhaseEvent::Evaluating),
            AppEvent::TaskPhaseChanged(TaskPhaseEvent::Completed {
                result_state: "confirmed".into(),
            }),
            AppEvent::FindingReceived {
                summary: "Revenue mismatch found".into(),
            },
            AppEvent::EvidenceReceived {
                evidence_id: "ev_1".into(),
                summary: "DAX result".into(),
            },
            AppEvent::WaitingForUser {
                question: "Which report?".into(),
            },
            AppEvent::SessionResumed {
                session_id: "s1".into(),
            },
            AppEvent::RequestQuit,
        ];

        // Verify we can construct all variants without panic
        assert_eq!(events.len(), 19);
    }

    #[test]
    fn plan_mode_action_all_variants() {
        let actions = vec![
            PlanModeAction::SelectLob("fin".into()),
            PlanModeAction::SetScope { workspace: "ws".into() },
            PlanModeAction::AddArtifact {
                artifact_type: "report".into(),
                reference: "R".into(),
            },
            PlanModeAction::RemoveArtifact { index: 0 },
            PlanModeAction::SelectRecipe("r1".into()),
            PlanModeAction::ClearRecipe,
            PlanModeAction::SetIntent("investigate".into()),
            PlanModeAction::AddAssumption("user is admin".into()),
            PlanModeAction::FinalizeContract,
            PlanModeAction::KeepRefining,
        ];
        assert_eq!(actions.len(), 10);
    }

    #[test]
    fn event_action_derives_debug() {
        let event = AppEvent::Tick;
        let debug_str = format!("{event:?}");
        assert!(debug_str.contains("Tick"));
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cd spool && cargo test -p spool-tui -- event`
Expected: FAIL -- types not defined yet

**Step 3: Write the implementation**

Replace the placeholder content of `spool/spool-tui/src/event.rs`:

```rust
// spool/spool-tui/src/event.rs
use crossterm::event::KeyCode;

/// Plan mode actions correspond to the plan mode UX flow from Spec Section 13.1.
///
/// Each action maps to a discrete user operation during task contract construction:
/// refine request, select scope, define artifacts, select recipes, finalize.
#[derive(Debug, Clone)]
pub enum PlanModeAction {
    SelectLob(String),
    SetScope { workspace: String },
    AddArtifact { artifact_type: String, reference: String },
    RemoveArtifact { index: usize },
    SelectRecipe(String),
    ClearRecipe,
    SetIntent(String),
    AddAssumption(String),
    FinalizeContract,
    KeepRefining,
}

/// Task phase events emitted by the harness during execution.
///
/// These drive the progress surface (Spec Section 13.2) and the
/// task execution lifecycle rendering (Spec Section 15.2).
#[derive(Debug, Clone)]
pub enum TaskPhaseEvent {
    Planning,
    Investigating { artifact: String },
    Evaluating,
    Completed { result_state: String },
    Blocked { reason: String },
    Interrupted,
}

/// Application events consumed by the TUI event loop.
///
/// Follows the crossterm event -> app state update -> ratatui render
/// pattern borrowed from codex-rs tui architecture.
#[derive(Debug, Clone)]
pub enum AppEvent {
    /// A key press from the terminal.
    TerminalKey(KeyCode),

    /// Terminal resize event.
    TerminalResize { width: u16, height: u16 },

    /// Periodic tick for animation and polling.
    Tick,

    /// User submitted text input from the chat input widget.
    UserSubmitInput(String),

    /// Plan mode action from the plan mode widget.
    PlanModeAction(PlanModeAction),

    /// Toggle between progress and advanced view.
    ToggleAdvancedView,

    /// Task phase changed during execution.
    TaskPhaseChanged(TaskPhaseEvent),

    /// A new finding was produced by the generator.
    FindingReceived { summary: String },

    /// New evidence was collected and appended to the ledger.
    EvidenceReceived { evidence_id: String, summary: String },

    /// Spool is waiting for user input (checkpoint question).
    WaitingForUser { question: String },

    /// A session was resumed from persisted state.
    SessionResumed { session_id: String },

    /// User requested to quit the application.
    RequestQuit,
}

// tests at bottom of file (from Step 1)
```

**Step 4: Run test to verify it passes**

Run: `cd spool && cargo test -p spool-tui -- event`
Expected: 3 tests PASS

**Step 5: Commit**

```bash
git add spool/spool-tui/src/event.rs
git commit -m "feat(spool-tui): TUI event types with plan mode actions, task phase events, and terminal events"
```

---

## Task 4: Structured Compaction

**Files:**

- Modify: `spool/spool-tui/src/compaction.rs`

**Step 1: Write the failing test**

Add to `spool/spool-tui/src/compaction.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use spool_protocol::artifact::ArtifactType;
    use spool_protocol::checkpoint::{
        AskUserQuestion, CheckpointPolicy, CheckpointTrigger, CheckpointClass, UserAnswer,
    };
    use spool_protocol::contradiction::{
        ContradictionId, ContradictionRecord, ContradictionStatus, MaterialityLevel,
    };
    use spool_protocol::evidence::{EvidenceClass, EvidenceId, EvidenceItem, EvidenceType};
    use spool_protocol::evaluator::{EvaluatorOutcome, EvidenceTarget};
    use spool_protocol::task_contract::*;
    use spool_protocol::task_result::*;

    fn sample_contract() -> TaskContract {
        TaskContract {
            task_id: TaskId("task_1".into()),
            intent: "Find revenue mismatch".into(),
            scope: Scope {
                lob: "finance".into(),
                workspace: "Executive BI".into(),
                artifacts: vec![ArtifactRef {
                    artifact_type: ArtifactType::Report,
                    reference: "Revenue Report".into(),
                }],
            },
            selected_recipe: Some("report_number_mismatch".into()),
            selected_recipe_selection_mode: None,
            assumptions: vec!["Published report".into()],
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
            created_at: Some(Utc::now()),
            updated_at: None,
        }
    }

    fn make_evidence(id: &str, class: EvidenceClass, summary: &str) -> EvidenceItem {
        EvidenceItem {
            id: EvidenceId(id.into()),
            evidence_type: EvidenceType::Observed,
            evidence_class: class,
            source: "test".into(),
            summary: summary.into(),
            artifact_refs: vec![],
            observed_at: Some(Utc::now()),
            detail: None,
        }
    }

    fn make_contradiction(id: &str) -> ContradictionRecord {
        let now = Utc::now();
        ContradictionRecord {
            id: ContradictionId(id.into()),
            disputed_claim: "Revenue mismatch between sources".into(),
            conflicting_evidence: vec![
                EvidenceId("ev_1".into()),
                EvidenceId("ev_2".into()),
            ],
            materiality: MaterialityLevel::Material,
            freshness_notes: None,
            resolution_attempted: false,
            resolution_detail: None,
            status: ContradictionStatus::Open,
            created_at: now,
            updated_at: now,
        }
    }

    fn make_question(id: &str, answered: bool) -> AskUserQuestion {
        let now = Utc::now();
        AskUserQuestion {
            id: id.into(),
            checkpoint_class: CheckpointClass::Information,
            trigger: CheckpointTrigger::Ambiguous,
            question: "Which report do you mean?".into(),
            options: Some(vec!["Report A".into(), "Report B".into()]),
            allows_free_text: true,
            asked_at: now,
            answer: if answered {
                Some(UserAnswer {
                    selected_option: Some("Report A".into()),
                    free_text: None,
                    answered_at: now,
                })
            } else {
                None
            },
        }
    }

    #[test]
    fn compact_preserves_contract() {
        let input = CompactionInput {
            task_contract: sample_contract(),
            task_phase: "investigating".into(),
            evidence_items: vec![
                make_evidence("ev_1", EvidenceClass::DaxQueryResult, "DAX returned 12.4M"),
            ],
            contradiction_records: vec![],
            unresolved_questions: vec![],
            unresolved_evaluator_requests: vec![],
            artifact_focus: Some("Revenue Report".into()),
            latest_result: None,
        };

        let output = compact(&input);

        assert_eq!(output.task_contract.task_id.0, "task_1");
        assert_eq!(output.task_phase, "investigating");
        assert_eq!(output.artifact_focus, Some("Revenue Report".into()));
    }

    #[test]
    fn compact_produces_evidence_summary() {
        let input = CompactionInput {
            task_contract: sample_contract(),
            task_phase: "investigating".into(),
            evidence_items: vec![
                make_evidence("ev_1", EvidenceClass::DaxQueryResult, "DAX returned 12.4M"),
                make_evidence("ev_2", EvidenceClass::WarehouseQueryResult, "Warehouse returned 12.4M"),
                make_evidence("ev_3", EvidenceClass::ReportMetadata, "Report has 3 pages"),
            ],
            contradiction_records: vec![],
            unresolved_questions: vec![],
            unresolved_evaluator_requests: vec![],
            artifact_focus: None,
            latest_result: None,
        };

        let output = compact(&input);

        assert_eq!(output.evidence_summary.total_count, 3);
        assert_eq!(output.evidence_summary.by_class.len(), 3);
        assert!(output.evidence_summary.by_class.contains_key("dax_query_result"));
        assert!(output.evidence_summary.by_class.contains_key("warehouse_query_result"));
        assert!(output.evidence_summary.by_class.contains_key("report_metadata"));
    }

    #[test]
    fn compact_preserves_contradiction_summary() {
        let input = CompactionInput {
            task_contract: sample_contract(),
            task_phase: "evaluating".into(),
            evidence_items: vec![],
            contradiction_records: vec![make_contradiction("c_1")],
            unresolved_questions: vec![],
            unresolved_evaluator_requests: vec![],
            artifact_focus: None,
            latest_result: None,
        };

        let output = compact(&input);

        assert_eq!(output.contradiction_summary.total_count, 1);
        assert_eq!(output.contradiction_summary.unresolved_count, 1);
        assert!(output.contradiction_summary.has_material_unresolved);
    }

    #[test]
    fn compact_preserves_unresolved_questions() {
        let input = CompactionInput {
            task_contract: sample_contract(),
            task_phase: "planning".into(),
            evidence_items: vec![],
            contradiction_records: vec![],
            unresolved_questions: vec![make_question("q_1", false)],
            unresolved_evaluator_requests: vec![],
            artifact_focus: None,
            latest_result: None,
        };

        let output = compact(&input);

        assert_eq!(output.unresolved_questions.len(), 1);
        assert_eq!(output.unresolved_questions[0].id, "q_1");
    }

    #[test]
    fn compact_preserves_unresolved_evaluator_requests() {
        let target = EvidenceTarget {
            description: "Run DAX query for disputed measure".into(),
            target_artifact: Some("art_1".into()),
            target_evidence_class: Some("dax_query_result".into()),
        };

        let input = CompactionInput {
            task_contract: sample_contract(),
            task_phase: "evaluating".into(),
            evidence_items: vec![],
            contradiction_records: vec![],
            unresolved_questions: vec![],
            unresolved_evaluator_requests: vec![target],
            artifact_focus: None,
            latest_result: None,
        };

        let output = compact(&input);

        assert_eq!(output.unresolved_evaluator_requests.len(), 1);
        assert!(output.unresolved_evaluator_requests[0]
            .description
            .contains("DAX"));
    }

    #[test]
    fn compact_includes_latest_result() {
        let result = TaskResult {
            task_id: TaskId("task_1".into()),
            proposed_state: Some(ResultState::Confirmed),
            state: ResultState::SupportedHypothesis,
            confidence: Confidence::Medium,
            summary: "Leading hypothesis".into(),
            findings: vec![],
            evidence_refs: vec![],
            validation_results: vec![],
            recommended_actions: vec![],
            blockers: vec![],
            open_questions: vec![],
            proposed_changes: vec![],
            contradiction_refs: vec![],
            result_generated_at: Some(Utc::now()),
            result_version: Some(1),
        };

        let input = CompactionInput {
            task_contract: sample_contract(),
            task_phase: "evaluating".into(),
            evidence_items: vec![],
            contradiction_records: vec![],
            unresolved_questions: vec![],
            unresolved_evaluator_requests: vec![],
            artifact_focus: None,
            latest_result: Some(result),
        };

        let output = compact(&input);

        assert!(output.latest_result.is_some());
        let r = output.latest_result.unwrap();
        assert_eq!(r.state, ResultState::SupportedHypothesis);
    }

    #[test]
    fn compaction_output_serializes() {
        let input = CompactionInput {
            task_contract: sample_contract(),
            task_phase: "investigating".into(),
            evidence_items: vec![
                make_evidence("ev_1", EvidenceClass::DaxQueryResult, "DAX result"),
            ],
            contradiction_records: vec![make_contradiction("c_1")],
            unresolved_questions: vec![make_question("q_1", false)],
            unresolved_evaluator_requests: vec![],
            artifact_focus: Some("Report".into()),
            latest_result: None,
        };

        let output = compact(&input);
        let json = serde_json::to_string_pretty(&output).unwrap();

        // Verify round trip
        let restored: CompactionOutput = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.task_contract.task_id.0, "task_1");
        assert_eq!(restored.evidence_summary.total_count, 1);
        assert_eq!(restored.contradiction_summary.total_count, 1);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cd spool && cargo test -p spool-tui -- compaction`
Expected: FAIL -- types not defined yet

**Step 3: Write the implementation**

Replace the placeholder content of `spool/spool-tui/src/compaction.rs`:

```rust
// spool/spool-tui/src/compaction.rs
use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use spool_protocol::checkpoint::AskUserQuestion;
use spool_protocol::contradiction::{ContradictionRecord, ContradictionStatus, MaterialityLevel};
use spool_protocol::evaluator::EvidenceTarget;
use spool_protocol::evidence::EvidenceItem;
use spool_protocol::task_contract::TaskContract;
use spool_protocol::task_result::TaskResult;

/// Input to the compaction function.
///
/// Gathered from the current task state before compaction runs.
pub struct CompactionInput {
    pub task_contract: TaskContract,
    pub task_phase: String,
    pub evidence_items: Vec<EvidenceItem>,
    pub contradiction_records: Vec<ContradictionRecord>,
    pub unresolved_questions: Vec<AskUserQuestion>,
    pub unresolved_evaluator_requests: Vec<EvidenceTarget>,
    pub artifact_focus: Option<String>,
    pub latest_result: Option<TaskResult>,
}

/// Summary of evidence collected, grouped by class.
///
/// This is a compact representation for active model context,
/// not the full evidence ledger.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceSummary {
    pub total_count: usize,
    pub by_class: HashMap<String, EvidenceClassSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceClassSummary {
    pub count: usize,
    pub latest_summary: String,
}

/// Summary of contradiction state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContradictionSummary {
    pub total_count: usize,
    pub unresolved_count: usize,
    pub has_material_unresolved: bool,
    pub unresolved_claims: Vec<String>,
}

/// Structured compaction output per Spec Section 12.3.
///
/// Compaction produces structured working state rather than only
/// truncating text. This output preserves:
/// - active or last relevant task contract
/// - current task phase
/// - evidence ledger summary
/// - contradiction summary
/// - unresolved user questions
/// - unresolved evaluator requests
/// - active artifact focus
/// - latest canonical task result when one exists
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactionOutput {
    pub task_contract: TaskContract,
    pub task_phase: String,
    pub evidence_summary: EvidenceSummary,
    pub contradiction_summary: ContradictionSummary,
    pub unresolved_questions: Vec<AskUserQuestion>,
    pub unresolved_evaluator_requests: Vec<EvidenceTarget>,
    pub artifact_focus: Option<String>,
    pub latest_result: Option<TaskResult>,
}

/// Produce structured compaction output from current task state.
///
/// This implements the compaction rule from Spec Section 12.3.
/// The output is the primary input for active context composition
/// (Spec Section 12.4) and resume context rebuilding (Spec Section 12.7).
pub fn compact(input: &CompactionInput) -> CompactionOutput {
    let evidence_summary = build_evidence_summary(&input.evidence_items);
    let contradiction_summary = build_contradiction_summary(&input.contradiction_records);

    CompactionOutput {
        task_contract: input.task_contract.clone(),
        task_phase: input.task_phase.clone(),
        evidence_summary,
        contradiction_summary,
        unresolved_questions: input.unresolved_questions.clone(),
        unresolved_evaluator_requests: input.unresolved_evaluator_requests.clone(),
        artifact_focus: input.artifact_focus.clone(),
        latest_result: input.latest_result.clone(),
    }
}

fn build_evidence_summary(items: &[EvidenceItem]) -> EvidenceSummary {
    let mut by_class: HashMap<String, EvidenceClassSummary> = HashMap::new();

    for item in items {
        let class_key = serde_json::to_value(&item.evidence_class)
            .ok()
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_else(|| format!("{:?}", item.evidence_class));

        let entry = by_class.entry(class_key).or_insert_with(|| {
            EvidenceClassSummary {
                count: 0,
                latest_summary: String::new(),
            }
        });
        entry.count += 1;
        entry.latest_summary = item.summary.clone();
    }

    EvidenceSummary {
        total_count: items.len(),
        by_class,
    }
}

fn build_contradiction_summary(records: &[ContradictionRecord]) -> ContradictionSummary {
    let unresolved: Vec<&ContradictionRecord> = records
        .iter()
        .filter(|r| r.status == ContradictionStatus::Open)
        .collect();

    let has_material_unresolved = unresolved
        .iter()
        .any(|r| r.materiality == MaterialityLevel::Material);

    let unresolved_claims = unresolved
        .iter()
        .map(|r| r.disputed_claim.clone())
        .collect();

    ContradictionSummary {
        total_count: records.len(),
        unresolved_count: unresolved.len(),
        has_material_unresolved,
        unresolved_claims,
    }
}

// tests at bottom of file (from Step 1)
```

**Step 4: Run test to verify it passes**

Run: `cd spool && cargo test -p spool-tui -- compaction`
Expected: 7 tests PASS

**Step 5: Commit**

```bash
git add spool/spool-tui/src/compaction.rs
git commit -m "feat(spool-tui): structured compaction producing evidence and contradiction summaries per Spec Section 12.3"
```

---

## Task 5: Active Context Composition

**Files:**

- Modify: `spool/spool-tui/src/context.rs`

**Step 1: Write the failing test**

Add to `spool/spool-tui/src/context.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::compaction::{
        CompactionOutput, ContradictionSummary, EvidenceClassSummary, EvidenceSummary,
    };
    use chrono::Utc;
    use spool_protocol::artifact::ArtifactType;
    use spool_protocol::checkpoint::{CheckpointPolicy, CheckpointTrigger};
    use spool_protocol::evidence::EvidenceClass;
    use spool_protocol::task_contract::*;
    use std::collections::HashMap;

    fn sample_contract() -> TaskContract {
        TaskContract {
            task_id: TaskId("task_1".into()),
            intent: "Find revenue mismatch".into(),
            scope: Scope {
                lob: "finance".into(),
                workspace: "Executive BI".into(),
                artifacts: vec![ArtifactRef {
                    artifact_type: ArtifactType::Report,
                    reference: "Revenue Report".into(),
                }],
            },
            selected_recipe: Some("report_number_mismatch".into()),
            selected_recipe_selection_mode: None,
            assumptions: vec!["Published report".into()],
            expected_evidence_classes: vec![EvidenceClass::DaxQueryResult],
            validation_floor: ValidationFloor::DirectValidationRequired,
            checkpoint_policy: CheckpointPolicy {
                ask_on: vec![CheckpointTrigger::Ambiguous],
            },
            clarification_checkpoints: vec![],
            approval_checkpoints: vec![],
            expected_deliverable_shape: "structured_task_result".into(),
            evaluator_packet_requirements: vec![],
            task_status: TaskStatus::Active,
            created_at: Some(Utc::now()),
            updated_at: None,
        }
    }

    fn sample_compaction_output() -> CompactionOutput {
        let mut by_class = HashMap::new();
        by_class.insert(
            "dax_query_result".into(),
            EvidenceClassSummary {
                count: 2,
                latest_summary: "DAX returned 12.4M".into(),
            },
        );

        CompactionOutput {
            task_contract: sample_contract(),
            task_phase: "investigating".into(),
            evidence_summary: EvidenceSummary {
                total_count: 2,
                by_class,
            },
            contradiction_summary: ContradictionSummary {
                total_count: 0,
                unresolved_count: 0,
                has_material_unresolved: false,
                unresolved_claims: vec![],
            },
            unresolved_questions: vec![],
            unresolved_evaluator_requests: vec![],
            artifact_focus: Some("Revenue Report".into()),
            latest_result: None,
        }
    }

    #[test]
    fn compose_active_context_includes_contract() {
        let compaction = sample_compaction_output();
        let recent_tail = vec![
            "User: Why does the report show different numbers?".into(),
            "Spool: Investigating the Revenue Report...".into(),
        ];
        let knowledge_summary = Some("LOB: finance, Bundle: finance-v1".into());
        let durable_memory_summary = None;

        let context = compose_active_context(
            &compaction,
            &recent_tail,
            knowledge_summary.as_deref(),
            durable_memory_summary,
        );

        assert!(context.task_contract_section.contains("task_1"));
        assert!(context.task_contract_section.contains("revenue mismatch"));
    }

    #[test]
    fn compose_active_context_includes_evidence_summary() {
        let compaction = sample_compaction_output();
        let context = compose_active_context(&compaction, &[], None, None);

        assert!(context.evidence_section.contains("2"));
        assert!(context.evidence_section.contains("dax_query_result"));
    }

    #[test]
    fn compose_active_context_includes_recent_tail() {
        let compaction = sample_compaction_output();
        let recent_tail = vec![
            "User: Check the measure definition".into(),
            "Spool: Looking up Sales[QoQ Revenue]...".into(),
        ];

        let context = compose_active_context(&compaction, &recent_tail, None, None);

        assert!(context.recent_tail_section.contains("Check the measure"));
        assert!(context.recent_tail_section.contains("QoQ Revenue"));
    }

    #[test]
    fn compose_active_context_includes_knowledge() {
        let compaction = sample_compaction_output();
        let knowledge = "LOB: finance, Bundle: finance-v1, Tier1: 45 measures, Tier2: 12 rules";

        let context = compose_active_context(&compaction, &[], Some(knowledge), None);

        assert!(context.knowledge_section.is_some());
        assert!(context.knowledge_section.as_ref().unwrap().contains("finance"));
    }

    #[test]
    fn compose_active_context_without_knowledge() {
        let compaction = sample_compaction_output();
        let context = compose_active_context(&compaction, &[], None, None);

        assert!(context.knowledge_section.is_none());
    }

    #[test]
    fn compose_active_context_includes_durable_memory() {
        let compaction = sample_compaction_output();
        let memory = "Known pattern: QoQ measures often use stale quarter offsets";

        let context = compose_active_context(&compaction, &[], None, Some(memory));

        assert!(context.durable_memory_section.is_some());
        assert!(context.durable_memory_section.as_ref().unwrap().contains("QoQ"));
    }

    #[test]
    fn render_context_to_string_produces_nonempty_output() {
        let compaction = sample_compaction_output();
        let context = compose_active_context(
            &compaction,
            &["User: test".into()],
            Some("LOB knowledge"),
            None,
        );

        let rendered = context.render();
        assert!(!rendered.is_empty());
        assert!(rendered.contains("Task Contract"));
        assert!(rendered.contains("Evidence"));
        assert!(rendered.contains("Recent"));
        assert!(rendered.contains("Knowledge"));
    }

    #[test]
    fn recent_tail_truncation() {
        let compaction = sample_compaction_output();
        let mut long_tail: Vec<String> = (0..200)
            .map(|i| format!("Message {i}"))
            .collect();

        let context = compose_active_context(&compaction, &long_tail, None, None);

        // Should only include the last MAX_RECENT_TAIL_LINES messages
        let lines: Vec<&str> = context.recent_tail_section.lines().collect();
        assert!(lines.len() <= MAX_RECENT_TAIL_LINES + 2); // +2 for header/footer
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cd spool && cargo test -p spool-tui -- context`
Expected: FAIL -- types not defined yet

**Step 3: Write the implementation**

Replace the placeholder content of `spool/spool-tui/src/context.rs`:

```rust
// spool/spool-tui/src/context.rs
use crate::compaction::CompactionOutput;

/// Maximum number of recent tail lines to include in active context.
pub const MAX_RECENT_TAIL_LINES: usize = 50;

/// Active context composed for model consumption.
///
/// This implements the active context rule from Spec Section 12.4.
/// The context is composed from:
/// - task contract
/// - evidence ledger (via compaction summary)
/// - current working state
/// - recent unresolved tail
/// - selected knowledge bundle
/// - relevant durable memory
pub struct ActiveContext {
    pub task_contract_section: String,
    pub evidence_section: String,
    pub contradiction_section: String,
    pub working_state_section: String,
    pub recent_tail_section: String,
    pub knowledge_section: Option<String>,
    pub durable_memory_section: Option<String>,
}

impl ActiveContext {
    /// Render the active context to a single string for model consumption.
    pub fn render(&self) -> String {
        let mut parts = Vec::new();

        parts.push(format!("## Task Contract\n{}", self.task_contract_section));
        parts.push(format!("## Evidence Summary\n{}", self.evidence_section));

        if !self.contradiction_section.is_empty() {
            parts.push(format!("## Contradictions\n{}", self.contradiction_section));
        }

        parts.push(format!("## Working State\n{}", self.working_state_section));

        if !self.recent_tail_section.is_empty() {
            parts.push(format!("## Recent Conversation\n{}", self.recent_tail_section));
        }

        if let Some(ref knowledge) = self.knowledge_section {
            parts.push(format!("## Knowledge\n{knowledge}"));
        }

        if let Some(ref memory) = self.durable_memory_section {
            parts.push(format!("## Durable Memory\n{memory}"));
        }

        parts.join("\n\n")
    }
}

/// Compose the active context from compaction output and session state.
///
/// This is the primary context composition function called before
/// model invocations and during resume context rebuilding
/// (Spec Section 12.7).
pub fn compose_active_context(
    compaction: &CompactionOutput,
    recent_tail: &[String],
    knowledge_summary: Option<&str>,
    durable_memory_summary: Option<&str>,
) -> ActiveContext {
    let task_contract_section = render_task_contract_section(compaction);
    let evidence_section = render_evidence_section(compaction);
    let contradiction_section = render_contradiction_section(compaction);
    let working_state_section = render_working_state_section(compaction);
    let recent_tail_section = render_recent_tail_section(recent_tail);

    ActiveContext {
        task_contract_section,
        evidence_section,
        contradiction_section,
        working_state_section,
        recent_tail_section,
        knowledge_section: knowledge_summary.map(String::from),
        durable_memory_section: durable_memory_summary.map(String::from),
    }
}

fn render_task_contract_section(compaction: &CompactionOutput) -> String {
    let c = &compaction.task_contract;
    let mut lines = Vec::new();

    lines.push(format!("Task ID: {}", c.task_id.0));
    lines.push(format!("Intent: {}", c.intent));
    lines.push(format!("LOB: {}", c.scope.lob));
    lines.push(format!("Workspace: {}", c.scope.workspace));

    if !c.scope.artifacts.is_empty() {
        let artifacts: Vec<String> = c
            .scope
            .artifacts
            .iter()
            .map(|a| format!("{:?}:{}", a.artifact_type, a.reference))
            .collect();
        lines.push(format!("Artifacts: {}", artifacts.join(", ")));
    }

    if let Some(ref recipe) = c.selected_recipe {
        lines.push(format!("Recipe: {recipe}"));
    }

    if !c.assumptions.is_empty() {
        lines.push(format!("Assumptions: {}", c.assumptions.join("; ")));
    }

    lines.push(format!("Status: {:?}", c.task_status));

    lines.join("\n")
}

fn render_evidence_section(compaction: &CompactionOutput) -> String {
    let summary = &compaction.evidence_summary;
    let mut lines = Vec::new();

    lines.push(format!("Total evidence items: {}", summary.total_count));

    let mut classes: Vec<(&String, &crate::compaction::EvidenceClassSummary)> =
        summary.by_class.iter().collect();
    classes.sort_by_key(|(k, _)| k.clone());

    for (class, class_summary) in classes {
        lines.push(format!(
            "  {class}: {} item(s), latest: {}",
            class_summary.count, class_summary.latest_summary
        ));
    }

    lines.join("\n")
}

fn render_contradiction_section(compaction: &CompactionOutput) -> String {
    let summary = &compaction.contradiction_summary;

    if summary.total_count == 0 {
        return String::new();
    }

    let mut lines = Vec::new();
    lines.push(format!(
        "Contradictions: {} total, {} unresolved",
        summary.total_count, summary.unresolved_count
    ));

    if summary.has_material_unresolved {
        lines.push("WARNING: Material unresolved contradictions exist".into());
    }

    for claim in &summary.unresolved_claims {
        lines.push(format!("  - {claim}"));
    }

    lines.join("\n")
}

fn render_working_state_section(compaction: &CompactionOutput) -> String {
    let mut lines = Vec::new();

    lines.push(format!("Phase: {}", compaction.task_phase));

    if let Some(ref focus) = compaction.artifact_focus {
        lines.push(format!("Artifact focus: {focus}"));
    }

    if !compaction.unresolved_questions.is_empty() {
        lines.push(format!(
            "Pending user questions: {}",
            compaction.unresolved_questions.len()
        ));
        for q in &compaction.unresolved_questions {
            lines.push(format!("  - {}", q.question));
        }
    }

    if !compaction.unresolved_evaluator_requests.is_empty() {
        lines.push(format!(
            "Pending evaluator evidence requests: {}",
            compaction.unresolved_evaluator_requests.len()
        ));
        for r in &compaction.unresolved_evaluator_requests {
            lines.push(format!("  - {}", r.description));
        }
    }

    if let Some(ref result) = compaction.latest_result {
        lines.push(format!("Latest result state: {:?}", result.state));
        lines.push(format!("Latest confidence: {:?}", result.confidence));
    }

    lines.join("\n")
}

fn render_recent_tail_section(recent_tail: &[String]) -> String {
    if recent_tail.is_empty() {
        return String::new();
    }

    let start = if recent_tail.len() > MAX_RECENT_TAIL_LINES {
        recent_tail.len() - MAX_RECENT_TAIL_LINES
    } else {
        0
    };

    recent_tail[start..].join("\n")
}

// tests at bottom of file (from Step 1)
```

**Step 4: Run test to verify it passes**

Run: `cd spool && cargo test -p spool-tui -- context`
Expected: 8 tests PASS

**Step 5: Commit**

```bash
git add spool/spool-tui/src/context.rs
git commit -m "feat(spool-tui): active context composition from compaction output per Spec Section 12.4"
```

---

## Task 6: Resume Semantics

**Files:**

- Modify: `spool/spool-tui/src/resume.rs`

**Step 1: Write the failing test**

Add to `spool/spool-tui/src/resume.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use spool_protocol::artifact::ArtifactType;
    use spool_protocol::checkpoint::{
        AskUserQuestion, CheckpointClass, CheckpointPolicy, CheckpointTrigger, UserAnswer,
    };
    use spool_protocol::contradiction::{
        ContradictionId, ContradictionRecord, ContradictionStatus, MaterialityLevel,
    };
    use spool_protocol::evidence::{EvidenceClass, EvidenceId, EvidenceItem, EvidenceType};
    use spool_protocol::task_contract::*;
    use spool_protocol::task_result::*;
    use spool_core::persistence::SessionState;

    fn make_contract(id: &str, status: TaskStatus) -> TaskContract {
        TaskContract {
            task_id: TaskId(id.into()),
            intent: format!("Task {id}"),
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
            expected_evidence_classes: vec![],
            validation_floor: ValidationFloor::DirectValidationRequired,
            checkpoint_policy: CheckpointPolicy {
                ask_on: vec![CheckpointTrigger::Ambiguous],
            },
            clarification_checkpoints: vec![],
            approval_checkpoints: vec![],
            expected_deliverable_shape: "structured_task_result".into(),
            evaluator_packet_requirements: vec![],
            task_status: status,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        }
    }

    fn sample_session_state() -> SessionState {
        SessionState {
            session_id: "session_1".into(),
            created_at: Utc::now(),
            selected_lob: "finance".into(),
            workspace_scope: "Executive BI".into(),
            task_contracts: vec![
                make_contract("task_1", TaskStatus::Completed),
                make_contract("task_2", TaskStatus::Active),
            ],
            task_results: vec![TaskResult {
                task_id: TaskId("task_1".into()),
                proposed_state: Some(ResultState::Confirmed),
                state: ResultState::Confirmed,
                confidence: Confidence::High,
                summary: "Completed".into(),
                findings: vec![],
                evidence_refs: vec![],
                validation_results: vec![],
                recommended_actions: vec![],
                blockers: vec![],
                open_questions: vec![],
                proposed_changes: vec![],
                contradiction_refs: vec![],
                result_generated_at: Some(Utc::now()),
                result_version: Some(1),
            }],
            evidence_items: vec![EvidenceItem {
                id: EvidenceId("ev_1".into()),
                evidence_type: EvidenceType::Observed,
                evidence_class: EvidenceClass::DaxQueryResult,
                source: "test".into(),
                summary: "DAX result".into(),
                artifact_refs: vec![],
                observed_at: Some(Utc::now()),
                detail: None,
            }],
            contradiction_records: vec![],
            checkpoint_history: vec![],
        }
    }

    #[test]
    fn resolve_active_task() {
        let state = sample_session_state();
        let resolved = resolve_resume_task(&state);

        // Should prefer active task (task_2) over completed (task_1)
        assert_eq!(resolved.task_contract.task_id.0, "task_2");
        assert_eq!(resolved.resolution_rule, ResumeResolutionRule::ActiveTask);
    }

    #[test]
    fn resolve_most_recent_incomplete_when_no_active() {
        let mut state = sample_session_state();
        // Change task_2 to Interrupted (incomplete but not Active)
        state.task_contracts[1].task_status = TaskStatus::Interrupted;

        let resolved = resolve_resume_task(&state);
        assert_eq!(resolved.task_contract.task_id.0, "task_2");
        assert_eq!(
            resolved.resolution_rule,
            ResumeResolutionRule::MostRecentIncomplete
        );
    }

    #[test]
    fn resolve_most_recent_completed_when_all_complete() {
        let mut state = sample_session_state();
        state.task_contracts[1].task_status = TaskStatus::Completed;

        let resolved = resolve_resume_task(&state);
        // task_2 is last in the list, so it is "most recent"
        assert_eq!(resolved.task_contract.task_id.0, "task_2");
        assert_eq!(
            resolved.resolution_rule,
            ResumeResolutionRule::MostRecentCompleted
        );
    }

    #[test]
    fn build_resume_context_from_session_state() {
        let state = sample_session_state();
        let context = build_resume_context(&state);

        assert_eq!(context.session_id, "session_1");
        assert_eq!(context.selected_lob, "finance");
        assert_eq!(context.workspace_scope, "Executive BI");
        assert_eq!(context.active_task.task_id.0, "task_2");
        assert_eq!(context.evidence_items.len(), 1);
        assert_eq!(context.contradiction_records.len(), 0);
        assert_eq!(context.task_results.len(), 1);
    }

    #[test]
    fn interrupted_task_surfaces_explicitly() {
        let mut state = sample_session_state();
        state.task_contracts[1].task_status = TaskStatus::Interrupted;

        let context = build_resume_context(&state);
        let info = context.interrupted_task_info.unwrap();

        assert_eq!(info.task_id, "task_2");
        assert!(info.was_interrupted);
        // No pending question or evaluator request in this fixture
        assert!(!info.user_answer_pending);
        assert!(!info.evaluator_evidence_pending);
    }

    #[test]
    fn interrupted_task_with_pending_question() {
        let mut state = sample_session_state();
        state.task_contracts[1].task_status = TaskStatus::Interrupted;
        state.checkpoint_history.push(AskUserQuestion {
            id: "q_1".into(),
            checkpoint_class: CheckpointClass::Information,
            trigger: CheckpointTrigger::Ambiguous,
            question: "Which report?".into(),
            options: None,
            allows_free_text: true,
            asked_at: Utc::now(),
            answer: None, // unanswered
        });

        let context = build_resume_context(&state);
        let info = context.interrupted_task_info.unwrap();

        assert!(info.user_answer_pending);
        assert_eq!(info.pending_questions.len(), 1);
    }

    #[test]
    fn resume_context_serializes() {
        let state = sample_session_state();
        let context = build_resume_context(&state);
        let json = serde_json::to_string_pretty(&context).unwrap();
        let restored: ResumeContext = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.session_id, "session_1");
        assert_eq!(restored.active_task.task_id.0, "task_2");
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cd spool && cargo test -p spool-tui -- resume`
Expected: FAIL -- types not defined yet

**Step 3: Write the implementation**

Replace the placeholder content of `spool/spool-tui/src/resume.rs`:

```rust
// spool/spool-tui/src/resume.rs
use serde::{Deserialize, Serialize};
use spool_core::persistence::SessionState;
use spool_protocol::checkpoint::AskUserQuestion;
use spool_protocol::contradiction::ContradictionRecord;
use spool_protocol::evidence::EvidenceItem;
use spool_protocol::task_contract::{TaskContract, TaskStatus};
use spool_protocol::task_result::TaskResult;

/// Which resolution rule was used to select the resume task.
///
/// Per Spec Section 12.6:
/// 1. active task if one is marked active
/// 2. most recently updated incomplete task
/// 3. most recent completed task
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResumeResolutionRule {
    ActiveTask,
    MostRecentIncomplete,
    MostRecentCompleted,
}

/// Result of resume task resolution.
pub struct ResolvedResumeTask {
    pub task_contract: TaskContract,
    pub resolution_rule: ResumeResolutionRule,
}

/// Information about an interrupted task for explicit surfacing.
///
/// Per Spec Section 12.8, resume should surface interrupted state
/// explicitly rather than pretending the task ended cleanly.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterruptedTaskInfo {
    pub task_id: String,
    pub was_interrupted: bool,
    pub last_phase: String,
    pub user_answer_pending: bool,
    pub evaluator_evidence_pending: bool,
    pub pending_questions: Vec<AskUserQuestion>,
    pub last_known_state: String,
}

/// Full resume context rebuilt from persisted session state.
///
/// This implements the resume semantics from Spec Section 12.5.
/// Resume is session-level from the user point of view while
/// internally restoring all structured state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResumeContext {
    pub session_id: String,
    pub selected_lob: String,
    pub workspace_scope: String,
    pub active_task: TaskContract,
    pub resolution_rule: ResumeResolutionRule,
    pub evidence_items: Vec<EvidenceItem>,
    pub contradiction_records: Vec<ContradictionRecord>,
    pub checkpoint_history: Vec<AskUserQuestion>,
    pub task_results: Vec<TaskResult>,
    pub interrupted_task_info: Option<InterruptedTaskInfo>,
}

/// Resolve which task to resume from persisted session state.
///
/// Per Spec Section 12.6:
/// 1. the active task if one is marked active
/// 2. otherwise the most recently updated incomplete task
/// 3. otherwise the most recent completed task
pub fn resolve_resume_task(state: &SessionState) -> ResolvedResumeTask {
    // Rule 1: active task
    if let Some(active) = state
        .task_contracts
        .iter()
        .find(|t| t.task_status == TaskStatus::Active)
    {
        return ResolvedResumeTask {
            task_contract: active.clone(),
            resolution_rule: ResumeResolutionRule::ActiveTask,
        };
    }

    // Rule 2: most recently updated incomplete task
    let incomplete_statuses = [
        TaskStatus::Planning,
        TaskStatus::Evaluating,
        TaskStatus::Blocked,
        TaskStatus::Interrupted,
    ];

    if let Some(incomplete) = state
        .task_contracts
        .iter()
        .rev()
        .find(|t| incomplete_statuses.contains(&t.task_status))
    {
        return ResolvedResumeTask {
            task_contract: incomplete.clone(),
            resolution_rule: ResumeResolutionRule::MostRecentIncomplete,
        };
    }

    // Rule 3: most recent completed task
    let last = state
        .task_contracts
        .last()
        .expect("session state must have at least one task contract");

    ResolvedResumeTask {
        task_contract: last.clone(),
        resolution_rule: ResumeResolutionRule::MostRecentCompleted,
    }
}

/// Build the full resume context from persisted session state.
///
/// This implements the resume context composition from Spec Section 12.7.
/// The context prioritizes structured state over raw transcript replay.
pub fn build_resume_context(state: &SessionState) -> ResumeContext {
    let resolved = resolve_resume_task(state);
    let interrupted_task_info = build_interrupted_task_info(state, &resolved);

    ResumeContext {
        session_id: state.session_id.clone(),
        selected_lob: state.selected_lob.clone(),
        workspace_scope: state.workspace_scope.clone(),
        active_task: resolved.task_contract,
        resolution_rule: resolved.resolution_rule,
        evidence_items: state.evidence_items.clone(),
        contradiction_records: state.contradiction_records.clone(),
        checkpoint_history: state.checkpoint_history.clone(),
        task_results: state.task_results.clone(),
        interrupted_task_info,
    }
}

fn build_interrupted_task_info(
    state: &SessionState,
    resolved: &ResolvedResumeTask,
) -> Option<InterruptedTaskInfo> {
    let task = &resolved.task_contract;

    if task.task_status != TaskStatus::Interrupted {
        return None;
    }

    let pending_questions: Vec<AskUserQuestion> = state
        .checkpoint_history
        .iter()
        .filter(|q| q.answer.is_none())
        .cloned()
        .collect();

    let user_answer_pending = !pending_questions.is_empty();

    // In v1, evaluator evidence pending is detected by checking if the
    // task was in evaluating phase. Full evaluator request tracking
    // requires integration with the evaluator loop state.
    let evaluator_evidence_pending = false;

    let last_phase = format!("{:?}", task.task_status);
    let last_known_state = if let Some(result) = state.task_results.last() {
        format!("{:?}", result.state)
    } else {
        "not_yet_finalized".into()
    };

    Some(InterruptedTaskInfo {
        task_id: task.task_id.0.clone(),
        was_interrupted: true,
        last_phase,
        user_answer_pending,
        evaluator_evidence_pending,
        pending_questions,
        last_known_state,
    })
}

// tests at bottom of file (from Step 1)
```

**Step 4: Run test to verify it passes**

Run: `cd spool && cargo test -p spool-tui -- resume`
Expected: 7 tests PASS

**Step 5: Commit**

```bash
git add spool/spool-tui/src/resume.rs
git commit -m "feat(spool-tui): resume semantics with task resolution rules and interrupted task surfacing per Spec Sections 12.5-12.8"
```

---

## Task 7: Status Bar Widget

**Files:**

- Modify: `spool/spool-tui/src/widgets/status_bar.rs`

**Step 1: Write the failing test**

Add to `spool/spool-tui/src/widgets/status_bar.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;

    #[test]
    fn status_bar_renders_session_info() {
        let props = StatusBarProps {
            session_id: Some("session_1".into()),
            selected_lob: "finance".into(),
            workspace_scope: "Executive BI".into(),
            view_mode: "Progress".into(),
            task_phase: Some("Investigating".into()),
        };

        let backend = TestBackend::new(80, 1);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                let area = frame.area();
                render_status_bar(frame, area, &props);
            })
            .unwrap();

        let buffer = terminal.backend().buffer().clone();
        let content = buffer_to_string(&buffer);

        assert!(content.contains("finance"));
        assert!(content.contains("Executive BI"));
        assert!(content.contains("Progress"));
    }

    #[test]
    fn status_bar_renders_without_session() {
        let props = StatusBarProps {
            session_id: None,
            selected_lob: String::new(),
            workspace_scope: String::new(),
            view_mode: "Startup".into(),
            task_phase: None,
        };

        let backend = TestBackend::new(80, 1);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                let area = frame.area();
                render_status_bar(frame, area, &props);
            })
            .unwrap();

        let buffer = terminal.backend().buffer().clone();
        let content = buffer_to_string(&buffer);

        assert!(content.contains("Spool"));
    }

    fn buffer_to_string(buffer: &Buffer) -> String {
        let mut output = String::new();
        for y in 0..buffer.area.height {
            for x in 0..buffer.area.width {
                let cell = &buffer[(x, y)];
                output.push_str(cell.symbol());
            }
        }
        output
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cd spool && cargo test -p spool-tui -- widgets::status_bar`
Expected: FAIL -- types not defined yet

**Step 3: Write the implementation**

Replace the placeholder content of `spool/spool-tui/src/widgets/status_bar.rs`:

```rust
// spool/spool-tui/src/widgets/status_bar.rs
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

/// Props for the status bar widget.
pub struct StatusBarProps {
    pub session_id: Option<String>,
    pub selected_lob: String,
    pub workspace_scope: String,
    pub view_mode: String,
    pub task_phase: Option<String>,
}

/// Render the status bar at the bottom of the terminal.
///
/// The status bar shows session identity, LOB, workspace, current view
/// mode, and task phase. This provides persistent orientation context
/// across all view modes.
pub fn render_status_bar(frame: &mut Frame, area: Rect, props: &StatusBarProps) {
    let style = Style::default()
        .bg(Color::DarkGray)
        .fg(Color::White);

    let mut spans = Vec::new();

    spans.push(Span::styled(
        " Spool ",
        Style::default()
            .bg(Color::Blue)
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    ));

    if let Some(ref session_id) = props.session_id {
        spans.push(Span::styled(
            format!(" {} ", props.selected_lob),
            style,
        ));
        spans.push(Span::styled(
            format!("| {} ", props.workspace_scope),
            style,
        ));
    }

    spans.push(Span::styled(
        format!("| {} ", props.view_mode),
        Style::default()
            .bg(Color::DarkGray)
            .fg(Color::Yellow),
    ));

    if let Some(ref phase) = props.task_phase {
        spans.push(Span::styled(
            format!("| {phase} "),
            style,
        ));
    }

    // Fill remaining width with background
    let line = Line::from(spans);
    let paragraph = Paragraph::new(line).style(style);
    frame.render_widget(paragraph, area);
}

// tests at bottom of file (from Step 1)
```

**Step 4: Run test to verify it passes**

Run: `cd spool && cargo test -p spool-tui -- widgets::status_bar`
Expected: 2 tests PASS

**Step 5: Commit**

```bash
git add spool/spool-tui/src/widgets/status_bar.rs
git commit -m "feat(spool-tui): status bar widget showing session, LOB, workspace, view mode, and task phase"
```

---

## Task 8: Plan Mode Widget

**Files:**

- Modify: `spool/spool-tui/src/widgets/plan_mode.rs`

**Step 1: Write the failing test**

Add to `spool/spool-tui/src/widgets/plan_mode.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;

    fn buffer_to_string(buffer: &Buffer) -> String {
        let mut output = String::new();
        for y in 0..buffer.area.height {
            for x in 0..buffer.area.width {
                let cell = &buffer[(x, y)];
                output.push_str(cell.symbol());
            }
        }
        output
    }

    #[test]
    fn plan_mode_renders_empty_state() {
        let props = PlanModeProps {
            intent: String::new(),
            selected_lob: "finance".into(),
            workspace_scope: "Executive BI".into(),
            artifacts: vec![],
            selected_recipe: None,
            assumptions: vec![],
            can_finalize: false,
            focused_field: PlanModeField::Intent,
        };

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                let area = frame.area();
                render_plan_mode(frame, area, &props);
            })
            .unwrap();

        let buffer = terminal.backend().buffer().clone();
        let content = buffer_to_string(&buffer);

        assert!(content.contains("Plan Mode"));
        assert!(content.contains("finance"));
        assert!(content.contains("Executive BI"));
    }

    #[test]
    fn plan_mode_renders_populated_state() {
        let props = PlanModeProps {
            intent: "Investigate revenue mismatch".into(),
            selected_lob: "finance".into(),
            workspace_scope: "Executive BI".into(),
            artifacts: vec![
                PlanModeArtifact {
                    artifact_type: "report".into(),
                    reference: "Executive Revenue Report".into(),
                },
                PlanModeArtifact {
                    artifact_type: "measure".into(),
                    reference: "Sales Model.Revenue".into(),
                },
            ],
            selected_recipe: Some("report_number_mismatch".into()),
            assumptions: vec!["Published report in Executive BI".into()],
            can_finalize: true,
            focused_field: PlanModeField::Artifacts,
        };

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                let area = frame.area();
                render_plan_mode(frame, area, &props);
            })
            .unwrap();

        let buffer = terminal.backend().buffer().clone();
        let content = buffer_to_string(&buffer);

        assert!(content.contains("revenue mismatch"));
        assert!(content.contains("Revenue Report"));
        assert!(content.contains("report_number_mismatch"));
    }

    #[test]
    fn plan_mode_shows_finalize_option() {
        let props = PlanModeProps {
            intent: "Test".into(),
            selected_lob: "test".into(),
            workspace_scope: "ws".into(),
            artifacts: vec![PlanModeArtifact {
                artifact_type: "report".into(),
                reference: "R".into(),
            }],
            selected_recipe: None,
            assumptions: vec![],
            can_finalize: true,
            focused_field: PlanModeField::Intent,
        };

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                let area = frame.area();
                render_plan_mode(frame, area, &props);
            })
            .unwrap();

        let buffer = terminal.backend().buffer().clone();
        let content = buffer_to_string(&buffer);

        assert!(content.contains("Start") || content.contains("Finalize"));
    }

    #[test]
    fn all_plan_mode_fields_exist() {
        let fields = vec![
            PlanModeField::Intent,
            PlanModeField::Lob,
            PlanModeField::Workspace,
            PlanModeField::Artifacts,
            PlanModeField::Recipe,
            PlanModeField::Assumptions,
        ];
        assert_eq!(fields.len(), 6);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cd spool && cargo test -p spool-tui -- widgets::plan_mode`
Expected: FAIL -- types not defined yet

**Step 3: Write the implementation**

Replace the placeholder content of `spool/spool-tui/src/widgets/plan_mode.rs`:

```rust
// spool/spool-tui/src/widgets/plan_mode.rs
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};

/// Which field is currently focused in plan mode navigation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanModeField {
    Intent,
    Lob,
    Workspace,
    Artifacts,
    Recipe,
    Assumptions,
}

/// An artifact entry in the plan mode artifact list.
#[derive(Debug, Clone)]
pub struct PlanModeArtifact {
    pub artifact_type: String,
    pub reference: String,
}

/// Props for the plan mode widget.
///
/// Plan mode is analytics-native (Spec Section 13.1) and used to:
/// refine the request, select scope, define artifacts, select recipes,
/// define evidence expectations, and finalize the task contract.
pub struct PlanModeProps {
    pub intent: String,
    pub selected_lob: String,
    pub workspace_scope: String,
    pub artifacts: Vec<PlanModeArtifact>,
    pub selected_recipe: Option<String>,
    pub assumptions: Vec<String>,
    pub can_finalize: bool,
    pub focused_field: PlanModeField,
}

/// Render the plan mode surface.
///
/// Layout:
/// - Header: "Plan Mode" title
/// - Intent field
/// - LOB and workspace fields
/// - Artifacts list
/// - Recipe selection
/// - Assumptions
/// - Footer: finalize/keep refining options
pub fn render_plan_mode(frame: &mut Frame, area: Rect, props: &PlanModeProps) {
    let chunks = Layout::vertical([
        Constraint::Length(3),  // header
        Constraint::Length(3),  // intent
        Constraint::Length(3),  // lob + workspace
        Constraint::Min(4),    // artifacts
        Constraint::Length(3),  // recipe
        Constraint::Length(3),  // assumptions
        Constraint::Length(2),  // footer
    ])
    .split(area);

    // Header
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            " Plan Mode ",
            Style::default()
                .fg(Color::White)
                .bg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Define your investigation task contract"),
    ]))
    .block(Block::default().borders(Borders::BOTTOM));
    frame.render_widget(header, chunks[0]);

    // Intent
    let intent_style = if props.focused_field == PlanModeField::Intent {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
    };
    let intent_text = if props.intent.is_empty() {
        "Type your investigation request..."
    } else {
        &props.intent
    };
    let intent = Paragraph::new(Line::from(vec![
        Span::styled("Intent: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::styled(intent_text, intent_style),
    ]))
    .block(Block::default().borders(Borders::BOTTOM));
    frame.render_widget(intent, chunks[1]);

    // LOB and Workspace
    let scope = Paragraph::new(Line::from(vec![
        Span::styled("LOB: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(format!("{} ", props.selected_lob)),
        Span::styled("| Workspace: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(&props.workspace_scope),
    ]))
    .block(Block::default().borders(Borders::BOTTOM));
    frame.render_widget(scope, chunks[2]);

    // Artifacts
    let artifact_items: Vec<ListItem> = if props.artifacts.is_empty() {
        vec![ListItem::new("  (no artifacts defined)")]
    } else {
        props
            .artifacts
            .iter()
            .enumerate()
            .map(|(i, a)| {
                ListItem::new(format!("  {}. [{}] {}", i + 1, a.artifact_type, a.reference))
            })
            .collect()
    };
    let artifacts_style = if props.focused_field == PlanModeField::Artifacts {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let artifacts = List::new(artifact_items)
        .block(
            Block::default()
                .title("Artifacts")
                .borders(Borders::ALL)
                .border_style(artifacts_style),
        );
    frame.render_widget(artifacts, chunks[3]);

    // Recipe
    let recipe_text = props
        .selected_recipe
        .as_deref()
        .unwrap_or("(none selected)");
    let recipe = Paragraph::new(Line::from(vec![
        Span::styled("Recipe: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(recipe_text),
    ]))
    .block(Block::default().borders(Borders::BOTTOM));
    frame.render_widget(recipe, chunks[4]);

    // Assumptions
    let assumptions_text = if props.assumptions.is_empty() {
        "(none)".into()
    } else {
        props.assumptions.join("; ")
    };
    let assumptions = Paragraph::new(Line::from(vec![
        Span::styled("Assumptions: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(assumptions_text),
    ]));
    frame.render_widget(assumptions, chunks[5]);

    // Footer
    let footer_text = if props.can_finalize {
        "[Enter] Start investigation  [Tab] Keep refining  [Esc] Cancel"
    } else {
        "[Tab] Navigate fields  [Esc] Cancel"
    };
    let footer = Paragraph::new(Line::from(Span::styled(
        footer_text,
        Style::default().fg(Color::DarkGray),
    )));
    frame.render_widget(footer, chunks[6]);
}

// tests at bottom of file (from Step 1)
```

**Step 4: Run test to verify it passes**

Run: `cd spool && cargo test -p spool-tui -- widgets::plan_mode`
Expected: 4 tests PASS

**Step 5: Commit**

```bash
git add spool/spool-tui/src/widgets/plan_mode.rs
git commit -m "feat(spool-tui): plan mode widget with intent, scope, artifacts, recipe, and finalize flow per Spec Section 13.1"
```

---

## Task 9: Progress Surface Widget

**Files:**

- Modify: `spool/spool-tui/src/widgets/progress.rs`

**Step 1: Write the failing test**

Add to `spool/spool-tui/src/widgets/progress.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    use ratatui::buffer::Buffer;

    fn buffer_to_string(buffer: &Buffer) -> String {
        let mut output = String::new();
        for y in 0..buffer.area.height {
            for x in 0..buffer.area.width {
                let cell = &buffer[(x, y)];
                output.push_str(cell.symbol());
            }
        }
        output
    }

    #[test]
    fn progress_renders_investigating_phase() {
        let props = ProgressProps {
            task_phase: "Investigating".into(),
            artifact_focus: Some("Executive Revenue Report".into()),
            latest_finding: Some("Measure uses stale quarter offset".into()),
            waiting_state: None,
            evidence_count: 3,
            contradiction_count: 0,
        };

        let backend = TestBackend::new(80, 12);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                let area = frame.area();
                render_progress(frame, area, &props);
            })
            .unwrap();

        let buffer = terminal.backend().buffer().clone();
        let content = buffer_to_string(&buffer);

        assert!(content.contains("Investigating"));
        assert!(content.contains("Revenue Report"));
        assert!(content.contains("stale quarter"));
    }

    #[test]
    fn progress_renders_waiting_state() {
        let props = ProgressProps {
            task_phase: "Investigating".into(),
            artifact_focus: None,
            latest_finding: None,
            waiting_state: Some(ProgressWaiting::UserInput {
                question: "Which report do you mean?".into(),
            }),
            evidence_count: 0,
            contradiction_count: 0,
        };

        let backend = TestBackend::new(80, 12);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                let area = frame.area();
                render_progress(frame, area, &props);
            })
            .unwrap();

        let buffer = terminal.backend().buffer().clone();
        let content = buffer_to_string(&buffer);

        assert!(content.contains("Waiting"));
        assert!(content.contains("report"));
    }

    #[test]
    fn progress_renders_evaluating_phase() {
        let props = ProgressProps {
            task_phase: "Evaluating".into(),
            artifact_focus: None,
            latest_finding: Some("Revenue variance identified".into()),
            waiting_state: None,
            evidence_count: 5,
            contradiction_count: 1,
        };

        let backend = TestBackend::new(80, 12);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                let area = frame.area();
                render_progress(frame, area, &props);
            })
            .unwrap();

        let buffer = terminal.backend().buffer().clone();
        let content = buffer_to_string(&buffer);

        assert!(content.contains("Evaluating"));
        assert!(content.contains("5"));
    }

    #[test]
    fn progress_renders_with_contradictions() {
        let props = ProgressProps {
            task_phase: "Investigating".into(),
            artifact_focus: None,
            latest_finding: None,
            waiting_state: None,
            evidence_count: 2,
            contradiction_count: 1,
        };

        let backend = TestBackend::new(80, 12);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                let area = frame.area();
                render_progress(frame, area, &props);
            })
            .unwrap();

        let buffer = terminal.backend().buffer().clone();
        let content = buffer_to_string(&buffer);

        assert!(content.contains("1"));
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cd spool && cargo test -p spool-tui -- widgets::progress`
Expected: FAIL -- types not defined yet

**Step 3: Write the implementation**

Replace the placeholder content of `spool/spool-tui/src/widgets/progress.rs`:

```rust
// spool/spool-tui/src/widgets/progress.rs
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

/// Waiting state for the progress surface.
#[derive(Debug, Clone)]
pub enum ProgressWaiting {
    UserInput { question: String },
    ExternalResponse { description: String },
    EvaluatorReview,
}

/// Props for the progress surface widget.
///
/// The default progress surface emphasizes (Spec Section 13.2):
/// - current phase
/// - current artifact under investigation
/// - latest meaningful finding
/// - waiting state if blocked
pub struct ProgressProps {
    pub task_phase: String,
    pub artifact_focus: Option<String>,
    pub latest_finding: Option<String>,
    pub waiting_state: Option<ProgressWaiting>,
    pub evidence_count: usize,
    pub contradiction_count: usize,
}

/// Render the progress surface.
///
/// Layout:
/// - Phase indicator with status
/// - Artifact focus (if any)
/// - Latest finding (if any)
/// - Waiting state (if blocked/waiting)
/// - Evidence and contradiction counts
pub fn render_progress(frame: &mut Frame, area: Rect, props: &ProgressProps) {
    let chunks = Layout::vertical([
        Constraint::Length(3),  // phase header
        Constraint::Length(2),  // artifact focus
        Constraint::Min(3),    // latest finding
        Constraint::Length(2),  // waiting state
        Constraint::Length(2),  // counters
    ])
    .split(area);

    // Phase header
    let phase_color = match props.task_phase.as_str() {
        "Investigating" => Color::Green,
        "Evaluating" => Color::Yellow,
        "Planning" => Color::Cyan,
        "Finalizing" => Color::Blue,
        _ => Color::White,
    };

    let phase = Paragraph::new(Line::from(vec![
        Span::styled(
            format!(" {} ", props.task_phase),
            Style::default()
                .fg(Color::White)
                .bg(phase_color)
                .add_modifier(Modifier::BOLD),
        ),
    ]))
    .block(
        Block::default()
            .title("Progress")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(phase_color)),
    );
    frame.render_widget(phase, chunks[0]);

    // Artifact focus
    let artifact_text = props
        .artifact_focus
        .as_deref()
        .map(|a| format!("Artifact: {a}"))
        .unwrap_or_default();
    let artifact = Paragraph::new(Line::from(Span::styled(
        artifact_text,
        Style::default().fg(Color::White),
    )));
    frame.render_widget(artifact, chunks[1]);

    // Latest finding
    let finding_text = props
        .latest_finding
        .as_deref()
        .map(|f| format!("Latest: {f}"))
        .unwrap_or_else(|| "No findings yet".into());
    let finding = Paragraph::new(Line::from(Span::styled(
        finding_text,
        Style::default().fg(Color::White),
    )))
    .block(Block::default().borders(Borders::TOP));
    frame.render_widget(finding, chunks[2]);

    // Waiting state
    if let Some(ref waiting) = props.waiting_state {
        let waiting_text = match waiting {
            ProgressWaiting::UserInput { question } => {
                format!("Waiting for input: {question}")
            }
            ProgressWaiting::ExternalResponse { description } => {
                format!("Waiting: {description}")
            }
            ProgressWaiting::EvaluatorReview => "Evaluator reviewing...".into(),
        };
        let waiting_widget = Paragraph::new(Line::from(Span::styled(
            waiting_text,
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )));
        frame.render_widget(waiting_widget, chunks[3]);
    }

    // Counters
    let mut counter_spans = vec![
        Span::styled(
            format!("Evidence: {} ", props.evidence_count),
            Style::default().fg(Color::Green),
        ),
    ];

    if props.contradiction_count > 0 {
        counter_spans.push(Span::styled(
            format!("| Contradictions: {} ", props.contradiction_count),
            Style::default().fg(Color::Red),
        ));
    }

    let counters = Paragraph::new(Line::from(counter_spans));
    frame.render_widget(counters, chunks[4]);
}

// tests at bottom of file (from Step 1)
```

**Step 4: Run test to verify it passes**

Run: `cd spool && cargo test -p spool-tui -- widgets::progress`
Expected: 4 tests PASS

**Step 5: Commit**

```bash
git add spool/spool-tui/src/widgets/progress.rs
git commit -m "feat(spool-tui): progress surface widget with phase, artifact focus, findings, and waiting state per Spec Section 13.2"
```

---

## Task 10: Advanced View Widget

**Files:**

- Modify: `spool/spool-tui/src/widgets/advanced_view.rs`

**Step 1: Write the failing test**

Add to `spool/spool-tui/src/widgets/advanced_view.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    use ratatui::buffer::Buffer;

    fn buffer_to_string(buffer: &Buffer) -> String {
        let mut output = String::new();
        for y in 0..buffer.area.height {
            for x in 0..buffer.area.width {
                let cell = &buffer[(x, y)];
                output.push_str(cell.symbol());
            }
        }
        output
    }

    #[test]
    fn advanced_view_renders_transcript() {
        let props = AdvancedViewProps {
            transcript_entries: vec![
                TranscriptEntry {
                    role: HarnessRole::Planner,
                    action: "Created task contract for revenue mismatch investigation".into(),
                    timestamp: "10:30:01".into(),
                },
                TranscriptEntry {
                    role: HarnessRole::Generator,
                    action: "Inspecting report metadata for Executive Revenue Report".into(),
                    timestamp: "10:30:05".into(),
                },
                TranscriptEntry {
                    role: HarnessRole::Generator,
                    action: "Running DAX query for QoQ Revenue measure".into(),
                    timestamp: "10:30:08".into(),
                },
                TranscriptEntry {
                    role: HarnessRole::Evaluator,
                    action: "Reviewing evidence packet: 3 items, 0 contradictions".into(),
                    timestamp: "10:30:12".into(),
                },
            ],
            evidence_entries: vec![
                EvidenceLedgerEntry {
                    id: "ev_1".into(),
                    class: "dax_query_result".into(),
                    summary: "DAX returned 12.4M for Q1".into(),
                },
                EvidenceLedgerEntry {
                    id: "ev_2".into(),
                    class: "warehouse_query_result".into(),
                    summary: "Warehouse returned 12.4M for Q1".into(),
                },
            ],
            contradiction_entries: vec![],
            evaluator_disagreement: None,
        };

        let backend = TestBackend::new(100, 30);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                let area = frame.area();
                render_advanced_view(frame, area, &props);
            })
            .unwrap();

        let buffer = terminal.backend().buffer().clone();
        let content = buffer_to_string(&buffer);

        assert!(content.contains("Planner"));
        assert!(content.contains("Generator"));
        assert!(content.contains("Evaluator"));
        assert!(content.contains("dax_query_result"));
    }

    #[test]
    fn advanced_view_renders_contradictions() {
        let props = AdvancedViewProps {
            transcript_entries: vec![],
            evidence_entries: vec![],
            contradiction_entries: vec![AdvancedContradictionEntry {
                id: "c_1".into(),
                status: "Open".into(),
                claim: "Revenue mismatch between DAX and warehouse".into(),
                materiality: "Material".into(),
            }],
            evaluator_disagreement: None,
        };

        let backend = TestBackend::new(100, 20);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                let area = frame.area();
                render_advanced_view(frame, area, &props);
            })
            .unwrap();

        let buffer = terminal.backend().buffer().clone();
        let content = buffer_to_string(&buffer);

        assert!(content.contains("Contradiction"));
        assert!(content.contains("Material"));
    }

    #[test]
    fn advanced_view_renders_evaluator_disagreement() {
        let props = AdvancedViewProps {
            transcript_entries: vec![],
            evidence_entries: vec![],
            contradiction_entries: vec![],
            evaluator_disagreement: Some(EvaluatorDisagreement {
                generator_proposed: "confirmed".into(),
                evaluator_assigned: "supported_hypothesis".into(),
                reason: "Evidence insufficient for full confirmation".into(),
            }),
        };

        let backend = TestBackend::new(100, 20);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                let area = frame.area();
                render_advanced_view(frame, area, &props);
            })
            .unwrap();

        let buffer = terminal.backend().buffer().clone();
        let content = buffer_to_string(&buffer);

        assert!(content.contains("Disagreement") || content.contains("disagreement"));
    }

    #[test]
    fn harness_roles_exist() {
        let roles = vec![
            HarnessRole::Planner,
            HarnessRole::Generator,
            HarnessRole::Evaluator,
        ];
        assert_eq!(roles.len(), 3);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cd spool && cargo test -p spool-tui -- widgets::advanced_view`
Expected: FAIL -- types not defined yet

**Step 3: Write the implementation**

Replace the placeholder content of `spool/spool-tui/src/widgets/advanced_view.rs`:

```rust
// spool/spool-tui/src/widgets/advanced_view.rs
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};

/// Harness role for transcript entries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HarnessRole {
    Planner,
    Generator,
    Evaluator,
}

impl HarnessRole {
    fn label(&self) -> &str {
        match self {
            HarnessRole::Planner => "Planner",
            HarnessRole::Generator => "Generator",
            HarnessRole::Evaluator => "Evaluator",
        }
    }

    fn color(&self) -> Color {
        match self {
            HarnessRole::Planner => Color::Cyan,
            HarnessRole::Generator => Color::Green,
            HarnessRole::Evaluator => Color::Yellow,
        }
    }
}

/// A single entry in the structured transcript.
#[derive(Debug, Clone)]
pub struct TranscriptEntry {
    pub role: HarnessRole,
    pub action: String,
    pub timestamp: String,
}

/// An entry in the evidence ledger detail view.
#[derive(Debug, Clone)]
pub struct EvidenceLedgerEntry {
    pub id: String,
    pub class: String,
    pub summary: String,
}

/// A contradiction entry for the advanced view.
#[derive(Debug, Clone)]
pub struct AdvancedContradictionEntry {
    pub id: String,
    pub status: String,
    pub claim: String,
    pub materiality: String,
}

/// Evaluator/generator disagreement detail.
///
/// Per Spec Section 13.3, advanced view should expand on disagreement
/// details, but material disagreement must already be visible in the
/// normal answer.
#[derive(Debug, Clone)]
pub struct EvaluatorDisagreement {
    pub generator_proposed: String,
    pub evaluator_assigned: String,
    pub reason: String,
}

/// Props for the advanced view widget.
///
/// Advanced view exposes (Spec Section 13.3):
/// - structured transcript
/// - planner/generator/evaluator activity
/// - evidence ledger detail
/// - intermediate artifacts
///
/// It should not expose hidden chain-of-thought.
pub struct AdvancedViewProps {
    pub transcript_entries: Vec<TranscriptEntry>,
    pub evidence_entries: Vec<EvidenceLedgerEntry>,
    pub contradiction_entries: Vec<AdvancedContradictionEntry>,
    pub evaluator_disagreement: Option<EvaluatorDisagreement>,
}

/// Render the advanced view overlay.
///
/// Layout:
/// - Left panel: structured transcript with role-colored entries
/// - Right panel: evidence ledger and contradiction detail
/// - Bottom: evaluator disagreement if present
pub fn render_advanced_view(frame: &mut Frame, area: Rect, props: &AdvancedViewProps) {
    let main_chunks = Layout::vertical([
        Constraint::Length(1),  // header
        Constraint::Min(10),   // content
        Constraint::Length(4), // disagreement footer
    ])
    .split(area);

    // Header
    let header = Paragraph::new(Line::from(Span::styled(
        " Advanced View  [Ctrl+T] toggle ",
        Style::default()
            .fg(Color::White)
            .bg(Color::Magenta)
            .add_modifier(Modifier::BOLD),
    )));
    frame.render_widget(header, main_chunks[0]);

    // Content: left transcript, right evidence/contradictions
    let content_chunks = Layout::horizontal([
        Constraint::Percentage(55),
        Constraint::Percentage(45),
    ])
    .split(main_chunks[1]);

    // Left: structured transcript
    let transcript_items: Vec<ListItem> = props
        .transcript_entries
        .iter()
        .map(|entry| {
            let line = Line::from(vec![
                Span::styled(
                    format!("[{}] ", entry.timestamp),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    format!("{}: ", entry.role.label()),
                    Style::default()
                        .fg(entry.role.color())
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(&entry.action),
            ]);
            ListItem::new(line)
        })
        .collect();

    let transcript = List::new(transcript_items).block(
        Block::default()
            .title("Transcript")
            .borders(Borders::ALL),
    );
    frame.render_widget(transcript, content_chunks[0]);

    // Right: evidence and contradictions stacked
    let right_chunks = Layout::vertical([
        Constraint::Percentage(60),
        Constraint::Percentage(40),
    ])
    .split(content_chunks[1]);

    // Evidence ledger
    let evidence_items: Vec<ListItem> = props
        .evidence_entries
        .iter()
        .map(|entry| {
            let line = Line::from(vec![
                Span::styled(
                    format!("[{}] ", entry.id),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    format!("{}: ", entry.class),
                    Style::default().fg(Color::Green),
                ),
                Span::raw(&entry.summary),
            ]);
            ListItem::new(line)
        })
        .collect();

    let evidence = List::new(evidence_items).block(
        Block::default()
            .title("Evidence Ledger")
            .borders(Borders::ALL),
    );
    frame.render_widget(evidence, right_chunks[0]);

    // Contradictions
    let contradiction_items: Vec<ListItem> = props
        .contradiction_entries
        .iter()
        .map(|entry| {
            let status_color = if entry.status == "Open" {
                Color::Red
            } else {
                Color::Green
            };
            let line = Line::from(vec![
                Span::styled(
                    format!("[{}] ", entry.id),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    format!("{} ", entry.status),
                    Style::default().fg(status_color),
                ),
                Span::styled(
                    format!("({}) ", entry.materiality),
                    Style::default().fg(Color::Yellow),
                ),
                Span::raw(&entry.claim),
            ]);
            ListItem::new(line)
        })
        .collect();

    let contradictions = List::new(contradiction_items).block(
        Block::default()
            .title("Contradictions")
            .borders(Borders::ALL),
    );
    frame.render_widget(contradictions, right_chunks[1]);

    // Evaluator disagreement footer
    if let Some(ref disagreement) = props.evaluator_disagreement {
        let disagreement_widget = Paragraph::new(vec![
            Line::from(vec![
                Span::styled(
                    "Evaluator Disagreement: ",
                    Style::default()
                        .fg(Color::Red)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(format!(
                    "Generator proposed '{}', evaluator assigned '{}'",
                    disagreement.generator_proposed, disagreement.evaluator_assigned
                )),
            ]),
            Line::from(vec![
                Span::styled("Reason: ", Style::default().fg(Color::Yellow)),
                Span::raw(&disagreement.reason),
            ]),
        ])
        .block(Block::default().borders(Borders::TOP));
        frame.render_widget(disagreement_widget, main_chunks[2]);
    }
}

// tests at bottom of file (from Step 1)
```

**Step 4: Run test to verify it passes**

Run: `cd spool && cargo test -p spool-tui -- widgets::advanced_view`
Expected: 4 tests PASS

**Step 5: Commit**

```bash
git add spool/spool-tui/src/widgets/advanced_view.rs
git commit -m "feat(spool-tui): advanced view widget with structured transcript, evidence ledger, contradictions, and evaluator disagreement per Spec Section 13.3"
```

---

## Task 11: Chat Input Widget

**Files:**

- Modify: `spool/spool-tui/src/widgets/chat_input.rs`

**Step 1: Write the failing test**

Add to `spool/spool-tui/src/widgets/chat_input.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    use ratatui::buffer::Buffer;

    fn buffer_to_string(buffer: &Buffer) -> String {
        let mut output = String::new();
        for y in 0..buffer.area.height {
            for x in 0..buffer.area.width {
                let cell = &buffer[(x, y)];
                output.push_str(cell.symbol());
            }
        }
        output
    }

    #[test]
    fn chat_input_renders_empty() {
        let mut input = ChatInputState::new();

        let backend = TestBackend::new(80, 3);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                let area = frame.area();
                render_chat_input(frame, area, &input);
            })
            .unwrap();

        let buffer = terminal.backend().buffer().clone();
        let content = buffer_to_string(&buffer);

        // Should show prompt indicator
        assert!(content.contains(">") || content.contains("spool"));
    }

    #[test]
    fn chat_input_insert_and_read() {
        let mut input = ChatInputState::new();
        input.insert('H');
        input.insert('i');
        assert_eq!(input.text(), "Hi");
        assert_eq!(input.cursor_position(), 2);
    }

    #[test]
    fn chat_input_backspace() {
        let mut input = ChatInputState::new();
        input.insert('A');
        input.insert('B');
        input.insert('C');
        input.backspace();
        assert_eq!(input.text(), "AB");
        assert_eq!(input.cursor_position(), 2);
    }

    #[test]
    fn chat_input_backspace_empty() {
        let mut input = ChatInputState::new();
        input.backspace();
        assert_eq!(input.text(), "");
        assert_eq!(input.cursor_position(), 0);
    }

    #[test]
    fn chat_input_submit_clears() {
        let mut input = ChatInputState::new();
        input.insert('t');
        input.insert('e');
        input.insert('s');
        input.insert('t');

        let submitted = input.submit();
        assert_eq!(submitted, Some("test".into()));
        assert_eq!(input.text(), "");
        assert_eq!(input.cursor_position(), 0);
    }

    #[test]
    fn chat_input_submit_empty_returns_none() {
        let mut input = ChatInputState::new();
        let submitted = input.submit();
        assert!(submitted.is_none());
    }

    #[test]
    fn chat_input_cursor_movement() {
        let mut input = ChatInputState::new();
        input.insert('a');
        input.insert('b');
        input.insert('c');

        assert_eq!(input.cursor_position(), 3);

        input.move_cursor_left();
        assert_eq!(input.cursor_position(), 2);

        input.move_cursor_left();
        assert_eq!(input.cursor_position(), 1);

        input.move_cursor_right();
        assert_eq!(input.cursor_position(), 2);

        // Cannot go past end
        input.move_cursor_right();
        input.move_cursor_right();
        assert_eq!(input.cursor_position(), 3);
    }

    #[test]
    fn chat_input_move_left_at_zero() {
        let mut input = ChatInputState::new();
        input.move_cursor_left();
        assert_eq!(input.cursor_position(), 0);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cd spool && cargo test -p spool-tui -- widgets::chat_input`
Expected: FAIL -- types not defined yet

**Step 3: Write the implementation**

Replace the placeholder content of `spool/spool-tui/src/widgets/chat_input.rs`:

```rust
// spool/spool-tui/src/widgets/chat_input.rs
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

/// Mutable state for the chat input widget.
///
/// Tracks the current text buffer and cursor position.
/// Supports insert, backspace, cursor movement, and submit.
pub struct ChatInputState {
    buffer: String,
    cursor: usize,
}

impl ChatInputState {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            cursor: 0,
        }
    }

    pub fn text(&self) -> &str {
        &self.buffer
    }

    pub fn cursor_position(&self) -> usize {
        self.cursor
    }

    pub fn insert(&mut self, c: char) {
        self.buffer.insert(self.cursor, c);
        self.cursor += c.len_utf8();
    }

    pub fn backspace(&mut self) {
        if self.cursor > 0 {
            // Find the previous character boundary
            let prev = self.buffer[..self.cursor]
                .char_indices()
                .next_back()
                .map(|(i, _)| i)
                .unwrap_or(0);
            self.buffer.drain(prev..self.cursor);
            self.cursor = prev;
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor > 0 {
            self.cursor = self.buffer[..self.cursor]
                .char_indices()
                .next_back()
                .map(|(i, _)| i)
                .unwrap_or(0);
        }
    }

    pub fn move_cursor_right(&mut self) {
        if self.cursor < self.buffer.len() {
            self.cursor = self.buffer[self.cursor..]
                .char_indices()
                .nth(1)
                .map(|(i, _)| self.cursor + i)
                .unwrap_or(self.buffer.len());
        }
    }

    /// Submit the current input buffer and clear it.
    ///
    /// Returns `None` if the buffer is empty.
    pub fn submit(&mut self) -> Option<String> {
        if self.buffer.is_empty() {
            return None;
        }
        let text = std::mem::take(&mut self.buffer);
        self.cursor = 0;
        Some(text)
    }
}

impl Default for ChatInputState {
    fn default() -> Self {
        Self::new()
    }
}

/// Render the chat input widget.
///
/// Shows a prompt indicator and the current input buffer with cursor.
pub fn render_chat_input(frame: &mut Frame, area: Rect, state: &ChatInputState) {
    let prompt = "> ";
    let text = state.text();

    let line = Line::from(vec![
        Span::styled(
            prompt,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(text),
    ]);

    let input = Paragraph::new(line).block(
        Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(Color::DarkGray)),
    );

    frame.render_widget(input, area);

    // Set cursor position
    let cursor_x = area.x + prompt.len() as u16 + state.cursor_position() as u16;
    let cursor_y = area.y + 1; // +1 for the border
    if cursor_x < area.x + area.width && cursor_y < area.y + area.height {
        frame.set_cursor_position((cursor_x, cursor_y));
    }
}

// tests at bottom of file (from Step 1)
```

**Step 4: Run test to verify it passes**

Run: `cd spool && cargo test -p spool-tui -- widgets::chat_input`
Expected: 8 tests PASS

**Step 5: Commit**

```bash
git add spool/spool-tui/src/widgets/chat_input.rs
git commit -m "feat(spool-tui): chat input widget with cursor movement, backspace, and submit"
```

---

## Task 12: Session Startup Lifecycle

**Files:**

- Create: `spool/spool-tui/src/lifecycle.rs`
- Modify: `spool/spool-tui/src/lib.rs`

**Step 1: Write the failing test**

Add `pub mod lifecycle;` to `spool/spool-tui/src/lib.rs`.

Create `spool/spool-tui/src/lifecycle.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_state::ViewMode;

    #[test]
    fn startup_sequence_steps() {
        let steps = StartupSequence::new("finance".into(), "Executive BI".into());

        assert_eq!(steps.current_step(), StartupStep::Authenticating);
        assert!(!steps.is_complete());
    }

    #[test]
    fn startup_sequence_progression() {
        let mut seq = StartupSequence::new("finance".into(), "Executive BI".into());

        assert_eq!(seq.current_step(), StartupStep::Authenticating);

        seq.advance();
        assert_eq!(seq.current_step(), StartupStep::ConnectingFabric);

        seq.advance();
        assert_eq!(seq.current_step(), StartupStep::SelectingLob);

        seq.advance();
        assert_eq!(seq.current_step(), StartupStep::EstablishingWorkspace);

        seq.advance();
        assert_eq!(seq.current_step(), StartupStep::LoadingKnowledge);

        seq.advance();
        assert_eq!(seq.current_step(), StartupStep::BuildingContext);

        seq.advance();
        assert_eq!(seq.current_step(), StartupStep::Ready);
        assert!(seq.is_complete());
    }

    #[test]
    fn startup_sequence_labels() {
        let steps = vec![
            StartupStep::Authenticating,
            StartupStep::ConnectingFabric,
            StartupStep::SelectingLob,
            StartupStep::EstablishingWorkspace,
            StartupStep::LoadingKnowledge,
            StartupStep::BuildingContext,
            StartupStep::Ready,
        ];

        for step in &steps {
            let label = step.label();
            assert!(!label.is_empty());
        }

        assert_eq!(steps.len(), 7);
    }

    #[test]
    fn startup_to_session_produces_app_state() {
        let mut seq = StartupSequence::new("finance".into(), "Executive BI".into());

        // Advance to ready
        for _ in 0..6 {
            seq.advance();
        }
        assert!(seq.is_complete());

        let result = seq.finalize("session_123".into());
        assert_eq!(result.session_id, "session_123");
        assert_eq!(result.selected_lob, "finance");
        assert_eq!(result.workspace_scope, "Executive BI");
    }

    #[test]
    fn task_execution_lifecycle_steps() {
        let steps = vec![
            TaskExecutionStep::UserAsksQuestion,
            TaskExecutionStep::PlannerCreatesContract,
            TaskExecutionStep::UserStartsTask,
            TaskExecutionStep::GeneratorInvestigates,
            TaskExecutionStep::GeneratorCollectsEvidence,
            TaskExecutionStep::EvaluatorReviews,
            TaskExecutionStep::ResultEmitted,
            TaskExecutionStep::UserInspects,
        ];
        assert_eq!(steps.len(), 8);

        for step in &steps {
            assert!(!step.label().is_empty());
        }
    }

    #[test]
    fn task_execution_tracker_progression() {
        let mut tracker = TaskExecutionTracker::new();
        assert_eq!(tracker.current_step(), TaskExecutionStep::UserAsksQuestion);

        tracker.advance();
        assert_eq!(
            tracker.current_step(),
            TaskExecutionStep::PlannerCreatesContract
        );

        tracker.advance();
        assert_eq!(tracker.current_step(), TaskExecutionStep::UserStartsTask);
    }

    #[test]
    fn task_execution_tracker_history() {
        let mut tracker = TaskExecutionTracker::new();
        tracker.advance();
        tracker.advance();

        assert_eq!(tracker.history().len(), 3); // initial + 2 advances
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cd spool && cargo test -p spool-tui -- lifecycle`
Expected: FAIL -- types not defined yet

**Step 3: Write the implementation**

```rust
// spool/spool-tui/src/lifecycle.rs

/// Session startup step per Spec Section 15.1.
///
/// The startup sequence is:
/// 1. Authenticate product login
/// 2. Authenticate Fabric access
/// 3. Select LOB
/// 4. Establish workspace scope
/// 5. Load selected LOB Tier 1 + Tier 2
/// 6. Build prompt context
/// 7. Enter chat session (ready)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StartupStep {
    Authenticating,
    ConnectingFabric,
    SelectingLob,
    EstablishingWorkspace,
    LoadingKnowledge,
    BuildingContext,
    Ready,
}

impl StartupStep {
    pub fn label(&self) -> &str {
        match self {
            StartupStep::Authenticating => "Authenticating...",
            StartupStep::ConnectingFabric => "Connecting to Fabric...",
            StartupStep::SelectingLob => "Selecting LOB...",
            StartupStep::EstablishingWorkspace => "Establishing workspace scope...",
            StartupStep::LoadingKnowledge => "Loading knowledge bundle...",
            StartupStep::BuildingContext => "Building prompt context...",
            StartupStep::Ready => "Ready",
        }
    }

    fn next(&self) -> StartupStep {
        match self {
            StartupStep::Authenticating => StartupStep::ConnectingFabric,
            StartupStep::ConnectingFabric => StartupStep::SelectingLob,
            StartupStep::SelectingLob => StartupStep::EstablishingWorkspace,
            StartupStep::EstablishingWorkspace => StartupStep::LoadingKnowledge,
            StartupStep::LoadingKnowledge => StartupStep::BuildingContext,
            StartupStep::BuildingContext => StartupStep::Ready,
            StartupStep::Ready => StartupStep::Ready,
        }
    }
}

/// Tracks progression through the startup sequence.
pub struct StartupSequence {
    current: StartupStep,
    selected_lob: String,
    workspace_scope: String,
}

/// Result of a completed startup sequence.
pub struct StartupResult {
    pub session_id: String,
    pub selected_lob: String,
    pub workspace_scope: String,
}

impl StartupSequence {
    pub fn new(selected_lob: String, workspace_scope: String) -> Self {
        Self {
            current: StartupStep::Authenticating,
            selected_lob,
            workspace_scope,
        }
    }

    pub fn current_step(&self) -> StartupStep {
        self.current.clone()
    }

    pub fn is_complete(&self) -> bool {
        self.current == StartupStep::Ready
    }

    pub fn advance(&mut self) {
        self.current = self.current.next();
    }

    pub fn finalize(self, session_id: String) -> StartupResult {
        StartupResult {
            session_id,
            selected_lob: self.selected_lob,
            workspace_scope: self.workspace_scope,
        }
    }
}

/// Task execution lifecycle step per Spec Section 15.2.
///
/// The execution sequence is:
/// 1. User asks question
/// 2. Planner creates or refines task contract
/// 3. User starts task
/// 4. Generator investigates
/// 5. Generator collects evidence and validations
/// 6. Evaluator subagent reviews bounded packet
/// 7. Result is emitted
/// 8. User may inspect, continue, or resume later
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskExecutionStep {
    UserAsksQuestion,
    PlannerCreatesContract,
    UserStartsTask,
    GeneratorInvestigates,
    GeneratorCollectsEvidence,
    EvaluatorReviews,
    ResultEmitted,
    UserInspects,
}

impl TaskExecutionStep {
    pub fn label(&self) -> &str {
        match self {
            TaskExecutionStep::UserAsksQuestion => "User asks question",
            TaskExecutionStep::PlannerCreatesContract => "Planner creating task contract",
            TaskExecutionStep::UserStartsTask => "User starts task",
            TaskExecutionStep::GeneratorInvestigates => "Generator investigating",
            TaskExecutionStep::GeneratorCollectsEvidence => "Collecting evidence",
            TaskExecutionStep::EvaluatorReviews => "Evaluator reviewing",
            TaskExecutionStep::ResultEmitted => "Result emitted",
            TaskExecutionStep::UserInspects => "User inspecting result",
        }
    }

    fn next(&self) -> TaskExecutionStep {
        match self {
            TaskExecutionStep::UserAsksQuestion => TaskExecutionStep::PlannerCreatesContract,
            TaskExecutionStep::PlannerCreatesContract => TaskExecutionStep::UserStartsTask,
            TaskExecutionStep::UserStartsTask => TaskExecutionStep::GeneratorInvestigates,
            TaskExecutionStep::GeneratorInvestigates => {
                TaskExecutionStep::GeneratorCollectsEvidence
            }
            TaskExecutionStep::GeneratorCollectsEvidence => TaskExecutionStep::EvaluatorReviews,
            TaskExecutionStep::EvaluatorReviews => TaskExecutionStep::ResultEmitted,
            TaskExecutionStep::ResultEmitted => TaskExecutionStep::UserInspects,
            TaskExecutionStep::UserInspects => TaskExecutionStep::UserInspects,
        }
    }
}

/// Tracks progression through task execution lifecycle.
pub struct TaskExecutionTracker {
    current: TaskExecutionStep,
    history: Vec<TaskExecutionStep>,
}

impl TaskExecutionTracker {
    pub fn new() -> Self {
        let initial = TaskExecutionStep::UserAsksQuestion;
        Self {
            current: initial.clone(),
            history: vec![initial],
        }
    }

    pub fn current_step(&self) -> TaskExecutionStep {
        self.current.clone()
    }

    pub fn advance(&mut self) {
        self.current = self.current.next();
        self.history.push(self.current.clone());
    }

    pub fn history(&self) -> &[TaskExecutionStep] {
        &self.history
    }
}

impl Default for TaskExecutionTracker {
    fn default() -> Self {
        Self::new()
    }
}

// tests at bottom of file (from Step 1)
```

**Step 4: Run test to verify it passes**

Run: `cd spool && cargo test -p spool-tui -- lifecycle`
Expected: 6 tests PASS

**Step 5: Commit**

```bash
git add spool/spool-tui/src/lifecycle.rs spool/spool-tui/src/lib.rs
git commit -m "feat(spool-tui): session startup and task execution lifecycle trackers per Spec Sections 15.1-15.2"
```

---

## Task 13: Terminal Event Loop

**Files:**

- Create: `spool/spool-tui/src/terminal.rs`
- Modify: `spool/spool-tui/src/lib.rs`

**Step 1: Write the failing test**

Add `pub mod terminal;` to `spool/spool-tui/src/lib.rs`.

Create `spool/spool-tui/src/terminal.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_state::{AppState, ViewMode};
    use crate::event::{AppEvent, PlanModeAction, TaskPhaseEvent};

    #[test]
    fn event_handler_quit() {
        let mut state = AppState::new();
        let result = handle_app_event(&mut state, AppEvent::RequestQuit);
        assert!(matches!(result, EventResult::Quit));
    }

    #[test]
    fn event_handler_initialize_session() {
        let mut state = AppState::new();
        state.initialize_session("s1".into(), "fin".into(), "ws".into());
        assert!(matches!(state.view(), ViewMode::PlanMode));
    }

    #[test]
    fn event_handler_toggle_advanced_from_progress() {
        let mut state = AppState::new();
        state.initialize_session("s1".into(), "fin".into(), "ws".into());
        state.transition_view(ViewMode::Progress).unwrap();

        let result = handle_app_event(&mut state, AppEvent::ToggleAdvancedView);
        assert!(matches!(result, EventResult::Continue));
        assert!(matches!(state.view(), ViewMode::Advanced));

        let result = handle_app_event(&mut state, AppEvent::ToggleAdvancedView);
        assert!(matches!(result, EventResult::Continue));
        assert!(matches!(state.view(), ViewMode::Progress));
    }

    #[test]
    fn event_handler_task_phase_changed() {
        let mut state = AppState::new();
        state.initialize_session("s1".into(), "fin".into(), "ws".into());

        let result = handle_app_event(
            &mut state,
            AppEvent::TaskPhaseChanged(TaskPhaseEvent::Investigating {
                artifact: "Report A".into(),
            }),
        );
        assert!(matches!(result, EventResult::Continue));
        assert_eq!(
            state.task_phase(),
            Some(&crate::app_state::TaskPhase::Investigating)
        );
        assert_eq!(state.artifact_focus(), Some("Report A"));
    }

    #[test]
    fn event_handler_finding_received() {
        let mut state = AppState::new();
        state.initialize_session("s1".into(), "fin".into(), "ws".into());

        let result = handle_app_event(
            &mut state,
            AppEvent::FindingReceived {
                summary: "Revenue variance found".into(),
            },
        );
        assert!(matches!(result, EventResult::Continue));
        assert_eq!(state.latest_finding(), Some("Revenue variance found"));
    }

    #[test]
    fn event_handler_waiting_for_user() {
        let mut state = AppState::new();
        state.initialize_session("s1".into(), "fin".into(), "ws".into());

        let result = handle_app_event(
            &mut state,
            AppEvent::WaitingForUser {
                question: "Which report?".into(),
            },
        );
        assert!(matches!(result, EventResult::Continue));
        assert!(state.waiting_state().is_some());
    }

    #[test]
    fn event_handler_tick_is_noop() {
        let mut state = AppState::new();
        let result = handle_app_event(&mut state, AppEvent::Tick);
        assert!(matches!(result, EventResult::Continue));
    }

    #[test]
    fn event_result_variants() {
        let results = vec![EventResult::Continue, EventResult::Quit];
        assert_eq!(results.len(), 2);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cd spool && cargo test -p spool-tui -- terminal`
Expected: FAIL -- types not defined yet

**Step 3: Write the implementation**

```rust
// spool/spool-tui/src/terminal.rs
use crate::app_state::{AppState, TaskPhase, ViewMode, WaitingState};
use crate::event::{AppEvent, TaskPhaseEvent};

/// Result of processing an application event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventResult {
    Continue,
    Quit,
}

/// Process an application event and update state accordingly.
///
/// This is the core event handler that follows the crossterm events ->
/// app state update -> ratatui render pattern borrowed from codex-rs
/// tui architecture.
pub fn handle_app_event(state: &mut AppState, event: AppEvent) -> EventResult {
    match event {
        AppEvent::RequestQuit => EventResult::Quit,

        AppEvent::Tick => EventResult::Continue,

        AppEvent::TerminalKey(_key) => {
            // Key handling is delegated to view-specific handlers
            // in the main render loop. This is a placeholder for
            // events not captured by widgets.
            EventResult::Continue
        }

        AppEvent::TerminalResize { .. } => {
            // Resize is handled by ratatui automatically during draw
            EventResult::Continue
        }

        AppEvent::ToggleAdvancedView => {
            match state.view() {
                ViewMode::Progress => {
                    let _ = state.transition_view(ViewMode::Advanced);
                }
                ViewMode::Advanced => {
                    let _ = state.transition_view(ViewMode::Progress);
                }
                _ => {}
            }
            EventResult::Continue
        }

        AppEvent::TaskPhaseChanged(phase_event) => {
            match phase_event {
                TaskPhaseEvent::Planning => {
                    state.set_task_phase(TaskPhase::Planning);
                    state.clear_artifact_focus();
                }
                TaskPhaseEvent::Investigating { artifact } => {
                    state.set_task_phase(TaskPhase::Investigating);
                    state.set_artifact_focus(artifact);
                }
                TaskPhaseEvent::Evaluating => {
                    state.set_task_phase(TaskPhase::Evaluating);
                }
                TaskPhaseEvent::Completed { .. } => {
                    state.set_task_phase(TaskPhase::Finalizing);
                    state.clear_waiting();
                }
                TaskPhaseEvent::Blocked { reason } => {
                    state.set_waiting(WaitingState::ExternalResponse {
                        description: reason,
                    });
                }
                TaskPhaseEvent::Interrupted => {
                    // Interrupted state is handled by the lifecycle
                    // and resume modules
                }
            }
            EventResult::Continue
        }

        AppEvent::FindingReceived { summary } => {
            state.set_latest_finding(summary);
            EventResult::Continue
        }

        AppEvent::EvidenceReceived { .. } => {
            // Evidence count is tracked by the harness state,
            // not directly by the app state
            EventResult::Continue
        }

        AppEvent::WaitingForUser { question } => {
            state.set_waiting(WaitingState::UserInput { question });
            EventResult::Continue
        }

        AppEvent::SessionResumed { .. } => {
            // Resume is handled by the resume module
            EventResult::Continue
        }

        AppEvent::UserSubmitInput(_) => {
            // Input handling is delegated to the chat input widget
            state.clear_waiting();
            EventResult::Continue
        }

        AppEvent::PlanModeAction(_) => {
            // Plan mode actions are handled by the plan mode widget state
            EventResult::Continue
        }
    }
}

// tests at bottom of file (from Step 1)
```

**Step 4: Run test to verify it passes**

Run: `cd spool && cargo test -p spool-tui -- terminal`
Expected: 8 tests PASS

**Step 5: Commit**

```bash
git add spool/spool-tui/src/terminal.rs spool/spool-tui/src/lib.rs
git commit -m "feat(spool-tui): terminal event loop handler routing events to app state updates"
```

---

## Task 14: Resume Integration With Persistence

**Files:**

- Create: `spool/spool-tui/tests/resume_integration.rs`

This task validates the full resume flow: persist session state, load from disk, resolve resume task, build resume context, verify interrupted task surfacing.

**Step 1: Write the integration tests**

```rust
// spool/spool-tui/tests/resume_integration.rs
//!
//! Integration tests for Plan 5: resume flow through persistence.
//!
//! These tests prove that session state can be persisted, reloaded,
//! and correctly resumed with interrupted task surfacing, per
//! Spec Sections 12.5-12.8.

use chrono::Utc;
use spool_core::persistence::{JsonlPersistence, PersistenceProvider, SessionState};
use spool_protocol::artifact::ArtifactType;
use spool_protocol::checkpoint::{
    AskUserQuestion, CheckpointClass, CheckpointPolicy, CheckpointTrigger,
};
use spool_protocol::contradiction::{
    ContradictionId, ContradictionRecord, ContradictionStatus, MaterialityLevel,
};
use spool_protocol::evidence::{EvidenceClass, EvidenceId, EvidenceItem, EvidenceType};
use spool_protocol::task_contract::*;
use spool_protocol::task_result::*;
use spool_tui::compaction::{compact, CompactionInput};
use spool_tui::context::compose_active_context;
use spool_tui::resume::{build_resume_context, ResumeResolutionRule};
use tempfile::TempDir;

fn make_contract(id: &str, status: TaskStatus) -> TaskContract {
    TaskContract {
        task_id: TaskId(id.into()),
        intent: format!("Investigate {id}"),
        scope: Scope {
            lob: "finance".into(),
            workspace: "Executive BI".into(),
            artifacts: vec![ArtifactRef {
                artifact_type: ArtifactType::Report,
                reference: "Revenue Report".into(),
            }],
        },
        selected_recipe: Some("report_number_mismatch".into()),
        selected_recipe_selection_mode: None,
        assumptions: vec!["Published report".into()],
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
        task_status: status,
        created_at: Some(Utc::now()),
        updated_at: Some(Utc::now()),
    }
}

fn make_evidence(id: &str) -> EvidenceItem {
    EvidenceItem {
        id: EvidenceId(id.into()),
        evidence_type: EvidenceType::Observed,
        evidence_class: EvidenceClass::DaxQueryResult,
        source: "test".into(),
        summary: format!("Evidence {id}"),
        artifact_refs: vec![],
        observed_at: Some(Utc::now()),
        detail: None,
    }
}

/// Scenario: persist, reload, and resume an active session with one
/// completed task and one active task.
#[test]
fn scenario_persist_and_resume_active_session() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("session.jsonl");
    let persistence = JsonlPersistence::new(path.clone());

    let state = SessionState {
        session_id: "session_resume_1".into(),
        created_at: Utc::now(),
        selected_lob: "finance".into(),
        workspace_scope: "Executive BI".into(),
        task_contracts: vec![
            make_contract("task_1", TaskStatus::Completed),
            make_contract("task_2", TaskStatus::Active),
        ],
        task_results: vec![TaskResult {
            task_id: TaskId("task_1".into()),
            proposed_state: Some(ResultState::Confirmed),
            state: ResultState::Confirmed,
            confidence: Confidence::High,
            summary: "Task 1 completed".into(),
            findings: vec![],
            evidence_refs: vec![],
            validation_results: vec![],
            recommended_actions: vec![],
            blockers: vec![],
            open_questions: vec![],
            proposed_changes: vec![],
            contradiction_refs: vec![],
            result_generated_at: Some(Utc::now()),
            result_version: Some(1),
        }],
        evidence_items: vec![make_evidence("ev_1"), make_evidence("ev_2")],
        contradiction_records: vec![],
        checkpoint_history: vec![],
    };

    // Persist
    persistence.save(&state).unwrap();

    // Reload
    let restored = persistence.load().unwrap();
    assert_eq!(restored.session_id, "session_resume_1");
    assert_eq!(restored.task_contracts.len(), 2);

    // Resume
    let resume_context = build_resume_context(&restored);
    assert_eq!(resume_context.active_task.task_id.0, "task_2");
    assert_eq!(resume_context.resolution_rule, ResumeResolutionRule::ActiveTask);
    assert!(resume_context.interrupted_task_info.is_none());

    // Compact the resumed state
    let compaction_input = CompactionInput {
        task_contract: resume_context.active_task.clone(),
        task_phase: "investigating".into(),
        evidence_items: resume_context.evidence_items.clone(),
        contradiction_records: resume_context.contradiction_records.clone(),
        unresolved_questions: vec![],
        unresolved_evaluator_requests: vec![],
        artifact_focus: Some("Revenue Report".into()),
        latest_result: None,
    };

    let compaction_output = compact(&compaction_input);
    assert_eq!(compaction_output.evidence_summary.total_count, 2);

    // Compose active context
    let active_context = compose_active_context(
        &compaction_output,
        &["User: What happened with revenue?".into()],
        Some("LOB: finance"),
        None,
    );

    let rendered = active_context.render();
    assert!(rendered.contains("task_2"));
    assert!(rendered.contains("Evidence"));
    assert!(rendered.contains("finance"));
}

/// Scenario: persist and resume an interrupted session with pending user question.
#[test]
fn scenario_persist_and_resume_interrupted_with_pending_question() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("session_interrupted.jsonl");
    let persistence = JsonlPersistence::new(path.clone());

    let state = SessionState {
        session_id: "session_interrupted_1".into(),
        created_at: Utc::now(),
        selected_lob: "finance".into(),
        workspace_scope: "Executive BI".into(),
        task_contracts: vec![make_contract("task_int", TaskStatus::Interrupted)],
        task_results: vec![],
        evidence_items: vec![make_evidence("ev_1")],
        contradiction_records: vec![ContradictionRecord {
            id: ContradictionId("c_1".into()),
            disputed_claim: "Revenue mismatch".into(),
            conflicting_evidence: vec![EvidenceId("ev_1".into())],
            materiality: MaterialityLevel::Material,
            freshness_notes: None,
            resolution_attempted: false,
            resolution_detail: None,
            status: ContradictionStatus::Open,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }],
        checkpoint_history: vec![AskUserQuestion {
            id: "q_pending".into(),
            checkpoint_class: CheckpointClass::Information,
            trigger: CheckpointTrigger::Ambiguous,
            question: "Which report version?".into(),
            options: Some(vec!["v1".into(), "v2".into()]),
            allows_free_text: true,
            asked_at: Utc::now(),
            answer: None, // unanswered
        }],
    };

    persistence.save(&state).unwrap();

    let restored = persistence.load().unwrap();
    let resume_context = build_resume_context(&restored);

    // Should detect interrupted task
    assert!(resume_context.interrupted_task_info.is_some());
    let info = resume_context.interrupted_task_info.unwrap();
    assert!(info.was_interrupted);
    assert!(info.user_answer_pending);
    assert_eq!(info.pending_questions.len(), 1);
    assert_eq!(info.pending_questions[0].question, "Which report version?");

    // Compact should preserve contradiction
    let compaction_input = CompactionInput {
        task_contract: resume_context.active_task,
        task_phase: "interrupted".into(),
        evidence_items: resume_context.evidence_items,
        contradiction_records: resume_context.contradiction_records.clone(),
        unresolved_questions: resume_context
            .checkpoint_history
            .iter()
            .filter(|q| q.answer.is_none())
            .cloned()
            .collect(),
        unresolved_evaluator_requests: vec![],
        artifact_focus: None,
        latest_result: None,
    };

    let output = compact(&compaction_input);
    assert!(output.contradiction_summary.has_material_unresolved);
    assert_eq!(output.unresolved_questions.len(), 1);
}

/// Scenario: resume with all tasks completed should pick most recent.
#[test]
fn scenario_resume_all_completed() {
    let state = SessionState {
        session_id: "session_all_done".into(),
        created_at: Utc::now(),
        selected_lob: "finance".into(),
        workspace_scope: "Executive BI".into(),
        task_contracts: vec![
            make_contract("task_old", TaskStatus::Completed),
            make_contract("task_new", TaskStatus::Completed),
        ],
        task_results: vec![],
        evidence_items: vec![],
        contradiction_records: vec![],
        checkpoint_history: vec![],
    };

    let resume_context = build_resume_context(&state);
    assert_eq!(resume_context.active_task.task_id.0, "task_new");
    assert_eq!(
        resume_context.resolution_rule,
        ResumeResolutionRule::MostRecentCompleted
    );
}
```

**Step 2: Run the integration tests**

Run: `cd spool && cargo test -p spool-tui --test resume_integration`
Expected: 3 tests PASS

**Step 3: Commit**

```bash
git add spool/spool-tui/tests/resume_integration.rs
git commit -m "test(spool-tui): integration tests for persist-resume flow with active, interrupted, and completed sessions"
```

---

## Task 15: Full Session Lifecycle Integration Test

**Files:**

- Create: `spool/spool-tui/tests/session_lifecycle.rs`

This task validates the full session lifecycle: startup -> plan mode -> execution rendering -> compaction -> persist -> resume.

**Step 1: Write the integration test**

```rust
// spool/spool-tui/tests/session_lifecycle.rs
//!
//! Full session lifecycle integration test for Plan 5.
//!
//! Validates: startup -> plan mode state -> execution phase tracking ->
//! compaction -> persist -> resume -> context composition.

use chrono::Utc;
use spool_core::persistence::{JsonlPersistence, PersistenceProvider, SessionState};
use spool_protocol::artifact::ArtifactType;
use spool_protocol::checkpoint::{CheckpointPolicy, CheckpointTrigger};
use spool_protocol::evidence::{EvidenceClass, EvidenceId, EvidenceItem, EvidenceType};
use spool_protocol::task_contract::*;
use spool_protocol::task_result::*;
use spool_tui::app_state::{AppState, TaskPhase, ViewMode, WaitingState};
use spool_tui::compaction::{compact, CompactionInput};
use spool_tui::context::compose_active_context;
use spool_tui::event::{AppEvent, TaskPhaseEvent};
use spool_tui::lifecycle::{StartupSequence, StartupStep, TaskExecutionTracker, TaskExecutionStep};
use spool_tui::resume::build_resume_context;
use spool_tui::terminal::{handle_app_event, EventResult};
use tempfile::TempDir;

#[test]
fn full_session_lifecycle() {
    // --- Phase 1: Startup ---
    let mut startup = StartupSequence::new("finance".into(), "Executive BI".into());
    assert_eq!(startup.current_step(), StartupStep::Authenticating);

    // Advance through all startup steps
    for _ in 0..6 {
        startup.advance();
    }
    assert!(startup.is_complete());

    let startup_result = startup.finalize("session_full_lifecycle".into());

    // --- Phase 2: Initialize app state (enters plan mode) ---
    let mut state = AppState::new();
    state.initialize_session(
        startup_result.session_id.clone(),
        startup_result.selected_lob.clone(),
        startup_result.workspace_scope.clone(),
    );
    assert!(matches!(state.view(), ViewMode::PlanMode));

    // --- Phase 3: Transition to progress (task started) ---
    state.transition_view(ViewMode::Progress).unwrap();
    assert!(matches!(state.view(), ViewMode::Progress));

    // --- Phase 4: Task execution tracking ---
    let mut exec_tracker = TaskExecutionTracker::new();
    assert_eq!(exec_tracker.current_step(), TaskExecutionStep::UserAsksQuestion);

    exec_tracker.advance(); // Planner creates contract
    exec_tracker.advance(); // User starts task
    exec_tracker.advance(); // Generator investigates

    // Handle task phase events through the event handler
    let result = handle_app_event(
        &mut state,
        AppEvent::TaskPhaseChanged(TaskPhaseEvent::Investigating {
            artifact: "Executive Revenue Report".into(),
        }),
    );
    assert!(matches!(result, EventResult::Continue));
    assert_eq!(state.task_phase(), Some(&TaskPhase::Investigating));
    assert_eq!(state.artifact_focus(), Some("Executive Revenue Report"));

    // Receive a finding
    let result = handle_app_event(
        &mut state,
        AppEvent::FindingReceived {
            summary: "QoQ measure uses stale quarter offset".into(),
        },
    );
    assert!(matches!(result, EventResult::Continue));
    assert_eq!(
        state.latest_finding(),
        Some("QoQ measure uses stale quarter offset")
    );

    // --- Phase 5: Toggle advanced view ---
    let result = handle_app_event(&mut state, AppEvent::ToggleAdvancedView);
    assert!(matches!(result, EventResult::Continue));
    assert!(matches!(state.view(), ViewMode::Advanced));

    // Toggle back
    let result = handle_app_event(&mut state, AppEvent::ToggleAdvancedView);
    assert!(matches!(result, EventResult::Continue));
    assert!(matches!(state.view(), ViewMode::Progress));

    // --- Phase 6: Evaluating ---
    let result = handle_app_event(
        &mut state,
        AppEvent::TaskPhaseChanged(TaskPhaseEvent::Evaluating),
    );
    assert!(matches!(result, EventResult::Continue));
    assert_eq!(state.task_phase(), Some(&TaskPhase::Evaluating));

    // --- Phase 7: Compaction ---
    let contract = TaskContract {
        task_id: TaskId("task_lifecycle".into()),
        intent: "Find revenue mismatch".into(),
        scope: Scope {
            lob: "finance".into(),
            workspace: "Executive BI".into(),
            artifacts: vec![ArtifactRef {
                artifact_type: ArtifactType::Report,
                reference: "Executive Revenue Report".into(),
            }],
        },
        selected_recipe: Some("report_number_mismatch".into()),
        selected_recipe_selection_mode: None,
        assumptions: vec!["Published report".into()],
        expected_evidence_classes: vec![EvidenceClass::DaxQueryResult],
        validation_floor: ValidationFloor::DirectValidationRequired,
        checkpoint_policy: CheckpointPolicy {
            ask_on: vec![CheckpointTrigger::Ambiguous],
        },
        clarification_checkpoints: vec![],
        approval_checkpoints: vec![],
        expected_deliverable_shape: "structured_task_result".into(),
        evaluator_packet_requirements: vec![],
        task_status: TaskStatus::Active,
        created_at: Some(Utc::now()),
        updated_at: None,
    };

    let evidence = vec![
        EvidenceItem {
            id: EvidenceId("ev_1".into()),
            evidence_type: EvidenceType::Observed,
            evidence_class: EvidenceClass::DaxQueryResult,
            source: "dax".into(),
            summary: "DAX returned 12.4M".into(),
            artifact_refs: vec![],
            observed_at: Some(Utc::now()),
            detail: None,
        },
        EvidenceItem {
            id: EvidenceId("ev_2".into()),
            evidence_type: EvidenceType::Observed,
            evidence_class: EvidenceClass::WarehouseQueryResult,
            source: "warehouse".into(),
            summary: "Warehouse returned 12.4M".into(),
            artifact_refs: vec![],
            observed_at: Some(Utc::now()),
            detail: None,
        },
    ];

    let compaction_input = CompactionInput {
        task_contract: contract.clone(),
        task_phase: "evaluating".into(),
        evidence_items: evidence.clone(),
        contradiction_records: vec![],
        unresolved_questions: vec![],
        unresolved_evaluator_requests: vec![],
        artifact_focus: Some("Executive Revenue Report".into()),
        latest_result: None,
    };

    let compaction_output = compact(&compaction_input);
    assert_eq!(compaction_output.evidence_summary.total_count, 2);
    assert_eq!(compaction_output.task_phase, "evaluating");

    // --- Phase 8: Context composition ---
    let active_context = compose_active_context(
        &compaction_output,
        &[
            "User: Why does the report show wrong numbers?".into(),
            "Spool: Investigating Revenue Report...".into(),
        ],
        Some("LOB: finance, Tier1: 45 measures"),
        None,
    );

    let rendered = active_context.render();
    assert!(rendered.contains("task_lifecycle"));
    assert!(rendered.contains("Evidence"));
    assert!(rendered.contains("revenue mismatch"));

    // --- Phase 9: Persist ---
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("lifecycle_session.jsonl");
    let persistence = JsonlPersistence::new(path.clone());

    let session_state = SessionState {
        session_id: startup_result.session_id.clone(),
        created_at: Utc::now(),
        selected_lob: "finance".into(),
        workspace_scope: "Executive BI".into(),
        task_contracts: vec![contract],
        task_results: vec![],
        evidence_items: evidence,
        contradiction_records: vec![],
        checkpoint_history: vec![],
    };

    persistence.save(&session_state).unwrap();

    // --- Phase 10: Resume ---
    let restored = persistence.load().unwrap();
    let resume_context = build_resume_context(&restored);

    assert_eq!(resume_context.session_id, "session_full_lifecycle");
    assert_eq!(resume_context.active_task.task_id.0, "task_lifecycle");
    assert_eq!(resume_context.evidence_items.len(), 2);
    assert!(resume_context.interrupted_task_info.is_none());

    // Completion: all assertions pass
    let result = handle_app_event(
        &mut state,
        AppEvent::TaskPhaseChanged(TaskPhaseEvent::Completed {
            result_state: "confirmed".into(),
        }),
    );
    assert!(matches!(result, EventResult::Continue));
    assert_eq!(state.task_phase(), Some(&TaskPhase::Finalizing));
}
```

**Step 2: Run the integration test**

Run: `cd spool && cargo test -p spool-tui --test session_lifecycle`
Expected: 1 test PASS

**Step 3: Commit**

```bash
git add spool/spool-tui/tests/session_lifecycle.rs
git commit -m "test(spool-tui): full session lifecycle integration test from startup through plan mode, execution, compaction, persist, and resume"
```

---

## Task 16: Main Entry Point

**Files:**

- Modify: `spool/spool-tui/src/main.rs`

**Step 1: Write the main function**

This task creates the real main entry point that wires the event loop together. Since Plan 5 does not own live Fabric auth or LLM integration, the main function demonstrates the startup sequence and enters plan mode with fixture data.

Replace the placeholder `spool/spool-tui/src/main.rs`:

```rust
// spool/spool-tui/src/main.rs
use std::io::{self, stdout};
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Layout};
use ratatui::Terminal;

use spool_tui::app_state::{AppState, ViewMode};
use spool_tui::event::AppEvent;
use spool_tui::lifecycle::{StartupSequence, StartupStep};
use spool_tui::terminal::{handle_app_event, EventResult};
use spool_tui::widgets::chat_input::{render_chat_input, ChatInputState};
use spool_tui::widgets::plan_mode::{render_plan_mode, PlanModeField, PlanModeProps};
use spool_tui::widgets::progress::{render_progress, ProgressProps};
use spool_tui::widgets::status_bar::{render_status_bar, StatusBarProps};

fn main() -> io::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Initialize app state
    let mut app_state = AppState::new();
    let mut chat_input = ChatInputState::new();

    // Simulate startup sequence
    let mut startup = StartupSequence::new("finance".into(), "Executive BI".into());
    for _ in 0..6 {
        startup.advance();
    }
    let result = startup.finalize(uuid::Uuid::new_v4().to_string());
    app_state.initialize_session(
        result.session_id,
        result.selected_lob,
        result.workspace_scope,
    );

    // Main event loop
    loop {
        terminal.draw(|frame| {
            let area = frame.area();

            let chunks = Layout::vertical([
                Constraint::Min(1),    // main content
                Constraint::Length(3), // chat input
                Constraint::Length(1), // status bar
            ])
            .split(area);

            // Render view based on current mode
            match app_state.view() {
                ViewMode::PlanMode => {
                    let props = PlanModeProps {
                        intent: String::new(),
                        selected_lob: app_state.selected_lob().into(),
                        workspace_scope: app_state.workspace_scope().into(),
                        artifacts: vec![],
                        selected_recipe: None,
                        assumptions: vec![],
                        can_finalize: false,
                        focused_field: PlanModeField::Intent,
                    };
                    render_plan_mode(frame, chunks[0], &props);
                }
                ViewMode::Progress => {
                    let props = ProgressProps {
                        task_phase: app_state
                            .task_phase()
                            .map(|p| format!("{p:?}"))
                            .unwrap_or_else(|| "Idle".into()),
                        artifact_focus: app_state.artifact_focus().map(String::from),
                        latest_finding: app_state.latest_finding().map(String::from),
                        waiting_state: None,
                        evidence_count: 0,
                        contradiction_count: 0,
                    };
                    render_progress(frame, chunks[0], &props);
                }
                _ => {}
            }

            // Chat input
            render_chat_input(frame, chunks[1], &chat_input);

            // Status bar
            let status_props = StatusBarProps {
                session_id: app_state.session_id().map(String::from),
                selected_lob: app_state.selected_lob().into(),
                workspace_scope: app_state.workspace_scope().into(),
                view_mode: format!("{:?}", app_state.view()),
                task_phase: app_state.task_phase().map(|p| format!("{p:?}")),
            };
            render_status_bar(frame, chunks[2], &status_props);
        })?;

        // Handle input events
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(KeyEvent {
                code, modifiers, ..
            }) = event::read()?
            {
                match (code, modifiers) {
                    (KeyCode::Char('c'), KeyModifiers::CONTROL) | (KeyCode::Char('q'), _) => {
                        break;
                    }
                    (KeyCode::Char('t'), KeyModifiers::CONTROL) => {
                        let _ = handle_app_event(
                            &mut app_state,
                            AppEvent::ToggleAdvancedView,
                        );
                    }
                    (KeyCode::Enter, _) => {
                        if let Some(text) = chat_input.submit() {
                            let _ = handle_app_event(
                                &mut app_state,
                                AppEvent::UserSubmitInput(text),
                            );
                        }
                    }
                    (KeyCode::Backspace, _) => {
                        chat_input.backspace();
                    }
                    (KeyCode::Left, _) => {
                        chat_input.move_cursor_left();
                    }
                    (KeyCode::Right, _) => {
                        chat_input.move_cursor_right();
                    }
                    (KeyCode::Tab, _) => {
                        // Navigate plan mode fields (placeholder)
                    }
                    (KeyCode::Char(c), _) => {
                        chat_input.insert(c);
                    }
                    _ => {}
                }
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
```

**Step 2: Verify build**

Run: `cd spool && cargo build -p spool-tui`
Expected: compiles with no errors

**Step 3: Commit**

```bash
git add spool/spool-tui/src/main.rs
git commit -m "feat(spool-tui): main entry point with crossterm/ratatui event loop, plan mode, progress, and chat input rendering"
```
