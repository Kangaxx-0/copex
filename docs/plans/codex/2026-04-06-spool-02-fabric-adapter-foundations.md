# Spool Fabric Adapter Foundations Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the Fabric adapter foundation for Spool, including config-driven startup, auth and token restoration, structured capability declarations, explicit workspace and artifact resolution, and metadata inspection paths that feed the canonical harness contracts.

**Architecture:** This plan introduces a dedicated `spool-fabric` crate for Fabric-specific config, auth, HTTP, capability, resolution, and metadata inspection seams, plus a thin `spool` app crate for config-backed startup. It keeps platform-neutral task and evidence semantics in `spool-model` and `spool-core`, while proving the adapter through fixture-backed tests first and a live dev-workspace smoke path second.

**Tech Stack:** Rust 2024, Clap, Reqwest, Serde, serde_json, toml, Tokio, keyring, thiserror, pretty_assertions

---

## Context For The Implementer

This is the first live-platform plan. It consumes Plan 1’s canonical contract layer without redefining it.

The governing references are:

- `docs/superpowers/specs/2026-04-06-spool-refined-spec.md`
- `docs/superpowers/specs/2026-04-06-spool-dev-planning-readiness-design.md`
- `docs/plans/codex/2026-04-06-spool-01-harness-semantics-foundation.md`
- `docs/plans/codex/2026-04-06-spool-plan-index.md`

The adapter must support the v1 Fabric artifact set:

- report
- page
- visual
- semantic model
- measure
- table
- column
- relationship
- warehouse

It must also express ambiguous and unresolved resolution outcomes explicitly rather than hiding them inside `Option`.

## Out Of Scope

- DAX execution
- warehouse query execution
- recipe-driven planner behavior
- TUI rendering
- Fabric-side mutations
- durable-memory reuse

## Dependencies

- `docs/plans/codex/2026-04-06-spool-01-harness-semantics-foundation.md`

## Contract Impact

This plan implements Fabric-specific mappings for:

- capability declarations
- config-backed auth and token restoration
- workspace resolution
- artifact resolution
- report and semantic-model metadata inspection

This plan should not weaken the platform-neutral artifact, evidence, or task contracts from Plan 1.

## Requirement Traceability

| Requirement source | Required behavior in this plan | Planned coverage |
|---|---|---|
| Refined spec sections 2.4 and 3.2-3.5 | Fabric-only runtime behavior must still map into the platform-neutral artifact identity model | Tasks 4-7 define explicit resolution outcomes, artifact mappings, and metadata envelopes keyed by canonical identity |
| Refined spec section 3.1 | Users speak in report, page, visual, semantic-model, and warehouse vocabulary rather than raw IDs | Tasks 4-6 define scoped resolution behavior for human-facing names, parent-derived lookups, and GUID-backed inputs |
| Refined spec section 3.5 | Ambiguous or unresolved matches must surface structured outcomes rather than continue silently | Task 4 defines `resolved | ambiguous | unresolved` outcomes and Tasks 5-6 apply them across report-side and model-side artifacts |
| Planning spec section 5 | Later plans require an explicit real integration-validation path | Task 8 runs the live auth and metadata smoke path against the shared config and fixture contract |
| Plan index shared config contract | Live tests must use config-backed fixture values and token sources rather than shell-only discovery | Tasks 1, 3, and 8 define config loading, auth restoration, and the live read path from `~/.config/spool/dev.toml` or `spool --config <path>` |

## Execution Invariants

- `spool-fabric` owns Fabric transport and normalization only. It must not redefine Plan 1 task, evidence, or result semantics.
- Every successful resolution path must emit a canonical `ArtifactIdentity` with the strongest available `resolution_basis`.
- Ambiguity is a first-class outcome. If multiple plausible matches exist inside scope, the adapter returns structured candidates and stops there.
- Metadata inspection returns canonical envelopes that later plans can turn into evidence without scraping transport-specific JSON blobs.
- Auth restoration must honor the shared config contract first: the primary token environment variable comes from `access_token_env`, and any fallback env name is only a compatibility escape hatch.
- Capability declarations must describe what the current adapter/runtime can do, not what Fabric might support someday.

## Live Fixture Inputs And Success Conditions

All live steps in this plan use the shared fixture set from the plan index:

- workspace fixture: `workspace_name` and optional `workspace_id`
- report-side fixture path: `report_name` or `report_id`, plus `page_name` and `visual_name`
- semantic-model fixture path: `semantic_model_name` or `semantic_model_id`
- warehouse fixture presence only for resolution completeness; no query execution belongs in this plan
- token source: environment variable named by `access_token_env`

Task 8 is complete only when one live run proves all of the following:

- config loading found the shared fixture values
- auth restoration produced a usable bearer token without ad hoc shell scraping
- workspace resolution succeeded
- at least one report-side and one semantic-model-side fixture resolved into canonical identities
- metadata envelopes came back keyed by those identities and were ready for later evidence projection

## Handoff Artifacts For Later Plans

- config types and CLI loading behavior for the shared Fabric dev environment
- auth/session and HTTP client primitives reused by validation and index-building work
- stable resolution outcome types and metadata envelope types consumed by Plans 3-5
- an adapter architecture note recording resolution rules, ambiguity handling, and capability boundaries

## Integration Validation

Real validation gate:

- use the config-backed dev Fabric workspace
- acquire or restore auth through the adapter
- resolve one known workspace
- resolve at least one report-side artifact and one semantic-model-side artifact
- fetch metadata for those artifacts through the adapter
- prove the harness can ingest the observed metadata as canonical evidence without adapter-specific translation hacks

## Open Items / Deferred Decisions

### Owned By This Plan

- exact auth-client boundary between keyring, env, and runtime token use
- exact capability-contract field set for v1 Fabric
- exact fixture strategy for deterministic adapter tests
- exact metadata fetch split between report-side and semantic-model-side clients

### Deferred To Later Plans

- DAX transport
- warehouse SQL transport
- TUI interaction model
- planner recipe-selection behavior
- any write-path or MCP dependency for future mutation workflows

### Review Triggers

- if Fabric auth requires state that cannot be represented through config plus keyring-backed token restoration
- if one or more artifact kinds cannot be normalized into the Plan 1 artifact identity model
- if real API behavior requires a different ambiguity model than `resolved | ambiguous | unresolved`
- if metadata inspection requires capability fields not represented in the adapter contract

## File Structure

| Path | Responsibility |
|---|---|
| `spool/spool/Cargo.toml` | app manifest |
| `spool/spool/src/main.rs` | entrypoint and smoke commands |
| `spool/spool/src/config.rs` | config-file loading |
| `spool/spool-fabric/Cargo.toml` | Fabric adapter manifest |
| `spool/spool-fabric/src/lib.rs` | crate exports |
| `spool/spool-fabric/src/config.rs` | Fabric config types |
| `spool/spool-fabric/src/auth.rs` | token source, token cache, and auth session state |
| `spool/spool-fabric/src/client.rs` | shared HTTP client and auth injection |
| `spool/spool-fabric/src/capabilities.rs` | structured capability contract |
| `spool/spool-fabric/src/workspaces.rs` | workspace resolution |
| `spool/spool-fabric/src/artifacts.rs` | artifact resolution and disambiguation types |
| `spool/spool-fabric/src/metadata.rs` | metadata inspection requests and responses |
| `spool/spool-fabric/tests/config_load.rs` | config-load tests |
| `spool/spool-fabric/tests/capabilities.rs` | capability contract tests |
| `spool/spool-fabric/tests/resolution_fixtures.rs` | fixture-backed resolution tests |
| `spool/spool-fabric/tests/live_read_smoke.rs` | ignored live smoke test |
| `spool/docs/architecture/fabric-adapter.md` | architecture note for adapter boundaries |

### Task 1: Add The CLI And Fabric Crate Skeleton

**Files:**
- Modify: `spool/Cargo.toml`
- Create: `spool/spool/Cargo.toml`
- Create: `spool/spool/src/main.rs`
- Create: `spool/spool/src/config.rs`
- Create: `spool/spool-fabric/Cargo.toml`
- Create: `spool/spool-fabric/src/lib.rs`
- Create: `spool/spool-fabric/src/config.rs`
- Create: `spool/spool-fabric/tests/config_load.rs`

- [ ] **Step 1: Write the failing config smoke test**

Create `spool/spool-fabric/tests/config_load.rs`:

```rust
use spool_fabric::FabricConfig;

#[test]
fn fabric_config_roundtrips_with_workspace_and_tenant() {
    let raw = r#"
tenant_id = "tenant"
client_id = "client"
workspace_name = "Spool Dev"
"#;

    let parsed: FabricConfig = toml::from_str(raw).unwrap();

    assert_eq!(parsed.workspace_name, "Spool Dev");
    assert_eq!(parsed.tenant_id, "tenant");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:
```bash
cd spool
cargo test -p spool-fabric fabric_config_roundtrips_with_workspace_and_tenant
```

Expected: FAIL because the crate does not exist.

- [ ] **Step 3: Implement the crate skeleton**

Create `spool/spool-fabric/src/config.rs`:

```rust
use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct FabricConfig {
    pub tenant_id: String,
    pub client_id: String,
    pub workspace_name: String,
    pub workspace_id: Option<String>,
    pub report_name: Option<String>,
    pub page_name: Option<String>,
    pub visual_name: Option<String>,
    pub semantic_model_name: Option<String>,
    pub semantic_model_id: Option<String>,
    pub warehouse_name: Option<String>,
    pub warehouse_dsn: Option<String>,
    pub access_token_env: Option<String>,
}

impl FabricConfig {
    pub fn load_from_path(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let raw = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&raw)?)
    }
}
```

Create `spool/spool-fabric/src/lib.rs`:

```rust
mod config;

pub use config::FabricConfig;
```

Create `spool/spool/src/config.rs`:

```rust
use std::fs;
use std::path::Path;

use spool_fabric::FabricConfig;

pub fn load_fabric_config(path: &Path) -> FabricConfig {
    let _ = fs::metadata(path).unwrap();
    FabricConfig::load_from_path(path).unwrap()
}
```

Create `spool/spool/src/main.rs`:

```rust
use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
struct Args {
    #[arg(long)]
    config: PathBuf,
}

fn main() {
    let args = Args::parse();
    let config = crate::config::load_fabric_config(&args.config);
    println!("{}", config.workspace_name);
}

mod config;
```

- [ ] **Step 4: Run test to verify it passes**

Run:
```bash
cd spool
cargo test -p spool-fabric fabric_config_roundtrips_with_workspace_and_tenant
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
cd spool
git add Cargo.toml spool spool-fabric
git commit -m "feat: add spool fabric adapter skeleton"
```

### Task 2: Define The Structured Capability Contract

**Files:**
- Create: `spool/spool-fabric/src/capabilities.rs`
- Modify: `spool/spool-fabric/src/lib.rs`
- Create: `spool/spool-fabric/tests/capabilities.rs`

- [ ] **Step 1: Write the failing capability-contract test**

Create `spool/spool-fabric/tests/capabilities.rs`:

```rust
use spool_fabric::{fabric_capabilities, MutationMode};

#[test]
fn fabric_capabilities_are_structured_and_read_only_for_v1() {
    let capabilities = fabric_capabilities();

    assert_eq!(capabilities.adapter_id, "spool-fabric");
    assert_eq!(capabilities.mutation_mode, MutationMode::ProposalOnly);
    assert!(capabilities.artifact_kinds.contains(&"report".to_string()));
    assert!(capabilities.identity_locator_shapes.contains(&"workspace/report".to_string()));
    assert!(capabilities.evidence_classes.contains(&"report_metadata".to_string()));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:
```bash
cd spool
cargo test -p spool-fabric fabric_capabilities_are_structured_and_read_only_for_v1
```

Expected: FAIL because the capability contract does not exist.

- [ ] **Step 3: Implement the capability contract**

Create `spool/spool-fabric/src/capabilities.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum MutationMode {
    ProposalOnly,
    Enabled,
    Disallowed,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CapabilityContract {
    pub adapter_id: String,
    pub artifact_kinds: Vec<String>,
    pub identity_locator_shapes: Vec<String>,
    pub evidence_classes: Vec<String>,
    pub freshness_expectations: Vec<String>,
    pub auth_requirements: Vec<String>,
    pub safety_rules: Vec<String>,
    pub mutation_mode: MutationMode,
}
```

Expose `fabric_capabilities() -> CapabilityContract` from `spool-fabric/src/lib.rs`.

- [ ] **Step 4: Run test to verify it passes**

Run:
```bash
cd spool
cargo test -p spool-fabric fabric_capabilities_are_structured_and_read_only_for_v1
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
cd spool
git add spool-fabric
git commit -m "feat: add spool fabric capability contract"
```

### Task 3: Add Auth Session And Token Restoration

**Files:**
- Create: `spool/spool-fabric/src/auth.rs`
- Create: `spool/spool-fabric/src/client.rs`
- Modify: `spool/spool-fabric/src/lib.rs`
- Create: `spool/spool-fabric/tests/auth_session.rs`

- [ ] **Step 1: Write the failing auth-session test**

Create `spool/spool-fabric/tests/auth_session.rs`:

```rust
use spool_fabric::{FabricAuthSession, TokenSourceKind};

#[test]
fn auth_session_prefers_keyring_then_env() {
    let session = FabricAuthSession::restored(
        TokenSourceKind::Keyring,
        "token-value".into(),
    );

    assert_eq!(session.token_source, TokenSourceKind::Keyring);
    assert_eq!(session.access_token, "token-value");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:
```bash
cd spool
cargo test -p spool-fabric auth_session_prefers_keyring_then_env
```

Expected: FAIL because the auth session types do not exist.

- [ ] **Step 3: Implement auth session and HTTP client primitives**

Create `spool/spool-fabric/src/auth.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum TokenSourceKind {
    Keyring,
    Env,
    DeviceFlow,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct FabricAuthSession {
    pub token_source: TokenSourceKind,
    pub access_token: String,
}

impl FabricAuthSession {
    pub fn restored(token_source: TokenSourceKind, access_token: String) -> Self {
        Self {
            token_source,
            access_token,
        }
    }
}
```

Create `spool/spool-fabric/src/client.rs`:

```rust
use reqwest::Client;

use crate::auth::FabricAuthSession;

pub struct FabricHttpClient {
    http: Client,
    session: FabricAuthSession,
}
```

Expose both from `spool-fabric/src/lib.rs`.

- [ ] **Step 4: Run test to verify it passes**

Run:
```bash
cd spool
cargo test -p spool-fabric auth_session_prefers_keyring_then_env
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
cd spool
git add spool-fabric
git commit -m "feat: add spool fabric auth session primitives"
```

### Task 4: Define Explicit Workspace And Artifact Resolution Outcomes

**Files:**
- Create: `spool/spool-fabric/src/workspaces.rs`
- Create: `spool/spool-fabric/src/artifacts.rs`
- Modify: `spool/spool-fabric/src/lib.rs`
- Create: `spool/spool-fabric/tests/resolution_contracts.rs`

- [ ] **Step 1: Write the failing resolution-contract tests**

Create `spool/spool-fabric/tests/resolution_contracts.rs`:

```rust
use spool_fabric::{ResolutionCandidate, ResolutionOutcome, WorkspaceRef};

#[test]
fn ambiguous_resolution_retains_all_candidates() {
    let outcome = ResolutionOutcome::Ambiguous {
        candidates: vec![
            ResolutionCandidate::workspace(WorkspaceRef {
                workspace_id: "ws_1".into(),
                display_name: "Executive BI".into(),
            }),
            ResolutionCandidate::workspace(WorkspaceRef {
                workspace_id: "ws_2".into(),
                display_name: "Executive BI Sandbox".into(),
            }),
        ],
    };

    match outcome {
        ResolutionOutcome::Ambiguous { candidates } => assert_eq!(candidates.len(), 2),
        _ => panic!("expected ambiguous outcome"),
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:
```bash
cd spool
cargo test -p spool-fabric ambiguous_resolution_retains_all_candidates
```

Expected: FAIL because the resolution outcome types do not exist.

- [ ] **Step 3: Implement the resolution outcome contracts**

Create `spool/spool-fabric/src/workspaces.rs` and `spool/spool-fabric/src/artifacts.rs` with:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceRef {
    pub workspace_id: String,
    pub display_name: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedArtifact {
    pub identity: spool_model::ArtifactIdentity,
}
```

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ResolutionCandidate {
    Workspace(crate::WorkspaceRef),
    Artifact(crate::ResolvedArtifact),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ResolutionOutcome {
    Resolved(ResolutionCandidate),
    Ambiguous { candidates: Vec<ResolutionCandidate> },
    Unresolved { reason: String },
}
```

- [ ] **Step 4: Run test to verify it passes**

Run:
```bash
cd spool
cargo test -p spool-fabric ambiguous_resolution_retains_all_candidates
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
cd spool
git add spool-fabric
git commit -m "feat: add spool explicit resolution outcomes"
```

### Task 5: Implement Workspace, Report, Page, And Visual Resolution

**Files:**
- Modify: `spool/spool-fabric/src/workspaces.rs`
- Modify: `spool/spool-fabric/src/artifacts.rs`
- Create: `spool/spool-fabric/tests/resolution_fixtures.rs`

- [ ] **Step 1: Write the failing report-side resolution test**

Create `spool/spool-fabric/tests/resolution_fixtures.rs`:

```rust
use spool_fabric::{FixtureResolver, ResolutionOutcome};

#[test]
fn report_side_resolution_prefers_exact_workspace_then_child_artifacts() {
    let resolver = FixtureResolver::default();

    let workspace = resolver.resolve_workspace("Executive BI");
    let report = resolver.resolve_report("ws_exec", "Executive Revenue Report");
    let page = resolver.resolve_page("ws_exec", "rpt_exec", "Summary");
    let visual = resolver.resolve_visual("ws_exec", "rpt_exec", "Summary", "Revenue Card");

    assert!(matches!(workspace, ResolutionOutcome::Resolved(_)));
    assert!(matches!(report, ResolutionOutcome::Resolved(_)));
    assert!(matches!(page, ResolutionOutcome::Resolved(_)));
    assert!(matches!(visual, ResolutionOutcome::Resolved(_)));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:
```bash
cd spool
cargo test -p spool-fabric report_side_resolution_prefers_exact_workspace_then_child_artifacts
```

Expected: FAIL because the resolver methods do not exist.

- [ ] **Step 3: Implement report-side resolution**

Add resolver traits and fixture-backed implementations covering:

- workspace resolution
- report resolution
- page resolution
- visual resolution

The resolution logic should follow the priority order:

1. explicit ID or GUID input
2. parsed URL when applicable
3. exact API match within confirmed scope
4. unique scoped name match
5. derived child identity from resolved parent

- [ ] **Step 4: Run test to verify it passes**

Run:
```bash
cd spool
cargo test -p spool-fabric report_side_resolution_prefers_exact_workspace_then_child_artifacts
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
cd spool
git add spool-fabric
git commit -m "feat: add spool report-side resolution"
```

### Task 6: Implement Semantic-Model And Warehouse Resolution

**Files:**
- Modify: `spool/spool-fabric/src/artifacts.rs`
- Modify: `spool/spool-fabric/tests/resolution_fixtures.rs`

- [ ] **Step 1: Write the failing semantic-model resolution test**

Append to `spool/spool-fabric/tests/resolution_fixtures.rs`:

```rust
#[test]
fn model_side_resolution_covers_model_measure_table_column_relationship_and_warehouse() {
    let resolver = FixtureResolver::default();

    assert!(matches!(resolver.resolve_semantic_model("ws_exec", "Sales Model"), ResolutionOutcome::Resolved(_)));
    assert!(matches!(resolver.resolve_measure("ws_exec", "mod_sales", "Sales", "Revenue"), ResolutionOutcome::Resolved(_)));
    assert!(matches!(resolver.resolve_table("ws_exec", "mod_sales", "Sales"), ResolutionOutcome::Resolved(_)));
    assert!(matches!(resolver.resolve_column("ws_exec", "mod_sales", "Sales", "Order Date"), ResolutionOutcome::Resolved(_)));
    assert!(matches!(resolver.resolve_relationship("ws_exec", "mod_sales", "sales_to_calendar"), ResolutionOutcome::Resolved(_)));
    assert!(matches!(resolver.resolve_warehouse("ws_exec", "Finance Warehouse"), ResolutionOutcome::Resolved(_)));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:
```bash
cd spool
cargo test -p spool-fabric model_side_resolution_covers_model_measure_table_column_relationship_and_warehouse
```

Expected: FAIL because those resolver methods do not exist.

- [ ] **Step 3: Implement the remaining v1 artifact resolution**

Extend the artifact resolver and fixture implementation to cover:

- semantic model
- measure
- table
- column
- relationship
- warehouse

Normalize each resolved result into the Plan 1 artifact identity shape.

- [ ] **Step 4: Run test to verify it passes**

Run:
```bash
cd spool
cargo test -p spool-fabric model_side_resolution_covers_model_measure_table_column_relationship_and_warehouse
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
cd spool
git add spool-fabric
git commit -m "feat: add spool semantic model and warehouse resolution"
```

### Task 7: Add Metadata Inspection Surfaces

**Files:**
- Create: `spool/spool-fabric/src/metadata.rs`
- Modify: `spool/spool-fabric/src/lib.rs`
- Create: `spool/spool-fabric/tests/metadata_projection.rs`

- [ ] **Step 1: Write the failing metadata-projection test**

Create `spool/spool-fabric/tests/metadata_projection.rs`:

```rust
use spool_fabric::MetadataEnvelope;
use spool_model::{ArtifactIdentity, ArtifactKind, ResolutionBasis};

#[test]
fn metadata_envelope_retains_artifact_kind_and_payload_summary() {
    let envelope = MetadataEnvelope {
        artifact_identity: ArtifactIdentity::new(
            "art_report_exec",
            ArtifactKind::Report,
            Some("ws_exec"),
            None,
            "fabric://workspace/ws_exec/report/rpt_exec",
            "Executive Revenue Report",
            ResolutionBasis::ExactApiMatch,
        ),
        payload_summary: "report has 3 pages".into(),
    };

    assert_eq!(envelope.artifact_identity.artifact_type, ArtifactKind::Report);
    assert_eq!(envelope.payload_summary, "report has 3 pages");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:
```bash
cd spool
cargo test -p spool-fabric metadata_envelope_retains_artifact_kind_and_payload_summary
```

Expected: FAIL because the metadata types do not exist.

- [ ] **Step 3: Implement the metadata inspection contract**

Create `spool/spool-fabric/src/metadata.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MetadataEnvelope {
    pub artifact_identity: spool_model::ArtifactIdentity,
    pub payload_summary: String,
}
```

Expose report-side and model-side metadata inspection helpers from `spool-fabric/src/lib.rs`.
Metadata helpers must accept resolved canonical identities rather than raw display names so later plans do not need adapter-specific translation glue.

- [ ] **Step 4: Run test to verify it passes**

Run:
```bash
cd spool
cargo test -p spool-fabric metadata_envelope_retains_artifact_kind_and_payload_summary
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
cd spool
git add spool-fabric
git commit -m "feat: add spool metadata inspection contract"
```

### Task 8: Add The Live Auth And Metadata Smoke Path

**Files:**
- Create: `spool/spool-fabric/tests/live_read_smoke.rs`

- [ ] **Step 1: Write the ignored live smoke test**

Create `spool/spool-fabric/tests/live_read_smoke.rs`:

```rust
use spool_fabric::{
    read_live_fixture_metadata,
    FabricConfig,
    FabricAuthSession,
    FabricHttpClient,
    TokenSourceKind,
};

#[tokio::test]
#[ignore = "requires dev Fabric workspace and local config"]
async fn reads_workspace_metadata_from_dev_fabric() {
    let config_path = std::env::var("SPOOL_CONFIG_PATH").unwrap_or_else(|_| {
        format!(
            "{}/.config/spool/dev.toml",
            std::env::var("HOME").unwrap()
        )
    });
    let config = FabricConfig::load_from_path(std::path::Path::new(&config_path)).unwrap();
    let token_env = config
        .access_token_env
        .clone()
        .unwrap_or_else(|| "SPOOL_FABRIC_ACCESS_TOKEN".into());
    let session = FabricAuthSession {
        token_source: TokenSourceKind::Env,
        access_token: std::env::var(token_env).unwrap(),
    };
    let client = FabricHttpClient::new(session);

    let response = read_live_fixture_metadata(&client, &config)
        .await
        .unwrap();

    assert!(response.workspace.is_some());
    assert!(response.report_metadata.is_some());
    assert!(response.semantic_model_metadata.is_some());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:
```bash
cd spool
cargo test -p spool-fabric reads_workspace_metadata_from_dev_fabric -- --ignored
```

Expected: FAIL because the config-backed live read helpers do not exist.

- [ ] **Step 3: Implement the live client read path**

Extend `spool/spool-fabric/src/client.rs` and adapter modules with:

- `FabricHttpClient::new(session: FabricAuthSession) -> Self`
- report-side and model-side live metadata helpers that resolve the configured fixtures
- a `read_live_fixture_metadata(client, config)` helper that:
  - restores auth from the configured token environment variable
  - resolves the configured workspace fixture
  - resolves the configured report-side and semantic-model-side artifacts
  - fetches metadata for those artifacts
  - returns canonical metadata envelopes keyed by `ArtifactIdentity`

Use bearer auth from the session token and the shared config contract from the plan index.

- [ ] **Step 4: Run the live smoke path**

Run:
```bash
cd spool
SPOOL_CONFIG_PATH=~/.config/spool/dev.toml \
cargo test -p spool-fabric reads_workspace_metadata_from_dev_fabric -- --ignored --nocapture
```

Expected: PASS using the configured dev Fabric workspace and shared live fixture set, with canonical workspace, report-side, and semantic-model-side metadata envelopes returned by the adapter.

- [ ] **Step 5: Commit**

```bash
cd spool
git add spool-fabric
git commit -m "feat: add spool live fabric read smoke path"
```

### Task 9: Write The Fabric Adapter Architecture Note

**Files:**
- Create: `spool/docs/architecture/fabric-adapter.md`

- [ ] **Step 1: Write the architecture note**

Create `spool/docs/architecture/fabric-adapter.md` describing:

- why `spool-fabric` owns auth, capability, resolution, and metadata seams
- why the adapter returns explicit ambiguity rather than collapsing to `Option`
- how artifact resolution maps into Plan 1 canonical identities
- why fixture-backed tests come before live smoke tests
- what later plans may add without redefining the adapter contract

- [ ] **Step 2: Review the note for missing concepts**

Run:
```bash
cd spool
rg -n "ambiguity|canonical identity|fixture|live smoke|capability" docs/architecture/fabric-adapter.md
```

Expected: the note explicitly mentions all five concepts.

- [ ] **Step 3: Commit**

```bash
cd spool
git add docs/architecture/fabric-adapter.md
git commit -m "docs: add spool fabric adapter architecture note"
```

## Self-Review Checklist

- Spec coverage: This plan covers config-backed startup, structured capability declarations, auth restoration, workspace and artifact resolution, ambiguity handling, metadata inspection, fixture-backed tests, and a live smoke path.
- Placeholder scan: No step may collapse ambiguity into `Option`, reduce capabilities to flat strings only, or leave most artifact kinds unresolved.
- Type consistency: Keep `CapabilityContract`, `MutationMode`, `FabricAuthSession`, `TokenSourceKind`, `ResolutionOutcome`, `ResolutionCandidate`, `WorkspaceRef`, `ResolvedArtifact`, and `MetadataEnvelope` stable because Plans 3-6 consume them directly.
