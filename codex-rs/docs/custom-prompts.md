# Custom Prompts Module

> **DRAFT**: The module `core/src/custom_prompts.rs` and `codex_protocol::custom_prompts` referenced below do not yet exist in the codebase. This document describes planned functionality.

This document describes how `core/src/custom_prompts.rs` would discover user-authored prompts, enrich them with metadata, and expose them to other Codex components.

## Overview

- `default_prompts_dir` returns `$CODEX_HOME/prompts`, or `None` when the home directory cannot be resolved. The value is derived from `crate::config::find_codex_home`, keeping the lookup consistent with other filesystem helpers.
- `discover_prompts_in` and `discover_prompts_in_excluding` asynchronously scan a directory for Markdown prompts, producing a sorted `Vec<CustomPrompt>`. The results use the shared protocol struct from `codex_protocol::custom_prompts`.
- `parse_frontmatter` strips optional YAML-style metadata from the top of each file, returning `(description, argument_hint, body)` for higher layers to consume.

The core module only depends on Tokio’s async filesystem API and the protocol crate, so it can be reused by both server and UI layers without pulling UI-specific code.

## Discovery Flow

1. `discover_prompts_in` delegates to the excluding variant with an empty `HashSet`, keeping call sites concise when no built-in filtering is needed.
2. `discover_prompts_in_excluding` iterates `tokio::fs::read_dir` results. It keeps only regular files whose extension is `.md`, ignoring subdirectories and other formats.
3. Filenames are converted to command names via `Path::file_stem()`; non-UTF-8 names are dropped gracefully. The exclusion set allows core to hide built-in prompts while still returning user prompts.
4. File contents are read as UTF-8. Any read or decoding failure causes that file to be skipped instead of aborting the scan, matching expectations for user-managed directories.
5. Each prompt is pushed as a `CustomPrompt` with its path, content, and any parsed metadata. Results are sorted lexicographically by name before being returned, ensuring stable UI ordering regardless of filesystem state.

## Front Matter Contract

`parse_frontmatter` looks for a leading `---` fence and consumes YAML-like key/value lines until it encounters a closing `---`. Supported keys are:

- `description`: shown in the slash-command popup.
- `argument-hint` or `argument_hint`: displayed after the description when prompting for arguments.

Values may be wrapped in single or double quotes; both quoting styles are stripped. Lines that are blank or start with `#` are treated as comments. If the closing fence is missing, the routine returns the original content unchanged so a malformed header never hides the prompt body.

```markdown
---
description: "Summarize the selected file"
argument-hint: "[path] [tone]"
---
Please summarize the file at $1 using a ${TONE} tone.
```

## Integration Points

- **Session Service**: When the core session handles `Op::ListCustomPrompts`, it looks up the default directory and publishes a `ListCustomPromptsResponseEvent` containing the discovered prompts. This ensures subscribers receive both built-ins (supplied via exclusions) and user prompts through the same channel.
- **TUI Command Palette**: The TUI renders custom prompts alongside built-in slash commands. Each prompt appears as `/{PROMPTS_CMD_PREFIX}:{name}`, and its description and argument hint populate the command popup. Because discovery produces the same `CustomPrompt` struct used by protocol code, the UI does not need a translation layer.
- **Prompt Invocation**: After selection, custom commands are dispatched through the same execution path as built-ins. The only distinction is the `prompts:` namespace, which prevents name collisions while keeping the interaction model identical.

## Behavior and Edge Cases

- Missing directories, empty folders, and unreadable entries all yield an empty results list, letting callers assume “no prompts” without handling errors.
- Non-Markdown files, symbolic links, and files with invalid UTF-8 content are silently skipped.
- Tests cover missing directories, sorting, exclusions, invalid Unicode, front-matter parsing, and CRLF preservation, preventing regressions in the discovery pipeline.

## Authoring Guidelines

- Place custom prompts in `$CODEX_HOME/prompts` with unique `.md` filenames; the stem becomes the slash command suffix.
- Provide concise `description` and `argument-hint` front-matter fields to improve discoverability in the command popup. Close the front-matter fence (`---`) to avoid having the body treated as metadata.
- Use `$1`, `$ARGUMENTS`, or other accepted placeholders to reference user-supplied arguments; higher layers perform the substitution before execution.
- Organize prompts flat within the directory. Subdirectories are ignored by the discovery routine.

## Extensibility Notes

- Built-in prompts can be layered on top of user prompts by passing an exclusion set to `discover_prompts_in_excluding`. This avoids duplicate names while still allowing overrides.
- Additional metadata keys can be introduced by expanding `parse_frontmatter`. Downstream code already ignores unknown keys, so extending the schema is backward compatible.
- If the UI needs freshness or categorization metadata, consider augmenting the `CustomPrompt` struct in `protocol/src/custom_prompts.rs` and updating this module to populate the new fields.
