# Codex-RS Planning Tool: Deep Code Review & Implementation Guide

A comprehensive analysis of the `update_plan` tool implementation in codex-rs, examining architecture patterns, design decisions, and inspirational takeaways for building similar task tracking systems.

## Executive Summary

The planning tool in codex-rs is a **structured communication mechanism** that allows the AI model to express its task breakdown and progress to users in a machine-readable, renderable format. Rather than parsing free-form text, the system provides a formal tool that enforces structure while remaining lightweight and non-blocking.

**Key Insight**: The planning tool is intentionally "useless" from a functional standpoint - it doesn't drive execution or modify state. Its value lies entirely in the **structured input** it accepts, not any output it produces.

---

## Architecture Overview

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│   AI Model      │────▶│  Tool Handler   │────▶│  Event System   │
│                 │     │  (plan.rs)      │     │                 │
│ update_plan()   │     │                 │     │ EventMsg::      │
│ tool call       │     │ Parse → Emit    │     │ PlanUpdate      │
└─────────────────┘     └─────────────────┘     └────────┬────────┘
                                                         │
                              ┌───────────────────────────┤
                              │                           │
                    ┌─────────▼─────────┐     ┌──────────▼──────────┐
                    │   TUI Renderer    │     │   Other Clients     │
                    │ (history_cell.rs) │     │ (MCP, Exec, SDK)    │
                    │                   │     │                     │
                    │  ✔ Completed      │     │  JSON structured    │
                    │  □ In Progress    │     │  events             │
                    │  □ Pending        │     │                     │
                    └───────────────────┘     └─────────────────────┘
```

### Layer Separation

| Layer | Component | Responsibility |
|-------|-----------|----------------|
| **Protocol** | `protocol/src/plan_tool.rs` | Type definitions, serialization |
| **Handler** | `core/src/tools/handlers/plan.rs` | Tool invocation processing |
| **Event** | `protocol/src/protocol.rs` | Event enum with PlanUpdate variant |
| **Session** | `core/src/codex.rs` | Event emission and persistence |
| **Rendering** | `tui/src/history_cell.rs` | Visual display in terminal |

---

## Core Data Structures

### Protocol Types (`protocol/src/plan_tool.rs`)

```rust
/// Status of an individual plan step
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "snake_case")]
pub enum StepStatus {
    Pending,
    InProgress,
    Completed,
}

/// A single step in the plan
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
pub struct PlanItemArg {
    pub step: String,
    pub status: StepStatus,
}

/// Complete plan update payload
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
pub struct UpdatePlanArgs {
    #[serde(default)]
    pub explanation: Option<String>,  // Optional context for the update
    pub plan: Vec<PlanItemArg>,       // The actual plan steps
}
```

**Design Decisions**:
- `deny_unknown_fields`: Strict parsing prevents model from adding arbitrary fields
- `rename_all = "snake_case"`: JSON keys match Rust conventions
- `JsonSchema` + `TS`: Auto-generates JSON schema and TypeScript types for SDK

---

## Tool Handler Implementation

### Tool Specification (`core/src/tools/handlers/plan.rs:20-60`)

```rust
pub static PLAN_TOOL: LazyLock<ToolSpec> = LazyLock::new(|| {
    // Build JSON schema for plan items
    let mut plan_item_props = BTreeMap::new();
    plan_item_props.insert("step".to_string(), JsonSchema::String { description: None });
    plan_item_props.insert(
        "status".to_string(),
        JsonSchema::String {
            description: Some("One of: pending, in_progress, completed".to_string()),
        },
    );

    let plan_items_schema = JsonSchema::Array {
        description: Some("The list of steps".to_string()),
        items: Box::new(JsonSchema::Object {
            properties: plan_item_props,
            required: Some(vec!["step".to_string(), "status".to_string()]),
            additional_properties: Some(false.into()),
        }),
    };

    ToolSpec::Function(ResponsesApiTool {
        name: "update_plan".to_string(),
        description: r#"Updates the task plan.
Provide an optional explanation and a list of plan items, each with a step and status.
At most one step can be in_progress at a time.
"#.to_string(),
        strict: false,  // Allows flexibility in model output
        parameters: JsonSchema::Object {
            properties,
            required: Some(vec!["plan".to_string()]),
            additional_properties: Some(false.into()),
        },
    })
});
```

**Key Pattern**: The tool schema embeds behavioral constraints ("at most one step can be in_progress") in the description. This is enforced by **model prompting**, not code validation.

### Handler Logic (`core/src/tools/handlers/plan.rs:62-117`)

```rust
#[async_trait]
impl ToolHandler for PlanHandler {
    fn kind(&self) -> ToolKind {
        ToolKind::Function
    }

    async fn handle(&self, invocation: ToolInvocation) -> Result<ToolOutput, FunctionCallError> {
        let ToolInvocation { session, turn, payload, .. } = invocation;

        // Extract function arguments
        let arguments = match payload {
            ToolPayload::Function { arguments } => arguments,
            _ => return Err(FunctionCallError::RespondToModel(
                "update_plan handler received unsupported payload".to_string(),
            )),
        };

        // Delegate to core handler
        let content = handle_update_plan(session.as_ref(), turn.as_ref(), arguments, call_id).await?;

        Ok(ToolOutput::Function {
            content,
            content_items: None,
            success: Some(true),
        })
    }
}

/// Core handler - intentionally minimal
/// "This function doesn't do anything useful. However, it gives the model
/// a structured way to record its plan that clients can read and render."
pub(crate) async fn handle_update_plan(
    session: &Session,
    turn_context: &TurnContext,
    arguments: String,
    _call_id: String,
) -> Result<String, FunctionCallError> {
    let args = parse_update_plan_arguments(&arguments)?;
    session.send_event(turn_context, EventMsg::PlanUpdate(args)).await;
    Ok("Plan updated".to_string())
}
```

**Design Philosophy**: The handler is explicitly documented as "not doing anything useful" - this is intentional. The value is in:
1. Forcing the model to articulate its plan in structured form
2. Emitting that structure as a typed event for clients to render
3. Returning a minimal acknowledgment to the model

---

## Event Flow & Persistence

### Event Emission (`core/src/codex.rs`)

```rust
pub(crate) async fn send_event(&self, turn_context: &TurnContext, msg: EventMsg) {
    let event = Event {
        id: turn_context.sub_id.clone(),
        msg
    };

    // Persist for session history/rollback
    self.persist_rollout_items(&[RolloutItem::EventMsg(...)]).await;

    // Send to all subscribed clients
    self.tx_event.send(event).await;
}
```

### Approval Bypass (`rollout/src/policy.rs:165`)

```rust
// PlanUpdate explicitly bypasses approval workflow
EventMsg::PlanUpdate(_) => false,  // No approval required
```

**Why**: Plans are informational-only. They don't modify files or execute commands, so approval gates would add friction without safety benefit.

---

## TUI Rendering

### History Cell Implementation (`tui/src/history_cell.rs:2433+`)

```rust
pub(crate) struct PlanUpdateCell {
    explanation: Option<String>,
    plan: Vec<PlanItemArg>,
}

impl HistoryCell for PlanUpdateCell {
    fn display_lines(&self, width: u16) -> Vec<Line<'static>> {
        let render_step = |status: &StepStatus, text: &str| -> Vec<Line<'static>> {
            let (box_str, step_style) = match status {
                StepStatus::Completed => ("✔ ", Style::default().crossed_out().dim()),
                StepStatus::InProgress => ("□ ", Style::default().cyan().bold()),
                StepStatus::Pending => ("□ ", Style::default().dim()),
            };
            // Text wrapping and formatting...
        };

        let mut lines: Vec<Line<'static>> = vec![];
        lines.push(vec!["• ".dim(), "Updated Plan".bold()].into());

        // Render optional explanation
        if let Some(expl) = &self.explanation {
            indented_lines.extend(render_note(expl));
        }

        // Render each step with status indicator
        for PlanItemArg { step, status } in self.plan.iter() {
            indented_lines.extend(render_step(status, step));
        }

        lines
    }
}
```

**Visual Output**:
```
• Updated Plan
  └ Setting up authentication flow
    ✔ Create user model
    □ Implement JWT middleware
    □ Add login endpoint
    □ Add refresh token logic
```

---

## Model Prompting Strategy

### From `core/prompt.md`

```markdown
## Planning

You have access to an `update_plan` tool which tracks steps and progress and
renders them to the user. Using the tool helps demonstrate that you've understood
the task and convey how you're approaching it.

**Use a plan when:**
- The task is non-trivial and will require multiple actions over a long time horizon
- There are logical phases or dependencies where sequencing matters
- The work has ambiguity that benefits from outlining high-level goals
- You want intermediate checkpoints for feedback and validation
- When the user asked you to do more than one thing in a single prompt
- The user has asked you to use the plan tool (aka "TODOs")

**Do NOT use plans for:**
- Simple or single-step queries you can answer immediately
- Padding out work with filler steps or stating the obvious
- Things you aren't actually capable of doing

**Plan Guidelines:**
- Call `update_plan` with a short list of 1-sentence steps (no more than 5-7 words each)
- Each step has a `status`: `pending`, `in_progress`, or `completed`
- There should always be exactly one `in_progress` step until everything is done
- When steps complete, mark them `completed` and move to the next
- Do not repeat the full plan contents after an update - the harness displays it
```

---

## Test Coverage

### Positive Case (`core/tests/suite/tool_harness.rs:164-186`)

```rust
#[tokio::test]
async fn update_plan_tool_works() {
    // ... setup ...

    wait_for_event(&codex, |event| match event {
        EventMsg::PlanUpdate(update) => {
            saw_plan_update = true;
            assert_eq!(update.explanation.as_deref(), Some("Tool harness check"));
            assert_eq!(update.plan.len(), 2);
            assert_eq!(update.plan[0].step, "Inspect workspace");
            assert_matches!(update.plan[0].status, StepStatus::InProgress);
            assert_eq!(update.plan[1].step, "Report results");
            assert_matches!(update.plan[1].status, StepStatus::Pending);
            false
        }
        _ => false,
    }).await;

    let (output_text, _) = call_output(&req, call_id);
    assert_eq!(output_text, "Plan updated");
}
```

### Negative Case - Malformed Payload

```rust
#[tokio::test]
async fn update_plan_tool_rejects_malformed_payload() {
    let invalid_args = json!({ "explanation": "Missing plan data" }).to_string();

    // ... submit invalid payload ...

    assert!(!saw_plan_update, "did not expect PlanUpdate event for malformed payload");
    assert!(output_text.contains("failed to parse function arguments"));
}
```

---

## Inspirational Patterns for Your Own Implementation

### 1. **Structured Tool for Unstructured Information**

**Pattern**: When you want the AI to communicate something (plans, thoughts, analysis), don't rely on parsing prose. Create a tool with a defined schema.

```typescript
// Instead of parsing "I'll do A then B then C"
interface TaskPlan {
  explanation?: string;
  steps: Array<{
    description: string;
    status: 'pending' | 'in_progress' | 'completed';
    dependencies?: string[];
  }>;
}
```

### 2. **Event-Driven State Communication**

**Pattern**: Decouple the "recording" of state from the "rendering" of state.

```
Model → Tool Handler → Event Bus → Multiple Consumers
                                   ├── TUI (visual)
                                   ├── SDK (programmatic)
                                   └── Logs (persistence)
```

### 3. **Prompt-Based Constraints**

**Pattern**: For behavioral rules that don't affect correctness, embed them in tool descriptions rather than validation code.

```rust
description: "At most one step can be in_progress at a time."
// Not validated in code - model follows this from training
```

**Why**: This keeps the code simple and allows flexibility. If the model occasionally violates the rule, nothing breaks - the display just shows multiple in-progress items.

### 4. **Intentionally Minimal Return Values**

**Pattern**: When a tool exists for its inputs, not its outputs, return the simplest possible acknowledgment.

```rust
Ok("Plan updated".to_string())  // Just confirm receipt
```

This prevents the model from over-interpreting tool responses and keeps token usage low.

### 5. **Layered Type Generation**

**Pattern**: Define types once, generate for multiple targets.

```rust
#[derive(Serialize, Deserialize, JsonSchema, TS)]
pub struct UpdatePlanArgs { ... }
```

- `Serialize/Deserialize`: Rust serialization
- `JsonSchema`: JSON Schema for tool definition
- `TS`: TypeScript types for SDK clients

### 6. **Approval-Free Informational Tools**

**Pattern**: Categorize tools by their side effects. Pure-informational tools should bypass approval workflows.

```rust
fn requires_approval(event: &EventMsg) -> bool {
    match event {
        EventMsg::PlanUpdate(_) => false,  // No side effects
        EventMsg::ExecCommand(_) => true,  // Has side effects
        // ...
    }
}
```

### 7. **Visual Status Encoding**

**Pattern**: Use consistent visual language for status states.

```
✔ Completed  (crossed-out, dim)     - Done, de-emphasized
□ In Progress (cyan, bold)          - Active focus
□ Pending     (dim)                 - Future, not yet relevant
```

---

## Integration Points Reference

| Component | File | Purpose |
|-----------|------|---------|
| Protocol Types | `protocol/src/plan_tool.rs` | Serialization structures |
| Event Variant | `protocol/src/protocol.rs:1374` | `EventMsg::PlanUpdate` |
| Tool Handler | `core/src/tools/handlers/plan.rs` | Invocation processing |
| Tool Registration | `core/src/tools/spec.rs` | Standard tool set |
| Event Emission | `core/src/codex.rs` | `send_event()` |
| Approval Policy | `rollout/src/policy.rs:165` | Bypass for plans |
| TUI Handler | `tui/src/chatwidget.rs` | `on_plan_update()` |
| TUI Renderer | `tui/src/history_cell.rs:2433` | Visual display |
| MCP Filter | `mcp-server/src/codex_tool_runner.rs` | Event filtering |
| Exec Output | `exec/src/event_processor_with_jsonl_output.rs` | JSON output |

---

## Summary: What Makes This Design Effective

1. **Separation of Concerns**: Model communicates intent → System emits structured event → Clients render appropriately

2. **Minimal Coupling**: The tool handler doesn't know or care how plans are displayed

3. **Type Safety End-to-End**: Same types used in Rust core, JSON protocol, and TypeScript SDK

4. **Graceful Degradation**: If plan parsing fails, error returns to model for correction; system continues

5. **No Blocking Operations**: Plan updates are fire-and-forget with async event emission

6. **Persistence Ready**: Events are persisted for session history, enabling rollback/resume

7. **Multi-Client Support**: Same event consumed by TUI, MCP server, exec mode, and SDK clients

---

## Potential Enhancements to Consider

1. **Plan Diffing**: Show what changed between plan updates (added/removed/reordered steps)

2. **Time Tracking**: Record timestamps for status transitions to measure actual vs. expected duration

3. **Dependency Graph**: Extend `PlanItemArg` with `depends_on: Vec<usize>` for complex workflows

4. **Plan Templates**: Pre-defined plan structures for common operations (refactoring, migration, etc.)

5. **Progress Metrics**: Emit metrics events for plan progress (steps completed, estimated remaining)

6. **Plan Persistence**: Store plans separately from conversation for cross-session task continuity

---

*Document generated from deep code review of codex-rs planning tool implementation.*
*Last updated: March 2026*
