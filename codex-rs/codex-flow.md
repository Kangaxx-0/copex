# Codex Flow: User Input to Response

## Flow Diagram

```
User Input → CLI Parser → Mode Router → AppServer → Codex → Model Client → Response
    ↓           ↓           ↓              ↓          ↓        ↓              ↓
[Terminal]  [cli/main]  [cli/main]   [app_server]  [codex]  [client]    [Terminal]
    ↑                                                  ↓
    └────────────────── Event Stream ←────────────────┘
```

## Connection Points & Data Flow

### 1. CLI → Mode Selection
**Connection**: Command line arguments determine execution mode
- **TUI Mode**: No subcommand → `codex_tui::run_main()` (`tui/src/lib.rs:587`)
- **Exec Mode**: `exec` subcommand → `codex_exec::run_main()` (`exec/src/lib.rs:177`)

### 2. Mode → Thread Start
**Connection**: Both modes create a conversation thread via `ClientRequest::ThreadStart`
- **TUI**: `AppServerSession::start_thread()` in `tui/src/app_server_session.rs:287`
- **Exec**: `ClientRequest::ThreadStart` in `exec/src/lib.rs:568`

### 3. Thread → CodexConversation
**Connection**: `CodexConversation` is a type alias for `CodexThread`
```rust
// core/src/lib.rs:131
pub type CodexConversation = CodexThread;
```

### 4. CodexConversation → Codex
**Connection**: `CodexThread` wraps `Codex` and delegates to it
```rust
// codex_thread.rs
pub async fn submit(&self, op: Op) -> CodexResult<String> {
    self.codex.submit(op).await  // Direct delegation
}
```

### 5. Codex → Model Client
**Connection**: `Codex::submit()` processes ops and calls model client
- Streaming via `client.stream()` in `core/src/client.rs:1289`
- Routes to either `stream_responses_websocket` or `stream_responses_api`

### 6. Response → Event Stream
**Connection**: Model responses become events via `next_event()`
- Events flow back through the same connection chain
- Both TUI and Exec modes listen to `conversation.next_event().await`

## Key Code Locations

| Stage | File | Key Function |
|-------|------|--------------|
| **Entry** | `cli/src/main.rs` | `cli_main()` |
| **TUI Mode** | `tui/src/app.rs` | `App::run()` |
| **Exec Mode** | `exec/src/lib.rs` | `run_main()` |
| **Thread Start (TUI)** | `tui/src/app_server_session.rs` | `start_thread()` |
| **Thread Start (Exec)** | `exec/src/lib.rs` | `ClientRequest::ThreadStart` |
| **Codex Engine** | `core/src/codex.rs` | `Codex::submit()` |
| **Model Client** | `core/src/client.rs` | `stream()` |
| **Event Processing** | `core/src/codex.rs` | `next_event()` |

## Connection Implementation Details

### TUI Mode Connection Pattern (`tui/src/app_server_session.rs`)

```rust
// 1. Start thread via AppServerSession
pub(crate) async fn start_thread(&mut self, config: &Config) -> Result<AppServerStartedThread> {
    // Sends ClientRequest::ThreadStart with params from config
}

// 2. Spawn Op submission task
let conversation_clone = conversation.clone();
tokio::spawn(async move {
    while let Some(op) = codex_op_rx.recv().await {
        let id = conversation_clone.submit(op).await;
        if let Err(e) = id {
            tracing::error!("failed to submit op: {e}");
        }
    }
});

// 3. Event streaming loop
while let Ok(event) = conversation.next_event().await {
    app_event_tx_clone.send(AppEvent::CodexEvent(event));
}
```

### Exec Mode Connection Pattern (`exec/src/lib.rs`)

```rust
// 1. Start thread via ClientRequest::ThreadStart
// 2. Spawn event handler with Op submission capability
let conversation = conversation.clone();
tokio::spawn(async move {
    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                conversation.submit(Op::Interrupt).await.ok();
                break;
            }
            res = conversation.next_event() => match res {
                Ok(event) => {
                    // Handle events...
                }
            }
        }
    }
});
```

### The Connection Chain

**Key Insight**: The `CodexConversation` (alias for `CodexThread`) provides both:
- `submit(op)` - for sending operations to Codex
- `next_event()` - for receiving responses as events

**Data Flow**:
1. Thread start → returns conversation handle
2. `conversation.submit(op)` → delegates to `self.codex.submit(op)`
3. `conversation.next_event()` → delegates to `self.codex.next_event()`
4. Both modes spawn async tasks that use the same `conversation` instance

## Data Flow Types

- **Input**: `Op` (operation/message)
- **Processing**: `Event`, `ResponseItem`, `ToolCall`
- **Output**: `Event` stream → UI rendering
