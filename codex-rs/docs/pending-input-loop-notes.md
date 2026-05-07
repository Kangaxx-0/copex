# Pending Input Loop

The Codex agent accepts user instructions, calls tools (like a shell), and streams its replies.  
Sometimes a user thinks of extra guidance while the agent is still working. Rather than restarting the
conversation, Codex keeps the current task alive and feeds the new guidance into the same turn.  
This pattern is the **pending input loop**.

## Mental Model
Imagine a conversation between a user and an assistant. The assistant is in the middle of running a command when the user realizes a flag is missing. The pending input loop lets the assistant keep going, finish the command, then immediately incorporate the user’s update when it talks to the model again. No work is lost, and the conversation stays coherent.

## Why Codex Uses a Loop
- **One turn = one task**: Tool runs, streaming output, and token accounting all belong to that single turn. If we tore everything down for each follow-up message, we would lose command state, diffs, and metrics.
- **Immediate course correction**: Additional instructions, approvals, or interrupts arrive while the current action is still running. The loop lets the agent enqueue those signals and respond on the next iteration.
- **Consistent history**: The same turn maintains the conversation archive. Review threads even keep their own self-contained history so they remain isolated from the main timeline.

## How It Works (Step by Step)
1. The task starts with the user’s initial message. Codex records it and informs the UI that the turn began.
2. While the model works, new UI actions (extra text, approvals, “stop”, etc.) are buffered in the turn state (`TurnState.pending_input`).
3. Each time the model finishes a response, `run_task` pulls any buffered items through `Session::get_pending_input`, appends them to the turn history, and sends the combined prompt back to the model.
4. The cycle repeats until the model produces a final assistant message or hits an error. At that point, Codex emits a `TaskComplete` event with the last assistant text.

## Benefits You Can Reuse Elsewhere
- **User experience**: Conversations feel fluid; people can nudge the system midstream without restarting it.
- **Tool stability**: Long-running tool invocations finish naturally, preventing the partial or duplicated work that task restarts can cause.
- **Clear audit trail**: A single unified log captures everything that happened in the turn, making debugging and compliance audits easier.

## Applying the Idea Beyond Codex
Any AI system that mixes automated steps with interactive feedback can benefit from this loop:
- Keep a per-task buffer for late-arriving input.
- Rebuild the model prompt or tool command after each step, inserting the buffered items.
- Only close the task when the agent produces a final response, so the history remains a continuous story.

By treating a task as a live conversation with room for mid-course updates, you get more resilient automations and a calmer user experience.
