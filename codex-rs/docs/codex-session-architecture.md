# Codex Session & Turn Context Management Architecture

**Purpose**: This document provides a comprehensive reference for understanding how OpenAI's Codex manages session state, turn context, and task execution. It is designed for engineers building agentic platforms who want to understand or borrow architectural patterns from Codex's implementation.

**Last Updated**: 2025-10-16

---

## Quick Start Summary

**Core Concepts**:
- **Session**: A complete conversation with the model - manages lifecycle, state, and history
- **Task**: A single user request that may require multiple LLM interactions
- **Turn**: One round-trip with the LLM (request → response with potential tool calls)

**Why This Hierarchy Matters**:
- **Sessions** provide conversation continuity and context accumulation
- **Tasks** encapsulate complete operations with abort/retry capability
- **Turns** enable iterative problem-solving where the LLM can request tools and see results

**Key Architecture Patterns**:
- **SQ/EQ Pattern**: Asynchronous communication via Submission Queue + Event Queue
- **State Isolation**: Persistent conversation state separate from ephemeral turn state
- **Dual Storage**: In-memory for speed + Persistent storage for durability
- **Event-Driven**: Decouple business logic from UI, enable reactive interfaces
- **Task-Based Execution**: Uniform interface for different operation types

---

## Table of Contents

1. [Overview & Architecture](#overview--architecture)
   - Key Terminology (Session, Task, Turn)
   - High-Level Architecture
   - Core Architectural Principles
2. [Core Components](#core-components)
   - Session, SessionState, TurnContext
   - SessionTask, SessionServices
   - ActiveTurn & TurnState
3. [Execution Flow](#execution-flow)
   - Task Lifecycle Pattern
   - Turn Loop Pattern
   - Submission Loop & Tool Execution
4. [State Management Patterns](#state-management-patterns)
   - Persistent vs Ephemeral State
   - Concurrency Patterns
   - History Management & Compaction
5. [Communication Patterns](#communication-patterns)
   - Submission Types & Event Types
   - Approval Workflow
   - Input Injection
6. [Key Design Decisions](#key-design-decisions)
7. [Reusable Patterns for Other Platforms](#reusable-patterns-for-other-platforms)
8. [Code Examples](#code-examples)

---

## Overview & Architecture

### Key Terminology

Before diving into the architecture, let's clarify the core concepts:

| Concept | Definition | Lifecycle | Example |
|---------|------------|-----------|---------|
| **Session** | A complete conversation with the model | Persistent across multiple tasks | From `codex spawn` to `codex shutdown` |
| **Task** | A single user request that may require multiple LLM interactions | Spans multiple turns until completion | User asks "analyze the codebase for security issues" |
| **Turn** | One round-trip with the LLM (request → response) | Single LLM API call | LLM requests Grep tool → executes → sends results back |

**Relationship**: `Session` contains multiple `Tasks`, each `Task` contains multiple `Turns`

```
Session
├─ Task 1 ("implement login feature")
│  ├─ Turn 1: User input → LLM requests Read tool
│  ├─ Turn 2: Tool results → LLM requests Write tool
│  └─ Turn 3: Tool results → LLM final response
├─ Task 2 ("fix the bug in auth.js")
│  ├─ Turn 1: User input → LLM requests Grep tool
│  └─ Turn 2: Tool results → LLM final response
└─ Task 3 ("write tests")
   └─ Turn 1: User input → LLM final response (no tools needed)
```

### High-Level Architecture

Codex implements a **Submission Queue (SQ) / Event Queue (EQ)** pattern for asynchronous communication between user and agent:

```
┌─────────────────────────────────────────────────────────────────────┐
│                            Codex System                              │
│                                                                      │
│  ┌──────────────┐                                                   │
│  │   Client     │                                                   │
│  │   (TUI/CLI)  │                                                   │
│  └──────┬───────┘                                                   │
│         │ Submissions                                               │
│         │ (Operations)                                              │
│         ▼                                                           │
│  ┌──────────────────────────────────────────────────────────┐      │
│  │            Submission Queue (SQ)                          │      │
│  │  Op::UserInput | Op::UserTurn | Op::Interrupt |          │      │
│  │  Op::ExecApproval | Op::PatchApproval | ...              │      │
│  └──────────────┬───────────────────────────────────────────┘      │
│                 │                                                   │
│                 ▼                                                   │
│  ┌──────────────────────────────────────────────────────────┐      │
│  │         Submission Loop (Event Dispatcher)                │      │
│  │   - Routes operations to appropriate handlers             │      │
│  │   - Manages session and turn lifecycle                    │      │
│  └──────────────┬───────────────────────────────────────────┘      │
│                 │                                                   │
│                 ▼                                                   │
│  ┌──────────────────────────────────────────────────────────┐      │
│  │                Session (Arc<Session>)                     │      │
│  │  ┌──────────────────────────────────────────────────┐    │      │
│  │  │  SessionState (Mutex)                             │    │      │
│  │  │  - Conversation history                          │    │      │
│  │  │  - Token usage & rate limits                     │    │      │
│  │  └──────────────────────────────────────────────────┘    │      │
│  │  ┌──────────────────────────────────────────────────┐    │      │
│  │  │  ActiveTurn (Mutex<Option<ActiveTurn>>)          │    │      │
│  │  │  - Running tasks (IndexMap<sub_id, RunningTask>) │    │      │
│  │  │  - TurnState (pending approvals, pending input)  │    │      │
│  │  └──────────────────────────────────────────────────┘    │      │
│  │  ┌──────────────────────────────────────────────────┐    │      │
│  │  │  SessionServices                                  │    │      │
│  │  │  - MCP connection manager                        │    │      │
│  │  │  - Exec session managers                         │    │      │
│  │  │  - Notifier, Rollout, Executor, Shell            │    │      │
│  │  └──────────────────────────────────────────────────┘    │      │
│  └──────────────┬───────────────────────────────────────────┘      │
│                 │                                                   │
│                 ▼                                                   │
│  ┌──────────────────────────────────────────────────────────┐      │
│  │              SessionTask Execution                        │      │
│  │  - RegularTask: Normal user interactions                 │      │
│  │  - CompactTask: History compaction                       │      │
│  │  - ReviewTask: Code review operations                    │      │
│  │                                                           │      │
│  │  Each task:                                               │      │
│  │  1. Builds turn input with history                       │      │
│  │  2. Calls run_turn with TurnContext                      │      │
│  │  3. Streams model responses                              │      │
│  │  4. Executes tool calls (parallel or sequential)         │      │
│  │  5. Records results to history and rollout               │      │
│  └──────────────┬───────────────────────────────────────────┘      │
│                 │ Events                                            │
│                 ▼                                                   │
│  ┌──────────────────────────────────────────────────────────┐      │
│  │            Event Queue (EQ)                               │      │
│  │  EventMsg::AgentMessage | EventMsg::ToolCall |            │      │
│  │  EventMsg::TokenCount | EventMsg::Error | ...             │      │
│  └──────────────┬───────────────────────────────────────────┘      │
│                 │ Events                                            │
│                 ▼                                                   │
│  ┌──────────────────────────────────────────────────────────┐      │
│  │   Client (TUI/CLI)                                        │      │
│  │   - Renders events to user                                │      │
│  │   - Collects user input/approvals                         │      │
│  └──────────────────────────────────────────────────────────┘      │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### Core Architectural Principles

1. **Async Communication**: Submissions and events flow through unbounded async channels, enabling non-blocking operation
2. **State Isolation**: Session state (persistent) is isolated from turn state (ephemeral)
3. **Task-Based Execution**: Operations are encapsulated as tasks with standard lifecycle (spawn → run → abort/complete)
4. **Event-Driven**: All operations emit events for UI/client consumption, enabling reactive interfaces
5. **Thread Safety**: Shared mutable state uses Arc + Mutex patterns for concurrent access

---

## Core Components

### 1. Session

**Location**: `codex-rs/core/src/codex.rs`

The `Session` is the central orchestrator for a conversation. It represents a single continuous interaction with the model and manages the lifecycle of tasks, state, and events.

#### Structure

```rust
pub(crate) struct Session {
    conversation_id: ConversationId,
    tx_event: Sender<Event>,
    state: Mutex<SessionState>,
    pub(crate) active_turn: Mutex<Option<ActiveTurn>>,
    pub(crate) services: SessionServices,
    next_internal_sub_id: AtomicU64,
}
```

#### Key Responsibilities

| Responsibility | Description | Methods |
|---------------|-------------|---------|
| **Task Management** | Spawn, track, and abort tasks | `spawn_task`, `abort_all_tasks`, `on_task_finished` |
| **Event Distribution** | Send events to clients via event queue | `send_event`, `get_tx_event` |
| **Approval Workflow** | Request and handle user approvals | `request_command_approval`, `request_patch_approval`, `notify_approval` |
| **State Management** | Manage conversation history and token usage | `record_conversation_items`, `update_token_usage_info`, `history_snapshot` |
| **Tool Execution** | Execute commands and patches with event emission | `run_exec_with_events`, `on_exec_command_begin`, `on_exec_command_end` |
| **History Recording** | Persist conversation to rollout log | `persist_rollout_items`, `persist_rollout_response_items` |

#### Session Lifecycle

```
┌─────────────┐
│   new()     │  Creates session with initial configuration
└──────┬──────┘
       │
       ▼
┌─────────────────────────────────────────────────┐
│  Session Initialization                         │
│  1. Initialize MCP connection manager           │
│  2. Load conversation history (new/resumed)     │
│  3. Create model client                         │
│  4. Initialize services (executor, notifier)    │
│  5. Send SessionConfigured event                │
└──────┬──────────────────────────────────────────┘
       │
       ▼
┌─────────────────────────────────────────────────┐
│  Active Session (submission_loop)               │
│  - Receives submissions from client             │
│  - Routes operations to handlers                │
│  - Spawns tasks as needed                       │
│  - Emits events to client                       │
│                                                  │
│  Operations:                                    │
│  • UserInput/UserTurn → spawn RegularTask       │
│  • Compact → spawn CompactTask                  │
│  • Review → spawn ReviewTask                    │
│  • Interrupt → abort_all_tasks                  │
│  • ExecApproval/PatchApproval → notify_approval │
│  • Shutdown → terminate loop                    │
└──────┬──────────────────────────────────────────┘
       │
       ▼
┌─────────────┐
│  Shutdown   │  Cleanup and terminate
└─────────────┘
```

#### Thread Safety

- **Session** is wrapped in `Arc<Session>` for safe sharing across async tasks
- **SessionState** uses `Mutex` for interior mutability
- **ActiveTurn** uses `Mutex<Option<ActiveTurn>>` to allow atomic turn replacement
- **AtomicU64** for lock-free ID generation

---

### 2. SessionState

**Location**: `codex-rs/core/src/state/session.rs`

`SessionState` holds **persistent, session-scoped mutable state**. This state survives across turns and represents the accumulated context of the conversation.

#### Structure

```rust
pub(crate) struct SessionState {
    pub(crate) history: ConversationHistory,
    pub(crate) token_info: Option<TokenUsageInfo>,
    pub(crate) latest_rate_limits: Option<RateLimitSnapshot>,
}
```

#### Components

| Component | Purpose | Key Methods |
|-----------|---------|-------------|
| **history** | Conversation history (messages, tool calls, results) | `record_items`, `replace`, `contents` |
| **token_info** | Cumulative and last-turn token usage | `new_or_append`, `append_last_usage`, `fill_to_context_window` |
| **latest_rate_limits** | Rate limit status from last API call | Stored as-is |

#### Why Separate from TurnState?

- **Persistence**: SessionState survives across multiple turns; TurnState is ephemeral
- **Scope**: SessionState is conversation-wide; TurnState is turn-specific
- **Isolation**: Separating persistent and ephemeral state simplifies reasoning about state lifecycle

#### History Management

```
┌─────────────────────────────────────────────────────────────┐
│  Conversation History in SessionState                       │
│                                                              │
│  ResponseItem:                                              │
│  1. Message (user/assistant)                                │
│  2. FunctionCall / FunctionCallOutput                       │
│  3. LocalShellCall (tool calls)                             │
│  4. CustomToolCall / CustomToolCallOutput (MCP)             │
│                                                              │
│  Operations:                                                │
│  • record_items: Append to history                          │
│  • replace: Replace entire history (used after compaction)  │
│  • contents: Get snapshot for turn input                    │
│                                                              │
│  History flows to model on each turn via turn_input         │
└─────────────────────────────────────────────────────────────┘
```

---

### 3. TurnContext

**Location**: `codex-rs/core/src/codex.rs`

`TurnContext` provides **immutable configuration for a single turn**. It can be overridden per-turn to dynamically change model, policies, or working directory.

#### Structure

```rust
pub(crate) struct TurnContext {
    pub(crate) client: ModelClient,
    pub(crate) cwd: PathBuf,
    pub(crate) base_instructions: Option<String>,
    pub(crate) user_instructions: Option<String>,
    pub(crate) approval_policy: AskForApproval,
    pub(crate) sandbox_policy: SandboxPolicy,
    pub(crate) shell_environment_policy: ShellEnvironmentPolicy,
    pub(crate) tools_config: ToolsConfig,
    pub(crate) is_review_mode: bool,
    pub(crate) final_output_json_schema: Option<Value>,
}
```

#### Components

| Component | Purpose | Examples |
|-----------|---------|----------|
| **client** | Model client for API calls | Configured with model, reasoning effort, API keys |
| **cwd** | Working directory for commands/tools | `/Users/gaxx/Github/codex` |
| **base_instructions** | System-level instructions | Core Codex behavior guidelines |
| **user_instructions** | User-provided custom instructions | Project-specific guidance |
| **approval_policy** | When to ask for approval | `UnlessTrusted`, `OnFailure`, `OnRequest`, `Never` |
| **sandbox_policy** | Execution restrictions | `ReadOnly`, `WorkspaceWrite`, `DangerFullAccess` |
| **tools_config** | Available tools | `exec`, `apply_patch`, `read_file`, `view_image`, `plan` |
| **is_review_mode** | Review task isolation | Isolates history for review threads |
| **final_output_json_schema** | Structured output schema | For JSON-formatted responses |

#### Context Override Mechanism

Codex supports two levels of context:

1. **Persistent TurnContext**: Lives in `submission_loop`, applies to all future turns
2. **Per-Turn Override**: Specified in `Op::UserTurn`, applies only to that turn

```rust
// Persistent override (affects all future turns)
Op::OverrideTurnContext {
    cwd: Some(new_path),
    model: Some("gpt-4".into()),
    // ... other overrides
}

// Per-turn override (affects only this turn)
Op::UserTurn {
    items,
    cwd: new_path,
    model: "gpt-4".into(),
    // ... other overrides
}
```

**Why Two Levels?**
- **Persistent**: For session-wide configuration changes (e.g., switching model)
- **Per-Turn**: For temporary overrides (e.g., testing with different policies)

---

### 4. SessionTask

**Location**: `codex-rs/core/src/tasks/mod.rs`

`SessionTask` is a **trait for executable tasks**. Tasks encapsulate different types of operations (normal turns, compaction, reviews) with a uniform interface.

#### Trait Definition

```rust
#[async_trait]
pub(crate) trait SessionTask: Send + Sync + 'static {
    fn kind(&self) -> TaskKind;

    async fn run(
        self: Arc<Self>,
        session: Arc<SessionTaskContext>,
        ctx: Arc<TurnContext>,
        sub_id: String,
        input: Vec<InputItem>,
    ) -> Option<String>;

    async fn abort(&self, session: Arc<SessionTaskContext>, sub_id: &str) {
        // Default: no-op
    }
}
```

#### Task Types

| Task Type | Purpose | Behavior |
|-----------|---------|----------|
| **RegularTask** | Normal user interactions | Executes `run_task` with full conversation history |
| **CompactTask** | History compaction | Summarizes conversation to reduce token usage |
| **ReviewTask** | Code review operations | Isolated history thread with structured output |

#### Task Lifecycle

```
┌──────────────┐
│ spawn_task() │  Called by submission_loop
└──────┬───────┘
       │
       ▼
┌─────────────────────────────────────────────────┐
│  1. Abort all existing tasks                    │
│     (TurnAbortReason::Replaced)                 │
└──────┬──────────────────────────────────────────┘
       │
       ▼
┌─────────────────────────────────────────────────┐
│  2. Spawn async task with tokio::spawn          │
│     - Wrap session in SessionTaskContext        │
│     - Clone TurnContext as Arc                  │
│     - Call task.run()                            │
│     - On completion, call on_task_finished()    │
└──────┬──────────────────────────────────────────┘
       │
       ▼
┌─────────────────────────────────────────────────┐
│  3. Register task in ActiveTurn                 │
│     - Store AbortHandle for cancellation        │
│     - Store TaskKind for identification         │
└──────┬──────────────────────────────────────────┘
       │
       ▼
┌─────────────────────────────────────────────────┐
│  4. Task execution (run_task)                   │
│     - Emit TaskStarted event                    │
│     - Loop: run_turn until complete or error    │
│     - Handle auto-compaction if needed          │
│     - Return last_agent_message                 │
└──────┬──────────────────────────────────────────┘
       │
       ▼
┌─────────────────────────────────────────────────┐
│  5. Cleanup (on_task_finished)                  │
│     - Remove task from ActiveTurn.tasks         │
│     - Clear ActiveTurn if no tasks remain       │
│     - Emit TaskComplete event                   │
└─────────────────────────────────────────────────┘
```

#### Abort Handling

```rust
// Abort all tasks (user interrupted)
sess.abort_all_tasks(TurnAbortReason::Interrupted).await;

// Per-task abort
task.handle.abort();  // Cancel tokio task
task.task.abort(session_ctx, &sub_id).await;  // Task-specific cleanup
```

---

### 5. SessionServices

**Location**: `codex-rs/core/src/state/service.rs`

`SessionServices` holds **session-scoped services** shared across all tasks and turns.

#### Structure

```rust
pub(crate) struct SessionServices {
    pub(crate) mcp_connection_manager: McpConnectionManager,
    pub(crate) session_manager: ExecSessionManager,
    pub(crate) unified_exec_manager: UnifiedExecSessionManager,
    pub(crate) notifier: UserNotifier,
    pub(crate) rollout: Mutex<Option<RolloutRecorder>>,
    pub(crate) user_shell: crate::shell::Shell,
    pub(crate) show_raw_agent_reasoning: bool,
    pub(crate) executor: Executor,
}
```

#### Service Descriptions

| Service | Purpose | Usage |
|---------|---------|-------|
| **mcp_connection_manager** | MCP server connections | Lists tools, calls MCP tools, manages OAuth |
| **session_manager** | Legacy exec session tracking | Manages long-running commands, stdin/stdout streaming |
| **unified_exec_manager** | New unified exec tracking | Simplified command execution |
| **notifier** | Desktop notifications | Sends notifications on task completion/errors |
| **rollout** | Session recording | Appends events to JSONL rollout file |
| **user_shell** | User's shell config | Bash, Zsh, Fish detection |
| **executor** | Sandboxed command executor | Enforces sandbox policies (Seatbelt/Landlock) |

#### Why Services Are Session-Scoped

- **MCP connections**: Expensive to create, reused across turns
- **Executor**: Sandbox policy consistent per-session
- **Rollout**: Single append-only log per conversation
- **Notifier**: Per-session notification preferences

---

### 6. ActiveTurn & TurnState

**Location**: `codex-rs/core/src/state/turn.rs`

`ActiveTurn` and `TurnState` manage **ephemeral turn-specific state**.

#### ActiveTurn Structure

```rust
pub(crate) struct ActiveTurn {
    pub(crate) tasks: IndexMap<String, RunningTask>,
    pub(crate) turn_state: Arc<Mutex<TurnState>>,
}

pub(crate) struct RunningTask {
    pub(crate) handle: AbortHandle,
    pub(crate) kind: TaskKind,
    pub(crate) task: Arc<dyn SessionTask>,
}
```

**ActiveTurn** tracks:
- **tasks**: Map of `sub_id` → `RunningTask` (indexed for deterministic iteration)
- **turn_state**: Shared mutable state for the turn

**RunningTask** contains:
- **handle**: Tokio abort handle for cancellation
- **kind**: Task type (Regular/Compact/Review)
- **task**: Task implementation (for calling `abort()`)

#### TurnState Structure

```rust
pub(crate) struct TurnState {
    pending_approvals: HashMap<String, oneshot::Sender<ReviewDecision>>,
    pending_input: Vec<ResponseInputItem>,
}
```

**TurnState** holds:
- **pending_approvals**: Awaiting user approval (keyed by `sub_id`)
- **pending_input**: User input buffered during task execution

#### State Lifecycle

```
┌─────────────────────────────────────────────────┐
│  ActiveTurn Lifecycle                           │
│                                                  │
│  None  ──[spawn_task]──→  Some(ActiveTurn)      │
│                                                  │
│  Some(ActiveTurn):                              │
│  • tasks: {sub_id → RunningTask}                │
│  • turn_state: {pending_approvals, pending_input}│
│                                                  │
│  Some(ActiveTurn) ──[task_finished]──→  None    │
│  (when last task completes)                     │
│                                                  │
│  Some(ActiveTurn) ──[interrupt]──→  None        │
│  (when user aborts all tasks)                   │
└─────────────────────────────────────────────────┘
```

#### Why TurnState Is Separate

- **Ephemeral**: TurnState is cleared when turn ends; SessionState persists
- **Scoped**: TurnState is specific to a task; SessionState is conversation-wide
- **Cleanup**: Easily clear pending approvals/input without affecting history

---

## Execution Flow

### Submission Loop

**Location**: `codex-rs/core/src/codex.rs` (`submission_loop`)

The submission loop is the **heart of Codex's event dispatcher**. It receives operations from clients and routes them to appropriate handlers.

```rust
async fn submission_loop(
    sess: Arc<Session>,
    turn_context: TurnContext,
    config: Arc<Config>,
    rx_sub: Receiver<Submission>,
) {
    let mut turn_context = Arc::new(turn_context);
    while let Ok(sub) = rx_sub.recv().await {
        match sub.op {
            Op::UserInput { items } => {
                // Try to inject into running task, else spawn new
                if let Err(items) = sess.inject_input(items).await {
                    sess.spawn_task(turn_context.clone(), sub.id, items, RegularTask).await;
                }
            }
            Op::UserTurn { items, cwd, model, ... } => {
                // Build per-turn context with overrides
                let fresh_turn_context = TurnContext { /* ... */ };
                if let Err(items) = sess.inject_input(items).await {
                    sess.spawn_task(Arc::new(fresh_turn_context), sub.id, items, RegularTask).await;
                }
            }
            Op::Interrupt => {
                sess.abort_all_tasks(TurnAbortReason::Interrupted).await;
            }
            Op::ExecApproval { id, decision } => {
                sess.notify_approval(&id, decision).await;
            }
            Op::Compact => {
                sess.spawn_task(turn_context.clone(), sub.id, vec![], CompactTask).await;
            }
            Op::Review { review_request } => {
                sess.spawn_task(turn_context_for_review, sub.id, input, ReviewTask).await;
            }
            Op::Shutdown => break,
            // ... other operations
        }
    }
}
```

### Understanding Tasks and Turns

#### Task Lifecycle Pattern

A **task** represents the complete execution of a user request:

```
User Request: "analyze security.js for vulnerabilities"
        ↓
┌─────────────────────────────────────────┐
│ Task Started                            │
│ - Record initial user input             │
│ - Initialize task state                 │
└─────────────────────────────────────────┘
        ↓
╔═════════════════════════════════════════╗
║         Turn Loop (iterates)            ║
║                                         ║
║  Turn 1: LLM → Request Read tool       ║
║  Turn 2: Tool results → Request Grep   ║
║  Turn 3: Analysis → Final response     ║
╚═════════════════════════════════════════╝
        ↓
┌─────────────────────────────────────────┐
│ Task Completed                          │
│ - Emit completion event                │
│ - Clean up task state                  │
└─────────────────────────────────────────┘
```

**Key Design Decision**: Tasks can be interrupted and aborted cleanly at turn boundaries.

#### Turn Loop Pattern

The turn loop enables **iterative problem-solving** where the LLM can request tools and see their results:

```
loop {
    1. Build turn input from conversation history
    2. Send to LLM
    3. Process LLM response:
       ├─ Tool calls → Execute → Record → Continue
       ├─ Final message → Record → Break
       └─ Error → Retry or Break
    4. Check resource limits (tokens, time)
}
```

**Why This Works**:
- LLM sees **cumulative conversation history** on each turn
- Tool results are automatically included in next turn's context
- Natural breakpoints for interruption and error handling

#### Multi-Turn Example

```
Turn 1: User: "analyze security.js"
        LLM:  "I'll read the file" → Read("security.js")

Turn 2: Tool: "function login(user, pass) { ... }"
        LLM:  "I see password handling, let me check for issues" → Grep("hardcoded")

Turn 3: Tool: "No hardcoded passwords found"
        LLM:  "Analysis complete. Found 3 vulnerabilities: ..." [FINAL]
```

**Pattern Benefit**: The LLM builds understanding progressively, similar to how humans debug - explore, hypothesize, verify, conclude.

### Tool Call Execution Pattern

Codex supports **parallel tool call execution** for improved performance:

```
┌────────────────────────────────────────────────┐
│  Parallel Tool Execution                       │
│                                                 │
│  LLM requests: [Read(file1), Read(file2), Grep]│
│                                                 │
│  If parallel supported:                        │
│    ┌─────────┐  ┌─────────┐  ┌─────────┐     │
│    │ Read 1  │  │ Read 2  │  │  Grep   │     │
│    └────┬────┘  └────┬────┘  └────┬────┘     │
│         │            │            │           │
│         └────────────┴────────────┘           │
│                      ▼                         │
│          Collect all results                  │
│          Return to LLM in next turn           │
│                                                 │
│  If sequential:                                │
│    Execute → Wait → Execute → Wait → ...      │
└────────────────────────────────────────────────┘
```

**Design Decision**: Model and tool must both support parallelism
- **Benefit**: Significant speedup for independent operations
- **Trade-off**: More complex error handling and state management
- **Fallback**: Graceful degradation to sequential execution

### Streaming and Progressive Response

```
┌────────────────────────────────────────────────┐
│  Streaming Pattern                             │
│                                                 │
│  LLM generates tokens → Stream to client       │
│                                                 │
│  Benefits:                                     │
│  • Lower perceived latency                    │
│  • Better UX (see thinking in real-time)      │
│  • Early interruption possible                │
│                                                 │
│  Events:                                       │
│  • AgentMessageDelta (text chunks)             │
│  • AgentReasoningDelta (thinking process)      │
│  • FunctionCall (tool requests)                │
│  • Completed (end of response)                 │
└────────────────────────────────────────────────┘
```

**Why Streaming?**
- Users see progress immediately, not after complete response
- Can interrupt long responses early
- Better experience for slow network connections

---

## State Management Patterns

### Persistent vs Ephemeral State

| State Type | Scope | Lifecycle | Storage | Example |
|------------|-------|-----------|---------|---------|
| **SessionState** | Conversation | Multi-turn, persistent | `Mutex<SessionState>` | Conversation history, cumulative tokens |
| **TurnState** | Single turn | Ephemeral, cleared on end | `Mutex<TurnState>` | Pending approvals, buffered input |
| **TurnContext** | Per-turn | Immutable for turn | `Arc<TurnContext>` | Model config, policies, cwd |

### State Access Patterns

```rust
// Access SessionState (persistent)
async fn update_history(&self, items: &[ResponseItem]) {
    let mut state = self.state.lock().await;
    state.record_items(items.iter());
}

// Access TurnState (ephemeral)
async fn request_approval(&self, sub_id: String, tx: oneshot::Sender<ReviewDecision>) {
    let mut active = self.active_turn.lock().await;
    if let Some(at) = active.as_mut() {
        let mut ts = at.turn_state.lock().await;
        ts.insert_pending_approval(sub_id, tx);
    }
}

// Replace ActiveTurn atomically
async fn spawn_task(&self, ...) {
    // Abort existing tasks
    self.abort_all_tasks(TurnAbortReason::Replaced).await;

    // Create new ActiveTurn
    let mut active = self.active_turn.lock().await;
    let mut turn = ActiveTurn::default();
    turn.add_task(sub_id, task);
    *active = Some(turn);
}
```

### Concurrency Patterns

1. **Arc for Sharing**: `Arc<Session>`, `Arc<TurnContext>` allow safe sharing across async tasks
2. **Mutex for Interior Mutability**: `Mutex<SessionState>`, `Mutex<Option<ActiveTurn>>` for exclusive access
3. **Channels for Communication**: `Sender<Event>`, `Receiver<Submission>` for async message passing
4. **Oneshot for Approvals**: `oneshot::Sender<ReviewDecision>` for single-response requests
5. **AtomicU64 for Counters**: Lock-free ID generation

### History Management

#### Dual Storage Pattern

Codex maintains conversation history in **two places** for different purposes:

| Storage Layer | Purpose | Trade-off |
|--------------|---------|-----------|
| **In-Memory** | Fast access for building turn context | Lost on session end |
| **Persistent** | Durable log for session resume | I/O overhead on writes |

**Why This Pattern?**
- **Performance**: In-memory history enables fast turn construction without disk I/O
- **Durability**: Persistent log allows resuming conversations after restart
- **Separation of Concerns**: Read operations don't block on write operations

#### History Growth and Compaction

```
┌──────────────────────────────────────────────────┐
│  History Lifecycle                               │
│                                                   │
│  User Input → LLM Response → Tool Calls →        │
│  Tool Results → LLM Response → ...               │
│                                                   │
│  ↓ (when approaching token limit)                │
│                                                   │
│  Compaction: Summarize old turns                 │
│  Keep: Recent context + User messages            │
│  Result: Reduced token usage, preserved context  │
└──────────────────────────────────────────────────┘
```

**Design Trade-off**:
- Send full history every turn → LLM has complete context
- Auto-compact when needed → Stay within token limits
- Preserve user messages → Don't lose original intent

#### Turn Input Construction

Each turn receives the **complete conversation history**:

```
Turn Input = [
  Environment Context,
  User Instructions,
  User Message,
  Previous LLM Response,
  Previous Tool Calls,
  Previous Tool Results,
  ... (repeated for all prior turns)
]
```

This enables the LLM to:
- See results of previously requested tools
- Build on prior analysis
- Maintain conversation coherence
- Make informed decisions about next steps

---

## Communication Patterns

### Submission Types

| Op Type | Purpose | Handler | Result |
|---------|---------|---------|--------|
| **UserInput** | User message, inject or spawn | `inject_input` → `spawn_task` | RegularTask spawned |
| **UserTurn** | User message with overrides | `inject_input` → `spawn_task` with per-turn context | RegularTask with custom context |
| **Interrupt** | Abort current tasks | `abort_all_tasks` | TurnAborted events |
| **OverrideTurnContext** | Update persistent context | Update `turn_context` Arc | No event |
| **ExecApproval** | Approve/deny command | `notify_approval` | Unblock approval request |
| **PatchApproval** | Approve/deny file changes | `notify_approval` | Unblock approval request |
| **Compact** | Trigger compaction | `spawn_task(CompactTask)` | History summarized |
| **Review** | Start code review | `spawn_task(ReviewTask)` | ReviewTask with isolated history |
| **Shutdown** | Terminate session | Break submission loop | Process exits |

### Event Types

| EventMsg Type | Purpose | Emitted By | Payload |
|---------------|---------|------------|---------|
| **SessionConfigured** | Session initialized | Session::new | conversation_id, model, rollout_path |
| **TaskStarted** | Task began | run_task | model_context_window |
| **TaskComplete** | Task finished | on_task_finished | last_agent_message |
| **AgentMessage** | Model text output | Stream processing | message |
| **AgentMessageDelta** | Streaming text | Stream processing | delta |
| **TokenCount** | Usage update | update_token_usage_info | TokenUsageInfo, RateLimitSnapshot |
| **ExecCommandBegin** | Command starting | on_exec_command_begin | call_id, command, cwd |
| **ExecCommandEnd** | Command finished | on_exec_command_end | stdout, stderr, exit_code |
| **ExecApprovalRequest** | Request approval | request_command_approval | call_id, command, reason |
| **PatchApplyBegin** | Patch starting | on_exec_command_begin | changes, auto_approved |
| **PatchApplyEnd** | Patch finished | on_exec_command_end | success, stdout/stderr |
| **TurnDiff** | Full turn diff | on_exec_command_end | unified_diff |
| **Error** | Error occurred | Various | message |
| **TurnAborted** | Task aborted | handle_task_abort | reason |

### Approval Workflow

```
┌─────────────────────────────────────────────────┐
│  Approval Request Flow                          │
│                                                  │
│  1. Tool execution requires approval            │
│     → session.request_command_approval()         │
│                                                  │
│  2. Create oneshot channel                      │
│     let (tx, rx) = oneshot::channel();          │
│                                                  │
│  3. Store tx in TurnState.pending_approvals     │
│     ts.insert_pending_approval(sub_id, tx)      │
│                                                  │
│  4. Emit ExecApprovalRequest event              │
│     → Client displays approval UI                │
│                                                  │
│  5. User responds via ExecApproval submission   │
│     → session.notify_approval(sub_id, decision)  │
│                                                  │
│  6. Remove tx from pending_approvals            │
│     let tx = ts.remove_pending_approval(sub_id) │
│                                                  │
│  7. Send decision through oneshot channel       │
│     tx.send(decision).ok()                      │
│                                                  │
│  8. Tool execution unblocks                     │
│     let decision = rx.await.unwrap_or_default() │
│     → Execute or skip based on decision         │
└─────────────────────────────────────────────────┘
```

### Input Injection

Codex supports **interrupting a running task** with new user input:

```rust
// User submits input while task is running
Op::UserInput { items } => {
    // Try to inject into running task
    if let Err(items) = sess.inject_input(items).await {
        // No running task, spawn new one
        sess.spawn_task(turn_context.clone(), sub.id, items, RegularTask).await;
    }
}

// inject_input implementation
pub async fn inject_input(&self, input: Vec<InputItem>) -> Result<(), Vec<InputItem>> {
    let mut active = self.active_turn.lock().await;
    match active.as_mut() {
        Some(at) => {
            let mut ts = at.turn_state.lock().await;
            ts.push_pending_input(input.into());
            Ok(())  // Input buffered
        }
        None => Err(input),  // No active task
    }
}

// Task retrieves buffered input in turn loop
let pending_input = sess.get_pending_input().await;
// Include in next turn_input sent to model
```

---

## Key Design Decisions

### 1. Why Session → Task → Turn Hierarchy?

**Problem**: How to manage long-lived conversations with multiple user requests?

**Solution**: Three-level hierarchy with clear responsibilities
- **Session**: Manages conversation lifetime and accumulated context
- **Task**: Encapsulates one user request with abort/retry capability
- **Turn**: Enables iterative LLM problem-solving with tool calls

**Why This Works**:
- Natural mapping to user mental model (conversation → request → interaction)
- Clear boundaries for state management and lifecycle
- Easy interruption and error recovery at appropriate levels

### 2. Why Event-Driven Architecture?

**Problem**: How to support multiple client types (CLI, TUI, web) without coupling?

**Solution**: Session emits events; clients consume independently

**Trade-offs**:
- ✅ Decouple business logic from presentation
- ✅ Easy to add new client types
- ✅ Natural fit for reactive UIs
- ❌ More complex than direct function calls
- ❌ Debugging across event boundaries

### 3. Why Separate Persistent and Ephemeral State?

**Problem**: How to prevent state leakage between operations?

**Solution**: Clear separation of concerns
- **SessionState**: Conversation history, cumulative metrics (persistent)
- **TurnState**: Pending approvals, buffered input (cleared on task end)

**Why This Works**:
- Lifecycle matches user expectations (history persists, approvals don't)
- Simplified cleanup - just drop TurnState
- Prevents accidental state pollution

### 4. Why Dual Storage (In-Memory + Persistent)?

**Problem**: Trade-off between performance and durability

**Solution**: In-memory for reads, persistent log for writes
- Fast turn construction without disk I/O
- Durable conversation history for resume
- Writes don't block reads

**Alternative Considered**: Database with caching
- **Why Not**: Added complexity, limited benefit for append-only workload

### 5. Why Approval Workflow Pattern?

**Problem**: How to give users control over sensitive operations?

**Solution**: Oneshot channel + pending approval map
- Request approval → Store channel → Emit event
- User responds → Send decision through channel
- Timeout/abort → Drop channel

**Why This Works**:
- Clean async await pattern
- Easy to implement timeouts and cancellation
- No polling or busy-waiting

### 6. Why Parallel Tool Execution?

**Problem**: Sequential tool calls waste time on independent operations

**Solution**: Execute independent tool calls concurrently when supported

**Trade-offs**:
- ✅ 2-5x speedup for independent operations
- ✅ Better resource utilization
- ❌ More complex error handling
- ❌ Requires model and tool support

### 7. Why Streaming Response Pattern?

**Problem**: Users wait for complete response before seeing anything

**Solution**: Stream tokens as generated

**Why This Works**:
- Lower perceived latency (see progress immediately)
- Can interrupt long responses early
- Better UX for slow network conditions
- Natural fit for agentic "thinking out loud"

---

## Reusable Patterns for Other Platforms

### 1. SQ/EQ Pattern Implementation

```rust
// Create bounded submission channel, unbounded event channel
let (tx_sub, rx_sub) = async_channel::bounded(64);
let (tx_event, rx_event) = async_channel::unbounded();

// Spawn submission loop
tokio::spawn(async move {
    while let Ok(submission) = rx_sub.recv().await {
        // Route submission to handlers
        match submission.op {
            Op::UserInput { .. } => { /* ... */ }
            Op::Interrupt => { /* ... */ }
            // ...
        }
    }
});

// Client submits operations
tx_sub.send(Submission { id: "1".into(), op: Op::UserInput { items } }).await?;

// Client receives events
while let Ok(event) = rx_event.recv().await {
    match event.msg {
        EventMsg::AgentMessage(msg) => println!("{}", msg.message),
        EventMsg::Error(err) => eprintln!("{}", err.message),
        // ...
    }
}
```

**Why This Works**:
- Bounded submission queue prevents unbounded memory growth
- Unbounded event queue ensures no events are dropped
- Async channels enable non-blocking send/recv

### 2. State Isolation Strategy

```rust
// Persistent state (survives across turns)
struct SessionState {
    history: ConversationHistory,
    token_usage: TokenUsage,
}

// Ephemeral state (cleared on task end)
struct TurnState {
    pending_approvals: HashMap<String, oneshot::Sender<Decision>>,
    pending_input: Vec<Input>,
}

// Session holds both
struct Session {
    state: Mutex<SessionState>,
    active_turn: Mutex<Option<ActiveTurn>>,
}

// Clear turn state without affecting session state
async fn end_turn(&self) {
    let mut active = self.active_turn.lock().await;
    *active = None;  // TurnState dropped here
}
```

**Why This Works**:
- Clear ownership boundaries
- Easy to reason about state lifecycle
- Prevents accidental state leakage

### 3. Approval and Interruption Handling

```rust
// Request approval with oneshot channel
async fn request_approval(&self, sub_id: String) -> Decision {
    let (tx, rx) = oneshot::channel();

    // Store sender in turn state
    {
        let mut turn_state = self.turn_state.lock().await;
        turn_state.pending_approvals.insert(sub_id.clone(), tx);
    }

    // Emit approval request event
    self.send_event(Event::ApprovalRequest { sub_id: sub_id.clone() }).await;

    // Wait for response (blocks until user responds)
    rx.await.unwrap_or(Decision::Denied)
}

// User responds
async fn notify_approval(&self, sub_id: &str, decision: Decision) {
    let mut turn_state = self.turn_state.lock().await;
    if let Some(tx) = turn_state.pending_approvals.remove(sub_id) {
        tx.send(decision).ok();
    }
}

// Interrupt (cancel approval)
async fn interrupt(&self) {
    let mut turn_state = self.turn_state.lock().await;
    turn_state.pending_approvals.clear();  // Drop all senders
}
```

**Why This Works**:
- Oneshot channel ensures single-response
- Clear ownership of approval lifecycle
- Interrupt is trivial (just clear map)

### 4. Token Tracking and Rate Limiting

```rust
struct SessionState {
    token_info: Option<TokenUsageInfo>,
    rate_limits: Option<RateLimitSnapshot>,
}

struct TokenUsageInfo {
    total_token_usage: TokenUsage,
    last_token_usage: TokenUsage,
    model_context_window: Option<u64>,
}

// Update after each turn
async fn update_token_usage(&self, usage: TokenUsage) {
    let mut state = self.state.lock().await;
    state.token_info = TokenUsageInfo::new_or_append(
        &state.token_info,
        &Some(usage),
        self.model_context_window,
    );
}

// Check if approaching limit
fn approaching_limit(&self, threshold: f64) -> bool {
    if let Some(info) = &self.token_info {
        let used = info.total_token_usage.total_tokens;
        let limit = info.model_context_window.unwrap_or(u64::MAX);
        (used as f64 / limit as f64) > threshold
    } else {
        false
    }
}
```

**Why This Works**:
- Cumulative and per-turn tracking
- Easy to trigger auto-compaction
- Rate limit awareness for retry logic

### 5. Task Orchestration with Abort Support

```rust
// Task trait
#[async_trait]
trait Task: Send + Sync + 'static {
    async fn run(&self, ctx: Context) -> Result<Output>;
    async fn abort(&self) { /* optional cleanup */ }
}

// Spawn task with abort handle
async fn spawn_task<T: Task>(&self, task: T) {
    let handle = tokio::spawn(async move {
        task.run(ctx).await
    }).abort_handle();

    // Store handle for later abort
    self.active_tasks.lock().await.insert(task_id, handle);
}

// Abort all tasks
async fn abort_all(&self) {
    let tasks = self.active_tasks.lock().await.drain().collect::<Vec<_>>();
    for (id, handle) in tasks {
        handle.abort();
    }
}
```

**Why This Works**:
- Tokio's abort_handle provides clean cancellation
- Task trait allows custom cleanup logic
- Easy to abort all tasks on interrupt

---

## Code Examples

### Example 1: Session Initialization

```rust
use codex_core::Codex;
use codex_protocol::protocol::{InitialHistory, SessionSource};

async fn init_session() -> Result<Codex> {
    let config = Config::load()?;
    let auth_manager = Arc::new(AuthManager::new()?);

    let history = InitialHistory::New;  // New conversation
    let source = SessionSource::Cli;

    let CodexSpawnOk { codex, conversation_id } =
        Codex::spawn(config, auth_manager, history, source).await?;

    println!("Session initialized: {}", conversation_id);
    Ok(codex)
}
```

### Example 2: Submitting User Input

```rust
use codex_protocol::protocol::{Op, InputItem};

async fn send_message(codex: &Codex, text: &str) -> Result<String> {
    let items = vec![InputItem::Text { text: text.into() }];
    let op = Op::UserInput { items };

    let sub_id = codex.submit(op).await?;
    println!("Submitted with ID: {}", sub_id);
    Ok(sub_id)
}
```

### Example 3: Handling Events

```rust
use codex_protocol::protocol::{Event, EventMsg};

async fn handle_events(codex: &Codex) -> Result<()> {
    loop {
        let event = codex.next_event().await?;

        match event.msg {
            EventMsg::AgentMessage(msg) => {
                println!("Agent: {}", msg.message);
            }
            EventMsg::TokenCount(info) => {
                if let Some(usage) = info.info {
                    println!("Tokens: {} total", usage.total_token_usage.total_tokens);
                }
            }
            EventMsg::ExecApprovalRequest(req) => {
                println!("Approve command? {:?}", req.command);
                // User responds with ExecApproval submission
            }
            EventMsg::TaskComplete(_) => {
                println!("Task finished");
                break;
            }
            EventMsg::Error(err) => {
                eprintln!("Error: {}", err.message);
                break;
            }
            _ => { /* handle other events */ }
        }
    }
    Ok(())
}
```

### Example 4: Approval Workflow

```rust
use codex_protocol::protocol::{Op, ReviewDecision};

async fn handle_approval(codex: &Codex, event: Event) -> Result<()> {
    if let EventMsg::ExecApprovalRequest(req) = event.msg {
        println!("Command: {:?}", req.command);
        println!("Working dir: {:?}", req.cwd);

        // Get user decision (simulate with auto-approve here)
        let decision = ReviewDecision::Approved;

        // Submit approval
        let op = Op::ExecApproval {
            id: event.id.clone(),
            decision,
        };
        codex.submit(op).await?;
    }
    Ok(())
}
```

### Example 5: Dynamic Context Override

```rust
use codex_protocol::protocol::{Op, AskForApproval, SandboxPolicy};
use std::path::PathBuf;

async fn switch_directory(codex: &Codex, new_dir: PathBuf) -> Result<()> {
    let op = Op::OverrideTurnContext {
        cwd: Some(new_dir.clone()),
        approval_policy: None,
        sandbox_policy: None,
        model: None,
        effort: None,
        summary: None,
    };

    codex.submit(op).await?;
    println!("Switched to directory: {}", new_dir.display());
    Ok(())
}

async fn run_with_readonly_sandbox(codex: &Codex, text: &str) -> Result<()> {
    let items = vec![InputItem::Text { text: text.into() }];

    let op = Op::UserTurn {
        items,
        cwd: std::env::current_dir()?,
        approval_policy: AskForApproval::OnRequest,
        sandbox_policy: SandboxPolicy::ReadOnly,  // Per-turn override
        model: "gpt-4".into(),
        effort: None,
        summary: Default::default(),
        final_output_json_schema: None,
    };

    codex.submit(op).await?;
    Ok(())
}
```

### Example 6: Interrupt Running Task

```rust
async fn interrupt_task(codex: &Codex) -> Result<()> {
    let op = Op::Interrupt;
    codex.submit(op).await?;
    println!("Task interrupted");
    Ok(())
}
```

---

## Appendix: File Reference

| Component | File Path |
|-----------|-----------|
| **Session, TurnContext** | `codex-rs/core/src/codex.rs` |
| **SessionState** | `codex-rs/core/src/state/session.rs` |
| **SessionServices** | `codex-rs/core/src/state/service.rs` |
| **ActiveTurn, TurnState** | `codex-rs/core/src/state/turn.rs` |
| **SessionTask trait** | `codex-rs/core/src/tasks/mod.rs` |
| **RegularTask** | `codex-rs/core/src/tasks/regular.rs` |
| **CompactTask** | `codex-rs/core/src/tasks/compact.rs` |
| **ReviewTask** | `codex-rs/core/src/tasks/review.rs` |
| **Protocol (Op, EventMsg)** | `codex-rs/protocol/src/protocol.rs` |

---

## Conclusion

Codex's architecture demonstrates several **reusable patterns** for building agentic platforms:

### Core Architectural Insights

**1. Hierarchical State Management**
- **Session** (lifetime) → **Task** (operation) → **Turn** (interaction)
- Clear separation of persistent vs ephemeral state
- Easy to reason about lifecycle and cleanup

**2. Event-Driven Architecture**
- Decouple business logic from presentation layer
- Enable reactive UIs and multiple client types
- Non-blocking communication via async channels

**3. Iterative Problem-Solving**
- Turn loop enables LLM to request tools and see results
- Cumulative history provides complete context each turn
- Natural breakpoints for interruption and error handling

**4. Flexible Configuration**
- Per-turn overrides without affecting session state
- Dynamic behavior changes without restart
- Support for different use cases (CLI, TUI, automation)

**5. Robust Concurrency**
- Thread-safe shared state via Arc + Mutex
- Parallel tool execution for performance
- Clean task abort and interruption

**6. User Control**
- Approval workflow for security-sensitive operations
- Transparent event stream for observability
- Input injection for mid-task interaction

### Applying These Patterns

When building an agentic platform, consider:

- **Will you need conversation continuity?** → Session pattern
- **Should users control sensitive operations?** → Approval workflow
- **Can operations be interrupted cleanly?** → Task-based execution
- **Do you need responsive UIs?** → Event-driven + streaming
- **Should the AI see previous tool results?** → Turn loop with history
- **Will operations take significant time?** → Async execution with progress events

These patterns work together to create a **responsive, controllable, and reliable** agentic system.

---

**Document Version**: 1.0
**Author**: Claude (Codex)
**Generated**: 2025-10-16
