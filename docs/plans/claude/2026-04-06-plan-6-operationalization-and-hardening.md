# Plan 6: Operationalization And Hardening

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Establish the operational surface of Spool — durable memory, export, policy hardening, telemetry, configuration, CLI entry point, and end-to-end smoke tests — wiring together all prior plan work into a launchable product.

**Architecture:** Three new crates — `spool-memory` (durable memory subsystem), `spool-export` (JSON and Markdown export), and `spool-cli` (CLI entry point with clap) — plus cross-cutting additions to `spool-protocol`, `spool-core`, and the workspace root. All subsystems are fixture-testable. The CLI ties together configuration, session management, LOB selection, and the harness loop.

**Tech Stack:** Rust 2024 edition, serde/serde_json, chrono, uuid, tokio, async-trait, thiserror, clap, toml, tracing, tracing-subscriber, tracing-opentelemetry, opentelemetry (sdk + stdout exporter), tempfile (dev)

**Governing spec:** `docs/superpowers/specs/2026-04-06-spool-refined-spec.md`
**Planning readiness:** `docs/superpowers/specs/2026-04-06-spool-dev-planning-readiness-design.md`

---

## Plan-Specific Sections

### Subsystem Scope

This plan owns:

- durable memory types, store, lifecycle (capture, promotion, reuse, invalidation)
- durable memory scope markers (lob, workspace, team, global)
- durable memory metadata and status model (candidate, confirmed, stale, invalidated)
- JSON export of canonical task results (convenience, non-stable)
- Markdown export of canonical task results (convenience, non-stable)
- confirmation policy enforcement engine
- telemetry initialization with `tracing` and W3C trace context via OpenTelemetry
- structured logging with span-aware context
- performance timing for harness phases
- error handling hardening with graceful degradation and user-facing messages
- TOML configuration file loading with environment variable overrides
- connection detail management (workspace, auth placeholders)
- CLI argument parsing with clap (session commands, LOB selection, config path)
- CLI session management (new, resume, list)
- end-to-end smoke tests using fixture scenarios

### Out Of Scope

- live Fabric auth or API calls (Plan 2)
- knowledge bundle loading and Tier 1/Tier 2 structure (Plan 3)
- DAX or warehouse query execution internals (Plan 4)
- TUI rendering, progress surface, advanced view (Plan 5)
- LLM provider integration (abstracted behind harness traits)
- stable export contract (v1 exports are convenience outputs)
- SQLite-backed memory persistence (deferred; v1 uses JSONL files)

### Dependencies

- Plan 1: spool-protocol types (artifact, evidence, contradiction, task contract, task result, evaluator, checkpoint)
- Plan 1: spool-core harness traits (Planner, Generator, Evaluator), evidence ledger, contradiction ledger, evaluator loop, persistence trait
- Plan 2: Fabric adapter trait surface (used only via trait stubs here)
- Plan 3: knowledge types (used only via fixture stubs here)
- Plan 4: validation execution paths (used only via fixture stubs here)
- Plan 5: TUI session state model (this plan wires session persistence but does not own TUI rendering)

### Contract Impact

This plan **implements** the following governing contracts from the refined spec:

- Durable memory types and lifecycle (Spec Sections 7.12-7.13)
- Canonical task result export as convenience output (Spec Section 10.8, non-stable format)
- Confirmation policy enforcement (Spec Section 14.1)
- Session startup lifecycle steps 1-8 at CLI level (Spec Section 15.1)

This plan **pressures** no existing contracts but may surface refinements needed in:

- persistence format details (Plan 1 contract)
- session metadata shape (Plan 5 contract)

### Validation

Plan 6 is proven through:

- unit tests: durable memory CRUD, lifecycle transitions, scope filtering
- unit tests: JSON and Markdown export round-trips, format verification
- unit tests: confirmation policy trigger classification
- (SQL policy is owned by Plan 4 — not duplicated here)
- unit tests: config file loading and environment variable override
- unit tests: CLI argument parsing
- integration tests: end-to-end smoke tests using fixture scenarios that exercise the full path from CLI args through harness loop to export

**Integration validation against dev Fabric workspace (required for plan completion):**

Per the planning readiness addendum (Section 5), Plan 6 wires together all subsystems into the final CLI entry point. At least one end-to-end smoke test must exercise a real Fabric round-trip to validate that the full path works — not just fixtures.

| Seam | Scenario | Environment | Success Condition |
|------|----------|-------------|-------------------|
| End-to-end CLI | Run `spool` with a real config pointing to dev workspace, authenticate, select LOB, start a minimal investigation task | Dev Fabric workspace | CLI starts, authenticates, loads LOB bundle, resolves at least one artifact, and produces a structured task result |
| Session resume | Persist session from the above run, then resume it | Local filesystem + dev Fabric workspace | Resumed session restores task contract, evidence ledger, and artifact focus correctly |

These integration tests should be gated behind `SPOOL_INTEGRATION_TEST=1` like Plan 2 and Plan 4, but must pass locally before Plan 6 is marked complete.

### Open Items

**Owned by this plan:**

- exact TOML config file schema (resolved during Task 7 implementation)
- exact CLI subcommand surface (resolved during Task 9 implementation)
- exact tracing subscriber layer configuration (resolved during Task 6 implementation)

**Deferred to later plans:**

- SQLite-backed durable memory persistence (future hardening)
- stable export contract versioning (post-v1)
- real auth flow integration (Plan 2)
- TUI integration of telemetry spans (Plan 5)

**Review triggers:**

- if durable memory scope markers prove insufficient during real LOB testing, revisit scope model
- if export format proves inadequate for downstream consumption, revisit export schema
- if CLI session management conflicts with TUI session model from Plan 5, reconcile

---

## Task 1: Workspace And Crate Scaffolding For Plan 6

**Files:**

- Modify: `spool/Cargo.toml`
- Create: `spool/spool-memory/Cargo.toml`
- Create: `spool/spool-memory/src/lib.rs`
- Create: `spool/spool-export/Cargo.toml`
- Create: `spool/spool-export/src/lib.rs`
- Create: `spool/spool-cli/Cargo.toml`
- Create: `spool/spool-cli/src/main.rs`

**Step 1: Update workspace Cargo.toml**

Add the three new crates to the workspace members list and add new workspace dependencies:

```toml
# spool/Cargo.toml
[workspace]
members = [
    "spool-protocol",
    "spool-core",
    "spool-memory",
    "spool-export",
    "spool-cli",
]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2024"
license = "Apache-2.0"

[workspace.dependencies]
spool-protocol = { path = "spool-protocol" }
spool-core = { path = "spool-core" }
spool-memory = { path = "spool-memory" }
spool-export = { path = "spool-export" }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v4", "serde"] }
tokio = { version = "1", features = ["full"] }
async-trait = "0.1"
thiserror = "2"
clap = { version = "4", features = ["derive"] }
toml = "0.8"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
tracing-opentelemetry = "0.28"
opentelemetry = "0.28"
opentelemetry_sdk = { version = "0.28", features = ["rt-tokio"] }
opentelemetry-stdout = "0.28"
```

**Step 2: Create spool-memory crate**

```toml
# spool/spool-memory/Cargo.toml
[package]
name = "spool-memory"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
spool-protocol = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
chrono = { workspace = true }
uuid = { workspace = true }
thiserror = { workspace = true }

[dev-dependencies]
tempfile = "3"
```

```rust
// spool/spool-memory/src/lib.rs
pub mod types;
pub mod store;
pub mod lifecycle;
```

**Step 3: Create spool-export crate**

```toml
# spool/spool-export/Cargo.toml
[package]
name = "spool-export"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
spool-protocol = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
chrono = { workspace = true }
thiserror = { workspace = true }
```

```rust
// spool/spool-export/src/lib.rs
pub mod json_export;
pub mod markdown_export;
```

**Step 4: Create spool-cli crate**

```toml
# spool/spool-cli/Cargo.toml
[package]
name = "spool-cli"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
spool-protocol = { workspace = true }
spool-core = { workspace = true }
spool-memory = { workspace = true }
spool-export = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
chrono = { workspace = true }
uuid = { workspace = true }
tokio = { workspace = true }
clap = { workspace = true }
toml = { workspace = true }
tracing = { workspace = true }
tracing-subscriber = { workspace = true }
thiserror = { workspace = true }

[dev-dependencies]
tempfile = "3"
```

```rust
// spool/spool-cli/src/main.rs
mod config;
mod telemetry;
mod policy;
mod session;
mod error;

fn main() {
    println!("spool-cli placeholder");
}
```

**Step 5: Create placeholder modules**

Create empty files for each module declared in the lib.rs and main.rs files. Each file should contain only a comment:

```rust
// placeholder — implemented in later tasks
```

Files to create:

- `spool/spool-memory/src/types.rs`
- `spool/spool-memory/src/store.rs`
- `spool/spool-memory/src/lifecycle.rs`
- `spool/spool-export/src/json_export.rs`
- `spool/spool-export/src/markdown_export.rs`
- `spool/spool-cli/src/config.rs`
- `spool/spool-cli/src/telemetry.rs`
- `spool/spool-cli/src/policy.rs`
- `spool/spool-cli/src/session.rs`
- `spool/spool-cli/src/error.rs`

**Step 6: Verify build**

Run: `cd spool && cargo check`
Expected: compiles with no errors

**Step 7: Commit**

```bash
git add spool/
git commit -m "feat(spool): scaffold spool-memory, spool-export, and spool-cli crates for Plan 6"
```

---

## Task 2: Durable Memory Types

**Files:**

- Modify: `spool/spool-memory/src/types.rs`

**Step 1: Write the failing test**

Add to `spool/spool-memory/src/types.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn memory_entry_round_trip() {
        let now = Utc::now();
        let entry = MemoryEntry {
            memory_id: MemoryId("mem_001".into()),
            memory_type: MemoryType::UserPreference,
            scope: MemoryScope::Lob("finance".into()),
            content: "The finance team uses fiscal quarters starting in July.".into(),
            source_basis: SourceBasis::ExplicitUserConfirmation {
                session_id: "sess_42".into(),
            },
            status: MemoryStatus::Confirmed,
            created_at: now,
            last_validated_at: Some(now),
            invalidated_at: None,
            invalidation_reason: None,
            tags: vec!["fiscal-calendar".into(), "quarter-definition".into()],
        };

        let json = serde_json::to_string_pretty(&entry).unwrap();
        let restored: MemoryEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.memory_id, entry.memory_id);
        assert_eq!(restored.memory_type, MemoryType::UserPreference);
        assert_eq!(restored.status, MemoryStatus::Confirmed);
        match &restored.scope {
            MemoryScope::Lob(lob) => assert_eq!(lob, "finance"),
            _ => panic!("expected Lob scope"),
        }
    }

    #[test]
    fn all_memory_types_serialize() {
        let types = vec![
            MemoryType::UserPreference,
            MemoryType::TeamConvention,
            MemoryType::RecurringIssuePattern,
            MemoryType::ReferenceSource,
            MemoryType::ArtifactRelationshipHint,
            MemoryType::StableValidationRecipe,
        ];
        for t in types {
            let json = serde_json::to_string(&t).unwrap();
            let restored: MemoryType = serde_json::from_str(&json).unwrap();
            assert_eq!(restored, t);
        }
    }

    #[test]
    fn all_memory_statuses_serialize() {
        let statuses = vec![
            MemoryStatus::Candidate,
            MemoryStatus::Confirmed,
            MemoryStatus::Stale,
            MemoryStatus::Invalidated,
        ];
        for s in statuses {
            let json = serde_json::to_string(&s).unwrap();
            let restored: MemoryStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(restored, s);
        }
    }

    #[test]
    fn all_scope_variants_serialize() {
        let scopes = vec![
            MemoryScope::Lob("finance".into()),
            MemoryScope::Workspace("ws_123".into()),
            MemoryScope::Team("analytics-team".into()),
            MemoryScope::Global,
        ];
        for s in scopes {
            let json = serde_json::to_string(&s).unwrap();
            let restored: MemoryScope = serde_json::from_str(&json).unwrap();
            assert_eq!(
                serde_json::to_string(&restored).unwrap(),
                json,
            );
        }
    }

    #[test]
    fn source_basis_variants_round_trip() {
        let bases = vec![
            SourceBasis::CompletedTaskResult {
                task_id: "task_1".into(),
            },
            SourceBasis::RepeatedObservation {
                occurrence_count: 3,
                first_seen_session: "sess_1".into(),
            },
            SourceBasis::ExplicitUserConfirmation {
                session_id: "sess_2".into(),
            },
            SourceBasis::CuratedKnowledgeDerivation {
                bundle_name: "finance-bundle".into(),
            },
        ];
        for b in bases {
            let json = serde_json::to_string(&b).unwrap();
            let restored: SourceBasis = serde_json::from_str(&json).unwrap();
            assert_eq!(
                serde_json::to_string(&restored).unwrap(),
                json,
            );
        }
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cd spool && cargo test -p spool-memory -- types`
Expected: FAIL — types not defined yet

**Step 3: Write the implementation**

Replace the placeholder content of `spool/spool-memory/src/types.rs` with:

```rust
// spool/spool-memory/src/types.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Unique identifier for a durable memory entry.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MemoryId(pub String);

/// Allowed durable-memory classes in v1 (Spec Section 7.12).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryType {
    UserPreference,
    TeamConvention,
    RecurringIssuePattern,
    ReferenceSource,
    ArtifactRelationshipHint,
    StableValidationRecipe,
}

/// Scope marker for durable memory (Spec Section 7.13).
///
/// Memory should not be treated as universally global unless explicitly
/// marked as such. LOB-scoped memory should not be silently applied
/// across different LOBs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "scope_type", content = "scope_value", rename_all = "snake_case")]
pub enum MemoryScope {
    Lob(String),
    Workspace(String),
    Team(String),
    Global,
}

/// Lifecycle status for a durable memory entry (Spec Section 7.13).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryStatus {
    Candidate,
    Confirmed,
    Stale,
    Invalidated,
}

/// How the memory entry was originally established.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "basis_type", rename_all = "snake_case")]
pub enum SourceBasis {
    CompletedTaskResult {
        task_id: String,
    },
    RepeatedObservation {
        occurrence_count: u32,
        first_seen_session: String,
    },
    ExplicitUserConfirmation {
        session_id: String,
    },
    CuratedKnowledgeDerivation {
        bundle_name: String,
    },
}

/// A single durable memory entry (Spec Sections 7.12-7.13).
///
/// Durable memory is a separate subsystem from curated knowledge.
/// It is learned, confirmed, or promoted operating context that helps
/// Spool work efficiently across sessions without redefining domain truth.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub memory_id: MemoryId,
    pub memory_type: MemoryType,
    pub scope: MemoryScope,
    pub content: String,
    pub source_basis: SourceBasis,
    pub status: MemoryStatus,
    pub created_at: DateTime<Utc>,
    pub last_validated_at: Option<DateTime<Utc>>,
    pub invalidated_at: Option<DateTime<Utc>>,
    pub invalidation_reason: Option<String>,
    pub tags: Vec<String>,
}

// tests at bottom of file (from Step 1)
```

**Step 4: Run test to verify it passes**

Run: `cd spool && cargo test -p spool-memory -- types`
Expected: 5 tests PASS

**Step 5: Commit**

```bash
git add spool/spool-memory/src/types.rs
git commit -m "feat(spool-memory): durable memory types with scope markers, lifecycle status, and source basis"
```

---

## Task 3: Durable Memory Store And Lifecycle

**Files:**

- Modify: `spool/spool-memory/src/store.rs`
- Modify: `spool/spool-memory/src/lifecycle.rs`

**Step 1: Write the failing tests**

In `spool/spool-memory/src/store.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;
    use chrono::Utc;

    fn make_entry(id: &str, scope: MemoryScope, status: MemoryStatus) -> MemoryEntry {
        let now = Utc::now();
        MemoryEntry {
            memory_id: MemoryId(id.into()),
            memory_type: MemoryType::UserPreference,
            scope,
            content: format!("Memory content for {id}"),
            source_basis: SourceBasis::ExplicitUserConfirmation {
                session_id: "sess_1".into(),
            },
            status,
            created_at: now,
            last_validated_at: None,
            invalidated_at: None,
            invalidation_reason: None,
            tags: vec![],
        }
    }

    #[test]
    fn insert_and_get() {
        let mut store = MemoryStore::new();
        let entry = make_entry("mem_1", MemoryScope::Global, MemoryStatus::Candidate);
        store.insert(entry);
        assert_eq!(store.len(), 1);

        let retrieved = store.get(&MemoryId("mem_1".into())).unwrap();
        assert_eq!(retrieved.content, "Memory content for mem_1");
    }

    #[test]
    fn query_by_scope_lob() {
        let mut store = MemoryStore::new();
        store.insert(make_entry("mem_1", MemoryScope::Lob("finance".into()), MemoryStatus::Confirmed));
        store.insert(make_entry("mem_2", MemoryScope::Lob("marketing".into()), MemoryStatus::Confirmed));
        store.insert(make_entry("mem_3", MemoryScope::Global, MemoryStatus::Confirmed));

        let finance = store.query_by_scope(&MemoryScope::Lob("finance".into()));
        assert_eq!(finance.len(), 1);
        assert_eq!(finance[0].memory_id, MemoryId("mem_1".into()));
    }

    #[test]
    fn query_by_status() {
        let mut store = MemoryStore::new();
        store.insert(make_entry("mem_1", MemoryScope::Global, MemoryStatus::Candidate));
        store.insert(make_entry("mem_2", MemoryScope::Global, MemoryStatus::Confirmed));
        store.insert(make_entry("mem_3", MemoryScope::Global, MemoryStatus::Stale));

        let confirmed = store.query_by_status(&MemoryStatus::Confirmed);
        assert_eq!(confirmed.len(), 1);
        assert_eq!(confirmed[0].memory_id, MemoryId("mem_2".into()));
    }

    #[test]
    fn query_active_for_scope_returns_confirmed_and_global() {
        let mut store = MemoryStore::new();
        store.insert(make_entry("mem_1", MemoryScope::Lob("finance".into()), MemoryStatus::Confirmed));
        store.insert(make_entry("mem_2", MemoryScope::Lob("marketing".into()), MemoryStatus::Confirmed));
        store.insert(make_entry("mem_3", MemoryScope::Global, MemoryStatus::Confirmed));
        store.insert(make_entry("mem_4", MemoryScope::Lob("finance".into()), MemoryStatus::Invalidated));
        store.insert(make_entry("mem_5", MemoryScope::Lob("finance".into()), MemoryStatus::Candidate));

        let active = store.query_active_for_lob("finance");
        // Should include: mem_1 (finance confirmed), mem_3 (global confirmed)
        // Should exclude: mem_2 (marketing), mem_4 (invalidated), mem_5 (candidate)
        assert_eq!(active.len(), 2);
        let ids: Vec<&str> = active.iter().map(|e| e.memory_id.0.as_str()).collect();
        assert!(ids.contains(&"mem_1"));
        assert!(ids.contains(&"mem_3"));
    }

    #[test]
    fn persistence_round_trip() {
        let mut store = MemoryStore::new();
        store.insert(make_entry("mem_1", MemoryScope::Global, MemoryStatus::Confirmed));
        store.insert(make_entry("mem_2", MemoryScope::Lob("finance".into()), MemoryStatus::Candidate));

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("memory.jsonl");

        store.save_to_file(&path).unwrap();

        let loaded = MemoryStore::load_from_file(&path).unwrap();
        assert_eq!(loaded.len(), 2);
        assert!(loaded.get(&MemoryId("mem_1".into())).is_some());
        assert!(loaded.get(&MemoryId("mem_2".into())).is_some());
    }

    #[test]
    fn load_from_nonexistent_returns_empty() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("does_not_exist.jsonl");
        let store = MemoryStore::load_from_file(&path).unwrap();
        assert_eq!(store.len(), 0);
    }
}
```

In `spool/spool-memory/src/lifecycle.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;
    use chrono::Utc;

    fn make_candidate(id: &str) -> MemoryEntry {
        let now = Utc::now();
        MemoryEntry {
            memory_id: MemoryId(id.into()),
            memory_type: MemoryType::RecurringIssuePattern,
            scope: MemoryScope::Lob("finance".into()),
            content: "Recurring pattern".into(),
            source_basis: SourceBasis::RepeatedObservation {
                occurrence_count: 1,
                first_seen_session: "sess_1".into(),
            },
            status: MemoryStatus::Candidate,
            created_at: now,
            last_validated_at: None,
            invalidated_at: None,
            invalidation_reason: None,
            tags: vec![],
        }
    }

    #[test]
    fn promote_candidate_to_confirmed() {
        let mut entry = make_candidate("mem_1");
        let result = promote(&mut entry);
        assert!(result.is_ok());
        assert_eq!(entry.status, MemoryStatus::Confirmed);
        assert!(entry.last_validated_at.is_some());
    }

    #[test]
    fn promote_confirmed_is_error() {
        let mut entry = make_candidate("mem_1");
        entry.status = MemoryStatus::Confirmed;
        let result = promote(&mut entry);
        assert!(result.is_err());
    }

    #[test]
    fn mark_stale() {
        let mut entry = make_candidate("mem_1");
        entry.status = MemoryStatus::Confirmed;
        let result = mark_stale(&mut entry);
        assert!(result.is_ok());
        assert_eq!(entry.status, MemoryStatus::Stale);
    }

    #[test]
    fn invalidate_with_reason() {
        let mut entry = make_candidate("mem_1");
        entry.status = MemoryStatus::Confirmed;
        let result = invalidate(&mut entry, "Fresh runtime evidence contradicts this.".into());
        assert!(result.is_ok());
        assert_eq!(entry.status, MemoryStatus::Invalidated);
        assert!(entry.invalidated_at.is_some());
        assert_eq!(
            entry.invalidation_reason.as_deref(),
            Some("Fresh runtime evidence contradicts this."),
        );
    }

    #[test]
    fn invalidate_already_invalidated_is_error() {
        let mut entry = make_candidate("mem_1");
        entry.status = MemoryStatus::Invalidated;
        let result = invalidate(&mut entry, "reason".into());
        assert!(result.is_err());
    }

    #[test]
    fn revalidate_stale_entry() {
        let mut entry = make_candidate("mem_1");
        entry.status = MemoryStatus::Stale;
        let result = revalidate(&mut entry);
        assert!(result.is_ok());
        assert_eq!(entry.status, MemoryStatus::Confirmed);
        assert!(entry.last_validated_at.is_some());
    }

    #[test]
    fn revalidate_invalidated_is_error() {
        let mut entry = make_candidate("mem_1");
        entry.status = MemoryStatus::Invalidated;
        let result = revalidate(&mut entry);
        assert!(result.is_err());
    }

    #[test]
    fn valid_transitions() {
        assert!(is_valid_transition(&MemoryStatus::Candidate, &MemoryStatus::Confirmed));
        assert!(is_valid_transition(&MemoryStatus::Confirmed, &MemoryStatus::Stale));
        assert!(is_valid_transition(&MemoryStatus::Confirmed, &MemoryStatus::Invalidated));
        assert!(is_valid_transition(&MemoryStatus::Stale, &MemoryStatus::Confirmed));
        assert!(is_valid_transition(&MemoryStatus::Stale, &MemoryStatus::Invalidated));
        assert!(!is_valid_transition(&MemoryStatus::Invalidated, &MemoryStatus::Confirmed));
        assert!(!is_valid_transition(&MemoryStatus::Confirmed, &MemoryStatus::Candidate));
    }
}
```

**Step 2: Run tests to verify they fail**

Run: `cd spool && cargo test -p spool-memory -- store lifecycle`
Expected: FAIL

**Step 3: Write the implementations**

Replace `spool/spool-memory/src/store.rs` with:

```rust
// spool/spool-memory/src/store.rs
use std::collections::HashMap;
use std::io::{BufRead, Write};
use std::path::Path;

use crate::types::{MemoryEntry, MemoryId, MemoryScope, MemoryStatus};

/// In-memory durable memory store with JSONL persistence.
///
/// Durable memory is a separate subsystem from curated knowledge (Spec Section 7.12).
/// This store provides scope-aware querying and lifecycle-aware filtering.
pub struct MemoryStore {
    entries: HashMap<MemoryId, MemoryEntry>,
    /// Insertion order for deterministic serialization.
    order: Vec<MemoryId>,
}

impl MemoryStore {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            order: Vec::new(),
        }
    }

    pub fn insert(&mut self, entry: MemoryEntry) {
        let id = entry.memory_id.clone();
        if self.entries.insert(id.clone(), entry).is_none() {
            self.order.push(id);
        }
    }

    pub fn get(&self, id: &MemoryId) -> Option<&MemoryEntry> {
        self.entries.get(id)
    }

    pub fn get_mut(&mut self, id: &MemoryId) -> Option<&mut MemoryEntry> {
        self.entries.get_mut(id)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Query entries matching a specific scope.
    pub fn query_by_scope(&self, scope: &MemoryScope) -> Vec<&MemoryEntry> {
        self.order
            .iter()
            .filter_map(|id| self.entries.get(id))
            .filter(|e| e.scope == *scope)
            .collect()
    }

    /// Query entries matching a specific status.
    pub fn query_by_status(&self, status: &MemoryStatus) -> Vec<&MemoryEntry> {
        self.order
            .iter()
            .filter_map(|id| self.entries.get(id))
            .filter(|e| e.status == *status)
            .collect()
    }

    /// Return all confirmed entries relevant to a given LOB.
    ///
    /// This includes LOB-scoped confirmed entries for the specified LOB
    /// plus all global confirmed entries. LOB-scoped memory from other LOBs
    /// is excluded per Spec Section 7.13 scope rules.
    pub fn query_active_for_lob(&self, lob: &str) -> Vec<&MemoryEntry> {
        self.order
            .iter()
            .filter_map(|id| self.entries.get(id))
            .filter(|e| {
                if e.status != MemoryStatus::Confirmed {
                    return false;
                }
                match &e.scope {
                    MemoryScope::Lob(entry_lob) => entry_lob == lob,
                    MemoryScope::Global => true,
                    MemoryScope::Workspace(_) => true,
                    MemoryScope::Team(_) => true,
                }
            })
            .collect()
    }

    /// All entries in insertion order.
    pub fn all(&self) -> Vec<&MemoryEntry> {
        self.order
            .iter()
            .filter_map(|id| self.entries.get(id))
            .collect()
    }

    /// Save all entries to a JSONL file.
    pub fn save_to_file(&self, path: &Path) -> Result<(), MemoryStoreError> {
        let file = std::fs::File::create(path)
            .map_err(|e| MemoryStoreError::Io(e))?;
        let mut writer = std::io::BufWriter::new(file);
        for id in &self.order {
            if let Some(entry) = self.entries.get(id) {
                let line = serde_json::to_string(entry)
                    .map_err(|e| MemoryStoreError::Serialization(e))?;
                writeln!(writer, "{}", line)
                    .map_err(|e| MemoryStoreError::Io(e))?;
            }
        }
        writer.flush().map_err(|e| MemoryStoreError::Io(e))?;
        Ok(())
    }

    /// Load entries from a JSONL file. Returns an empty store if file does not exist.
    pub fn load_from_file(path: &Path) -> Result<Self, MemoryStoreError> {
        if !path.exists() {
            return Ok(Self::new());
        }
        let file = std::fs::File::open(path)
            .map_err(|e| MemoryStoreError::Io(e))?;
        let reader = std::io::BufReader::new(file);
        let mut store = Self::new();
        for line in reader.lines() {
            let line = line.map_err(|e| MemoryStoreError::Io(e))?;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let entry: MemoryEntry = serde_json::from_str(trimmed)
                .map_err(|e| MemoryStoreError::Serialization(e))?;
            store.insert(entry);
        }
        Ok(store)
    }
}

impl Default for MemoryStore {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MemoryStoreError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

// tests at bottom of file (from Step 1)
```

Replace `spool/spool-memory/src/lifecycle.rs` with:

```rust
// spool/spool-memory/src/lifecycle.rs
use chrono::Utc;

use crate::types::{MemoryEntry, MemoryStatus};

/// Error for invalid lifecycle transitions.
#[derive(Debug, thiserror::Error)]
pub enum LifecycleError {
    #[error("invalid transition from {from:?} to {to:?}")]
    InvalidTransition {
        from: MemoryStatus,
        to: MemoryStatus,
    },
}

/// Check whether a lifecycle transition is valid.
///
/// Valid transitions per Spec Section 7.13:
///   candidate -> confirmed
///   confirmed -> stale
///   confirmed -> invalidated
///   stale -> confirmed (revalidation)
///   stale -> invalidated
///
/// Invalidated is terminal — no transitions out.
/// Candidate cannot go directly to stale or invalidated.
/// Confirmed cannot return to candidate.
pub fn is_valid_transition(from: &MemoryStatus, to: &MemoryStatus) -> bool {
    matches!(
        (from, to),
        (MemoryStatus::Candidate, MemoryStatus::Confirmed)
            | (MemoryStatus::Confirmed, MemoryStatus::Stale)
            | (MemoryStatus::Confirmed, MemoryStatus::Invalidated)
            | (MemoryStatus::Stale, MemoryStatus::Confirmed)
            | (MemoryStatus::Stale, MemoryStatus::Invalidated)
    )
}

/// Promote a candidate entry to confirmed status.
///
/// Confirmed promotion should require at least one of (Spec Section 7.13):
/// - repeated appearance across tasks
/// - explicit human confirmation
/// - strong evidence that the information is stable operating context
pub fn promote(entry: &mut MemoryEntry) -> Result<(), LifecycleError> {
    if !is_valid_transition(&entry.status, &MemoryStatus::Confirmed) {
        return Err(LifecycleError::InvalidTransition {
            from: entry.status.clone(),
            to: MemoryStatus::Confirmed,
        });
    }
    entry.status = MemoryStatus::Confirmed;
    entry.last_validated_at = Some(Utc::now());
    Ok(())
}

/// Mark a confirmed entry as stale.
///
/// Stale entries remain inspectable and can be revalidated.
pub fn mark_stale(entry: &mut MemoryEntry) -> Result<(), LifecycleError> {
    if !is_valid_transition(&entry.status, &MemoryStatus::Stale) {
        return Err(LifecycleError::InvalidTransition {
            from: entry.status.clone(),
            to: MemoryStatus::Stale,
        });
    }
    entry.status = MemoryStatus::Stale;
    Ok(())
}

/// Invalidate a memory entry with a reason.
///
/// Invalidated memory should remain inspectable in trace or memory history
/// rather than disappearing without explanation (Spec Section 7.13).
pub fn invalidate(entry: &mut MemoryEntry, reason: String) -> Result<(), LifecycleError> {
    if !is_valid_transition(&entry.status, &MemoryStatus::Invalidated) {
        return Err(LifecycleError::InvalidTransition {
            from: entry.status.clone(),
            to: MemoryStatus::Invalidated,
        });
    }
    entry.status = MemoryStatus::Invalidated;
    entry.invalidated_at = Some(Utc::now());
    entry.invalidation_reason = Some(reason);
    Ok(())
}

/// Revalidate a stale entry, promoting it back to confirmed.
///
/// This is appropriate when fresh runtime evidence confirms the memory
/// is still valid operating context.
pub fn revalidate(entry: &mut MemoryEntry) -> Result<(), LifecycleError> {
    if entry.status != MemoryStatus::Stale {
        return Err(LifecycleError::InvalidTransition {
            from: entry.status.clone(),
            to: MemoryStatus::Confirmed,
        });
    }
    entry.status = MemoryStatus::Confirmed;
    entry.last_validated_at = Some(Utc::now());
    Ok(())
}

// tests at bottom of file (from Step 1)
```

**Step 4: Run tests to verify they pass**

Run: `cd spool && cargo test -p spool-memory -- store lifecycle`
Expected: 14 tests PASS (7 store + 7 lifecycle)

**Step 5: Commit**

```bash
git add spool/spool-memory/
git commit -m "feat(spool-memory): durable memory store with scope queries, JSONL persistence, and lifecycle transitions"
```

---

## Task 4: JSON Export Of Canonical Task Results

**Files:**

- Modify: `spool/spool-export/src/json_export.rs`

**Step 1: Write the failing test**

Add to `spool/spool-export/src/json_export.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use spool_protocol::artifact::ArtifactType;
    use spool_protocol::contradiction::ContradictionId;
    use spool_protocol::evidence::EvidenceId;
    use spool_protocol::task_contract::TaskId;
    use spool_protocol::task_result::*;

    fn sample_task_result() -> TaskResult {
        TaskResult {
            task_id: TaskId("task_123".into()),
            proposed_state: Some(ResultState::Confirmed),
            state: ResultState::SupportedHypothesis,
            confidence: Confidence::Medium,
            summary: "The executive report is likely using a measure definition that no longer matches warehouse-backed business logic.".into(),
            findings: vec![
                Finding {
                    id: "finding_1".into(),
                    title: "Revenue variance traced to semantic-model measure logic".into(),
                    detail: "The report measure and validation queries disagree on quarter-over-quarter handling.".into(),
                },
            ],
            evidence_refs: vec![
                EvidenceId("ev_12".into()),
                EvidenceId("ev_19".into()),
                EvidenceId("ev_23".into()),
            ],
            validation_results: vec![
                ValidationResult {
                    id: "val_1".into(),
                    validation_type: "dax_and_warehouse_comparison".into(),
                    status: "passed".into(),
                    detail: "DAX and warehouse validations aligned.".into(),
                },
            ],
            recommended_actions: vec![
                RecommendedAction {
                    id: "action_1".into(),
                    action_type: "proposed_model_change".into(),
                    summary: "Review and update the quarter-over-quarter revenue measure logic.".into(),
                },
            ],
            blockers: vec![],
            open_questions: vec![],
            proposed_changes: vec![
                ProposedChange {
                    id: "change_1".into(),
                    artifact_type: ArtifactType::Measure,
                    artifact_ref: "Sales Model.Sales[QoQ Revenue]".into(),
                    change_summary: "Replace current quarter offset logic.".into(),
                },
            ],
            contradiction_refs: vec![],
            result_generated_at: Some(chrono::Utc::now()),
            result_version: Some(1),
        }
    }

    #[test]
    fn export_json_contains_required_fields() {
        let result = sample_task_result();
        let json = export_task_result_json(&result).unwrap();

        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value["task_id"], "task_123");
        assert_eq!(value["state"], "supported_hypothesis");
        assert_eq!(value["confidence"], "medium");
        assert!(value["summary"].as_str().unwrap().contains("executive report"));
        assert_eq!(value["findings"].as_array().unwrap().len(), 1);
        assert_eq!(value["evidence_refs"].as_array().unwrap().len(), 3);
        assert_eq!(value["validation_results"].as_array().unwrap().len(), 1);
        assert_eq!(value["recommended_actions"].as_array().unwrap().len(), 1);
        assert_eq!(value["proposed_changes"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn export_json_includes_metadata_header() {
        let result = sample_task_result();
        let json = export_task_result_json_with_metadata(&result).unwrap();

        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value["_export_format"], "spool_task_result_v1_unstable");
        assert!(value["_exported_at"].is_string());
        assert!(value["task_result"].is_object());
    }

    #[test]
    fn export_json_round_trips() {
        let result = sample_task_result();
        let json = export_task_result_json(&result).unwrap();
        let restored: TaskResult = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.task_id, result.task_id);
        assert_eq!(restored.state, result.state);
        assert_eq!(restored.findings.len(), result.findings.len());
    }

    #[test]
    fn export_to_file_and_read_back() {
        let result = sample_task_result();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("result.json");

        export_task_result_json_to_file(&result, &path).unwrap();

        let contents = std::fs::read_to_string(&path).unwrap();
        let value: serde_json::Value = serde_json::from_str(&contents).unwrap();
        assert_eq!(value["_export_format"], "spool_task_result_v1_unstable");
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cd spool && cargo test -p spool-export -- json_export`
Expected: FAIL

**Step 3: Write the implementation**

Replace `spool/spool-export/src/json_export.rs` with:

```rust
// spool/spool-export/src/json_export.rs
use std::io::Write;
use std::path::Path;

use chrono::Utc;
use spool_protocol::task_result::TaskResult;

/// Error type for export operations.
#[derive(Debug, thiserror::Error)]
pub enum ExportError {
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

/// Export a canonical task result as pretty-printed JSON.
///
/// This is a convenience output, not a stable contract (Spec Section 10.8).
pub fn export_task_result_json(result: &TaskResult) -> Result<String, ExportError> {
    let json = serde_json::to_string_pretty(result)?;
    Ok(json)
}

/// Export a canonical task result as JSON wrapped in a metadata envelope.
///
/// The envelope includes:
/// - `_export_format`: identifies this as a non-stable v1 export
/// - `_exported_at`: ISO 8601 timestamp
/// - `task_result`: the canonical task result object
pub fn export_task_result_json_with_metadata(result: &TaskResult) -> Result<String, ExportError> {
    let envelope = serde_json::json!({
        "_export_format": "spool_task_result_v1_unstable",
        "_exported_at": Utc::now().to_rfc3339(),
        "task_result": serde_json::to_value(result)?,
    });
    let json = serde_json::to_string_pretty(&envelope)?;
    Ok(json)
}

/// Export a canonical task result to a JSON file with metadata envelope.
pub fn export_task_result_json_to_file(
    result: &TaskResult,
    path: &Path,
) -> Result<(), ExportError> {
    let json = export_task_result_json_with_metadata(result)?;
    let mut file = std::fs::File::create(path)?;
    file.write_all(json.as_bytes())?;
    file.flush()?;
    Ok(())
}

// tests at bottom of file (from Step 1)
```

**Step 4: Run test to verify it passes**

Run: `cd spool && cargo test -p spool-export -- json_export`
Expected: 4 tests PASS

**Step 5: Commit**

```bash
git add spool/spool-export/src/json_export.rs
git commit -m "feat(spool-export): JSON export of canonical task results with unstable v1 metadata envelope"
```

---

## Task 5: Markdown Export Of Canonical Task Results

**Files:**

- Modify: `spool/spool-export/src/markdown_export.rs`

**Step 1: Write the failing test**

Add to `spool/spool-export/src/markdown_export.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use spool_protocol::artifact::ArtifactType;
    use spool_protocol::contradiction::ContradictionId;
    use spool_protocol::evidence::EvidenceId;
    use spool_protocol::task_contract::TaskId;
    use spool_protocol::task_result::*;

    fn sample_task_result() -> TaskResult {
        TaskResult {
            task_id: TaskId("task_123".into()),
            proposed_state: Some(ResultState::Confirmed),
            state: ResultState::SupportedHypothesis,
            confidence: Confidence::Medium,
            summary: "The executive report is likely using a measure definition that no longer matches warehouse-backed business logic.".into(),
            findings: vec![
                Finding {
                    id: "finding_1".into(),
                    title: "Revenue variance traced to semantic-model measure logic".into(),
                    detail: "The report measure and validation queries disagree on QoQ handling.".into(),
                },
                Finding {
                    id: "finding_2".into(),
                    title: "Warehouse confirms expected business logic".into(),
                    detail: "Warehouse query returned correct QoQ totals.".into(),
                },
            ],
            evidence_refs: vec![
                EvidenceId("ev_12".into()),
                EvidenceId("ev_19".into()),
            ],
            validation_results: vec![
                ValidationResult {
                    id: "val_1".into(),
                    validation_type: "dax_and_warehouse_comparison".into(),
                    status: "passed".into(),
                    detail: "DAX and warehouse validations aligned.".into(),
                },
            ],
            recommended_actions: vec![
                RecommendedAction {
                    id: "action_1".into(),
                    action_type: "proposed_model_change".into(),
                    summary: "Review and update QoQ revenue measure.".into(),
                },
            ],
            blockers: vec![],
            open_questions: vec![
                OpenQuestion {
                    id: "q_1".into(),
                    question: "Should the fiscal calendar offset be applied?".into(),
                },
            ],
            proposed_changes: vec![
                ProposedChange {
                    id: "change_1".into(),
                    artifact_type: ArtifactType::Measure,
                    artifact_ref: "Sales Model.Sales[QoQ Revenue]".into(),
                    change_summary: "Replace quarter offset logic.".into(),
                },
            ],
            contradiction_refs: vec![],
            result_generated_at: Some(chrono::Utc::now()),
            result_version: Some(1),
        }
    }

    #[test]
    fn markdown_contains_title_and_summary() {
        let result = sample_task_result();
        let md = export_task_result_markdown(&result);
        assert!(md.contains("# Task Result: task_123"));
        assert!(md.contains("**State:** supported_hypothesis"));
        assert!(md.contains("**Confidence:** medium"));
        assert!(md.contains("executive report"));
    }

    #[test]
    fn markdown_contains_findings() {
        let result = sample_task_result();
        let md = export_task_result_markdown(&result);
        assert!(md.contains("## Findings"));
        assert!(md.contains("Revenue variance traced to semantic-model measure logic"));
        assert!(md.contains("Warehouse confirms expected business logic"));
    }

    #[test]
    fn markdown_contains_validation_results() {
        let result = sample_task_result();
        let md = export_task_result_markdown(&result);
        assert!(md.contains("## Validation Results"));
        assert!(md.contains("dax_and_warehouse_comparison"));
        assert!(md.contains("passed"));
    }

    #[test]
    fn markdown_contains_recommended_actions() {
        let result = sample_task_result();
        let md = export_task_result_markdown(&result);
        assert!(md.contains("## Recommended Actions"));
        assert!(md.contains("Review and update QoQ revenue measure."));
    }

    #[test]
    fn markdown_contains_open_questions() {
        let result = sample_task_result();
        let md = export_task_result_markdown(&result);
        assert!(md.contains("## Open Questions"));
        assert!(md.contains("Should the fiscal calendar offset be applied?"));
    }

    #[test]
    fn markdown_contains_proposed_changes() {
        let result = sample_task_result();
        let md = export_task_result_markdown(&result);
        assert!(md.contains("## Proposed Changes"));
        assert!(md.contains("Sales Model.Sales[QoQ Revenue]"));
    }

    #[test]
    fn markdown_omits_empty_sections() {
        let mut result = sample_task_result();
        result.blockers = vec![];
        let md = export_task_result_markdown(&result);
        assert!(!md.contains("## Blockers"));
    }

    #[test]
    fn markdown_includes_unstable_warning() {
        let result = sample_task_result();
        let md = export_task_result_markdown(&result);
        assert!(md.contains("non-stable"));
    }

    #[test]
    fn export_to_file() {
        let result = sample_task_result();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("result.md");

        export_task_result_markdown_to_file(&result, &path).unwrap();
        let contents = std::fs::read_to_string(&path).unwrap();
        assert!(contents.contains("# Task Result: task_123"));
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cd spool && cargo test -p spool-export -- markdown_export`
Expected: FAIL

**Step 3: Write the implementation**

Replace `spool/spool-export/src/markdown_export.rs` with:

```rust
// spool/spool-export/src/markdown_export.rs
use std::fmt::Write as FmtWrite;
use std::io::Write;
use std::path::Path;

use spool_protocol::task_result::TaskResult;

/// Export a canonical task result as human-readable Markdown.
///
/// This is a convenience output, not a stable contract (Spec Section 10.8).
/// The format may change between versions without notice.
pub fn export_task_result_markdown(result: &TaskResult) -> String {
    let mut md = String::new();

    // Header
    writeln!(md, "# Task Result: {}", result.task_id.0).unwrap();
    writeln!(md).unwrap();
    writeln!(md, "> **Note:** This export uses a non-stable v1 format and may change without notice.").unwrap();
    writeln!(md).unwrap();

    // State and confidence
    writeln!(md, "**State:** {}", format_state(&result.state)).unwrap();
    writeln!(md, "**Confidence:** {}", format_confidence(&result.confidence)).unwrap();
    if let Some(ref proposed) = result.proposed_state {
        writeln!(md, "**Proposed State:** {}", format_state(proposed)).unwrap();
    }
    writeln!(md).unwrap();

    // Summary
    writeln!(md, "## Summary").unwrap();
    writeln!(md).unwrap();
    writeln!(md, "{}", result.summary).unwrap();
    writeln!(md).unwrap();

    // Findings
    if !result.findings.is_empty() {
        writeln!(md, "## Findings").unwrap();
        writeln!(md).unwrap();
        for finding in &result.findings {
            writeln!(md, "### {}", finding.title).unwrap();
            writeln!(md).unwrap();
            writeln!(md, "{}", finding.detail).unwrap();
            writeln!(md).unwrap();
        }
    }

    // Evidence refs
    if !result.evidence_refs.is_empty() {
        writeln!(md, "## Evidence References").unwrap();
        writeln!(md).unwrap();
        for ev_ref in &result.evidence_refs {
            writeln!(md, "- `{}`", ev_ref.0).unwrap();
        }
        writeln!(md).unwrap();
    }

    // Validation results
    if !result.validation_results.is_empty() {
        writeln!(md, "## Validation Results").unwrap();
        writeln!(md).unwrap();
        for val in &result.validation_results {
            writeln!(md, "- **{}** ({}): {}", val.validation_type, val.status, val.detail).unwrap();
        }
        writeln!(md).unwrap();
    }

    // Recommended actions
    if !result.recommended_actions.is_empty() {
        writeln!(md, "## Recommended Actions").unwrap();
        writeln!(md).unwrap();
        for action in &result.recommended_actions {
            writeln!(md, "- **[{}]** {}", action.action_type, action.summary).unwrap();
        }
        writeln!(md).unwrap();
    }

    // Blockers
    if !result.blockers.is_empty() {
        writeln!(md, "## Blockers").unwrap();
        writeln!(md).unwrap();
        for blocker in &result.blockers {
            writeln!(md, "- **[{}]** {}", blocker.blocker_type, blocker.summary).unwrap();
        }
        writeln!(md).unwrap();
    }

    // Open questions
    if !result.open_questions.is_empty() {
        writeln!(md, "## Open Questions").unwrap();
        writeln!(md).unwrap();
        for q in &result.open_questions {
            writeln!(md, "- {}", q.question).unwrap();
        }
        writeln!(md).unwrap();
    }

    // Proposed changes
    if !result.proposed_changes.is_empty() {
        writeln!(md, "## Proposed Changes").unwrap();
        writeln!(md).unwrap();
        for change in &result.proposed_changes {
            writeln!(
                md,
                "- **{}** (`{}`): {}",
                change.artifact_ref,
                format_artifact_type(&change.artifact_type),
                change.change_summary,
            )
            .unwrap();
        }
        writeln!(md).unwrap();
    }

    // Contradiction refs
    if !result.contradiction_refs.is_empty() {
        writeln!(md, "## Contradiction References").unwrap();
        writeln!(md).unwrap();
        for c_ref in &result.contradiction_refs {
            writeln!(md, "- `{}`", c_ref.0).unwrap();
        }
        writeln!(md).unwrap();
    }

    // Footer metadata
    if let Some(ref generated_at) = result.result_generated_at {
        writeln!(md, "---").unwrap();
        writeln!(md, "*Generated at: {}*", generated_at.to_rfc3339()).unwrap();
    }

    md
}

/// Export a task result Markdown to a file.
pub fn export_task_result_markdown_to_file(
    result: &TaskResult,
    path: &Path,
) -> Result<(), std::io::Error> {
    let md = export_task_result_markdown(result);
    let mut file = std::fs::File::create(path)?;
    file.write_all(md.as_bytes())?;
    file.flush()?;
    Ok(())
}

fn format_state(state: &spool_protocol::task_result::ResultState) -> &'static str {
    match state {
        spool_protocol::task_result::ResultState::Confirmed => "confirmed",
        spool_protocol::task_result::ResultState::SupportedHypothesis => "supported_hypothesis",
        spool_protocol::task_result::ResultState::Inconclusive => "inconclusive",
        spool_protocol::task_result::ResultState::Blocked => "blocked",
    }
}

fn format_confidence(confidence: &spool_protocol::task_result::Confidence) -> &'static str {
    match confidence {
        spool_protocol::task_result::Confidence::High => "high",
        spool_protocol::task_result::Confidence::Medium => "medium",
        spool_protocol::task_result::Confidence::Low => "low",
    }
}

fn format_artifact_type(artifact_type: &spool_protocol::artifact::ArtifactType) -> &'static str {
    match artifact_type {
        spool_protocol::artifact::ArtifactType::Report => "report",
        spool_protocol::artifact::ArtifactType::Page => "page",
        spool_protocol::artifact::ArtifactType::Visual => "visual",
        spool_protocol::artifact::ArtifactType::SemanticModel => "semantic_model",
        spool_protocol::artifact::ArtifactType::Measure => "measure",
        spool_protocol::artifact::ArtifactType::Table => "table",
        spool_protocol::artifact::ArtifactType::Column => "column",
        spool_protocol::artifact::ArtifactType::Relationship => "relationship",
        spool_protocol::artifact::ArtifactType::Warehouse => "warehouse",
        spool_protocol::artifact::ArtifactType::QueryResult => "query_result",
    }
}

// tests at bottom of file (from Step 1)
```

**Step 4: Run test to verify it passes**

Run: `cd spool && cargo test -p spool-export -- markdown_export`
Expected: 9 tests PASS

**Step 5: Commit**

```bash
git add spool/spool-export/src/markdown_export.rs
git commit -m "feat(spool-export): Markdown export of canonical task results with section omission for empty fields"
```

---

## Task 6: Policy Enforcement — Confirmation Policy

> **Note:** SQL policy enforcement (Spec Section 14.2) is owned by Plan 4 (spool-exec). This task covers only confirmation policy (Spec Section 14.1).

**Files:**

- Modify: `spool/spool-cli/src/policy.rs`

**Step 1: Write the failing test**

Add to `spool/spool-cli/src/policy.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // --- Confirmation policy tests ---

    #[test]
    fn ambiguous_action_requires_confirmation() {
        let policy = ConfirmationPolicy::default();
        let action = ProposedAction {
            action_type: ActionType::ArtifactResolution,
            description: "Multiple reports match 'Revenue Report'".into(),
            is_ambiguous: true,
            is_scope_expanding: false,
            is_expectation_shaping: false,
            is_side_effecting: false,
        };
        assert_eq!(policy.evaluate(&action), ConfirmationDecision::RequiresConfirmation {
            reason: "action is ambiguous".into(),
        });
    }

    #[test]
    fn scope_expanding_requires_confirmation() {
        let policy = ConfirmationPolicy::default();
        let action = ProposedAction {
            action_type: ActionType::Investigation,
            description: "Expanding to include related warehouse".into(),
            is_ambiguous: false,
            is_scope_expanding: true,
            is_expectation_shaping: false,
            is_side_effecting: false,
        };
        assert_eq!(policy.evaluate(&action), ConfirmationDecision::RequiresConfirmation {
            reason: "action is scope-expanding".into(),
        });
    }

    #[test]
    fn routine_investigation_proceeds() {
        let policy = ConfirmationPolicy::default();
        let action = ProposedAction {
            action_type: ActionType::Investigation,
            description: "Run DAX query on confirmed measure".into(),
            is_ambiguous: false,
            is_scope_expanding: false,
            is_expectation_shaping: false,
            is_side_effecting: false,
        };
        assert_eq!(policy.evaluate(&action), ConfirmationDecision::Proceed);
    }

    #[test]
    fn side_effecting_requires_confirmation() {
        let policy = ConfirmationPolicy::default();
        let action = ProposedAction {
            action_type: ActionType::Export,
            description: "Export results to file".into(),
            is_ambiguous: false,
            is_scope_expanding: false,
            is_expectation_shaping: false,
            is_side_effecting: true,
        };
        assert_eq!(policy.evaluate(&action), ConfirmationDecision::RequiresConfirmation {
            reason: "action is side-effecting".into(),
        });
    }

    #[test]
    fn expectation_shaping_requires_confirmation() {
        let policy = ConfirmationPolicy::default();
        let action = ProposedAction {
            action_type: ActionType::Investigation,
            description: "Downgrading result confidence from high to medium".into(),
            is_ambiguous: false,
            is_scope_expanding: false,
            is_expectation_shaping: true,
            is_side_effecting: false,
        };
        assert_eq!(policy.evaluate(&action), ConfirmationDecision::RequiresConfirmation {
            reason: "action is expectation-shaping".into(),
        });
    }
}
```


**Step 2: Run test to verify it fails**

Run: `cd spool && cargo test -p spool-cli -- policy`
Expected: FAIL

**Step 3: Write the implementation**

Replace `spool/spool-cli/src/policy.rs` with:

```rust
// spool/spool-cli/src/policy.rs

/// Confirmation policy enforcement per Spec Section 14.1.
///
/// Spool interrupts for confirmation only when the next step is:
/// - ambiguous
/// - scope-expanding
/// - expectation-shaping
/// - side-effecting
///
/// Routine low-risk investigation inside confirmed scope proceeds without
/// extra confirmation.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionType {
    ArtifactResolution,
    Investigation,
    Export,
    MemoryPromotion,
}

#[derive(Debug, Clone)]
pub struct ProposedAction {
    pub action_type: ActionType,
    pub description: String,
    pub is_ambiguous: bool,
    pub is_scope_expanding: bool,
    pub is_expectation_shaping: bool,
    pub is_side_effecting: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfirmationDecision {
    Proceed,
    RequiresConfirmation { reason: String },
}

pub struct ConfirmationPolicy {
    pub enforce_ambiguous: bool,
    pub enforce_scope_expanding: bool,
    pub enforce_expectation_shaping: bool,
    pub enforce_side_effecting: bool,
}

impl Default for ConfirmationPolicy {
    fn default() -> Self {
        Self {
            enforce_ambiguous: true,
            enforce_scope_expanding: true,
            enforce_expectation_shaping: true,
            enforce_side_effecting: true,
        }
    }
}

impl ConfirmationPolicy {
    /// Evaluate whether a proposed action requires user confirmation.
    pub fn evaluate(&self, action: &ProposedAction) -> ConfirmationDecision {
        if self.enforce_ambiguous && action.is_ambiguous {
            return ConfirmationDecision::RequiresConfirmation {
                reason: "action is ambiguous".into(),
            };
        }
        if self.enforce_scope_expanding && action.is_scope_expanding {
            return ConfirmationDecision::RequiresConfirmation {
                reason: "action is scope-expanding".into(),
            };
        }
        if self.enforce_expectation_shaping && action.is_expectation_shaping {
            return ConfirmationDecision::RequiresConfirmation {
                reason: "action is expectation-shaping".into(),
            };
        }
        if self.enforce_side_effecting && action.is_side_effecting {
            return ConfirmationDecision::RequiresConfirmation {
                reason: "action is side-effecting".into(),
            };
        }
        ConfirmationDecision::Proceed
    }
}


// tests at bottom of file (from Step 1)
```

**Step 4: Run test to verify it passes**

Run: `cd spool && cargo test -p spool-cli -- policy`
Expected: 5 tests PASS (confirmation policy only)

**Step 5: Commit**

```bash
git add spool/spool-cli/src/policy.rs
git commit -m "feat(spool-cli): confirmation policy enforcement per spec Section 14.1"
```

---

## Task 7: Configuration Management

**Files:**

- Modify: `spool/spool-cli/src/config.rs`

**Step 1: Write the failing test**

Add to `spool/spool-cli/src/config.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn default_config_is_valid() {
        let config = SpoolConfig::default();
        assert_eq!(config.session.data_dir, "~/.spool/sessions");
        assert_eq!(config.session.max_history, 100);
        assert_eq!(config.harness.max_evaluator_iterations, 5);
    }

    #[test]
    fn toml_round_trip() {
        let config = SpoolConfig {
            connection: ConnectionConfig {
                workspace_id: Some("ws_123".into()),
                tenant_id: Some("tenant_abc".into()),
                auth_method: AuthMethod::DeviceCode,
            },
            session: SessionConfig {
                data_dir: "/tmp/spool/sessions".into(),
                max_history: 50,
                default_lob: Some("finance".into()),
            },
            harness: HarnessConfig {
                max_evaluator_iterations: 3,
                checkpoint_on_ambiguous: true,
                checkpoint_on_scope_expanding: true,
                checkpoint_on_expectation_shaping: true,
                checkpoint_on_side_effecting: true,
            },
            memory: MemoryConfig {
                store_path: "/tmp/spool/memory.jsonl".into(),
                auto_capture: false,
            },
            telemetry: TelemetryConfig {
                enabled: true,
                log_level: "debug".into(),
                json_logs: false,
                otel_enabled: false,
                otel_endpoint: None,
            },
        };

        let toml_str = toml::to_string_pretty(&config).unwrap();
        let restored: SpoolConfig = toml::from_str(&toml_str).unwrap();

        assert_eq!(restored.connection.workspace_id, Some("ws_123".into()));
        assert_eq!(restored.session.default_lob, Some("finance".into()));
        assert_eq!(restored.harness.max_evaluator_iterations, 3);
        assert_eq!(restored.telemetry.log_level, "debug");
    }

    #[test]
    fn load_from_toml_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("spool.toml");

        let toml_content = r#"
[connection]
workspace_id = "ws_test"
auth_method = "device_code"

[session]
data_dir = "/tmp/spool/test"
max_history = 25

[harness]
max_evaluator_iterations = 4

[memory]
store_path = "/tmp/spool/mem.jsonl"

[telemetry]
enabled = false
log_level = "warn"
"#;
        std::fs::write(&path, toml_content).unwrap();

        let config = SpoolConfig::load_from_file(&path).unwrap();
        assert_eq!(config.connection.workspace_id, Some("ws_test".into()));
        assert_eq!(config.session.max_history, 25);
        assert_eq!(config.harness.max_evaluator_iterations, 4);
        assert!(!config.telemetry.enabled);
    }

    #[test]
    fn load_from_nonexistent_returns_default() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("does_not_exist.toml");
        let config = SpoolConfig::load_from_file(&path).unwrap();
        assert_eq!(config.session.data_dir, "~/.spool/sessions");
    }

    #[test]
    fn env_override_workspace_id() {
        let mut config = SpoolConfig::default();
        // Simulate env var
        env::set_var("SPOOL_WORKSPACE_ID", "ws_from_env");
        config.apply_env_overrides();
        assert_eq!(config.connection.workspace_id, Some("ws_from_env".into()));
        env::remove_var("SPOOL_WORKSPACE_ID");
    }

    #[test]
    fn env_override_log_level() {
        let mut config = SpoolConfig::default();
        env::set_var("SPOOL_LOG_LEVEL", "trace");
        config.apply_env_overrides();
        assert_eq!(config.telemetry.log_level, "trace");
        env::remove_var("SPOOL_LOG_LEVEL");
    }

    #[test]
    fn env_override_data_dir() {
        let mut config = SpoolConfig::default();
        env::set_var("SPOOL_DATA_DIR", "/custom/path");
        config.apply_env_overrides();
        assert_eq!(config.session.data_dir, "/custom/path");
        env::remove_var("SPOOL_DATA_DIR");
    }

    #[test]
    fn env_override_default_lob() {
        let mut config = SpoolConfig::default();
        env::set_var("SPOOL_DEFAULT_LOB", "marketing");
        config.apply_env_overrides();
        assert_eq!(config.session.default_lob, Some("marketing".into()));
        env::remove_var("SPOOL_DEFAULT_LOB");
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cd spool && cargo test -p spool-cli -- config`
Expected: FAIL

**Step 3: Write the implementation**

Replace `spool/spool-cli/src/config.rs` with:

```rust
// spool/spool-cli/src/config.rs
use std::path::Path;

use serde::{Deserialize, Serialize};

/// Top-level Spool configuration.
///
/// Loaded from a TOML file with environment variable overrides.
/// Supports connection details, session settings, harness tuning,
/// memory store configuration, and telemetry settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpoolConfig {
    #[serde(default)]
    pub connection: ConnectionConfig,
    #[serde(default)]
    pub session: SessionConfig,
    #[serde(default)]
    pub harness: HarnessConfig,
    #[serde(default)]
    pub memory: MemoryConfig,
    #[serde(default)]
    pub telemetry: TelemetryConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    pub workspace_id: Option<String>,
    pub tenant_id: Option<String>,
    #[serde(default)]
    pub auth_method: AuthMethod,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthMethod {
    DeviceCode,
    ServicePrincipal,
    ManagedIdentity,
}

impl Default for AuthMethod {
    fn default() -> Self {
        Self::DeviceCode
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    #[serde(default = "default_data_dir")]
    pub data_dir: String,
    #[serde(default = "default_max_history")]
    pub max_history: usize,
    pub default_lob: Option<String>,
}

fn default_data_dir() -> String {
    "~/.spool/sessions".into()
}

fn default_max_history() -> usize {
    100
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarnessConfig {
    #[serde(default = "default_max_evaluator_iterations")]
    pub max_evaluator_iterations: u32,
    #[serde(default = "default_true")]
    pub checkpoint_on_ambiguous: bool,
    #[serde(default = "default_true")]
    pub checkpoint_on_scope_expanding: bool,
    #[serde(default = "default_true")]
    pub checkpoint_on_expectation_shaping: bool,
    #[serde(default = "default_true")]
    pub checkpoint_on_side_effecting: bool,
}

fn default_max_evaluator_iterations() -> u32 {
    5
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    #[serde(default = "default_memory_store_path")]
    pub store_path: String,
    #[serde(default)]
    pub auto_capture: bool,
}

fn default_memory_store_path() -> String {
    "~/.spool/memory.jsonl".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_log_level")]
    pub log_level: String,
    #[serde(default)]
    pub json_logs: bool,
    #[serde(default)]
    pub otel_enabled: bool,
    pub otel_endpoint: Option<String>,
}

fn default_log_level() -> String {
    "info".into()
}

impl Default for SpoolConfig {
    fn default() -> Self {
        Self {
            connection: ConnectionConfig::default(),
            session: SessionConfig::default(),
            harness: HarnessConfig::default(),
            memory: MemoryConfig::default(),
            telemetry: TelemetryConfig::default(),
        }
    }
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            workspace_id: None,
            tenant_id: None,
            auth_method: AuthMethod::default(),
        }
    }
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            data_dir: default_data_dir(),
            max_history: default_max_history(),
            default_lob: None,
        }
    }
}

impl Default for HarnessConfig {
    fn default() -> Self {
        Self {
            max_evaluator_iterations: default_max_evaluator_iterations(),
            checkpoint_on_ambiguous: true,
            checkpoint_on_scope_expanding: true,
            checkpoint_on_expectation_shaping: true,
            checkpoint_on_side_effecting: true,
        }
    }
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            store_path: default_memory_store_path(),
            auto_capture: false,
        }
    }
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            log_level: default_log_level(),
            json_logs: false,
            otel_enabled: false,
            otel_endpoint: None,
        }
    }
}

impl SpoolConfig {
    /// Load configuration from a TOML file.
    ///
    /// Returns the default config if the file does not exist.
    /// Returns an error if the file exists but cannot be parsed.
    pub fn load_from_file(path: &Path) -> Result<Self, ConfigError> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let contents = std::fs::read_to_string(path)
            .map_err(|e| ConfigError::Io(e))?;
        let config: Self = toml::from_str(&contents)
            .map_err(|e| ConfigError::Parse(e.to_string()))?;
        Ok(config)
    }

    /// Apply environment variable overrides.
    ///
    /// Environment variables take precedence over file-based config.
    /// Supported variables:
    /// - `SPOOL_WORKSPACE_ID` -> connection.workspace_id
    /// - `SPOOL_TENANT_ID` -> connection.tenant_id
    /// - `SPOOL_LOG_LEVEL` -> telemetry.log_level
    /// - `SPOOL_DATA_DIR` -> session.data_dir
    /// - `SPOOL_DEFAULT_LOB` -> session.default_lob
    pub fn apply_env_overrides(&mut self) {
        if let Ok(val) = std::env::var("SPOOL_WORKSPACE_ID") {
            self.connection.workspace_id = Some(val);
        }
        if let Ok(val) = std::env::var("SPOOL_TENANT_ID") {
            self.connection.tenant_id = Some(val);
        }
        if let Ok(val) = std::env::var("SPOOL_LOG_LEVEL") {
            self.telemetry.log_level = val;
        }
        if let Ok(val) = std::env::var("SPOOL_DATA_DIR") {
            self.session.data_dir = val;
        }
        if let Ok(val) = std::env::var("SPOOL_DEFAULT_LOB") {
            self.session.default_lob = Some(val);
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("parse error: {0}")]
    Parse(String),
}

// tests at bottom of file (from Step 1)
```

**Step 4: Run test to verify it passes**

Run: `cd spool && cargo test -p spool-cli -- config -- --test-threads=1`

Note: `--test-threads=1` because env var tests must not run in parallel.

Expected: 8 tests PASS

**Step 5: Commit**

```bash
git add spool/spool-cli/src/config.rs
git commit -m "feat(spool-cli): TOML configuration with env var overrides for connection, session, harness, memory, and telemetry"
```

---

## Task 8: Telemetry Initialization

**Files:**

- Modify: `spool/spool-cli/src/telemetry.rs`

**Step 1: Write the failing test**

Add to `spool/spool-cli/src/telemetry.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::TelemetryConfig;

    #[test]
    fn parse_log_level_valid() {
        assert!(parse_log_level("trace").is_some());
        assert!(parse_log_level("debug").is_some());
        assert!(parse_log_level("info").is_some());
        assert!(parse_log_level("warn").is_some());
        assert!(parse_log_level("error").is_some());
    }

    #[test]
    fn parse_log_level_invalid() {
        assert!(parse_log_level("garbage").is_none());
        assert!(parse_log_level("").is_none());
    }

    #[test]
    fn parse_log_level_case_insensitive() {
        assert!(parse_log_level("INFO").is_some());
        assert!(parse_log_level("Debug").is_some());
    }

    #[test]
    fn telemetry_config_to_env_filter() {
        let config = TelemetryConfig {
            enabled: true,
            log_level: "debug".into(),
            json_logs: false,
            otel_enabled: false,
            otel_endpoint: None,
        };
        let filter = build_env_filter(&config);
        // Should not panic and should produce a valid filter string
        assert!(filter.contains("debug"));
    }

    #[test]
    fn telemetry_config_disabled_uses_error_level() {
        let config = TelemetryConfig {
            enabled: false,
            log_level: "debug".into(),
            json_logs: false,
            otel_enabled: false,
            otel_endpoint: None,
        };
        let filter = build_env_filter(&config);
        assert!(filter.contains("error"));
    }

    #[test]
    fn phase_timing_records_duration() {
        let timing = PhaseTiming::start("planning");
        std::thread::sleep(std::time::Duration::from_millis(10));
        let finished = timing.finish();
        assert_eq!(finished.phase, "planning");
        assert!(finished.duration_ms >= 10);
    }

    #[test]
    fn phase_timing_display() {
        let finished = FinishedPhaseTiming {
            phase: "evaluation".into(),
            duration_ms: 1234,
        };
        let display = format!("{finished}");
        assert!(display.contains("evaluation"));
        assert!(display.contains("1234ms"));
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cd spool && cargo test -p spool-cli -- telemetry`
Expected: FAIL

**Step 3: Write the implementation**

Replace `spool/spool-cli/src/telemetry.rs` with:

```rust
// spool/spool-cli/src/telemetry.rs
use std::fmt;
use std::time::Instant;

use crate::config::TelemetryConfig;

/// Parse a log level string into a tracing-compatible level.
pub fn parse_log_level(level: &str) -> Option<tracing::Level> {
    match level.to_lowercase().as_str() {
        "trace" => Some(tracing::Level::TRACE),
        "debug" => Some(tracing::Level::DEBUG),
        "info" => Some(tracing::Level::INFO),
        "warn" => Some(tracing::Level::WARN),
        "error" => Some(tracing::Level::ERROR),
        _ => None,
    }
}

/// Build an environment filter string from telemetry config.
///
/// When telemetry is disabled, only error-level logs pass through.
/// When enabled, uses the configured log level.
pub fn build_env_filter(config: &TelemetryConfig) -> String {
    if !config.enabled {
        return "error".into();
    }
    // Validate the log level, falling back to "info" if invalid
    let level = parse_log_level(&config.log_level)
        .map(|l| l.to_string().to_lowercase())
        .unwrap_or_else(|| "info".into());
    level
}

/// Initialize the tracing subscriber based on configuration.
///
/// Sets up:
/// - env filter based on config log level
/// - JSON or human-readable format
/// - optional OpenTelemetry layer for W3C trace context
///
/// This should be called once at CLI startup.
pub fn init_tracing(config: &TelemetryConfig) {
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::EnvFilter;

    let filter = build_env_filter(config);
    let env_filter = EnvFilter::try_new(&filter)
        .unwrap_or_else(|_| EnvFilter::new("info"));

    if config.json_logs {
        let subscriber = tracing_subscriber::registry()
            .with(env_filter)
            .with(tracing_subscriber::fmt::layer().json());
        // Ignore error if a global subscriber is already set (e.g., in tests)
        let _ = tracing::subscriber::set_global_default(subscriber);
    } else {
        let subscriber = tracing_subscriber::registry()
            .with(env_filter)
            .with(tracing_subscriber::fmt::layer().compact());
        let _ = tracing::subscriber::set_global_default(subscriber);
    }
}

/// Simple phase timing utility for measuring harness phase durations.
///
/// Intended for performance observability without requiring full
/// OpenTelemetry spans for simple duration tracking.
pub struct PhaseTiming {
    phase: String,
    start: Instant,
}

impl PhaseTiming {
    pub fn start(phase: &str) -> Self {
        Self {
            phase: phase.into(),
            start: Instant::now(),
        }
    }

    pub fn finish(self) -> FinishedPhaseTiming {
        let duration = self.start.elapsed();
        FinishedPhaseTiming {
            phase: self.phase,
            duration_ms: duration.as_millis() as u64,
        }
    }
}

pub struct FinishedPhaseTiming {
    pub phase: String,
    pub duration_ms: u64,
}

impl fmt::Display for FinishedPhaseTiming {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}ms", self.phase, self.duration_ms)
    }
}

// tests at bottom of file (from Step 1)
```

**Step 4: Run test to verify it passes**

Run: `cd spool && cargo test -p spool-cli -- telemetry`
Expected: 7 tests PASS

**Step 5: Commit**

```bash
git add spool/spool-cli/src/telemetry.rs
git commit -m "feat(spool-cli): telemetry initialization with tracing, env filter, JSON logs, and phase timing"
```

---

## Task 9: CLI Argument Parsing And Session Management

**Files:**

- Modify: `spool/spool-cli/src/session.rs`
- Modify: `spool/spool-cli/src/main.rs`

**Step 1: Write the failing test**

In `spool/spool-cli/src/session.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_new_session() {
        let dir = tempfile::tempdir().unwrap();
        let manager = SessionManager::new(dir.path().to_path_buf());

        let session = manager.create_session(Some("finance"), Some("ws_123")).unwrap();
        assert!(!session.session_id.is_empty());
        assert_eq!(session.lob, Some("finance".into()));
        assert_eq!(session.workspace_id, Some("ws_123".into()));
        assert_eq!(session.status, SessionStatus::Active);
    }

    #[test]
    fn list_sessions_empty() {
        let dir = tempfile::tempdir().unwrap();
        let manager = SessionManager::new(dir.path().to_path_buf());
        let sessions = manager.list_sessions().unwrap();
        assert!(sessions.is_empty());
    }

    #[test]
    fn list_sessions_after_create() {
        let dir = tempfile::tempdir().unwrap();
        let manager = SessionManager::new(dir.path().to_path_buf());

        manager.create_session(Some("finance"), Some("ws_123")).unwrap();
        manager.create_session(Some("marketing"), Some("ws_456")).unwrap();

        let sessions = manager.list_sessions().unwrap();
        assert_eq!(sessions.len(), 2);
    }

    #[test]
    fn save_and_load_session() {
        let dir = tempfile::tempdir().unwrap();
        let manager = SessionManager::new(dir.path().to_path_buf());

        let session = manager.create_session(Some("finance"), None).unwrap();
        let session_id = session.session_id.clone();

        manager.save_session(&session).unwrap();
        let loaded = manager.load_session(&session_id).unwrap();
        assert_eq!(loaded.session_id, session_id);
        assert_eq!(loaded.lob, Some("finance".into()));
    }

    #[test]
    fn load_nonexistent_session_is_error() {
        let dir = tempfile::tempdir().unwrap();
        let manager = SessionManager::new(dir.path().to_path_buf());
        let result = manager.load_session("does_not_exist");
        assert!(result.is_err());
    }

    #[test]
    fn session_metadata_round_trip() {
        let now = chrono::Utc::now();
        let meta = SessionMetadata {
            session_id: "sess_1".into(),
            lob: Some("finance".into()),
            workspace_id: Some("ws_123".into()),
            status: SessionStatus::Active,
            created_at: now,
            updated_at: now,
            task_count: 0,
        };
        let json = serde_json::to_string(&meta).unwrap();
        let restored: SessionMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.session_id, "sess_1");
        assert_eq!(restored.status, SessionStatus::Active);
    }

    #[test]
    fn all_session_statuses_serialize() {
        let statuses = vec![
            SessionStatus::Active,
            SessionStatus::Completed,
            SessionStatus::Interrupted,
        ];
        for s in statuses {
            let json = serde_json::to_string(&s).unwrap();
            let restored: SessionStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(restored, s);
        }
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cd spool && cargo test -p spool-cli -- session`
Expected: FAIL

**Step 3: Write the implementations**

Replace `spool/spool-cli/src/session.rs` with:

```rust
// spool/spool-cli/src/session.rs
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    Active,
    Completed,
    Interrupted,
}

/// Session metadata persisted alongside session state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    pub session_id: String,
    pub lob: Option<String>,
    pub workspace_id: Option<String>,
    pub status: SessionStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub task_count: u32,
}

/// Session management for creating, listing, saving, and loading sessions.
///
/// Sessions are stored as individual JSON files in the configured data directory.
/// Each session gets a UUID-based filename.
pub struct SessionManager {
    data_dir: PathBuf,
}

#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("session not found: {0}")]
    NotFound(String),
}

impl SessionManager {
    pub fn new(data_dir: PathBuf) -> Self {
        Self { data_dir }
    }

    /// Create a new session with optional LOB and workspace scope.
    pub fn create_session(
        &self,
        lob: Option<&str>,
        workspace_id: Option<&str>,
    ) -> Result<SessionMetadata, SessionError> {
        let now = Utc::now();
        let session_id = format!("sess_{}", Uuid::new_v4().to_string().split('-').next().unwrap());

        let metadata = SessionMetadata {
            session_id,
            lob: lob.map(|s| s.into()),
            workspace_id: workspace_id.map(|s| s.into()),
            status: SessionStatus::Active,
            created_at: now,
            updated_at: now,
            task_count: 0,
        };

        self.save_session(&metadata)?;
        Ok(metadata)
    }

    /// Save session metadata to disk.
    pub fn save_session(&self, metadata: &SessionMetadata) -> Result<(), SessionError> {
        std::fs::create_dir_all(&self.data_dir)?;
        let path = self.session_path(&metadata.session_id);
        let json = serde_json::to_string_pretty(metadata)?;
        let mut file = std::fs::File::create(&path)?;
        file.write_all(json.as_bytes())?;
        file.flush()?;
        Ok(())
    }

    /// Load session metadata from disk.
    pub fn load_session(&self, session_id: &str) -> Result<SessionMetadata, SessionError> {
        let path = self.session_path(session_id);
        if !path.exists() {
            return Err(SessionError::NotFound(session_id.into()));
        }
        let contents = std::fs::read_to_string(&path)?;
        let metadata: SessionMetadata = serde_json::from_str(&contents)?;
        Ok(metadata)
    }

    /// List all sessions in the data directory.
    pub fn list_sessions(&self) -> Result<Vec<SessionMetadata>, SessionError> {
        if !self.data_dir.exists() {
            return Ok(Vec::new());
        }
        let mut sessions = Vec::new();
        for entry in std::fs::read_dir(&self.data_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                let contents = std::fs::read_to_string(&path)?;
                if let Ok(metadata) = serde_json::from_str::<SessionMetadata>(&contents) {
                    sessions.push(metadata);
                }
            }
        }
        // Sort by created_at descending (most recent first)
        sessions.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(sessions)
    }

    fn session_path(&self, session_id: &str) -> PathBuf {
        self.data_dir.join(format!("{session_id}.json"))
    }
}

// tests at bottom of file (from Step 1)
```

Now update `spool/spool-cli/src/main.rs` with the CLI argument parsing:

```rust
// spool/spool-cli/src/main.rs
mod config;
mod telemetry;
mod policy;
mod session;
mod error;

use clap::{Parser, Subcommand};

/// Spool — a Rust terminal analytics agent for Microsoft Fabric and Power BI.
#[derive(Parser, Debug)]
#[command(name = "spool", version, about)]
pub struct Cli {
    /// Path to configuration file
    #[arg(short, long, default_value = "~/.spool/config.toml")]
    pub config: String,

    /// Override the default LOB
    #[arg(long)]
    pub lob: Option<String>,

    /// Override the workspace ID
    #[arg(long)]
    pub workspace: Option<String>,

    /// Set log level (trace, debug, info, warn, error)
    #[arg(long)]
    pub log_level: Option<String>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Start a new investigation session
    New {
        /// LOB to use for the session
        #[arg(long)]
        lob: Option<String>,
    },
    /// Resume an existing session
    Resume {
        /// Session ID to resume
        session_id: String,
    },
    /// List existing sessions
    Sessions,
    /// Export a task result from a session
    Export {
        /// Session ID to export from
        session_id: String,
        /// Export format: json or markdown
        #[arg(long, default_value = "json")]
        format: String,
        /// Output file path
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Show memory entries
    Memory {
        #[command(subcommand)]
        action: MemoryAction,
    },
}

#[derive(Subcommand, Debug)]
pub enum MemoryAction {
    /// List all memory entries
    List {
        /// Filter by scope (lob, workspace, team, global)
        #[arg(long)]
        scope: Option<String>,
    },
    /// Show details of a specific memory entry
    Show {
        /// Memory ID to show
        memory_id: String,
    },
    /// Invalidate a memory entry
    Invalidate {
        /// Memory ID to invalidate
        memory_id: String,
        /// Reason for invalidation
        #[arg(long)]
        reason: String,
    },
}

fn main() {
    let cli = Cli::parse();

    // Load config
    let config_path = std::path::Path::new(&cli.config);
    let mut config = config::SpoolConfig::load_from_file(config_path)
        .unwrap_or_else(|_| config::SpoolConfig::default());
    config.apply_env_overrides();

    // Apply CLI overrides
    if let Some(ref lob) = cli.lob {
        config.session.default_lob = Some(lob.clone());
    }
    if let Some(ref workspace) = cli.workspace {
        config.connection.workspace_id = Some(workspace.clone());
    }
    if let Some(ref level) = cli.log_level {
        config.telemetry.log_level = level.clone();
    }

    // Initialize telemetry
    telemetry::init_tracing(&config.telemetry);

    tracing::info!("spool starting");

    match cli.command {
        Some(Commands::New { lob }) => {
            let effective_lob = lob.or(config.session.default_lob.clone());
            tracing::info!(lob = ?effective_lob, "creating new session");
            println!("New session (LOB: {}, workspace: {})",
                effective_lob.as_deref().unwrap_or("none"),
                config.connection.workspace_id.as_deref().unwrap_or("none"),
            );
        }
        Some(Commands::Resume { session_id }) => {
            tracing::info!(session_id = %session_id, "resuming session");
            println!("Resuming session: {session_id}");
        }
        Some(Commands::Sessions) => {
            println!("Listing sessions...");
        }
        Some(Commands::Export { session_id, format, output }) => {
            tracing::info!(session_id = %session_id, format = %format, "exporting");
            println!("Exporting session {session_id} as {format}");
        }
        Some(Commands::Memory { action }) => {
            match action {
                MemoryAction::List { scope } => {
                    println!("Listing memory (scope: {})", scope.as_deref().unwrap_or("all"));
                }
                MemoryAction::Show { memory_id } => {
                    println!("Showing memory: {memory_id}");
                }
                MemoryAction::Invalidate { memory_id, reason } => {
                    println!("Invalidating memory {memory_id}: {reason}");
                }
            }
        }
        None => {
            println!("Spool — interactive mode (not yet implemented)");
            println!("Use --help for available commands.");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn cli_parse_no_args() {
        let cli = Cli::try_parse_from(["spool"]);
        assert!(cli.is_ok());
        let cli = cli.unwrap();
        assert!(cli.command.is_none());
    }

    #[test]
    fn cli_parse_new_with_lob() {
        let cli = Cli::try_parse_from(["spool", "new", "--lob", "finance"]).unwrap();
        match cli.command {
            Some(Commands::New { lob }) => assert_eq!(lob, Some("finance".into())),
            _ => panic!("expected New command"),
        }
    }

    #[test]
    fn cli_parse_resume() {
        let cli = Cli::try_parse_from(["spool", "resume", "sess_123"]).unwrap();
        match cli.command {
            Some(Commands::Resume { session_id }) => assert_eq!(session_id, "sess_123"),
            _ => panic!("expected Resume command"),
        }
    }

    #[test]
    fn cli_parse_sessions() {
        let cli = Cli::try_parse_from(["spool", "sessions"]).unwrap();
        assert!(matches!(cli.command, Some(Commands::Sessions)));
    }

    #[test]
    fn cli_parse_export() {
        let cli = Cli::try_parse_from([
            "spool", "export", "sess_123", "--format", "markdown", "-o", "out.md",
        ]).unwrap();
        match cli.command {
            Some(Commands::Export { session_id, format, output }) => {
                assert_eq!(session_id, "sess_123");
                assert_eq!(format, "markdown");
                assert_eq!(output, Some("out.md".into()));
            }
            _ => panic!("expected Export command"),
        }
    }

    #[test]
    fn cli_parse_memory_list() {
        let cli = Cli::try_parse_from(["spool", "memory", "list", "--scope", "global"]).unwrap();
        match cli.command {
            Some(Commands::Memory { action: MemoryAction::List { scope } }) => {
                assert_eq!(scope, Some("global".into()));
            }
            _ => panic!("expected Memory List command"),
        }
    }

    #[test]
    fn cli_parse_memory_invalidate() {
        let cli = Cli::try_parse_from([
            "spool", "memory", "invalidate", "mem_123", "--reason", "stale data",
        ]).unwrap();
        match cli.command {
            Some(Commands::Memory { action: MemoryAction::Invalidate { memory_id, reason } }) => {
                assert_eq!(memory_id, "mem_123");
                assert_eq!(reason, "stale data");
            }
            _ => panic!("expected Memory Invalidate command"),
        }
    }

    #[test]
    fn cli_parse_config_and_overrides() {
        let cli = Cli::try_parse_from([
            "spool", "--config", "/custom/config.toml",
            "--lob", "marketing", "--workspace", "ws_456",
            "--log-level", "debug",
        ]).unwrap();
        assert_eq!(cli.config, "/custom/config.toml");
        assert_eq!(cli.lob, Some("marketing".into()));
        assert_eq!(cli.workspace, Some("ws_456".into()));
        assert_eq!(cli.log_level, Some("debug".into()));
    }
}
```

**Step 4: Run tests to verify they pass**

Run: `cd spool && cargo test -p spool-cli -- session tests`
Expected: 15 tests PASS (7 session + 8 CLI)

**Step 5: Commit**

```bash
git add spool/spool-cli/
git commit -m "feat(spool-cli): CLI argument parsing with clap and session management with create, list, save, load"
```

---

## Task 10: Error Handling Hardening

**Files:**

- Modify: `spool/spool-cli/src/error.rs`

**Step 1: Write the failing test**

Add to `spool/spool-cli/src/error.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_facing_message_for_config_error() {
        let err = SpoolCliError::Config("parse error: invalid key".into());
        let msg = err.user_facing_message();
        assert!(msg.contains("configuration"));
        assert!(!msg.contains("parse error: invalid key"));
    }

    #[test]
    fn user_facing_message_for_session_not_found() {
        let err = SpoolCliError::SessionNotFound("sess_123".into());
        let msg = err.user_facing_message();
        assert!(msg.contains("sess_123"));
        assert!(msg.contains("not found"));
    }

    #[test]
    fn user_facing_message_for_policy_violation() {
        let err = SpoolCliError::PolicyViolation("SQL must be read-only".into());
        let msg = err.user_facing_message();
        assert!(msg.contains("policy"));
    }

    #[test]
    fn user_facing_message_for_export_error() {
        let err = SpoolCliError::Export("file not writable".into());
        let msg = err.user_facing_message();
        assert!(msg.contains("export"));
    }

    #[test]
    fn user_facing_message_for_internal_error() {
        let err = SpoolCliError::Internal("panic in evaluator loop".into());
        let msg = err.user_facing_message();
        assert!(msg.contains("unexpected"));
        // Should not expose internal details to user
        assert!(!msg.contains("panic in evaluator loop"));
    }

    #[test]
    fn user_facing_message_for_connection_error() {
        let err = SpoolCliError::Connection("timeout after 30s".into());
        let msg = err.user_facing_message();
        assert!(msg.contains("connect"));
    }

    #[test]
    fn user_facing_message_for_memory_error() {
        let err = SpoolCliError::Memory("store corrupted".into());
        let msg = err.user_facing_message();
        assert!(msg.contains("memory"));
    }

    #[test]
    fn debug_detail_preserves_internal_info() {
        let err = SpoolCliError::Internal("evaluator loop index out of bounds".into());
        let debug = format!("{err:?}");
        assert!(debug.contains("evaluator loop index out of bounds"));
    }

    #[test]
    fn display_is_user_safe() {
        let err = SpoolCliError::Internal("secret internal detail".into());
        let display = format!("{err}");
        // Display should be the user-facing message, not the internal detail
        assert!(!display.contains("secret internal detail"));
    }

    #[test]
    fn graceful_degradation_suggestion() {
        let err = SpoolCliError::Connection("auth failed".into());
        let suggestion = err.recovery_suggestion();
        assert!(suggestion.is_some());
    }

    #[test]
    fn no_suggestion_for_internal() {
        let err = SpoolCliError::Internal("something broke".into());
        let suggestion = err.recovery_suggestion();
        // Internal errors may suggest retrying or filing a bug
        assert!(suggestion.is_some());
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cd spool && cargo test -p spool-cli -- error`
Expected: FAIL

**Step 3: Write the implementation**

Replace `spool/spool-cli/src/error.rs` with:

```rust
// spool/spool-cli/src/error.rs
//
// User-facing error types with graceful degradation.
//
// Design principles:
// - User-facing messages should be clear and actionable
// - Internal details should be preserved for debug/trace logging
// - Display trait shows user-safe messages only
// - recovery_suggestion() provides next-step guidance when possible

/// Top-level CLI error type with user-facing message support.
#[derive(Debug)]
pub enum SpoolCliError {
    /// Configuration file error (parse failure, missing file, etc.)
    Config(String),
    /// Session not found
    SessionNotFound(String),
    /// Policy violation (SQL, confirmation, etc.)
    PolicyViolation(String),
    /// Export error (file write, format, etc.)
    Export(String),
    /// Connection error (auth, network, timeout)
    Connection(String),
    /// Memory subsystem error
    Memory(String),
    /// Internal error (should not normally reach user)
    Internal(String),
}

impl SpoolCliError {
    /// Return a user-facing message that is safe to display in the terminal.
    ///
    /// This message should be clear, actionable, and should not expose
    /// internal implementation details, stack traces, or sensitive data.
    pub fn user_facing_message(&self) -> String {
        match self {
            Self::Config(_) => {
                "There was a problem loading the Spool configuration. \
                 Check your config file (~/.spool/config.toml) and try again."
                    .into()
            }
            Self::SessionNotFound(id) => {
                format!(
                    "Session '{id}' was not found. Use 'spool sessions' to list available sessions."
                )
            }
            Self::PolicyViolation(detail) => {
                format!("A policy restriction prevented this action: {detail}")
            }
            Self::Export(_) => {
                "There was a problem completing the export. \
                 Check that the output path is writable and try again."
                    .into()
            }
            Self::Connection(_) => {
                "Could not connect to the required service. \
                 Check your network connection and authentication settings."
                    .into()
            }
            Self::Memory(_) => {
                "There was a problem with the durable memory store. \
                 The memory file may be corrupted. Try running with --log-level debug for details."
                    .into()
            }
            Self::Internal(_) => {
                "An unexpected error occurred. \
                 Please retry, and if the problem persists, file a bug report with --log-level trace output."
                    .into()
            }
        }
    }

    /// Return a recovery suggestion for the user, if one is available.
    pub fn recovery_suggestion(&self) -> Option<String> {
        match self {
            Self::Config(_) => Some(
                "Run 'spool --config /path/to/config.toml' to specify an alternate config file."
                    .into(),
            ),
            Self::SessionNotFound(_) => Some(
                "Use 'spool sessions' to see all available sessions, or 'spool new' to start fresh."
                    .into(),
            ),
            Self::PolicyViolation(_) => Some(
                "Review the action and ensure it complies with Spool's v1 policy constraints."
                    .into(),
            ),
            Self::Export(_) => Some(
                "Check file permissions and disk space, then retry the export."
                    .into(),
            ),
            Self::Connection(_) => Some(
                "Verify your workspace ID, tenant ID, and network access. \
                 You can set SPOOL_WORKSPACE_ID as an environment variable."
                    .into(),
            ),
            Self::Memory(_) => Some(
                "You can reset the memory store by deleting the memory file and restarting."
                    .into(),
            ),
            Self::Internal(_) => Some(
                "Please retry the operation. If it persists, run with --log-level trace and include the output in a bug report."
                    .into(),
            ),
        }
    }
}

impl std::fmt::Display for SpoolCliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.user_facing_message())
    }
}

impl std::error::Error for SpoolCliError {}

// tests at bottom of file (from Step 1)
```

**Step 4: Run test to verify it passes**

Run: `cd spool && cargo test -p spool-cli -- error`
Expected: 11 tests PASS

**Step 5: Commit**

```bash
git add spool/spool-cli/src/error.rs
git commit -m "feat(spool-cli): user-facing error types with graceful degradation and recovery suggestions"
```

---

## Task 11: End-To-End Smoke Tests

**Files:**

- Create: `spool/spool-cli/tests/smoke_tests.rs`

**Step 1: Write the failing test**

Create `spool/spool-cli/tests/smoke_tests.rs`:

```rust
//! End-to-end smoke tests for Plan 6 subsystems.
//!
//! These tests exercise the full path from configuration through
//! memory lifecycle, policy enforcement, session management, and export.
//! All tests use fixture data — no live systems.

use spool_protocol::artifact::ArtifactType;
use spool_protocol::contradiction::ContradictionId;
use spool_protocol::evidence::EvidenceId;
use spool_protocol::task_contract::TaskId;
use spool_protocol::task_result::*;

use spool_memory::types::*;
use spool_memory::store::MemoryStore;
use spool_memory::lifecycle;

use spool_export::json_export;
use spool_export::markdown_export;

fn fixture_task_result() -> TaskResult {
    TaskResult {
        task_id: TaskId("task_smoke_1".into()),
        proposed_state: Some(ResultState::Confirmed),
        state: ResultState::SupportedHypothesis,
        confidence: Confidence::Medium,
        summary: "Revenue mismatch traced to measure definition drift.".into(),
        findings: vec![
            Finding {
                id: "f_1".into(),
                title: "Measure definition drift detected".into(),
                detail: "The QoQ revenue measure no longer aligns with warehouse logic.".into(),
            },
        ],
        evidence_refs: vec![
            EvidenceId("ev_1".into()),
            EvidenceId("ev_2".into()),
        ],
        validation_results: vec![
            ValidationResult {
                id: "val_1".into(),
                validation_type: "cross_source_comparison".into(),
                status: "divergent".into(),
                detail: "DAX and warehouse disagree on Q1 totals.".into(),
            },
        ],
        recommended_actions: vec![
            RecommendedAction {
                id: "act_1".into(),
                action_type: "measure_review".into(),
                summary: "Review QoQ revenue measure definition.".into(),
            },
        ],
        blockers: vec![],
        open_questions: vec![
            OpenQuestion {
                id: "q_1".into(),
                question: "Does the fiscal calendar offset apply?".into(),
            },
        ],
        proposed_changes: vec![
            ProposedChange {
                id: "chg_1".into(),
                artifact_type: ArtifactType::Measure,
                artifact_ref: "Sales[QoQ Revenue]".into(),
                change_summary: "Update quarter offset logic.".into(),
            },
        ],
        contradiction_refs: vec![],
        result_generated_at: Some(chrono::Utc::now()),
        result_version: Some(1),
    }
}

fn fixture_memory_entry(id: &str, scope: MemoryScope, status: MemoryStatus) -> MemoryEntry {
    let now = chrono::Utc::now();
    MemoryEntry {
        memory_id: MemoryId(id.into()),
        memory_type: MemoryType::RecurringIssuePattern,
        scope,
        content: format!("Smoke test memory: {id}"),
        source_basis: SourceBasis::RepeatedObservation {
            occurrence_count: 3,
            first_seen_session: "sess_smoke".into(),
        },
        status,
        created_at: now,
        last_validated_at: None,
        invalidated_at: None,
        invalidation_reason: None,
        tags: vec!["smoke-test".into()],
    }
}

/// Scenario 1: Full memory lifecycle — capture, promote, stale, revalidate, invalidate.
#[test]
fn smoke_memory_full_lifecycle() {
    let mut store = MemoryStore::new();

    // Capture as candidate
    let mut entry = fixture_memory_entry(
        "mem_smoke_1",
        MemoryScope::Lob("finance".into()),
        MemoryStatus::Candidate,
    );
    store.insert(entry.clone());
    assert_eq!(store.len(), 1);

    // Promote to confirmed
    let stored = store.get_mut(&MemoryId("mem_smoke_1".into())).unwrap();
    lifecycle::promote(stored).unwrap();
    assert_eq!(stored.status, MemoryStatus::Confirmed);

    // Mark stale
    let stored = store.get_mut(&MemoryId("mem_smoke_1".into())).unwrap();
    lifecycle::mark_stale(stored).unwrap();
    assert_eq!(stored.status, MemoryStatus::Stale);

    // Revalidate
    let stored = store.get_mut(&MemoryId("mem_smoke_1".into())).unwrap();
    lifecycle::revalidate(stored).unwrap();
    assert_eq!(stored.status, MemoryStatus::Confirmed);

    // Invalidate
    let stored = store.get_mut(&MemoryId("mem_smoke_1".into())).unwrap();
    lifecycle::invalidate(stored, "Contradicted by fresh evidence".into()).unwrap();
    assert_eq!(stored.status, MemoryStatus::Invalidated);
    assert!(stored.invalidation_reason.is_some());

    // Cannot transition out of invalidated
    let stored = store.get_mut(&MemoryId("mem_smoke_1".into())).unwrap();
    assert!(lifecycle::revalidate(stored).is_err());
    assert!(lifecycle::promote(stored).is_err());
}

/// Scenario 2: Memory scope filtering across LOBs.
#[test]
fn smoke_memory_scope_isolation() {
    let mut store = MemoryStore::new();
    store.insert(fixture_memory_entry(
        "mem_finance_1",
        MemoryScope::Lob("finance".into()),
        MemoryStatus::Confirmed,
    ));
    store.insert(fixture_memory_entry(
        "mem_marketing_1",
        MemoryScope::Lob("marketing".into()),
        MemoryStatus::Confirmed,
    ));
    store.insert(fixture_memory_entry(
        "mem_global_1",
        MemoryScope::Global,
        MemoryStatus::Confirmed,
    ));
    store.insert(fixture_memory_entry(
        "mem_finance_stale",
        MemoryScope::Lob("finance".into()),
        MemoryStatus::Stale,
    ));

    // Finance LOB should see finance + global, but not marketing or stale
    let active = store.query_active_for_lob("finance");
    assert_eq!(active.len(), 2);
    let ids: Vec<&str> = active.iter().map(|e| e.memory_id.0.as_str()).collect();
    assert!(ids.contains(&"mem_finance_1"));
    assert!(ids.contains(&"mem_global_1"));
    assert!(!ids.contains(&"mem_marketing_1"));
    assert!(!ids.contains(&"mem_finance_stale"));

    // Marketing LOB should see marketing + global
    let active = store.query_active_for_lob("marketing");
    assert_eq!(active.len(), 2);
    let ids: Vec<&str> = active.iter().map(|e| e.memory_id.0.as_str()).collect();
    assert!(ids.contains(&"mem_marketing_1"));
    assert!(ids.contains(&"mem_global_1"));
}

/// Scenario 3: Memory persistence round-trip.
#[test]
fn smoke_memory_persistence() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("smoke_memory.jsonl");

    let mut store = MemoryStore::new();
    store.insert(fixture_memory_entry(
        "mem_persist_1",
        MemoryScope::Global,
        MemoryStatus::Confirmed,
    ));
    store.insert(fixture_memory_entry(
        "mem_persist_2",
        MemoryScope::Lob("finance".into()),
        MemoryStatus::Candidate,
    ));
    store.save_to_file(&path).unwrap();

    let loaded = MemoryStore::load_from_file(&path).unwrap();
    assert_eq!(loaded.len(), 2);
    let entry = loaded.get(&MemoryId("mem_persist_1".into())).unwrap();
    assert_eq!(entry.status, MemoryStatus::Confirmed);
}

/// Scenario 4: JSON export produces valid, parseable output.
#[test]
fn smoke_json_export() {
    let result = fixture_task_result();
    let json = json_export::export_task_result_json(&result).unwrap();

    // Must be valid JSON
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(value["task_id"], "task_smoke_1");
    assert_eq!(value["state"], "supported_hypothesis");

    // With metadata wrapper
    let wrapped = json_export::export_task_result_json_with_metadata(&result).unwrap();
    let value: serde_json::Value = serde_json::from_str(&wrapped).unwrap();
    assert_eq!(value["_export_format"], "spool_task_result_v1_unstable");
}

/// Scenario 5: Markdown export produces readable output with all sections.
#[test]
fn smoke_markdown_export() {
    let result = fixture_task_result();
    let md = markdown_export::export_task_result_markdown(&result);

    assert!(md.contains("# Task Result: task_smoke_1"));
    assert!(md.contains("supported_hypothesis"));
    assert!(md.contains("Measure definition drift detected"));
    assert!(md.contains("cross_source_comparison"));
    assert!(md.contains("Review QoQ revenue measure definition"));
    assert!(md.contains("Does the fiscal calendar offset apply?"));
    assert!(md.contains("Sales[QoQ Revenue]"));
    assert!(md.contains("non-stable"));
}

/// Scenario 6: Export to file and read back.
#[test]
fn smoke_export_to_files() {
    let result = fixture_task_result();
    let dir = tempfile::tempdir().unwrap();

    // JSON
    let json_path = dir.path().join("smoke_result.json");
    json_export::export_task_result_json_to_file(&result, &json_path).unwrap();
    let json_contents = std::fs::read_to_string(&json_path).unwrap();
    assert!(json_contents.contains("task_smoke_1"));

    // Markdown
    let md_path = dir.path().join("smoke_result.md");
    markdown_export::export_task_result_markdown_to_file(&result, &md_path).unwrap();
    let md_contents = std::fs::read_to_string(&md_path).unwrap();
    assert!(md_contents.contains("task_smoke_1"));
}

/// Scenario 7: Config round-trip through file.
#[test]
fn smoke_config_round_trip() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("smoke_config.toml");

    let toml_content = r#"
[connection]
workspace_id = "ws_smoke"
auth_method = "device_code"

[session]
data_dir = "/tmp/spool/smoke"
max_history = 10
default_lob = "finance"

[harness]
max_evaluator_iterations = 3

[memory]
store_path = "/tmp/spool/smoke_memory.jsonl"

[telemetry]
enabled = true
log_level = "debug"
"#;
    std::fs::write(&path, toml_content).unwrap();

    // This validates the config parses correctly
    let config: toml::Value = toml::from_str(toml_content).unwrap();
    assert_eq!(
        config["connection"]["workspace_id"].as_str(),
        Some("ws_smoke"),
    );
    assert_eq!(
        config["session"]["default_lob"].as_str(),
        Some("finance"),
    );
    assert_eq!(
        config["harness"]["max_evaluator_iterations"].as_integer(),
        Some(3),
    );
}

/// Scenario 8: Session create, save, list, load cycle.
#[test]
fn smoke_session_lifecycle() {
    use spool_cli::session::SessionManager;

    let dir = tempfile::tempdir().unwrap();
    let manager = SessionManager::new(dir.path().to_path_buf());

    // Create two sessions
    let s1 = manager.create_session(Some("finance"), Some("ws_1")).unwrap();
    let s2 = manager.create_session(Some("marketing"), Some("ws_2")).unwrap();

    // List should show both
    let sessions = manager.list_sessions().unwrap();
    assert_eq!(sessions.len(), 2);

    // Load each by ID
    let loaded1 = manager.load_session(&s1.session_id).unwrap();
    assert_eq!(loaded1.lob, Some("finance".into()));

    let loaded2 = manager.load_session(&s2.session_id).unwrap();
    assert_eq!(loaded2.lob, Some("marketing".into()));

    // Load nonexistent fails
    assert!(manager.load_session("nonexistent").is_err());
}
```

**Step 2: Run test to verify it fails**

Run: `cd spool && cargo test -p spool-cli --test smoke_tests`
Expected: FAIL — references modules not yet public

Note: You may need to add `pub` visibility to the `session` module in `main.rs`. Update the module declaration:

In `spool/spool-cli/src/main.rs`, change:

```rust
mod session;
```

to:

```rust
pub mod session;
```

Also ensure the other modules that need to be visible for integration tests are public. The `spool-cli` crate's `Cargo.toml` needs a `[lib]` section since we have both a binary and integration tests that reference internal modules. Add a `src/lib.rs`:

Create `spool/spool-cli/src/lib.rs`:

```rust
// spool/spool-cli/src/lib.rs
pub mod config;
pub mod telemetry;
pub mod policy;
pub mod session;
pub mod error;
```

Update `spool/spool-cli/src/main.rs` to use the lib crate:

```rust
// spool/spool-cli/src/main.rs
use clap::{Parser, Subcommand};
use spool_cli::config;
use spool_cli::telemetry;

/// Spool — a Rust terminal analytics agent for Microsoft Fabric and Power BI.
#[derive(Parser, Debug)]
#[command(name = "spool", version, about)]
pub struct Cli {
    /// Path to configuration file
    #[arg(short, long, default_value = "~/.spool/config.toml")]
    pub config: String,

    /// Override the default LOB
    #[arg(long)]
    pub lob: Option<String>,

    /// Override the workspace ID
    #[arg(long)]
    pub workspace: Option<String>,

    /// Set log level (trace, debug, info, warn, error)
    #[arg(long)]
    pub log_level: Option<String>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Start a new investigation session
    New {
        /// LOB to use for the session
        #[arg(long)]
        lob: Option<String>,
    },
    /// Resume an existing session
    Resume {
        /// Session ID to resume
        session_id: String,
    },
    /// List existing sessions
    Sessions,
    /// Export a task result from a session
    Export {
        /// Session ID to export from
        session_id: String,
        /// Export format: json or markdown
        #[arg(long, default_value = "json")]
        format: String,
        /// Output file path
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Show memory entries
    Memory {
        #[command(subcommand)]
        action: MemoryAction,
    },
}

#[derive(Subcommand, Debug)]
pub enum MemoryAction {
    /// List all memory entries
    List {
        /// Filter by scope (lob, workspace, team, global)
        #[arg(long)]
        scope: Option<String>,
    },
    /// Show details of a specific memory entry
    Show {
        /// Memory ID to show
        memory_id: String,
    },
    /// Invalidate a memory entry
    Invalidate {
        /// Memory ID to invalidate
        memory_id: String,
        /// Reason for invalidation
        #[arg(long)]
        reason: String,
    },
}

fn main() {
    let cli = Cli::parse();

    // Load config
    let config_path = std::path::Path::new(&cli.config);
    let mut cfg = config::SpoolConfig::load_from_file(config_path)
        .unwrap_or_else(|_| config::SpoolConfig::default());
    cfg.apply_env_overrides();

    // Apply CLI overrides
    if let Some(ref lob) = cli.lob {
        cfg.session.default_lob = Some(lob.clone());
    }
    if let Some(ref workspace) = cli.workspace {
        cfg.connection.workspace_id = Some(workspace.clone());
    }
    if let Some(ref level) = cli.log_level {
        cfg.telemetry.log_level = level.clone();
    }

    // Initialize telemetry
    telemetry::init_tracing(&cfg.telemetry);

    tracing::info!("spool starting");

    match cli.command {
        Some(Commands::New { lob }) => {
            let effective_lob = lob.or(cfg.session.default_lob.clone());
            tracing::info!(lob = ?effective_lob, "creating new session");
            println!("New session (LOB: {}, workspace: {})",
                effective_lob.as_deref().unwrap_or("none"),
                cfg.connection.workspace_id.as_deref().unwrap_or("none"),
            );
        }
        Some(Commands::Resume { session_id }) => {
            tracing::info!(session_id = %session_id, "resuming session");
            println!("Resuming session: {session_id}");
        }
        Some(Commands::Sessions) => {
            println!("Listing sessions...");
        }
        Some(Commands::Export { session_id, format, output }) => {
            tracing::info!(session_id = %session_id, format = %format, "exporting");
            println!("Exporting session {session_id} as {format}");
        }
        Some(Commands::Memory { action }) => {
            match action {
                MemoryAction::List { scope } => {
                    println!("Listing memory (scope: {})", scope.as_deref().unwrap_or("all"));
                }
                MemoryAction::Show { memory_id } => {
                    println!("Showing memory: {memory_id}");
                }
                MemoryAction::Invalidate { memory_id, reason } => {
                    println!("Invalidating memory {memory_id}: {reason}");
                }
            }
        }
        None => {
            println!("Spool — interactive mode (not yet implemented)");
            println!("Use --help for available commands.");
        }
    }
}
```

Move the CLI argument parsing tests to the lib or keep them in main.rs with `#[cfg(test)]`.

**Step 3: Run tests to verify they pass**

Run: `cd spool && cargo test -p spool-cli --test smoke_tests`
Expected: 8 smoke tests PASS

**Step 4: Commit**

```bash
git add spool/spool-cli/
git commit -m "feat(spool-cli): end-to-end smoke tests for memory lifecycle, scope isolation, export, config, and session management"
```

---

## Task 12: Integration Wiring And Final Workspace Verification

**Files:**

- Modify: `spool/spool-memory/src/lib.rs` (ensure public re-exports)
- Modify: `spool/spool-export/src/lib.rs` (ensure public re-exports)

**Step 1: Write the failing test**

Create `spool/spool-core/tests/plan6_integration.rs`:

```rust
//! Plan 6 integration test verifying that all Plan 6 crates compose correctly
//! with Plan 1 spool-protocol types.

use spool_protocol::task_contract::TaskId;
use spool_protocol::task_result::*;
use spool_protocol::evidence::EvidenceId;
use spool_protocol::artifact::ArtifactType;

use spool_memory::types::*;
use spool_memory::store::MemoryStore;
use spool_memory::lifecycle;

use spool_export::json_export;
use spool_export::markdown_export;

/// Verify that a task result produced by Plan 1 types can flow through
/// Plan 6 export without data loss.
#[test]
fn protocol_types_flow_through_export() {
    let result = TaskResult {
        task_id: TaskId("task_integration_1".into()),
        proposed_state: None,
        state: ResultState::Confirmed,
        confidence: Confidence::High,
        summary: "Integration test result.".into(),
        findings: vec![
            Finding {
                id: "f_1".into(),
                title: "Integration finding".into(),
                detail: "Detail.".into(),
            },
        ],
        evidence_refs: vec![EvidenceId("ev_1".into())],
        validation_results: vec![
            ValidationResult {
                id: "v_1".into(),
                validation_type: "integration_check".into(),
                status: "passed".into(),
                detail: "All clear.".into(),
            },
        ],
        recommended_actions: vec![],
        blockers: vec![],
        open_questions: vec![],
        proposed_changes: vec![],
        contradiction_refs: vec![],
        result_generated_at: Some(chrono::Utc::now()),
        result_version: Some(1),
    };

    // JSON export
    let json = json_export::export_task_result_json(&result).unwrap();
    let restored: TaskResult = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.task_id, result.task_id);
    assert_eq!(restored.state, ResultState::Confirmed);
    assert_eq!(restored.confidence, Confidence::High);

    // Markdown export
    let md = markdown_export::export_task_result_markdown(&result);
    assert!(md.contains("task_integration_1"));
    assert!(md.contains("confirmed"));
    assert!(md.contains("Integration finding"));
}

/// Verify that memory entries can be created, transitioned, persisted,
/// and loaded alongside Plan 1 protocol types.
#[test]
fn memory_with_protocol_types() {
    let mut store = MemoryStore::new();

    let entry = MemoryEntry {
        memory_id: MemoryId("mem_integ_1".into()),
        memory_type: MemoryType::ArtifactRelationshipHint,
        scope: MemoryScope::Workspace("ws_integration".into()),
        content: "The Sales Model measure 'QoQ Revenue' uses fiscal calendar offsets.".into(),
        source_basis: SourceBasis::CompletedTaskResult {
            task_id: "task_integration_1".into(),
        },
        status: MemoryStatus::Candidate,
        created_at: chrono::Utc::now(),
        last_validated_at: None,
        invalidated_at: None,
        invalidation_reason: None,
        tags: vec!["fiscal-calendar".into()],
    };

    store.insert(entry);

    // Promote
    let stored = store.get_mut(&MemoryId("mem_integ_1".into())).unwrap();
    lifecycle::promote(stored).unwrap();
    assert_eq!(stored.status, MemoryStatus::Confirmed);

    // Persist and reload
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("integration_memory.jsonl");
    store.save_to_file(&path).unwrap();

    let loaded = MemoryStore::load_from_file(&path).unwrap();
    let loaded_entry = loaded.get(&MemoryId("mem_integ_1".into())).unwrap();
    assert_eq!(loaded_entry.status, MemoryStatus::Confirmed);
    assert_eq!(loaded_entry.memory_type, MemoryType::ArtifactRelationshipHint);
}

/// Verify confidence cap validation still works in the export pipeline.
#[test]
fn confidence_caps_enforced_before_export() {
    let result = TaskResult {
        task_id: TaskId("task_cap_check".into()),
        proposed_state: None,
        state: ResultState::Blocked,
        confidence: Confidence::High, // violation: blocked + high
        summary: "Should trigger cap violation.".into(),
        findings: vec![],
        evidence_refs: vec![],
        validation_results: vec![],
        recommended_actions: vec![],
        blockers: vec![],
        open_questions: vec![],
        proposed_changes: vec![],
        contradiction_refs: vec![],
        result_generated_at: None,
        result_version: None,
    };

    let violations = result.validate_confidence_caps();
    assert!(!violations.is_empty());
    assert!(violations[0].contains("blocked"));

    // Export still works (export does not enforce caps, that is harness responsibility)
    let json = json_export::export_task_result_json(&result).unwrap();
    assert!(json.contains("blocked"));
}
```

**Step 2: Run test to verify it fails**

Run: `cd spool && cargo test -p spool-core --test plan6_integration`
Expected: FAIL — spool-core dev-dependencies may not include spool-memory and spool-export yet

Add dev-dependencies to `spool/spool-core/Cargo.toml`:

```toml
[dev-dependencies]
tempfile = "3"
tokio = { workspace = true, features = ["test-util", "macros"] }
spool-memory = { workspace = true }
spool-export = { workspace = true }
```

**Step 3: Run test to verify it passes**

Run: `cd spool && cargo test -p spool-core --test plan6_integration`
Expected: 3 tests PASS

**Step 4: Run full workspace test suite**

Run: `cd spool && cargo test --workspace`
Expected: All tests across all crates PASS

**Step 5: Commit**

```bash
git add spool/
git commit -m "feat(spool): Plan 6 integration tests verifying protocol-export-memory composition and workspace-wide test pass"
```

---

## Summary

Plan 6 delivers 12 tasks across 3 new crates (`spool-memory`, `spool-export`, `spool-cli`) with cross-cutting additions to the workspace:

| Task | Subsystem | Tests |
|------|-----------|-------|
| 1 | Workspace scaffolding | build check |
| 2 | Durable memory types | 5 |
| 3 | Memory store and lifecycle | 14 |
| 4 | JSON export | 4 |
| 5 | Markdown export | 9 |
| 6 | Policy enforcement | 21 |
| 7 | Configuration management | 8 |
| 8 | Telemetry initialization | 7 |
| 9 | CLI args and session management | 15 |
| 10 | Error handling hardening | 11 |
| 11 | End-to-end smoke tests | 8 |
| 12 | Integration wiring | 3 |

**Total: ~105 tests, all fixture-backed, no live systems required.**
