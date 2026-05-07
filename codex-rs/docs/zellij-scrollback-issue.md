# Zellij Scrollback Incompatibility — Root Cause Analysis

## The Problem

When running Codex CLI inside Zellij, conversation history is truncated and inaccessible. Users cannot scroll back to review earlier parts of the session using Zellij's native scrollback (mouse wheel, Esc-Esc scroll mode, or keyboard navigation). The only way to access full history is through Codex's built-in transcript pager (Ctrl+T).

This is not a Zellij bug. It is a fundamental architectural mismatch between how Codex renders its TUI and how terminal multiplexers build scrollback buffers.

## Background: How Terminal Scrollback Works

A terminal emulator maintains two buffers:

- **Main buffer** — the normal scrollback. Text written with newlines (`\n`) flows into this buffer. When content scrolls off the top of the visible area, it enters the scrollback history. Terminal multiplexers (Zellij, tmux) can capture and expose this history.
- **Alternate screen buffer** — an isolated full-screen viewport activated by `ESC[?1049h`. Per the xterm specification, alternate screen has no scrollback. It exists for full-screen applications (vim, less, htop) that manage their own viewport. When the application exits alternate screen, the original main buffer is restored.

Zellij follows the xterm specification strictly. [Zellij PR #1032](https://github.com/zellij-org/zellij/pull/1032) explicitly disabled scrollback in alternate screen mode because the spec says it should not exist there.

## Layer 1: Alternate Screen Buffer

Codex's TUI enters alternate screen by default (`codex-rs/tui/src/tui.rs:411`):

```rust
let _ = execute!(self.terminal.backend_mut(), EnterAlternateScreen);
```

Once in alternate screen, Zellij creates an isolated viewport. No scrollback accumulates. The user can only see what fits on the current screen.

The codebase already mitigates this. Zellij is auto-detected via environment variables (`ZELLIJ`, `ZELLIJ_SESSION_NAME`, `ZELLIJ_VERSION`) in `codex-rs/terminal-detection/src/lib.rs:378-384`, and alternate screen is disabled when running inside Zellij (`codex-rs/tui/src/lib.rs:1548-1551`):

```rust
AltScreenMode::Auto => {
    let terminal_info = terminal_info();
    !matches!(terminal_info.multiplexer, Some(Multiplexer::Zellij { .. }))
}
```

This is also available manually via `--no-alt-screen` or the `tui.alternate_screen = "never"` config.

**However, disabling alternate screen does not fix the problem.** The scrollback remains broken due to Layer 2.

## Layer 2: DECSTBM Scroll Region Manipulation

When completed messages are pushed into the terminal's scrollback, Codex uses `insert_history_lines()` (`codex-rs/tui/src/insert_history.rs:34-181`). This function inserts styled text above the active viewport by manipulating ANSI scroll regions:

1. **DECSTBM** (`ESC[top;bottom r`) restricts the scrollable region to a subset of rows
2. **Reverse Index** (`ESC M`) scrolls content within that region downward, making room for new lines
3. The new content is written into the freed space
4. The scroll region is reset

```rust
// Set scroll region to [viewport_top .. screen_height]
queue!(writer, SetScrollRegion(top_1based..screen_size.height))?;
queue!(writer, MoveTo(0, area.top()))?;
for _ in 0..scroll_amount {
    queue!(writer, Print("\x1bM"))?;  // Reverse Index
}
queue!(writer, ResetScrollRegion)?;

// Set scroll region to [screen_top .. viewport_top] for writing
queue!(writer, SetScrollRegion(1..area.top()))?;
```

Terminal multiplexers cannot build scrollback from DECSTBM-manipulated content. Zellij sees "content rearranged in-place within a restricted region" — not "new content appended to the output stream." The scroll region manipulation is invisible to Zellij's scrollback tracking because it happens within the pane's existing viewport, not at the natural scroll boundary.

## Layer 3: Cursor-Addressed Viewport Rendering

The active viewport area (current prompt, streaming response, status bar) is rendered using ratatui's diff-based buffer system (`codex-rs/tui/src/custom_terminal.rs:508-630`). Each frame:

1. The previous and current frame buffers are diffed
2. Only changed cells are emitted
3. Each cell is drawn via `MoveTo(x, y)` followed by `Print(character)`

```rust
// Move cursor to specific coordinates, then write
if !matches!(last_pos, Some(p) if x == p.x + 1 && y == p.y) {
    queue!(writer, MoveTo(x, y))?;
}
queue!(writer, Print(cell.symbol()))?;
```

This is standard for any TUI framework (ratatui, ncurses, etc.) — it draws characters at absolute screen positions rather than appending text sequentially. Terminal multiplexers see fixed-position overwrites, not a growing stream of text. The viewport area never enters scrollback because nothing is "scrolling" in the terminal's sense — cells are being painted at coordinates.

Additionally, viewport management uses `CSI J` (erase in display) and `CSI K` (erase in line) commands during resize and viewport adjustments (`codex-rs/tui/src/tui.rs:477,494`), which can clear content that would otherwise be in scrollback.

## Why the Transcript Pager (Ctrl+T) Works

The transcript pager (`codex-rs/tui/src/pager_overlay.rs`) works in Zellij because it sidesteps the entire problem. It renders the full conversation history as a scrollable view *within Codex's own viewport*. Scrolling is handled internally by Codex (arrow keys, PgUp/PgDn), not by the terminal multiplexer. The terminal only sees the currently visible page of the transcript — Zellij's scrollback is irrelevant.

## Summary

The incompatibility has three compounding layers:

| Layer | Mechanism | Effect in Zellij |
|---|---|---|
| Alternate screen buffer | `ESC[?1049h` | Isolated viewport, no scrollback at all |
| DECSTBM scroll regions | `ESC[top;bottom r` + `ESC M` | History insertion invisible to scrollback tracking |
| Cursor-addressed rendering | `MoveTo(x,y)` + `Print(char)` | Live viewport never enters scrollback |

Disabling alternate screen (which the codebase already does automatically for Zellij) removes Layer 1 but leaves Layers 2 and 3 intact. The result is that even with `--no-alt-screen`, completed conversation history does not appear in Zellij's native scrollback.

The root cause is architectural: Codex's TUI is a full-screen application that manages its own viewport, while Zellij's scrollback relies on the sequential, append-only text stream that characterizes traditional terminal output.

## Related Issues

- [openai/codex#2558](https://github.com/openai/codex/issues/2558) — canonical tracker, output truncated in Zellij
- [openai/codex#9115](https://github.com/openai/codex/issues/9115) — Zellij incompatibility persists after `--no-alt-screen` fix
- [openai/codex#10331](https://github.com/openai/codex/issues/10331) — detailed diagnosis of full-screen redraw in main buffer
- [openai/codex#2836](https://github.com/openai/codex/issues/2836) — mouse/scroll broken, references Zellij author's xterm spec explanation
- [openai/codex#8352](https://github.com/openai/codex/issues/8352) — related tmux zoom issue with same root cause
- [zellij-org/zellij#1032](https://github.com/zellij-org/zellij/pull/1032) — Zellij's intentional disabling of alternate screen scrollback
