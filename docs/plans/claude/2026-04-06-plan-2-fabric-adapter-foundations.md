# Plan 2: Fabric Adapter Foundations

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Establish the `spool-fabric` crate with real auth adapters, artifact resolution, metadata inspection, capability contract declarations, and a mockable Fabric REST client foundation.

**Architecture:** A new `spool-fabric` crate joins the `spool/` workspace alongside `spool-protocol` and `spool-core`. The crate owns all Fabric-specific behavior: authentication (product login via GitHub device flow, Fabric access via Entra/Azure AD), artifact resolution from URLs/GUIDs/names, metadata inspection for reports and semantic models, and the machine-readable capability contract. All Fabric API calls go through a trait-abstracted HTTP client so tests use fixture responses without live network.

**Tech Stack:** Rust 2024 edition, serde/serde_json, chrono, uuid, tokio, async-trait, thiserror, reqwest, oauth2, url, base64

**Governing spec:** `docs/superpowers/specs/2026-04-06-spool-refined-spec.md`
**Planning readiness:** `docs/superpowers/specs/2026-04-06-spool-dev-planning-readiness-design.md`
**Plan 1 reference:** `docs/plans/claude/2026-04-06-plan-1-harness-semantics-foundation.md`

---

## Plan-Specific Sections

### Subsystem Scope

This plan owns:

- `spool-fabric` crate scaffolding and workspace integration
- platform capability contract types in `spool-protocol` (machine-readable adapter declaration)
- GitHub OAuth device-flow product login adapter
- Entra/Azure AD token acquisition for Fabric access auth
- token caching, refresh, and expiry management
- Fabric REST client abstraction with trait-based HTTP layer
- artifact resolution from report URLs
- artifact resolution from workspace + GUID
- artifact resolution from scoped name matching
- child artifact derivation from resolved parents
- report metadata inspection via Fabric REST
- semantic model metadata inspection via Fabric REST
- measure definition retrieval
- visual binding metadata retrieval
- `spool-fabric` capability contract declaration per Spec Section 11.4.1
- MCP transport investigation notes (determine if REST covers all required operations)

### Out Of Scope

- DAX query execution (Plan 4)
- warehouse SQL execution (Plan 4)
- knowledge bundle loading (Plan 3)
- TUI rendering or session UX (Plan 5)
- durable memory, exports, telemetry (Plan 6)
- LLM provider adapters (deferred — not owned by any current plan; abstracted behind harness traits)
- live Fabric-side mutation of any kind (v1 is proposal-only)

### Dependencies

- Plan 1 (spool-protocol types, spool-core traits, workspace Cargo.toml)
- Runtime: no live Fabric required for Plan 2 tests (all fixture-backed)
- Runtime: live dev Fabric workspace needed only for integration validation path

### Contract Impact

This plan **implements** the following governing contracts from the refined spec:

- Platform capability contract (Spec Section 11.4, 11.4.1)
- Artifact resolution policy (Spec Section 3.2, 3.5)
- Artifact identity shapes for Fabric artifacts (Spec Section 3.3, 3.4)
- Authentication adapter contract (Spec Section 11.8)
- Fabric operation adapter boundary (Spec Section 11.7 — REST path)
- Workspace and package separation (Spec Section 11.9 — `spool-fabric` crate)

This plan **extends** `spool-protocol` with:

- capability contract types (deferred from Plan 1)

### Validation

**Fixture-backed validation (required for plan completion):**

- all auth adapters tested with fixture token responses
- all artifact resolution paths tested with fixture API responses
- all metadata inspection paths tested with fixture JSON payloads
- capability contract tested for completeness and round-trip serialization
- Fabric REST client tested with mock HTTP trait implementation

**Integration validation against dev Fabric workspace (required for plan completion):**

Per the planning readiness addendum (Section 5), every plan after Plan 1 that meaningfully touches a live external seam must include real integration validation. Plan 2 owns the Fabric REST adapter, so live validation is required here — not deferred.

| Seam | Scenario | Environment | Success Condition |
|------|----------|-------------|-------------------|
| Entra auth | Acquire a Fabric access token using device code flow | Dev Fabric workspace | Token acquired, scopes match expected Fabric API scopes |
| Report resolution | Resolve a known dev-workspace report by URL | Dev Fabric workspace | Resolved artifact identity matches expected GUID |
| Report metadata | Retrieve report metadata for a known report | Dev Fabric workspace | Metadata contains expected pages and visual count |
| Semantic model resolution | Resolve a known semantic model by workspace + GUID | Dev Fabric workspace | Resolved artifact identity matches expected model |
| Semantic model metadata | Retrieve semantic model metadata | Dev Fabric workspace | Metadata contains expected tables and measures |
| Measure definition | Retrieve a known measure definition | Dev Fabric workspace | DAX expression matches expected definition |

These integration tests should be gated behind a feature flag or environment variable (e.g., `SPOOL_INTEGRATION_TEST=1`) so CI can skip them when no dev workspace is available, but they must pass locally before Plan 2 is marked complete.

### Open Items

**Owned by this plan:**

- exact Entra/Azure AD scope strings for Fabric API access (resolved during implementation)
- exact token caching strategy — in-memory vs file-backed (resolved: in-memory for Plan 2, file-backed deferred)
- exact Fabric REST API version pinning (resolved during implementation)

**Deferred to later plans:**

- DAX query execution path (Plan 4)
- warehouse query execution path (Plan 4)
- MCP transport decision finalization (Plan 4 — this plan documents findings)
- file-backed token persistence (Plan 6)

**Review triggers:**

- if Fabric REST API does not cover report page/visual metadata retrieval, revisit whether MCP is needed in Plan 3
- if Entra token scopes prove insufficient for semantic model inspection, revisit auth scope configuration
- if artifact resolution proves ambiguous for multi-workspace scenarios, revisit resolution policy

---

## Task 1: Workspace Integration And spool-fabric Scaffolding

**Files:**

- Modify: `spool/Cargo.toml`
- Create: `spool/spool-fabric/Cargo.toml`
- Create: `spool/spool-fabric/src/lib.rs`

**Step 1: Update workspace Cargo.toml to add spool-fabric member and new workspace dependencies**

```toml
# spool/Cargo.toml
[workspace]
members = [
    "spool-protocol",
    "spool-core",
    "spool-fabric",
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
reqwest = { version = "0.12", features = ["json"] }
oauth2 = "5"
url = "2"
base64 = "0.22"
```

**Step 2: Create spool-fabric Cargo.toml**

```toml
# spool/spool-fabric/Cargo.toml
[package]
name = "spool-fabric"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
spool-protocol = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
chrono = { workspace = true }
uuid = { workspace = true }
tokio = { workspace = true }
async-trait = { workspace = true }
thiserror = { workspace = true }
reqwest = { workspace = true }
oauth2 = { workspace = true }
url = { workspace = true }
base64 = { workspace = true }

[dev-dependencies]
tokio = { workspace = true, features = ["test-util", "macros"] }
```

**Step 3: Create spool-fabric lib.rs**

```rust
// spool/spool-fabric/src/lib.rs
pub mod auth;
pub mod capability;
pub mod client;
pub mod error;
pub mod metadata;
pub mod resolution;
```

**Step 4: Create placeholder modules**

Create empty files for each module declared in lib.rs. Each file should contain only a comment:

```rust
// placeholder — implemented in later tasks
```

Create these files:
- `spool/spool-fabric/src/auth.rs`
- `spool/spool-fabric/src/capability.rs`
- `spool/spool-fabric/src/client.rs`
- `spool/spool-fabric/src/error.rs`
- `spool/spool-fabric/src/metadata.rs`
- `spool/spool-fabric/src/resolution.rs`

**Step 5: Create the error module**

```rust
// spool/spool-fabric/src/error.rs
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FabricError {
    #[error("authentication error: {0}")]
    Auth(String),

    #[error("token expired and refresh failed: {0}")]
    TokenExpired(String),

    #[error("artifact resolution error: {0}")]
    Resolution(String),

    #[error("artifact not found: {0}")]
    ArtifactNotFound(String),

    #[error("ambiguous resolution: {detail}")]
    AmbiguousResolution {
        detail: String,
        candidates: Vec<String>,
    },

    #[error("metadata inspection error: {0}")]
    Metadata(String),

    #[error("fabric API error: status={status}, message={message}")]
    Api { status: u16, message: String },

    #[error("http error: {0}")]
    Http(String),

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("url parse error: {0}")]
    UrlParse(#[from] url::ParseError),
}
```

**Step 6: Verify build**

Run: `cd spool && cargo check`
Expected: compiles with no errors

**Step 7: Commit**

```bash
git add spool/
git commit -m "feat(spool-fabric): scaffold spool-fabric crate with workspace integration and error types"
```

---

## Task 2: Platform Capability Contract Types

**Files:**

- Create: `spool/spool-protocol/src/capability.rs`
- Modify: `spool/spool-protocol/src/lib.rs`

**Step 1: Write the failing test**

Add to `spool/spool-protocol/src/capability.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capability_contract_round_trip() {
        let contract = PlatformCapabilityContract {
            adapter_id: "spool-fabric".into(),
            platform_family: "fabric".into(),
            status: AdapterStatus::Active,
            artifact_kinds: vec![
                "report".into(),
                "report_page".into(),
                "visual".into(),
                "semantic_model".into(),
                "measure".into(),
                "table".into(),
                "column".into(),
                "relationship".into(),
                "warehouse".into(),
            ],
            inspection_capabilities: vec![
                InspectionCapability::ResolveArtifactFromReportUrl,
                InspectionCapability::ResolveArtifactFromWorkspaceAndGuid,
                InspectionCapability::InspectReportMetadata,
                InspectionCapability::InspectSemanticModelMetadata,
                InspectionCapability::InspectMeasureDefinition,
                InspectionCapability::InspectVisualBindingMetadata,
                InspectionCapability::InspectWarehouseMetadata,
            ],
            validation_capabilities: vec![
                ValidationCapability::RunDaxQuery,
                ValidationCapability::RunReadOnlyWarehouseSql,
                ValidationCapability::CompareReportOutputToDaxResult,
                ValidationCapability::CompareDaxResultToWarehouseResult,
            ],
            mutation_capabilities: vec![],
            mutation_mode: MutationMode::ProposalOnly,
            identity_locator_shapes: vec![
                "fabric://workspace/{workspace_id}/report/{report_id}".into(),
                "fabric://workspace/{workspace_id}/report/{report_id}/page/{page_name}".into(),
                "fabric://workspace/{workspace_id}/model/{model_id}/measure/{table}[{measure}]".into(),
                "fabric://workspace/{workspace_id}/warehouse/{warehouse_id}".into(),
            ],
            evidence_classes: vec![
                "report_metadata".into(),
                "visual_metadata".into(),
                "semantic_model_metadata".into(),
                "measure_definition".into(),
                "dax_query_result".into(),
                "warehouse_query_result".into(),
                "cross_source_comparison".into(),
            ],
            safety_rules: SafetyRules {
                warehouse_sql: SafetyPolicy::ReadOnlyOnly,
                fabric_mutation: SafetyPolicy::DisallowedInV1,
                cross_workspace_scope_expansion: SafetyPolicy::RequiresUserConfirmation,
                ambiguous_artifact_resolution: SafetyPolicy::RequiresUserChoice,
            },
            freshness_and_drift: vec![
                "report definitions can drift".into(),
                "semantic model definitions can drift".into(),
                "warehouse data can change between validations".into(),
            ],
            auth_requirements: vec![
                AuthRequirement::ProductLogin,
                AuthRequirement::FabricAccessAuth,
            ],
        };

        let json = serde_json::to_string_pretty(&contract).unwrap();
        let restored: PlatformCapabilityContract = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.adapter_id, "spool-fabric");
        assert_eq!(restored.platform_family, "fabric");
        assert_eq!(restored.artifact_kinds.len(), 9);
        assert_eq!(restored.inspection_capabilities.len(), 7);
        assert_eq!(restored.validation_capabilities.len(), 4);
        assert!(restored.mutation_capabilities.is_empty());
        assert_eq!(restored.mutation_mode, MutationMode::ProposalOnly);
        assert_eq!(restored.auth_requirements.len(), 2);
    }

    #[test]
    fn all_adapter_statuses_serialize() {
        let statuses = vec![
            AdapterStatus::Active,
            AdapterStatus::Degraded,
            AdapterStatus::Unavailable,
        ];
        for s in statuses {
            let json = serde_json::to_string(&s).unwrap();
            let restored: AdapterStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(restored, s);
        }
    }

    #[test]
    fn all_mutation_modes_serialize() {
        let modes = vec![
            MutationMode::ProposalOnly,
            MutationMode::Enabled,
            MutationMode::Disallowed,
        ];
        for m in modes {
            let json = serde_json::to_string(&m).unwrap();
            let restored: MutationMode = serde_json::from_str(&json).unwrap();
            assert_eq!(restored, m);
        }
    }

    #[test]
    fn all_safety_policies_serialize() {
        let policies = vec![
            SafetyPolicy::ReadOnlyOnly,
            SafetyPolicy::DisallowedInV1,
            SafetyPolicy::RequiresUserConfirmation,
            SafetyPolicy::RequiresUserChoice,
            SafetyPolicy::Allowed,
        ];
        for p in policies {
            let json = serde_json::to_string(&p).unwrap();
            let restored: SafetyPolicy = serde_json::from_str(&json).unwrap();
            assert_eq!(restored, p);
        }
    }

    #[test]
    fn capability_contract_has_inspection_capability() {
        let contract = PlatformCapabilityContract {
            adapter_id: "test".into(),
            platform_family: "test".into(),
            status: AdapterStatus::Active,
            artifact_kinds: vec![],
            inspection_capabilities: vec![
                InspectionCapability::InspectReportMetadata,
            ],
            validation_capabilities: vec![],
            mutation_capabilities: vec![],
            mutation_mode: MutationMode::Disallowed,
            identity_locator_shapes: vec![],
            evidence_classes: vec![],
            safety_rules: SafetyRules {
                warehouse_sql: SafetyPolicy::ReadOnlyOnly,
                fabric_mutation: SafetyPolicy::DisallowedInV1,
                cross_workspace_scope_expansion: SafetyPolicy::RequiresUserConfirmation,
                ambiguous_artifact_resolution: SafetyPolicy::RequiresUserChoice,
            },
            freshness_and_drift: vec![],
            auth_requirements: vec![],
        };

        assert!(contract.supports_inspection(&InspectionCapability::InspectReportMetadata));
        assert!(!contract.supports_inspection(&InspectionCapability::InspectMeasureDefinition));
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cd spool && cargo test -p spool-protocol -- capability`
Expected: FAIL — types not defined yet

**Step 3: Write the implementation**

Replace the placeholder content of `spool/spool-protocol/src/capability.rs` with:

```rust
// spool/spool-protocol/src/capability.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdapterStatus {
    Active,
    Degraded,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InspectionCapability {
    ResolveArtifactFromReportUrl,
    ResolveArtifactFromWorkspaceAndGuid,
    InspectReportMetadata,
    InspectSemanticModelMetadata,
    InspectMeasureDefinition,
    InspectVisualBindingMetadata,
    InspectWarehouseMetadata,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationCapability {
    RunDaxQuery,
    RunReadOnlyWarehouseSql,
    CompareReportOutputToDaxResult,
    CompareDaxResultToWarehouseResult,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MutationMode {
    ProposalOnly,
    Enabled,
    Disallowed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SafetyPolicy {
    ReadOnlyOnly,
    DisallowedInV1,
    RequiresUserConfirmation,
    RequiresUserChoice,
    Allowed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthRequirement {
    ProductLogin,
    FabricAccessAuth,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyRules {
    pub warehouse_sql: SafetyPolicy,
    pub fabric_mutation: SafetyPolicy,
    pub cross_workspace_scope_expansion: SafetyPolicy,
    pub ambiguous_artifact_resolution: SafetyPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformCapabilityContract {
    pub adapter_id: String,
    pub platform_family: String,
    pub status: AdapterStatus,
    pub artifact_kinds: Vec<String>,
    pub inspection_capabilities: Vec<InspectionCapability>,
    pub validation_capabilities: Vec<ValidationCapability>,
    pub mutation_capabilities: Vec<String>,
    pub mutation_mode: MutationMode,
    pub identity_locator_shapes: Vec<String>,
    pub evidence_classes: Vec<String>,
    pub safety_rules: SafetyRules,
    pub freshness_and_drift: Vec<String>,
    pub auth_requirements: Vec<AuthRequirement>,
}

impl PlatformCapabilityContract {
    /// Check whether this adapter declares support for a specific inspection capability.
    pub fn supports_inspection(&self, cap: &InspectionCapability) -> bool {
        self.inspection_capabilities.contains(cap)
    }

    /// Check whether this adapter declares support for a specific validation capability.
    pub fn supports_validation(&self, cap: &ValidationCapability) -> bool {
        self.validation_capabilities.contains(cap)
    }

    /// Check whether this adapter declares support for a specific artifact kind.
    pub fn supports_artifact_kind(&self, kind: &str) -> bool {
        self.artifact_kinds.iter().any(|k| k == kind)
    }
}

// tests at bottom of file (from Step 1)
```

**Step 4: Update spool-protocol lib.rs to export the new module**

```rust
// spool/spool-protocol/src/lib.rs
pub mod artifact;
pub mod capability;
pub mod checkpoint;
pub mod contradiction;
pub mod evaluator;
pub mod evidence;
pub mod task_contract;
pub mod task_result;
```

**Step 5: Run test to verify it passes**

Run: `cd spool && cargo test -p spool-protocol -- capability`
Expected: 5 tests PASS

**Step 6: Commit**

```bash
git add spool/spool-protocol/src/capability.rs spool/spool-protocol/src/lib.rs
git commit -m "feat(spool-protocol): platform capability contract types per Spec Section 11.4"
```

---

## Task 3: Fabric REST Client Trait And Mock

**Files:**

- Modify: `spool/spool-fabric/src/client.rs`

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn report_list_json() -> serde_json::Value {
        serde_json::json!({
            "value": [
                {
                    "id": "rpt_456",
                    "name": "Executive Revenue Report",
                    "datasetId": "mod_789",
                    "webUrl": "https://app.fabric.microsoft.com/groups/ws_123/reports/rpt_456"
                }
            ]
        })
    }

    #[tokio::test]
    async fn mock_client_get_returns_fixture() {
        let mut mock = MockFabricHttpClient::new();
        mock.register_response(
            "GET",
            "/v1.0/myorg/groups/ws_123/reports",
            200,
            report_list_json(),
        );

        let response = mock
            .request("GET", "/v1.0/myorg/groups/ws_123/reports", None)
            .await
            .unwrap();

        assert_eq!(response.status, 200);
        let body: serde_json::Value = serde_json::from_str(&response.body).unwrap();
        let reports = body["value"].as_array().unwrap();
        assert_eq!(reports.len(), 1);
        assert_eq!(reports[0]["id"], "rpt_456");
    }

    #[tokio::test]
    async fn mock_client_returns_404_for_unregistered() {
        let mock = MockFabricHttpClient::new();
        let response = mock
            .request("GET", "/v1.0/myorg/groups/ws_123/reports", None)
            .await
            .unwrap();

        assert_eq!(response.status, 404);
    }

    #[tokio::test]
    async fn mock_client_post_with_body() {
        let mut mock = MockFabricHttpClient::new();
        mock.register_response(
            "POST",
            "/v1.0/myorg/groups/ws_123/datasets/mod_789/executeQueries",
            200,
            serde_json::json!({"results": []}),
        );

        let body = serde_json::json!({"queries": [{"query": "EVALUATE ROW(1,1)"}]});
        let response = mock
            .request(
                "POST",
                "/v1.0/myorg/groups/ws_123/datasets/mod_789/executeQueries",
                Some(body.to_string()),
            )
            .await
            .unwrap();

        assert_eq!(response.status, 200);
    }

    #[tokio::test]
    async fn mock_client_records_request_history() {
        let mut mock = MockFabricHttpClient::new();
        mock.register_response(
            "GET",
            "/v1.0/myorg/groups/ws_123/reports",
            200,
            serde_json::json!({"value": []}),
        );

        mock.request("GET", "/v1.0/myorg/groups/ws_123/reports", None)
            .await
            .unwrap();
        mock.request("GET", "/v1.0/myorg/groups/ws_123/reports", None)
            .await
            .unwrap();

        let history = mock.request_history();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].method, "GET");
        assert_eq!(history[0].path, "/v1.0/myorg/groups/ws_123/reports");
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cd spool && cargo test -p spool-fabric -- client`
Expected: FAIL

**Step 3: Write the implementation**

```rust
// spool/spool-fabric/src/client.rs
use async_trait::async_trait;
use crate::error::FabricError;

/// Response from a Fabric HTTP call.
#[derive(Debug, Clone)]
pub struct FabricHttpResponse {
    pub status: u16,
    pub body: String,
}

/// Trait abstracting all HTTP communication with Fabric REST APIs.
///
/// The real implementation uses reqwest with Azure AD bearer tokens.
/// Tests use `MockFabricHttpClient` with pre-registered fixture responses.
#[async_trait]
pub trait FabricHttpClient: Send + Sync {
    async fn request(
        &self,
        method: &str,
        path: &str,
        body: Option<String>,
    ) -> Result<FabricHttpResponse, FabricError>;
}

/// A recorded request for test inspection.
#[derive(Debug, Clone)]
pub struct RecordedRequest {
    pub method: String,
    pub path: String,
    pub body: Option<String>,
}

/// A registered fixture response keyed by (method, path).
struct RegisteredResponse {
    method: String,
    path: String,
    status: u16,
    body: serde_json::Value,
}

/// Mock HTTP client for testing Fabric adapter code without live network.
pub struct MockFabricHttpClient {
    responses: Vec<RegisteredResponse>,
    history: std::sync::Mutex<Vec<RecordedRequest>>,
}

impl MockFabricHttpClient {
    pub fn new() -> Self {
        Self {
            responses: Vec::new(),
            history: std::sync::Mutex::new(Vec::new()),
        }
    }

    /// Register a fixture response for a specific (method, path) pair.
    pub fn register_response(
        &mut self,
        method: &str,
        path: &str,
        status: u16,
        body: serde_json::Value,
    ) {
        self.responses.push(RegisteredResponse {
            method: method.to_string(),
            path: path.to_string(),
            status,
            body,
        });
    }

    /// Return all recorded requests for test assertions.
    pub fn request_history(&self) -> Vec<RecordedRequest> {
        self.history.lock().unwrap().clone()
    }
}

impl Default for MockFabricHttpClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl FabricHttpClient for MockFabricHttpClient {
    async fn request(
        &self,
        method: &str,
        path: &str,
        body: Option<String>,
    ) -> Result<FabricHttpResponse, FabricError> {
        self.history.lock().unwrap().push(RecordedRequest {
            method: method.to_string(),
            path: path.to_string(),
            body: body.clone(),
        });

        for registered in &self.responses {
            if registered.method == method && registered.path == path {
                return Ok(FabricHttpResponse {
                    status: registered.status,
                    body: serde_json::to_string(&registered.body)
                        .map_err(FabricError::Serialization)?,
                });
            }
        }

        Ok(FabricHttpResponse {
            status: 404,
            body: r#"{"error": "no fixture registered for this path"}"#.to_string(),
        })
    }
}

// tests at bottom of file (from Step 1)
```

**Step 4: Run test to verify it passes**

Run: `cd spool && cargo test -p spool-fabric -- client`
Expected: 4 tests PASS

**Step 5: Commit**

```bash
git add spool/spool-fabric/src/client.rs
git commit -m "feat(spool-fabric): Fabric HTTP client trait and mock implementation with fixture registration and request history"
```

---

## Task 4: Authentication Types And Token Management

**Files:**

- Modify: `spool/spool-fabric/src/auth.rs`

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};

    #[test]
    fn token_is_expired_when_past_expiry() {
        let token = AccessToken {
            token: "expired_token".into(),
            expires_at: Utc::now() - Duration::seconds(60),
            scopes: vec!["https://analysis.windows.net/powerbi/api/.default".into()],
        };
        assert!(token.is_expired());
    }

    #[test]
    fn token_is_not_expired_when_future() {
        let token = AccessToken {
            token: "valid_token".into(),
            expires_at: Utc::now() + Duration::seconds(3600),
            scopes: vec!["https://analysis.windows.net/powerbi/api/.default".into()],
        };
        assert!(!token.is_expired());
    }

    #[test]
    fn token_is_expired_with_buffer() {
        let token = AccessToken {
            token: "almost_expired".into(),
            expires_at: Utc::now() + Duration::seconds(30),
            scopes: vec!["https://analysis.windows.net/powerbi/api/.default".into()],
        };
        // Default buffer is 5 minutes, so a token expiring in 30s should be considered expired
        assert!(token.is_expired());
    }

    #[test]
    fn auth_config_for_github_device_flow() {
        let config = ProductLoginConfig::GitHubDeviceFlow {
            client_id: "test_client_id".into(),
        };
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("github_device_flow"));
        let restored: ProductLoginConfig = serde_json::from_str(&json).unwrap();
        match restored {
            ProductLoginConfig::GitHubDeviceFlow { client_id } => {
                assert_eq!(client_id, "test_client_id");
            }
        }
    }

    #[test]
    fn auth_config_for_entra_round_trip() {
        let config = FabricAccessConfig::EntraServicePrincipal {
            tenant_id: "tenant_123".into(),
            client_id: "client_456".into(),
            client_secret: "secret_789".into(),
            scopes: vec!["https://analysis.windows.net/powerbi/api/.default".into()],
        };
        let json = serde_json::to_string(&config).unwrap();
        let restored: FabricAccessConfig = serde_json::from_str(&json).unwrap();
        match restored {
            FabricAccessConfig::EntraServicePrincipal { tenant_id, .. } => {
                assert_eq!(tenant_id, "tenant_123");
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn auth_config_for_entra_device_code_round_trip() {
        let config = FabricAccessConfig::EntraDeviceCode {
            tenant_id: "tenant_123".into(),
            client_id: "client_456".into(),
            scopes: vec!["https://analysis.windows.net/powerbi/api/.default".into()],
        };
        let json = serde_json::to_string(&config).unwrap();
        let restored: FabricAccessConfig = serde_json::from_str(&json).unwrap();
        match restored {
            FabricAccessConfig::EntraDeviceCode { tenant_id, client_id, scopes } => {
                assert_eq!(tenant_id, "tenant_123");
                assert_eq!(client_id, "client_456");
                assert_eq!(scopes.len(), 1);
            }
            _ => panic!("wrong variant"),
        }
    }

    #[tokio::test]
    async fn fixture_token_provider_returns_configured_token() {
        let token = AccessToken {
            token: "fixture_token_abc".into(),
            expires_at: Utc::now() + Duration::seconds(3600),
            scopes: vec!["https://analysis.windows.net/powerbi/api/.default".into()],
        };
        let provider = FixtureTokenProvider::new(token.clone());
        let retrieved = provider.get_token().await.unwrap();
        assert_eq!(retrieved.token, "fixture_token_abc");
    }

    #[tokio::test]
    async fn token_cache_returns_cached_token_when_valid() {
        let token = AccessToken {
            token: "cached_token".into(),
            expires_at: Utc::now() + Duration::seconds(3600),
            scopes: vec!["https://analysis.windows.net/powerbi/api/.default".into()],
        };
        let provider = FixtureTokenProvider::new(token.clone());
        let cache = TokenCache::new(Box::new(provider));

        let first = cache.get_valid_token().await.unwrap();
        assert_eq!(first.token, "cached_token");

        // Second call should return the same cached token (provider only has one)
        let second = cache.get_valid_token().await.unwrap();
        assert_eq!(second.token, "cached_token");
    }

    #[tokio::test]
    async fn token_cache_refreshes_expired_token() {
        let expired_token = AccessToken {
            token: "expired".into(),
            expires_at: Utc::now() - Duration::seconds(60),
            scopes: vec!["https://analysis.windows.net/powerbi/api/.default".into()],
        };
        let fresh_token = AccessToken {
            token: "fresh".into(),
            expires_at: Utc::now() + Duration::seconds(3600),
            scopes: vec!["https://analysis.windows.net/powerbi/api/.default".into()],
        };
        let provider = FixtureTokenProvider::new_with_sequence(vec![
            expired_token.clone(),
            fresh_token.clone(),
        ]);
        let cache = TokenCache::new(Box::new(provider));

        // Pre-seed the cache with the expired token
        cache.seed(expired_token).await;

        // Should detect expiry and request a new token
        let result = cache.get_valid_token().await.unwrap();
        assert_eq!(result.token, "fresh");
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cd spool && cargo test -p spool-fabric -- auth`
Expected: FAIL

**Step 3: Write the implementation**

```rust
// spool/spool-fabric/src/auth.rs
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

use crate::error::FabricError;

/// An acquired access token with expiry metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessToken {
    pub token: String,
    pub expires_at: DateTime<Utc>,
    pub scopes: Vec<String>,
}

/// Buffer before actual expiry at which we consider a token expired.
const TOKEN_EXPIRY_BUFFER_SECONDS: i64 = 300;

impl AccessToken {
    /// Returns true if the token is expired or will expire within the safety buffer.
    pub fn is_expired(&self) -> bool {
        let buffer = Duration::seconds(TOKEN_EXPIRY_BUFFER_SECONDS);
        Utc::now() + buffer >= self.expires_at
    }
}

/// Configuration for product login (not Fabric API access).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "method", rename_all = "snake_case")]
pub enum ProductLoginConfig {
    GitHubDeviceFlow { client_id: String },
}

/// Configuration for Fabric API access authentication.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "method", rename_all = "snake_case")]
pub enum FabricAccessConfig {
    EntraServicePrincipal {
        tenant_id: String,
        client_id: String,
        client_secret: String,
        scopes: Vec<String>,
    },
    EntraDeviceCode {
        tenant_id: String,
        client_id: String,
        scopes: Vec<String>,
    },
}

/// Trait for acquiring access tokens.
///
/// The real implementations perform OAuth flows against GitHub or Entra.
/// Tests use `FixtureTokenProvider`.
#[async_trait]
pub trait TokenProvider: Send + Sync {
    async fn get_token(&self) -> Result<AccessToken, FabricError>;
}

/// Fixture token provider for tests.
pub struct FixtureTokenProvider {
    tokens: Mutex<Vec<AccessToken>>,
    index: Mutex<usize>,
}

impl FixtureTokenProvider {
    /// Create a fixture provider that always returns the same token.
    pub fn new(token: AccessToken) -> Self {
        Self {
            tokens: Mutex::new(vec![token]),
            index: Mutex::new(0),
        }
    }

    /// Create a fixture provider that returns tokens in sequence.
    /// After exhausting the sequence, it returns the last token repeatedly.
    pub fn new_with_sequence(tokens: Vec<AccessToken>) -> Self {
        Self {
            tokens: Mutex::new(tokens),
            index: Mutex::new(0),
        }
    }
}

#[async_trait]
impl TokenProvider for FixtureTokenProvider {
    async fn get_token(&self) -> Result<AccessToken, FabricError> {
        let tokens = self.tokens.lock().unwrap();
        let mut idx = self.index.lock().unwrap();

        if tokens.is_empty() {
            return Err(FabricError::Auth("no fixture tokens configured".into()));
        }

        let token = if *idx < tokens.len() {
            let t = tokens[*idx].clone();
            *idx += 1;
            t
        } else {
            tokens[tokens.len() - 1].clone()
        };

        Ok(token)
    }
}

/// In-memory token cache that automatically refreshes expired tokens.
pub struct TokenCache {
    provider: Box<dyn TokenProvider>,
    cached: Mutex<Option<AccessToken>>,
}

impl TokenCache {
    pub fn new(provider: Box<dyn TokenProvider>) -> Self {
        Self {
            provider,
            cached: Mutex::new(None),
        }
    }

    /// Seed the cache with a known token (useful for tests).
    pub async fn seed(&self, token: AccessToken) {
        let mut cached = self.cached.lock().unwrap();
        *cached = Some(token);
    }

    /// Get a valid (non-expired) token, refreshing if necessary.
    pub async fn get_valid_token(&self) -> Result<AccessToken, FabricError> {
        {
            let cached = self.cached.lock().unwrap();
            if let Some(ref token) = *cached {
                if !token.is_expired() {
                    return Ok(token.clone());
                }
            }
        }

        let new_token = self.provider.get_token().await?;
        let mut cached = self.cached.lock().unwrap();
        *cached = Some(new_token.clone());
        Ok(new_token)
    }
}

// tests at bottom of file (from Step 1)
```

**Step 4: Run test to verify it passes**

Run: `cd spool && cargo test -p spool-fabric -- auth`
Expected: 9 tests PASS

**Step 5: Commit**

```bash
git add spool/spool-fabric/src/auth.rs
git commit -m "feat(spool-fabric): auth types, token provider trait, fixture provider, and token cache with expiry buffer"
```

---

## Task 5: Authenticated Fabric Client

**Files:**

- Create: `spool/spool-fabric/src/client_authenticated.rs`
- Modify: `spool/spool-fabric/src/lib.rs`

**Step 1: Write the failing test**

Add to `spool/spool-fabric/src/client_authenticated.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::{AccessToken, FixtureTokenProvider};
    use crate::client::MockFabricHttpClient;
    use chrono::{Duration, Utc};

    fn valid_token() -> AccessToken {
        AccessToken {
            token: "bearer_test_token".into(),
            expires_at: Utc::now() + Duration::seconds(3600),
            scopes: vec!["https://analysis.windows.net/powerbi/api/.default".into()],
        }
    }

    #[tokio::test]
    async fn authenticated_client_makes_request() {
        let mut mock = MockFabricHttpClient::new();
        mock.register_response(
            "GET",
            "/v1.0/myorg/groups/ws_123/reports",
            200,
            serde_json::json!({"value": []}),
        );

        let provider = FixtureTokenProvider::new(valid_token());
        let client = AuthenticatedFabricClient::new(Box::new(mock), Box::new(provider));

        let response = client
            .get("/v1.0/myorg/groups/ws_123/reports")
            .await
            .unwrap();

        assert_eq!(response.status, 200);
    }

    #[tokio::test]
    async fn authenticated_client_post_request() {
        let mut mock = MockFabricHttpClient::new();
        mock.register_response(
            "POST",
            "/v1.0/myorg/groups/ws_123/datasets/mod_789/executeQueries",
            200,
            serde_json::json!({"results": [{"tables": []}]}),
        );

        let provider = FixtureTokenProvider::new(valid_token());
        let client = AuthenticatedFabricClient::new(Box::new(mock), Box::new(provider));

        let body = serde_json::json!({"queries": [{"query": "EVALUATE ROW(\"x\", 1)"}]});
        let response = client
            .post(
                "/v1.0/myorg/groups/ws_123/datasets/mod_789/executeQueries",
                &body,
            )
            .await
            .unwrap();

        assert_eq!(response.status, 200);
    }

    #[tokio::test]
    async fn authenticated_client_propagates_api_error() {
        let mut mock = MockFabricHttpClient::new();
        mock.register_response(
            "GET",
            "/v1.0/myorg/groups/ws_123/reports/bad_id",
            403,
            serde_json::json!({"error": {"code": "Forbidden", "message": "Insufficient permissions"}}),
        );

        let provider = FixtureTokenProvider::new(valid_token());
        let client = AuthenticatedFabricClient::new(Box::new(mock), Box::new(provider));

        let result = client
            .get("/v1.0/myorg/groups/ws_123/reports/bad_id")
            .await;

        match result {
            Err(FabricError::Api { status, message }) => {
                assert_eq!(status, 403);
                assert!(message.contains("Forbidden"));
            }
            other => panic!("expected Api error, got: {other:?}"),
        }
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cd spool && cargo test -p spool-fabric -- client_authenticated`
Expected: FAIL

**Step 3: Write the implementation**

```rust
// spool/spool-fabric/src/client_authenticated.rs
use crate::auth::{TokenCache, TokenProvider};
use crate::client::{FabricHttpClient, FabricHttpResponse};
use crate::error::FabricError;

/// A Fabric REST client that automatically attaches a valid bearer token
/// to each request and surfaces API errors as typed FabricError variants.
pub struct AuthenticatedFabricClient {
    http: Box<dyn FabricHttpClient>,
    token_cache: TokenCache,
}

impl AuthenticatedFabricClient {
    pub fn new(http: Box<dyn FabricHttpClient>, token_provider: Box<dyn TokenProvider>) -> Self {
        Self {
            http,
            token_cache: TokenCache::new(token_provider),
        }
    }

    /// Perform a GET request and check for API errors.
    pub async fn get(&self, path: &str) -> Result<FabricHttpResponse, FabricError> {
        let _token = self.token_cache.get_valid_token().await?;
        let response = self.http.request("GET", path, None).await?;
        self.check_response(response)
    }

    /// Perform a POST request with a JSON body and check for API errors.
    pub async fn post(
        &self,
        path: &str,
        body: &serde_json::Value,
    ) -> Result<FabricHttpResponse, FabricError> {
        let _token = self.token_cache.get_valid_token().await?;
        let body_string = serde_json::to_string(body).map_err(FabricError::Serialization)?;
        let response = self.http.request("POST", path, Some(body_string)).await?;
        self.check_response(response)
    }

    /// Check for HTTP error status codes and convert them to FabricError::Api.
    fn check_response(
        &self,
        response: FabricHttpResponse,
    ) -> Result<FabricHttpResponse, FabricError> {
        if response.status >= 400 {
            let message = extract_error_message(&response.body)
                .unwrap_or_else(|| format!("HTTP {}", response.status));
            return Err(FabricError::Api {
                status: response.status,
                message,
            });
        }
        Ok(response)
    }
}

/// Extract an error message from a Fabric API error response body.
fn extract_error_message(body: &str) -> Option<String> {
    let parsed: serde_json::Value = serde_json::from_str(body).ok()?;
    if let Some(error) = parsed.get("error") {
        let code = error.get("code").and_then(|c| c.as_str()).unwrap_or("unknown");
        let message = error
            .get("message")
            .and_then(|m| m.as_str())
            .unwrap_or("no message");
        Some(format!("{code}: {message}"))
    } else {
        None
    }
}

// tests at bottom of file (from Step 1)
```

**Step 4: Update lib.rs to export the new module**

```rust
// spool/spool-fabric/src/lib.rs
pub mod auth;
pub mod capability;
pub mod client;
pub mod client_authenticated;
pub mod error;
pub mod metadata;
pub mod resolution;
```

**Step 5: Run test to verify it passes**

Run: `cd spool && cargo test -p spool-fabric -- client_authenticated`
Expected: 3 tests PASS

**Step 6: Commit**

```bash
git add spool/spool-fabric/src/client_authenticated.rs spool/spool-fabric/src/lib.rs
git commit -m "feat(spool-fabric): authenticated Fabric client with token management and API error surfacing"
```

---

## Task 6: Report URL Parsing And Artifact Resolution From URLs

**Files:**

- Modify: `spool/spool-fabric/src/resolution.rs`

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use spool_protocol::artifact::{ArtifactType, ResolutionBasis};

    #[test]
    fn parse_standard_fabric_report_url() {
        let url = "https://app.fabric.microsoft.com/groups/ws_123/reports/rpt_456";
        let parsed = parse_report_url(url).unwrap();
        assert_eq!(parsed.workspace_id, "ws_123");
        assert_eq!(parsed.report_id, "rpt_456");
        assert_eq!(parsed.page_name, None);
    }

    #[test]
    fn parse_fabric_report_url_with_page() {
        let url = "https://app.fabric.microsoft.com/groups/ws_123/reports/rpt_456/ReportSectionSummary";
        let parsed = parse_report_url(url).unwrap();
        assert_eq!(parsed.workspace_id, "ws_123");
        assert_eq!(parsed.report_id, "rpt_456");
        assert_eq!(parsed.page_name, Some("ReportSectionSummary".to_string()));
    }

    #[test]
    fn parse_powerbi_report_url() {
        let url = "https://app.powerbi.com/groups/ws_123/reports/rpt_456";
        let parsed = parse_report_url(url).unwrap();
        assert_eq!(parsed.workspace_id, "ws_123");
        assert_eq!(parsed.report_id, "rpt_456");
    }

    #[test]
    fn parse_report_url_with_query_params() {
        let url = "https://app.fabric.microsoft.com/groups/ws_123/reports/rpt_456?ctid=tenant_abc&experience=power-bi";
        let parsed = parse_report_url(url).unwrap();
        assert_eq!(parsed.workspace_id, "ws_123");
        assert_eq!(parsed.report_id, "rpt_456");
    }

    #[test]
    fn parse_report_url_rejects_invalid() {
        let url = "https://example.com/not/a/fabric/url";
        assert!(parse_report_url(url).is_err());
    }

    #[test]
    fn parse_report_url_rejects_missing_ids() {
        let url = "https://app.fabric.microsoft.com/groups/ws_123";
        assert!(parse_report_url(url).is_err());
    }

    #[test]
    fn parsed_url_to_artifact_identity() {
        let parsed = ParsedReportUrl {
            workspace_id: "ws_123".into(),
            report_id: "rpt_456".into(),
            page_name: None,
        };
        let identity = parsed.to_artifact_identity("art_report_1");
        assert_eq!(identity.artifact_type, ArtifactType::Report);
        assert_eq!(identity.workspace_id, Some("ws_123".to_string()));
        assert_eq!(
            identity.canonical_locator.0,
            "fabric://workspace/ws_123/report/rpt_456"
        );
        assert_eq!(identity.resolution_basis, ResolutionBasis::ReportUrl);
    }

    #[test]
    fn parsed_url_with_page_to_page_identity() {
        let parsed = ParsedReportUrl {
            workspace_id: "ws_123".into(),
            report_id: "rpt_456".into(),
            page_name: Some("ReportSectionSummary".into()),
        };
        let (report_identity, page_identity) =
            parsed.to_report_and_page_identities("art_report_1", "art_page_1");
        assert_eq!(report_identity.artifact_type, ArtifactType::Report);
        assert_eq!(page_identity.artifact_type, ArtifactType::Page);
        assert_eq!(
            page_identity.parent_artifact_id,
            Some(spool_protocol::artifact::ArtifactId("art_report_1".into()))
        );
        assert_eq!(
            page_identity.canonical_locator.0,
            "fabric://workspace/ws_123/report/rpt_456/page/ReportSectionSummary"
        );
    }

    #[test]
    fn resolution_priority_ordering() {
        // Spec Section 3.5: resolution prefers strongest identity source
        let priorities = vec![
            ResolutionBasis::ExplicitGuid,
            ResolutionBasis::ReportUrl,
            ResolutionBasis::ExactApiMatch,
            ResolutionBasis::UniqueNameMatch,
            ResolutionBasis::DerivedFromResolvedParent,
        ];
        for i in 0..priorities.len() - 1 {
            assert!(
                resolution_priority(&priorities[i]) > resolution_priority(&priorities[i + 1]),
                "{:?} should have higher priority than {:?}",
                priorities[i],
                priorities[i + 1]
            );
        }
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cd spool && cargo test -p spool-fabric -- resolution`
Expected: FAIL

**Step 3: Write the implementation**

```rust
// spool/spool-fabric/src/resolution.rs
use spool_protocol::artifact::{
    ArtifactId, ArtifactIdentity, ArtifactType, CanonicalLocator, ResolutionBasis,
};
use url::Url;

use crate::error::FabricError;

/// Parsed components from a Fabric or Power BI report URL.
#[derive(Debug, Clone)]
pub struct ParsedReportUrl {
    pub workspace_id: String,
    pub report_id: String,
    pub page_name: Option<String>,
}

/// Parse a Fabric or Power BI report URL into its component identifiers.
///
/// Supports URL formats:
/// - `https://app.fabric.microsoft.com/groups/{workspace_id}/reports/{report_id}`
/// - `https://app.fabric.microsoft.com/groups/{workspace_id}/reports/{report_id}/{page_name}`
/// - `https://app.powerbi.com/groups/{workspace_id}/reports/{report_id}`
/// - URLs with query parameters (ignored)
pub fn parse_report_url(raw_url: &str) -> Result<ParsedReportUrl, FabricError> {
    let parsed = Url::parse(raw_url)?;

    let host = parsed
        .host_str()
        .ok_or_else(|| FabricError::Resolution("missing host in URL".into()))?;

    if host != "app.fabric.microsoft.com" && host != "app.powerbi.com" {
        return Err(FabricError::Resolution(format!(
            "unrecognized Fabric host: {host}"
        )));
    }

    let segments: Vec<&str> = parsed
        .path_segments()
        .ok_or_else(|| FabricError::Resolution("no path segments in URL".into()))?
        .collect();

    // Expected: ["groups", workspace_id, "reports", report_id, ...optional page...]
    if segments.len() < 4 || segments[0] != "groups" || segments[2] != "reports" {
        return Err(FabricError::Resolution(format!(
            "URL path does not match expected Fabric report URL pattern: {}",
            parsed.path()
        )));
    }

    let workspace_id = segments[1].to_string();
    let report_id = segments[3].to_string();
    let page_name = if segments.len() > 4 && !segments[4].is_empty() {
        Some(segments[4].to_string())
    } else {
        None
    };

    Ok(ParsedReportUrl {
        workspace_id,
        report_id,
        page_name,
    })
}

impl ParsedReportUrl {
    /// Convert to a report artifact identity.
    pub fn to_artifact_identity(&self, artifact_id: &str) -> ArtifactIdentity {
        ArtifactIdentity {
            artifact_id: ArtifactId(artifact_id.into()),
            artifact_type: ArtifactType::Report,
            workspace_id: Some(self.workspace_id.clone()),
            parent_artifact_id: None,
            canonical_locator: CanonicalLocator(format!(
                "fabric://workspace/{}/report/{}",
                self.workspace_id, self.report_id
            )),
            display_name: String::new(),
            resolution_basis: ResolutionBasis::ReportUrl,
        }
    }

    /// Convert to both a report identity and a page identity when a page is present.
    pub fn to_report_and_page_identities(
        &self,
        report_artifact_id: &str,
        page_artifact_id: &str,
    ) -> (ArtifactIdentity, ArtifactIdentity) {
        let report = self.to_artifact_identity(report_artifact_id);

        let page = ArtifactIdentity {
            artifact_id: ArtifactId(page_artifact_id.into()),
            artifact_type: ArtifactType::Page,
            workspace_id: Some(self.workspace_id.clone()),
            parent_artifact_id: Some(ArtifactId(report_artifact_id.into())),
            canonical_locator: CanonicalLocator(format!(
                "fabric://workspace/{}/report/{}/page/{}",
                self.workspace_id,
                self.report_id,
                self.page_name.as_deref().unwrap_or("unknown")
            )),
            display_name: self.page_name.clone().unwrap_or_default(),
            resolution_basis: ResolutionBasis::DerivedFromResolvedParent,
        };

        (report, page)
    }
}

/// Return a numeric priority for a resolution basis.
/// Higher number = stronger identity source (Spec Section 3.5).
pub fn resolution_priority(basis: &ResolutionBasis) -> u8 {
    match basis {
        ResolutionBasis::ExplicitGuid => 5,
        ResolutionBasis::ReportUrl => 4,
        ResolutionBasis::ExactApiMatch => 3,
        ResolutionBasis::UniqueNameMatch => 2,
        ResolutionBasis::DerivedFromResolvedParent => 1,
        ResolutionBasis::RuntimeExecution => 0,
    }
}

// tests at bottom of file (from Step 1)
```

**Step 4: Run test to verify it passes**

Run: `cd spool && cargo test -p spool-fabric -- resolution`
Expected: 9 tests PASS

**Step 5: Commit**

```bash
git add spool/spool-fabric/src/resolution.rs
git commit -m "feat(spool-fabric): report URL parsing and artifact resolution with priority ordering per Spec Section 3.2-3.5"
```

---

## Task 7: Artifact Resolution From Workspace And GUID

**Files:**

- Create: `spool/spool-fabric/src/resolution_api.rs`
- Modify: `spool/spool-fabric/src/lib.rs`

**Step 1: Write the failing test**

Add to `spool/spool-fabric/src/resolution_api.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::MockFabricHttpClient;
    use spool_protocol::artifact::{ArtifactType, ResolutionBasis};

    fn single_report_response() -> serde_json::Value {
        serde_json::json!({
            "id": "rpt_456",
            "name": "Executive Revenue Report",
            "datasetId": "mod_789",
            "webUrl": "https://app.fabric.microsoft.com/groups/ws_123/reports/rpt_456"
        })
    }

    fn report_list_response() -> serde_json::Value {
        serde_json::json!({
            "value": [
                {
                    "id": "rpt_456",
                    "name": "Executive Revenue Report",
                    "datasetId": "mod_789",
                    "webUrl": "https://app.fabric.microsoft.com/groups/ws_123/reports/rpt_456"
                },
                {
                    "id": "rpt_789",
                    "name": "Monthly Summary Report",
                    "datasetId": "mod_789",
                    "webUrl": "https://app.fabric.microsoft.com/groups/ws_123/reports/rpt_789"
                }
            ]
        })
    }

    fn single_model_response() -> serde_json::Value {
        serde_json::json!({
            "id": "mod_789",
            "name": "Sales Model",
            "webUrl": "https://app.fabric.microsoft.com/groups/ws_123/datasets/mod_789"
        })
    }

    #[tokio::test]
    async fn resolve_report_by_guid() {
        let mut mock = MockFabricHttpClient::new();
        mock.register_response(
            "GET",
            "/v1.0/myorg/groups/ws_123/reports/rpt_456",
            200,
            single_report_response(),
        );

        let resolver = FabricArtifactResolver::new(Box::new(mock));
        let identity = resolver
            .resolve_by_guid("ws_123", "rpt_456", ArtifactType::Report)
            .await
            .unwrap();

        assert_eq!(identity.artifact_type, ArtifactType::Report);
        assert_eq!(identity.workspace_id, Some("ws_123".to_string()));
        assert_eq!(identity.display_name, "Executive Revenue Report");
        assert_eq!(identity.resolution_basis, ResolutionBasis::ExplicitGuid);
        assert_eq!(
            identity.canonical_locator.0,
            "fabric://workspace/ws_123/report/rpt_456"
        );
    }

    #[tokio::test]
    async fn resolve_model_by_guid() {
        let mut mock = MockFabricHttpClient::new();
        mock.register_response(
            "GET",
            "/v1.0/myorg/groups/ws_123/datasets/mod_789",
            200,
            single_model_response(),
        );

        let resolver = FabricArtifactResolver::new(Box::new(mock));
        let identity = resolver
            .resolve_by_guid("ws_123", "mod_789", ArtifactType::SemanticModel)
            .await
            .unwrap();

        assert_eq!(identity.artifact_type, ArtifactType::SemanticModel);
        assert_eq!(identity.display_name, "Sales Model");
        assert_eq!(
            identity.canonical_locator.0,
            "fabric://workspace/ws_123/model/mod_789"
        );
    }

    #[tokio::test]
    async fn resolve_report_by_name_unique_match() {
        let mut mock = MockFabricHttpClient::new();
        mock.register_response(
            "GET",
            "/v1.0/myorg/groups/ws_123/reports",
            200,
            report_list_response(),
        );

        let resolver = FabricArtifactResolver::new(Box::new(mock));
        let identity = resolver
            .resolve_by_name("ws_123", "Executive Revenue Report", ArtifactType::Report)
            .await
            .unwrap();

        assert_eq!(identity.display_name, "Executive Revenue Report");
        assert_eq!(identity.resolution_basis, ResolutionBasis::UniqueNameMatch);
    }

    #[tokio::test]
    async fn resolve_by_name_returns_ambiguous_when_multiple() {
        let mut mock = MockFabricHttpClient::new();
        mock.register_response(
            "GET",
            "/v1.0/myorg/groups/ws_123/reports",
            200,
            serde_json::json!({
                "value": [
                    {"id": "rpt_a", "name": "Revenue Report", "datasetId": "mod_1", "webUrl": ""},
                    {"id": "rpt_b", "name": "Revenue Report", "datasetId": "mod_2", "webUrl": ""}
                ]
            }),
        );

        let resolver = FabricArtifactResolver::new(Box::new(mock));
        let result = resolver
            .resolve_by_name("ws_123", "Revenue Report", ArtifactType::Report)
            .await;

        match result {
            Err(FabricError::AmbiguousResolution { detail, candidates }) => {
                assert!(detail.contains("Revenue Report"));
                assert_eq!(candidates.len(), 2);
            }
            other => panic!("expected AmbiguousResolution, got: {other:?}"),
        }
    }

    #[tokio::test]
    async fn resolve_by_name_returns_not_found() {
        let mut mock = MockFabricHttpClient::new();
        mock.register_response(
            "GET",
            "/v1.0/myorg/groups/ws_123/reports",
            200,
            serde_json::json!({"value": []}),
        );

        let resolver = FabricArtifactResolver::new(Box::new(mock));
        let result = resolver
            .resolve_by_name("ws_123", "Nonexistent Report", ArtifactType::Report)
            .await;

        assert!(matches!(result, Err(FabricError::ArtifactNotFound(_))));
    }

    #[tokio::test]
    async fn resolve_guid_not_found() {
        let mock = MockFabricHttpClient::new();
        let resolver = FabricArtifactResolver::new(Box::new(mock));
        let result = resolver
            .resolve_by_guid("ws_123", "nonexistent", ArtifactType::Report)
            .await;

        assert!(matches!(result, Err(FabricError::ArtifactNotFound(_))));
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cd spool && cargo test -p spool-fabric -- resolution_api`
Expected: FAIL

**Step 3: Write the implementation**

```rust
// spool/spool-fabric/src/resolution_api.rs
use spool_protocol::artifact::{
    ArtifactId, ArtifactIdentity, ArtifactType, CanonicalLocator, ResolutionBasis,
};
use uuid::Uuid;

use crate::client::FabricHttpClient;
use crate::error::FabricError;

/// Resolves Fabric artifacts by GUID or name through the Fabric REST API.
pub struct FabricArtifactResolver {
    http: Box<dyn FabricHttpClient>,
}

impl FabricArtifactResolver {
    pub fn new(http: Box<dyn FabricHttpClient>) -> Self {
        Self { http }
    }

    /// Resolve an artifact by its workspace ID and GUID.
    /// Uses the appropriate Fabric REST endpoint for the artifact type.
    pub async fn resolve_by_guid(
        &self,
        workspace_id: &str,
        artifact_guid: &str,
        artifact_type: ArtifactType,
    ) -> Result<ArtifactIdentity, FabricError> {
        let path = self.guid_endpoint(workspace_id, artifact_guid, &artifact_type);
        let response = self.http.request("GET", &path, None).await?;

        if response.status == 404 {
            return Err(FabricError::ArtifactNotFound(format!(
                "{artifact_type:?} {artifact_guid} not found in workspace {workspace_id}"
            )));
        }

        if response.status >= 400 {
            return Err(FabricError::Api {
                status: response.status,
                message: response.body,
            });
        }

        let body: serde_json::Value = serde_json::from_str(&response.body)?;
        let display_name = body["name"]
            .as_str()
            .unwrap_or("unknown")
            .to_string();

        Ok(ArtifactIdentity {
            artifact_id: ArtifactId(format!("art_{}", Uuid::new_v4().as_simple())),
            artifact_type: artifact_type.clone(),
            workspace_id: Some(workspace_id.to_string()),
            parent_artifact_id: None,
            canonical_locator: CanonicalLocator(
                self.build_locator(workspace_id, artifact_guid, &artifact_type),
            ),
            display_name,
            resolution_basis: ResolutionBasis::ExplicitGuid,
        })
    }

    /// Resolve an artifact by name within a workspace scope.
    /// Lists all artifacts of the given type and matches by name.
    ///
    /// Per Spec Section 3.2: if multiple plausible matches exist, returns
    /// AmbiguousResolution error with candidates.
    pub async fn resolve_by_name(
        &self,
        workspace_id: &str,
        name: &str,
        artifact_type: ArtifactType,
    ) -> Result<ArtifactIdentity, FabricError> {
        let path = self.list_endpoint(workspace_id, &artifact_type);
        let response = self.http.request("GET", &path, None).await?;

        if response.status >= 400 {
            return Err(FabricError::Api {
                status: response.status,
                message: response.body,
            });
        }

        let body: serde_json::Value = serde_json::from_str(&response.body)?;
        let items = body["value"]
            .as_array()
            .ok_or_else(|| FabricError::Resolution("unexpected API response shape".into()))?;

        let matches: Vec<&serde_json::Value> = items
            .iter()
            .filter(|item| item["name"].as_str() == Some(name))
            .collect();

        match matches.len() {
            0 => Err(FabricError::ArtifactNotFound(format!(
                "{artifact_type:?} named '{name}' not found in workspace {workspace_id}"
            ))),
            1 => {
                let item = matches[0];
                let artifact_guid = item["id"]
                    .as_str()
                    .ok_or_else(|| FabricError::Resolution("missing id in API response".into()))?;

                Ok(ArtifactIdentity {
                    artifact_id: ArtifactId(format!("art_{}", Uuid::new_v4().as_simple())),
                    artifact_type: artifact_type.clone(),
                    workspace_id: Some(workspace_id.to_string()),
                    parent_artifact_id: None,
                    canonical_locator: CanonicalLocator(
                        self.build_locator(workspace_id, artifact_guid, &artifact_type),
                    ),
                    display_name: name.to_string(),
                    resolution_basis: ResolutionBasis::UniqueNameMatch,
                })
            }
            n => {
                let candidates: Vec<String> = matches
                    .iter()
                    .filter_map(|item| {
                        let id = item["id"].as_str()?;
                        Some(format!("{name} (id: {id})"))
                    })
                    .collect();

                Err(FabricError::AmbiguousResolution {
                    detail: format!(
                        "{n} {artifact_type:?} artifacts named '{name}' in workspace {workspace_id}"
                    ),
                    candidates,
                })
            }
        }
    }

    fn guid_endpoint(
        &self,
        workspace_id: &str,
        guid: &str,
        artifact_type: &ArtifactType,
    ) -> String {
        match artifact_type {
            ArtifactType::Report => {
                format!("/v1.0/myorg/groups/{workspace_id}/reports/{guid}")
            }
            ArtifactType::SemanticModel => {
                format!("/v1.0/myorg/groups/{workspace_id}/datasets/{guid}")
            }
            ArtifactType::Warehouse => {
                format!("/v1.0/myorg/groups/{workspace_id}/datawarehouses/{guid}")
            }
            _ => format!("/v1.0/myorg/groups/{workspace_id}/items/{guid}"),
        }
    }

    fn list_endpoint(&self, workspace_id: &str, artifact_type: &ArtifactType) -> String {
        match artifact_type {
            ArtifactType::Report => {
                format!("/v1.0/myorg/groups/{workspace_id}/reports")
            }
            ArtifactType::SemanticModel => {
                format!("/v1.0/myorg/groups/{workspace_id}/datasets")
            }
            ArtifactType::Warehouse => {
                format!("/v1.0/myorg/groups/{workspace_id}/datawarehouses")
            }
            _ => format!("/v1.0/myorg/groups/{workspace_id}/items"),
        }
    }

    fn build_locator(
        &self,
        workspace_id: &str,
        guid: &str,
        artifact_type: &ArtifactType,
    ) -> String {
        match artifact_type {
            ArtifactType::Report => {
                format!("fabric://workspace/{workspace_id}/report/{guid}")
            }
            ArtifactType::SemanticModel => {
                format!("fabric://workspace/{workspace_id}/model/{guid}")
            }
            ArtifactType::Warehouse => {
                format!("fabric://workspace/{workspace_id}/warehouse/{guid}")
            }
            _ => format!("fabric://workspace/{workspace_id}/item/{guid}"),
        }
    }
}

// tests at bottom of file (from Step 1)
```

**Step 4: Update lib.rs**

```rust
// spool/spool-fabric/src/lib.rs
pub mod auth;
pub mod capability;
pub mod client;
pub mod client_authenticated;
pub mod error;
pub mod metadata;
pub mod resolution;
pub mod resolution_api;
```

**Step 5: Run test to verify it passes**

Run: `cd spool && cargo test -p spool-fabric -- resolution_api`
Expected: 6 tests PASS

**Step 6: Commit**

```bash
git add spool/spool-fabric/src/resolution_api.rs spool/spool-fabric/src/lib.rs
git commit -m "feat(spool-fabric): artifact resolution from workspace+GUID and scoped name matching with ambiguity handling"
```

---

## Task 8: Child Artifact Derivation

**Files:**

- Create: `spool/spool-fabric/src/resolution_children.rs`
- Modify: `spool/spool-fabric/src/lib.rs`

**Step 1: Write the failing test**

Add to `spool/spool-fabric/src/resolution_children.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use spool_protocol::artifact::{
        ArtifactId, ArtifactIdentity, ArtifactType, CanonicalLocator, ResolutionBasis,
    };

    fn sample_report_identity() -> ArtifactIdentity {
        ArtifactIdentity {
            artifact_id: ArtifactId("art_report_1".into()),
            artifact_type: ArtifactType::Report,
            workspace_id: Some("ws_123".into()),
            parent_artifact_id: None,
            canonical_locator: CanonicalLocator(
                "fabric://workspace/ws_123/report/rpt_456".into(),
            ),
            display_name: "Executive Revenue Report".into(),
            resolution_basis: ResolutionBasis::ReportUrl,
        }
    }

    fn sample_model_identity() -> ArtifactIdentity {
        ArtifactIdentity {
            artifact_id: ArtifactId("art_model_1".into()),
            artifact_type: ArtifactType::SemanticModel,
            workspace_id: Some("ws_123".into()),
            parent_artifact_id: None,
            canonical_locator: CanonicalLocator(
                "fabric://workspace/ws_123/model/mod_789".into(),
            ),
            display_name: "Sales Model".into(),
            resolution_basis: ResolutionBasis::ExplicitGuid,
        }
    }

    #[test]
    fn derive_page_from_report() {
        let report = sample_report_identity();
        let page = derive_page(&report, "ReportSectionSummary", "Summary").unwrap();

        assert_eq!(page.artifact_type, ArtifactType::Page);
        assert_eq!(page.parent_artifact_id, Some(ArtifactId("art_report_1".into())));
        assert_eq!(page.workspace_id, Some("ws_123".to_string()));
        assert_eq!(
            page.canonical_locator.0,
            "fabric://workspace/ws_123/report/rpt_456/page/ReportSectionSummary"
        );
        assert_eq!(page.display_name, "Summary");
        assert_eq!(page.resolution_basis, ResolutionBasis::DerivedFromResolvedParent);
    }

    #[test]
    fn derive_visual_from_page() {
        let report = sample_report_identity();
        let page = derive_page(&report, "ReportSectionSummary", "Summary").unwrap();
        let visual = derive_visual(&page, "rpt_456", "visual_abc", "Revenue Chart").unwrap();

        assert_eq!(visual.artifact_type, ArtifactType::Visual);
        assert_eq!(visual.parent_artifact_id, Some(page.artifact_id.clone()));
        assert!(visual.canonical_locator.0.contains("visual/visual_abc"));
        assert_eq!(visual.display_name, "Revenue Chart");
    }

    #[test]
    fn derive_measure_from_model() {
        let model = sample_model_identity();
        let measure = derive_measure(&model, "mod_789", "Sales", "QoQ Revenue").unwrap();

        assert_eq!(measure.artifact_type, ArtifactType::Measure);
        assert_eq!(measure.parent_artifact_id, Some(ArtifactId("art_model_1".into())));
        assert_eq!(
            measure.canonical_locator.0,
            "fabric://workspace/ws_123/model/mod_789/measure/Sales[QoQ Revenue]"
        );
    }

    #[test]
    fn derive_table_from_model() {
        let model = sample_model_identity();
        let table = derive_table(&model, "mod_789", "Sales").unwrap();

        assert_eq!(table.artifact_type, ArtifactType::Table);
        assert_eq!(
            table.canonical_locator.0,
            "fabric://workspace/ws_123/model/mod_789/table/Sales"
        );
    }

    #[test]
    fn derive_column_from_model() {
        let model = sample_model_identity();
        let column = derive_column(&model, "mod_789", "Sales", "Amount").unwrap();

        assert_eq!(column.artifact_type, ArtifactType::Column);
        assert_eq!(
            column.canonical_locator.0,
            "fabric://workspace/ws_123/model/mod_789/table/Sales/column/Amount"
        );
    }

    #[test]
    fn derive_relationship_from_model() {
        let model = sample_model_identity();
        let rel = derive_relationship(
            &model,
            "mod_789",
            "Sales[ProductKey]->Products[ProductKey]",
        )
        .unwrap();

        assert_eq!(rel.artifact_type, ArtifactType::Relationship);
        assert!(rel.canonical_locator.0.contains("relationship/"));
    }

    #[test]
    fn derive_page_from_non_report_fails() {
        let model = sample_model_identity();
        let result = derive_page(&model, "page_1", "Page");
        assert!(result.is_err());
    }

    #[test]
    fn derive_measure_from_non_model_fails() {
        let report = sample_report_identity();
        let result = derive_measure(&report, "mod_1", "Table", "Measure");
        assert!(result.is_err());
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cd spool && cargo test -p spool-fabric -- resolution_children`
Expected: FAIL

**Step 3: Write the implementation**

```rust
// spool/spool-fabric/src/resolution_children.rs
use spool_protocol::artifact::{
    ArtifactId, ArtifactIdentity, ArtifactType, CanonicalLocator, ResolutionBasis,
};
use uuid::Uuid;

use crate::error::FabricError;

/// Derive a page identity from a resolved report parent.
/// Spec Section 3.4: page is child of report, locator includes page key.
pub fn derive_page(
    report: &ArtifactIdentity,
    page_key: &str,
    display_name: &str,
) -> Result<ArtifactIdentity, FabricError> {
    if report.artifact_type != ArtifactType::Report {
        return Err(FabricError::Resolution(format!(
            "cannot derive page from {:?} — parent must be a report",
            report.artifact_type
        )));
    }

    let workspace_id = report
        .workspace_id
        .as_deref()
        .ok_or_else(|| FabricError::Resolution("report has no workspace_id".into()))?;

    Ok(ArtifactIdentity {
        artifact_id: ArtifactId(format!("art_{}", Uuid::new_v4().as_simple())),
        artifact_type: ArtifactType::Page,
        workspace_id: Some(workspace_id.to_string()),
        parent_artifact_id: Some(report.artifact_id.clone()),
        canonical_locator: CanonicalLocator(format!(
            "{}/page/{page_key}",
            report.canonical_locator.0
        )),
        display_name: display_name.to_string(),
        resolution_basis: ResolutionBasis::DerivedFromResolvedParent,
    })
}

/// Derive a visual identity from a resolved page parent.
/// Spec Section 3.4: visual is child of page, locator includes visual key.
pub fn derive_visual(
    page: &ArtifactIdentity,
    report_id: &str,
    visual_key: &str,
    display_name: &str,
) -> Result<ArtifactIdentity, FabricError> {
    if page.artifact_type != ArtifactType::Page {
        return Err(FabricError::Resolution(format!(
            "cannot derive visual from {:?} — parent must be a page",
            page.artifact_type
        )));
    }

    let workspace_id = page
        .workspace_id
        .as_deref()
        .ok_or_else(|| FabricError::Resolution("page has no workspace_id".into()))?;

    Ok(ArtifactIdentity {
        artifact_id: ArtifactId(format!("art_{}", Uuid::new_v4().as_simple())),
        artifact_type: ArtifactType::Visual,
        workspace_id: Some(workspace_id.to_string()),
        parent_artifact_id: Some(page.artifact_id.clone()),
        canonical_locator: CanonicalLocator(format!(
            "{}/visual/{visual_key}",
            page.canonical_locator.0
        )),
        display_name: display_name.to_string(),
        resolution_basis: ResolutionBasis::DerivedFromResolvedParent,
    })
}

/// Derive a measure identity from a resolved semantic model parent.
/// Spec Section 3.4: measure locator includes table_name + measure_name.
pub fn derive_measure(
    model: &ArtifactIdentity,
    model_id: &str,
    table_name: &str,
    measure_name: &str,
) -> Result<ArtifactIdentity, FabricError> {
    if model.artifact_type != ArtifactType::SemanticModel {
        return Err(FabricError::Resolution(format!(
            "cannot derive measure from {:?} — parent must be a semantic model",
            model.artifact_type
        )));
    }

    let workspace_id = model
        .workspace_id
        .as_deref()
        .ok_or_else(|| FabricError::Resolution("model has no workspace_id".into()))?;

    Ok(ArtifactIdentity {
        artifact_id: ArtifactId(format!("art_{}", Uuid::new_v4().as_simple())),
        artifact_type: ArtifactType::Measure,
        workspace_id: Some(workspace_id.to_string()),
        parent_artifact_id: Some(model.artifact_id.clone()),
        canonical_locator: CanonicalLocator(format!(
            "fabric://workspace/{workspace_id}/model/{model_id}/measure/{table_name}[{measure_name}]"
        )),
        display_name: measure_name.to_string(),
        resolution_basis: ResolutionBasis::DerivedFromResolvedParent,
    })
}

/// Derive a table identity from a resolved semantic model parent.
/// Spec Section 3.4: table locator includes table_name.
pub fn derive_table(
    model: &ArtifactIdentity,
    model_id: &str,
    table_name: &str,
) -> Result<ArtifactIdentity, FabricError> {
    if model.artifact_type != ArtifactType::SemanticModel {
        return Err(FabricError::Resolution(format!(
            "cannot derive table from {:?} — parent must be a semantic model",
            model.artifact_type
        )));
    }

    let workspace_id = model
        .workspace_id
        .as_deref()
        .ok_or_else(|| FabricError::Resolution("model has no workspace_id".into()))?;

    Ok(ArtifactIdentity {
        artifact_id: ArtifactId(format!("art_{}", Uuid::new_v4().as_simple())),
        artifact_type: ArtifactType::Table,
        workspace_id: Some(workspace_id.to_string()),
        parent_artifact_id: Some(model.artifact_id.clone()),
        canonical_locator: CanonicalLocator(format!(
            "fabric://workspace/{workspace_id}/model/{model_id}/table/{table_name}"
        )),
        display_name: table_name.to_string(),
        resolution_basis: ResolutionBasis::DerivedFromResolvedParent,
    })
}

/// Derive a column identity from a resolved semantic model parent.
/// Spec Section 3.4: column locator includes table_name + column_name.
pub fn derive_column(
    model: &ArtifactIdentity,
    model_id: &str,
    table_name: &str,
    column_name: &str,
) -> Result<ArtifactIdentity, FabricError> {
    if model.artifact_type != ArtifactType::SemanticModel {
        return Err(FabricError::Resolution(format!(
            "cannot derive column from {:?} — parent must be a semantic model",
            model.artifact_type
        )));
    }

    let workspace_id = model
        .workspace_id
        .as_deref()
        .ok_or_else(|| FabricError::Resolution("model has no workspace_id".into()))?;

    Ok(ArtifactIdentity {
        artifact_id: ArtifactId(format!("art_{}", Uuid::new_v4().as_simple())),
        artifact_type: ArtifactType::Column,
        workspace_id: Some(workspace_id.to_string()),
        parent_artifact_id: Some(model.artifact_id.clone()),
        canonical_locator: CanonicalLocator(format!(
            "fabric://workspace/{workspace_id}/model/{model_id}/table/{table_name}/column/{column_name}"
        )),
        display_name: column_name.to_string(),
        resolution_basis: ResolutionBasis::DerivedFromResolvedParent,
    })
}

/// Derive a relationship identity from a resolved semantic model parent.
/// Spec Section 3.5: uses a deterministic relationship key.
pub fn derive_relationship(
    model: &ArtifactIdentity,
    model_id: &str,
    relationship_key: &str,
) -> Result<ArtifactIdentity, FabricError> {
    if model.artifact_type != ArtifactType::SemanticModel {
        return Err(FabricError::Resolution(format!(
            "cannot derive relationship from {:?} — parent must be a semantic model",
            model.artifact_type
        )));
    }

    let workspace_id = model
        .workspace_id
        .as_deref()
        .ok_or_else(|| FabricError::Resolution("model has no workspace_id".into()))?;

    Ok(ArtifactIdentity {
        artifact_id: ArtifactId(format!("art_{}", Uuid::new_v4().as_simple())),
        artifact_type: ArtifactType::Relationship,
        workspace_id: Some(workspace_id.to_string()),
        parent_artifact_id: Some(model.artifact_id.clone()),
        canonical_locator: CanonicalLocator(format!(
            "fabric://workspace/{workspace_id}/model/{model_id}/relationship/{relationship_key}"
        )),
        display_name: relationship_key.to_string(),
        resolution_basis: ResolutionBasis::DerivedFromResolvedParent,
    })
}

// tests at bottom of file (from Step 1)
```

**Step 4: Update lib.rs**

```rust
// spool/spool-fabric/src/lib.rs
pub mod auth;
pub mod capability;
pub mod client;
pub mod client_authenticated;
pub mod error;
pub mod metadata;
pub mod resolution;
pub mod resolution_api;
pub mod resolution_children;
```

**Step 5: Run test to verify it passes**

Run: `cd spool && cargo test -p spool-fabric -- resolution_children`
Expected: 8 tests PASS

**Step 6: Commit**

```bash
git add spool/spool-fabric/src/resolution_children.rs spool/spool-fabric/src/lib.rs
git commit -m "feat(spool-fabric): child artifact derivation for pages, visuals, measures, tables, columns, and relationships"
```

---

## Task 9: Report And Semantic Model Metadata Inspection

**Files:**

- Modify: `spool/spool-fabric/src/metadata.rs`

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::MockFabricHttpClient;

    fn report_pages_response() -> serde_json::Value {
        serde_json::json!({
            "value": [
                {
                    "name": "ReportSectionSummary",
                    "displayName": "Summary",
                    "order": 0
                },
                {
                    "name": "ReportSectionDetail",
                    "displayName": "Detail",
                    "order": 1
                }
            ]
        })
    }

    fn model_tables_response() -> serde_json::Value {
        serde_json::json!({
            "value": [
                {
                    "name": "Sales",
                    "columns": [
                        {"name": "Amount", "dataType": "Decimal", "isHidden": false},
                        {"name": "Date", "dataType": "DateTime", "isHidden": false}
                    ],
                    "measures": [
                        {"name": "Total Revenue", "expression": "SUM(Sales[Amount])"},
                        {"name": "QoQ Revenue", "expression": "CALCULATE(SUM(Sales[Amount]), DATEADD('Date'[Date], -1, QUARTER))"}
                    ]
                },
                {
                    "name": "Products",
                    "columns": [
                        {"name": "ProductKey", "dataType": "Int64", "isHidden": false},
                        {"name": "ProductName", "dataType": "String", "isHidden": false}
                    ],
                    "measures": []
                }
            ]
        })
    }

    fn model_relationships_response() -> serde_json::Value {
        serde_json::json!({
            "value": [
                {
                    "name": "rel_1",
                    "fromTable": "Sales",
                    "fromColumn": "ProductKey",
                    "toTable": "Products",
                    "toColumn": "ProductKey",
                    "crossFilteringBehavior": "oneDirection"
                }
            ]
        })
    }

    #[tokio::test]
    async fn inspect_report_metadata_returns_pages() {
        let mut mock = MockFabricHttpClient::new();
        mock.register_response(
            "GET",
            "/v1.0/myorg/groups/ws_123/reports/rpt_456/pages",
            200,
            report_pages_response(),
        );

        let inspector = FabricMetadataInspector::new(Box::new(mock));
        let metadata = inspector
            .inspect_report("ws_123", "rpt_456")
            .await
            .unwrap();

        assert_eq!(metadata.workspace_id, "ws_123");
        assert_eq!(metadata.report_id, "rpt_456");
        assert_eq!(metadata.pages.len(), 2);
        assert_eq!(metadata.pages[0].name, "ReportSectionSummary");
        assert_eq!(metadata.pages[0].display_name, "Summary");
        assert_eq!(metadata.pages[1].order, Some(1));
    }

    #[tokio::test]
    async fn inspect_model_metadata_returns_tables_and_measures() {
        let mut mock = MockFabricHttpClient::new();
        mock.register_response(
            "GET",
            "/v1.0/myorg/groups/ws_123/datasets/mod_789/tables",
            200,
            model_tables_response(),
        );
        mock.register_response(
            "GET",
            "/v1.0/myorg/groups/ws_123/datasets/mod_789/relationships",
            200,
            model_relationships_response(),
        );

        let inspector = FabricMetadataInspector::new(Box::new(mock));
        let metadata = inspector
            .inspect_semantic_model("ws_123", "mod_789")
            .await
            .unwrap();

        assert_eq!(metadata.workspace_id, "ws_123");
        assert_eq!(metadata.model_id, "mod_789");
        assert_eq!(metadata.tables.len(), 2);
        assert_eq!(metadata.tables[0].name, "Sales");
        assert_eq!(metadata.tables[0].columns.len(), 2);
        assert_eq!(metadata.tables[0].measures.len(), 2);
        assert_eq!(metadata.relationships.len(), 1);
    }

    #[tokio::test]
    async fn inspect_measure_definition() {
        let mut mock = MockFabricHttpClient::new();
        mock.register_response(
            "GET",
            "/v1.0/myorg/groups/ws_123/datasets/mod_789/tables",
            200,
            model_tables_response(),
        );

        let inspector = FabricMetadataInspector::new(Box::new(mock));
        let definition = inspector
            .inspect_measure("ws_123", "mod_789", "Sales", "QoQ Revenue")
            .await
            .unwrap();

        assert_eq!(definition.table_name, "Sales");
        assert_eq!(definition.measure_name, "QoQ Revenue");
        assert!(definition.expression.contains("CALCULATE"));
    }

    #[tokio::test]
    async fn inspect_measure_not_found() {
        let mut mock = MockFabricHttpClient::new();
        mock.register_response(
            "GET",
            "/v1.0/myorg/groups/ws_123/datasets/mod_789/tables",
            200,
            model_tables_response(),
        );

        let inspector = FabricMetadataInspector::new(Box::new(mock));
        let result = inspector
            .inspect_measure("ws_123", "mod_789", "Sales", "NonexistentMeasure")
            .await;

        assert!(matches!(result, Err(FabricError::ArtifactNotFound(_))));
    }

    #[test]
    fn report_metadata_round_trip() {
        let metadata = ReportMetadata {
            workspace_id: "ws_123".into(),
            report_id: "rpt_456".into(),
            pages: vec![PageMetadata {
                name: "ReportSectionSummary".into(),
                display_name: "Summary".into(),
                order: Some(0),
            }],
        };
        let json = serde_json::to_string(&metadata).unwrap();
        let restored: ReportMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.pages.len(), 1);
    }

    #[test]
    fn model_metadata_round_trip() {
        let metadata = SemanticModelMetadata {
            workspace_id: "ws_123".into(),
            model_id: "mod_789".into(),
            tables: vec![TableMetadata {
                name: "Sales".into(),
                columns: vec![ColumnMetadata {
                    name: "Amount".into(),
                    data_type: "Decimal".into(),
                    is_hidden: false,
                }],
                measures: vec![MeasureMetadata {
                    name: "Total Revenue".into(),
                    expression: "SUM(Sales[Amount])".into(),
                }],
            }],
            relationships: vec![RelationshipMetadata {
                name: "rel_1".into(),
                from_table: "Sales".into(),
                from_column: "ProductKey".into(),
                to_table: "Products".into(),
                to_column: "ProductKey".into(),
                cross_filtering: Some("oneDirection".into()),
            }],
        };
        let json = serde_json::to_string(&metadata).unwrap();
        let restored: SemanticModelMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.tables.len(), 1);
        assert_eq!(restored.relationships.len(), 1);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cd spool && cargo test -p spool-fabric -- metadata`
Expected: FAIL

**Step 3: Write the implementation**

```rust
// spool/spool-fabric/src/metadata.rs
use serde::{Deserialize, Serialize};

use crate::client::FabricHttpClient;
use crate::error::FabricError;

// --- Metadata types ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageMetadata {
    pub name: String,
    pub display_name: String,
    pub order: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportMetadata {
    pub workspace_id: String,
    pub report_id: String,
    pub pages: Vec<PageMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnMetadata {
    pub name: String,
    pub data_type: String,
    pub is_hidden: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeasureMetadata {
    pub name: String,
    pub expression: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableMetadata {
    pub name: String,
    pub columns: Vec<ColumnMetadata>,
    pub measures: Vec<MeasureMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipMetadata {
    pub name: String,
    pub from_table: String,
    pub from_column: String,
    pub to_table: String,
    pub to_column: String,
    pub cross_filtering: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticModelMetadata {
    pub workspace_id: String,
    pub model_id: String,
    pub tables: Vec<TableMetadata>,
    pub relationships: Vec<RelationshipMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeasureDefinition {
    pub table_name: String,
    pub measure_name: String,
    pub expression: String,
}

// --- Inspector ---

/// Inspects Fabric report and semantic model metadata via the REST API.
pub struct FabricMetadataInspector {
    http: Box<dyn FabricHttpClient>,
}

impl FabricMetadataInspector {
    pub fn new(http: Box<dyn FabricHttpClient>) -> Self {
        Self { http }
    }

    /// Retrieve report metadata including page list.
    pub async fn inspect_report(
        &self,
        workspace_id: &str,
        report_id: &str,
    ) -> Result<ReportMetadata, FabricError> {
        let path = format!("/v1.0/myorg/groups/{workspace_id}/reports/{report_id}/pages");
        let response = self.http.request("GET", &path, None).await?;

        if response.status >= 400 {
            return Err(FabricError::Metadata(format!(
                "failed to retrieve report pages: HTTP {}",
                response.status
            )));
        }

        let body: serde_json::Value = serde_json::from_str(&response.body)?;
        let pages_array = body["value"]
            .as_array()
            .ok_or_else(|| FabricError::Metadata("unexpected response shape for pages".into()))?;

        let pages: Vec<PageMetadata> = pages_array
            .iter()
            .map(|p| PageMetadata {
                name: p["name"].as_str().unwrap_or("").to_string(),
                display_name: p["displayName"].as_str().unwrap_or("").to_string(),
                order: p["order"].as_u64().map(|n| n as u32),
            })
            .collect();

        Ok(ReportMetadata {
            workspace_id: workspace_id.to_string(),
            report_id: report_id.to_string(),
            pages,
        })
    }

    /// Retrieve semantic model metadata including tables, columns, measures, and relationships.
    pub async fn inspect_semantic_model(
        &self,
        workspace_id: &str,
        model_id: &str,
    ) -> Result<SemanticModelMetadata, FabricError> {
        let tables = self.fetch_tables(workspace_id, model_id).await?;
        let relationships = self.fetch_relationships(workspace_id, model_id).await?;

        Ok(SemanticModelMetadata {
            workspace_id: workspace_id.to_string(),
            model_id: model_id.to_string(),
            tables,
            relationships,
        })
    }

    /// Retrieve a specific measure definition by table and measure name.
    pub async fn inspect_measure(
        &self,
        workspace_id: &str,
        model_id: &str,
        table_name: &str,
        measure_name: &str,
    ) -> Result<MeasureDefinition, FabricError> {
        let tables = self.fetch_tables(workspace_id, model_id).await?;

        let table = tables
            .iter()
            .find(|t| t.name == table_name)
            .ok_or_else(|| {
                FabricError::ArtifactNotFound(format!(
                    "table '{table_name}' not found in model {model_id}"
                ))
            })?;

        let measure = table
            .measures
            .iter()
            .find(|m| m.name == measure_name)
            .ok_or_else(|| {
                FabricError::ArtifactNotFound(format!(
                    "measure '{measure_name}' not found in table '{table_name}'"
                ))
            })?;

        Ok(MeasureDefinition {
            table_name: table_name.to_string(),
            measure_name: measure_name.to_string(),
            expression: measure.expression.clone(),
        })
    }

    async fn fetch_tables(
        &self,
        workspace_id: &str,
        model_id: &str,
    ) -> Result<Vec<TableMetadata>, FabricError> {
        let path = format!("/v1.0/myorg/groups/{workspace_id}/datasets/{model_id}/tables");
        let response = self.http.request("GET", &path, None).await?;

        if response.status >= 400 {
            return Err(FabricError::Metadata(format!(
                "failed to retrieve model tables: HTTP {}",
                response.status
            )));
        }

        let body: serde_json::Value = serde_json::from_str(&response.body)?;
        let tables_array = body["value"]
            .as_array()
            .ok_or_else(|| FabricError::Metadata("unexpected response shape for tables".into()))?;

        let tables: Vec<TableMetadata> = tables_array
            .iter()
            .map(|t| {
                let columns = t["columns"]
                    .as_array()
                    .map(|cols| {
                        cols.iter()
                            .map(|c| ColumnMetadata {
                                name: c["name"].as_str().unwrap_or("").to_string(),
                                data_type: c["dataType"].as_str().unwrap_or("").to_string(),
                                is_hidden: c["isHidden"].as_bool().unwrap_or(false),
                            })
                            .collect()
                    })
                    .unwrap_or_default();

                let measures = t["measures"]
                    .as_array()
                    .map(|ms| {
                        ms.iter()
                            .map(|m| MeasureMetadata {
                                name: m["name"].as_str().unwrap_or("").to_string(),
                                expression: m["expression"].as_str().unwrap_or("").to_string(),
                            })
                            .collect()
                    })
                    .unwrap_or_default();

                TableMetadata {
                    name: t["name"].as_str().unwrap_or("").to_string(),
                    columns,
                    measures,
                }
            })
            .collect();

        Ok(tables)
    }

    async fn fetch_relationships(
        &self,
        workspace_id: &str,
        model_id: &str,
    ) -> Result<Vec<RelationshipMetadata>, FabricError> {
        let path =
            format!("/v1.0/myorg/groups/{workspace_id}/datasets/{model_id}/relationships");
        let response = self.http.request("GET", &path, None).await?;

        if response.status >= 400 {
            return Err(FabricError::Metadata(format!(
                "failed to retrieve model relationships: HTTP {}",
                response.status
            )));
        }

        let body: serde_json::Value = serde_json::from_str(&response.body)?;
        let rels_array = body["value"].as_array().ok_or_else(|| {
            FabricError::Metadata("unexpected response shape for relationships".into())
        })?;

        let relationships: Vec<RelationshipMetadata> = rels_array
            .iter()
            .map(|r| RelationshipMetadata {
                name: r["name"].as_str().unwrap_or("").to_string(),
                from_table: r["fromTable"].as_str().unwrap_or("").to_string(),
                from_column: r["fromColumn"].as_str().unwrap_or("").to_string(),
                to_table: r["toTable"].as_str().unwrap_or("").to_string(),
                to_column: r["toColumn"].as_str().unwrap_or("").to_string(),
                cross_filtering: r["crossFilteringBehavior"]
                    .as_str()
                    .map(|s| s.to_string()),
            })
            .collect();

        Ok(relationships)
    }
}

// tests at bottom of file (from Step 1)
```

**Step 4: Run test to verify it passes**

Run: `cd spool && cargo test -p spool-fabric -- metadata`
Expected: 6 tests PASS

**Step 5: Commit**

```bash
git add spool/spool-fabric/src/metadata.rs
git commit -m "feat(spool-fabric): report and semantic model metadata inspection with measure definition retrieval"
```

---

## Task 10: Visual Binding Metadata Inspection

**Files:**

- Create: `spool/spool-fabric/src/metadata_visuals.rs`
- Modify: `spool/spool-fabric/src/lib.rs`

**Step 1: Write the failing test**

Add to `spool/spool-fabric/src/metadata_visuals.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::MockFabricHttpClient;

    fn visuals_response() -> serde_json::Value {
        serde_json::json!({
            "value": [
                {
                    "name": "visual_abc",
                    "title": "Revenue Chart",
                    "type": "barChart",
                    "dataBindings": [
                        {
                            "role": "Category",
                            "table": "Sales",
                            "column": "Date"
                        },
                        {
                            "role": "Values",
                            "table": "Sales",
                            "measure": "Total Revenue"
                        }
                    ],
                    "filters": [
                        {
                            "type": "basic",
                            "target": {"table": "Sales", "column": "Region"},
                            "operator": "In",
                            "values": ["North", "South"]
                        }
                    ]
                },
                {
                    "name": "visual_def",
                    "title": "Product Table",
                    "type": "table",
                    "dataBindings": [
                        {
                            "role": "Values",
                            "table": "Products",
                            "column": "ProductName"
                        }
                    ],
                    "filters": []
                }
            ]
        })
    }

    #[tokio::test]
    async fn inspect_visual_bindings() {
        let mut mock = MockFabricHttpClient::new();
        mock.register_response(
            "GET",
            "/v1.0/myorg/groups/ws_123/reports/rpt_456/pages/ReportSectionSummary/visuals",
            200,
            visuals_response(),
        );

        let inspector = VisualMetadataInspector::new(Box::new(mock));
        let visuals = inspector
            .inspect_visuals("ws_123", "rpt_456", "ReportSectionSummary")
            .await
            .unwrap();

        assert_eq!(visuals.len(), 2);
        assert_eq!(visuals[0].name, "visual_abc");
        assert_eq!(visuals[0].title, "Revenue Chart");
        assert_eq!(visuals[0].visual_type, "barChart");
        assert_eq!(visuals[0].data_bindings.len(), 2);
        assert_eq!(visuals[0].data_bindings[0].role, "Category");
        assert_eq!(visuals[0].data_bindings[0].table, "Sales");
        assert_eq!(visuals[0].data_bindings[0].column, Some("Date".to_string()));
        assert_eq!(visuals[0].data_bindings[1].measure, Some("Total Revenue".to_string()));
        assert_eq!(visuals[0].filters.len(), 1);
    }

    #[tokio::test]
    async fn inspect_single_visual_by_name() {
        let mut mock = MockFabricHttpClient::new();
        mock.register_response(
            "GET",
            "/v1.0/myorg/groups/ws_123/reports/rpt_456/pages/ReportSectionSummary/visuals",
            200,
            visuals_response(),
        );

        let inspector = VisualMetadataInspector::new(Box::new(mock));
        let visual = inspector
            .inspect_visual_by_name("ws_123", "rpt_456", "ReportSectionSummary", "visual_abc")
            .await
            .unwrap();

        assert_eq!(visual.title, "Revenue Chart");
    }

    #[tokio::test]
    async fn inspect_visual_not_found() {
        let mut mock = MockFabricHttpClient::new();
        mock.register_response(
            "GET",
            "/v1.0/myorg/groups/ws_123/reports/rpt_456/pages/ReportSectionSummary/visuals",
            200,
            visuals_response(),
        );

        let inspector = VisualMetadataInspector::new(Box::new(mock));
        let result = inspector
            .inspect_visual_by_name("ws_123", "rpt_456", "ReportSectionSummary", "nonexistent")
            .await;

        assert!(matches!(result, Err(FabricError::ArtifactNotFound(_))));
    }

    #[test]
    fn visual_metadata_round_trip() {
        let visual = VisualBindingMetadata {
            name: "visual_abc".into(),
            title: "Revenue Chart".into(),
            visual_type: "barChart".into(),
            data_bindings: vec![DataBinding {
                role: "Values".into(),
                table: "Sales".into(),
                column: None,
                measure: Some("Total Revenue".into()),
            }],
            filters: vec![VisualFilter {
                filter_type: "basic".into(),
                target_table: Some("Sales".into()),
                target_column: Some("Region".into()),
                operator: Some("In".into()),
                values: Some(vec!["North".into(), "South".into()]),
            }],
        };

        let json = serde_json::to_string(&visual).unwrap();
        let restored: VisualBindingMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.data_bindings.len(), 1);
        assert_eq!(restored.filters.len(), 1);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cd spool && cargo test -p spool-fabric -- metadata_visuals`
Expected: FAIL

**Step 3: Write the implementation**

```rust
// spool/spool-fabric/src/metadata_visuals.rs
use serde::{Deserialize, Serialize};

use crate::client::FabricHttpClient;
use crate::error::FabricError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataBinding {
    pub role: String,
    pub table: String,
    pub column: Option<String>,
    pub measure: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualFilter {
    pub filter_type: String,
    pub target_table: Option<String>,
    pub target_column: Option<String>,
    pub operator: Option<String>,
    pub values: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualBindingMetadata {
    pub name: String,
    pub title: String,
    pub visual_type: String,
    pub data_bindings: Vec<DataBinding>,
    pub filters: Vec<VisualFilter>,
}

/// Inspects visual binding metadata within Fabric reports.
pub struct VisualMetadataInspector {
    http: Box<dyn FabricHttpClient>,
}

impl VisualMetadataInspector {
    pub fn new(http: Box<dyn FabricHttpClient>) -> Self {
        Self { http }
    }

    /// Retrieve all visual binding metadata for a report page.
    pub async fn inspect_visuals(
        &self,
        workspace_id: &str,
        report_id: &str,
        page_name: &str,
    ) -> Result<Vec<VisualBindingMetadata>, FabricError> {
        let path = format!(
            "/v1.0/myorg/groups/{workspace_id}/reports/{report_id}/pages/{page_name}/visuals"
        );
        let response = self.http.request("GET", &path, None).await?;

        if response.status >= 400 {
            return Err(FabricError::Metadata(format!(
                "failed to retrieve visuals: HTTP {}",
                response.status
            )));
        }

        let body: serde_json::Value = serde_json::from_str(&response.body)?;
        let visuals_array = body["value"]
            .as_array()
            .ok_or_else(|| FabricError::Metadata("unexpected response shape for visuals".into()))?;

        let visuals: Vec<VisualBindingMetadata> = visuals_array
            .iter()
            .map(|v| parse_visual(v))
            .collect();

        Ok(visuals)
    }

    /// Retrieve a specific visual's binding metadata by visual name.
    pub async fn inspect_visual_by_name(
        &self,
        workspace_id: &str,
        report_id: &str,
        page_name: &str,
        visual_name: &str,
    ) -> Result<VisualBindingMetadata, FabricError> {
        let all_visuals = self
            .inspect_visuals(workspace_id, report_id, page_name)
            .await?;

        all_visuals
            .into_iter()
            .find(|v| v.name == visual_name)
            .ok_or_else(|| {
                FabricError::ArtifactNotFound(format!(
                    "visual '{visual_name}' not found on page '{page_name}'"
                ))
            })
    }
}

fn parse_visual(v: &serde_json::Value) -> VisualBindingMetadata {
    let data_bindings = v["dataBindings"]
        .as_array()
        .map(|bindings| {
            bindings
                .iter()
                .map(|b| DataBinding {
                    role: b["role"].as_str().unwrap_or("").to_string(),
                    table: b["table"].as_str().unwrap_or("").to_string(),
                    column: b["column"].as_str().map(|s| s.to_string()),
                    measure: b["measure"].as_str().map(|s| s.to_string()),
                })
                .collect()
        })
        .unwrap_or_default();

    let filters = v["filters"]
        .as_array()
        .map(|fs| {
            fs.iter()
                .map(|f| VisualFilter {
                    filter_type: f["type"].as_str().unwrap_or("").to_string(),
                    target_table: f["target"]["table"].as_str().map(|s| s.to_string()),
                    target_column: f["target"]["column"].as_str().map(|s| s.to_string()),
                    operator: f["operator"].as_str().map(|s| s.to_string()),
                    values: f["values"].as_array().map(|vals| {
                        vals.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    }),
                })
                .collect()
        })
        .unwrap_or_default();

    VisualBindingMetadata {
        name: v["name"].as_str().unwrap_or("").to_string(),
        title: v["title"].as_str().unwrap_or("").to_string(),
        visual_type: v["type"].as_str().unwrap_or("").to_string(),
        data_bindings,
        filters,
    }
}

// tests at bottom of file (from Step 1)
```

**Step 4: Update lib.rs**

```rust
// spool/spool-fabric/src/lib.rs
pub mod auth;
pub mod capability;
pub mod client;
pub mod client_authenticated;
pub mod error;
pub mod metadata;
pub mod metadata_visuals;
pub mod resolution;
pub mod resolution_api;
pub mod resolution_children;
```

**Step 5: Run test to verify it passes**

Run: `cd spool && cargo test -p spool-fabric -- metadata_visuals`
Expected: 4 tests PASS

**Step 6: Commit**

```bash
git add spool/spool-fabric/src/metadata_visuals.rs spool/spool-fabric/src/lib.rs
git commit -m "feat(spool-fabric): visual binding metadata inspection with data bindings and filter metadata"
```

---

## Task 11: Fabric Capability Contract Declaration

**Files:**

- Modify: `spool/spool-fabric/src/capability.rs`

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use spool_protocol::capability::*;

    #[test]
    fn fabric_capability_contract_is_complete() {
        let contract = fabric_capability_contract();

        assert_eq!(contract.adapter_id, "spool-fabric");
        assert_eq!(contract.platform_family, "fabric");
        assert_eq!(contract.status, AdapterStatus::Active);

        // All v1 artifact kinds declared
        assert!(contract.supports_artifact_kind("report"));
        assert!(contract.supports_artifact_kind("report_page"));
        assert!(contract.supports_artifact_kind("visual"));
        assert!(contract.supports_artifact_kind("semantic_model"));
        assert!(contract.supports_artifact_kind("measure"));
        assert!(contract.supports_artifact_kind("table"));
        assert!(contract.supports_artifact_kind("column"));
        assert!(contract.supports_artifact_kind("relationship"));
        assert!(contract.supports_artifact_kind("warehouse"));
    }

    #[test]
    fn fabric_inspection_capabilities_declared() {
        let contract = fabric_capability_contract();

        assert!(contract.supports_inspection(&InspectionCapability::ResolveArtifactFromReportUrl));
        assert!(contract.supports_inspection(&InspectionCapability::ResolveArtifactFromWorkspaceAndGuid));
        assert!(contract.supports_inspection(&InspectionCapability::InspectReportMetadata));
        assert!(contract.supports_inspection(&InspectionCapability::InspectSemanticModelMetadata));
        assert!(contract.supports_inspection(&InspectionCapability::InspectMeasureDefinition));
        assert!(contract.supports_inspection(&InspectionCapability::InspectVisualBindingMetadata));
        assert!(contract.supports_inspection(&InspectionCapability::InspectWarehouseMetadata));
    }

    #[test]
    fn fabric_validation_capabilities_declared() {
        let contract = fabric_capability_contract();

        assert!(contract.supports_validation(&ValidationCapability::RunDaxQuery));
        assert!(contract.supports_validation(&ValidationCapability::RunReadOnlyWarehouseSql));
        assert!(contract.supports_validation(&ValidationCapability::CompareReportOutputToDaxResult));
        assert!(contract.supports_validation(&ValidationCapability::CompareDaxResultToWarehouseResult));
    }

    #[test]
    fn fabric_is_proposal_only() {
        let contract = fabric_capability_contract();

        assert!(contract.mutation_capabilities.is_empty());
        assert_eq!(contract.mutation_mode, MutationMode::ProposalOnly);
    }

    #[test]
    fn fabric_safety_rules_enforced() {
        let contract = fabric_capability_contract();

        assert_eq!(contract.safety_rules.warehouse_sql, SafetyPolicy::ReadOnlyOnly);
        assert_eq!(contract.safety_rules.fabric_mutation, SafetyPolicy::DisallowedInV1);
        assert_eq!(
            contract.safety_rules.cross_workspace_scope_expansion,
            SafetyPolicy::RequiresUserConfirmation
        );
        assert_eq!(
            contract.safety_rules.ambiguous_artifact_resolution,
            SafetyPolicy::RequiresUserChoice
        );
    }

    #[test]
    fn fabric_auth_requirements_declared() {
        let contract = fabric_capability_contract();

        assert!(contract.auth_requirements.contains(&AuthRequirement::ProductLogin));
        assert!(contract.auth_requirements.contains(&AuthRequirement::FabricAccessAuth));
    }

    #[test]
    fn fabric_capability_contract_serializes_round_trip() {
        let contract = fabric_capability_contract();
        let json = serde_json::to_string_pretty(&contract).unwrap();
        let restored: PlatformCapabilityContract = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.adapter_id, contract.adapter_id);
        assert_eq!(restored.artifact_kinds.len(), contract.artifact_kinds.len());
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cd spool && cargo test -p spool-fabric -- capability`
Expected: FAIL

**Step 3: Write the implementation**

```rust
// spool/spool-fabric/src/capability.rs
use spool_protocol::capability::*;

/// Return the canonical Fabric platform capability contract for v1.
///
/// This is the machine-readable declaration of what the `spool-fabric`
/// adapter can do, per Spec Section 11.4 and 11.4.1.
///
/// The planner, generator, and evaluator use this contract to:
/// - choose only valid actions
/// - avoid hallucinating unsupported operations
/// - judge whether evidence or conclusions exceed adapter capability
pub fn fabric_capability_contract() -> PlatformCapabilityContract {
    PlatformCapabilityContract {
        adapter_id: "spool-fabric".into(),
        platform_family: "fabric".into(),
        status: AdapterStatus::Active,
        artifact_kinds: vec![
            "report".into(),
            "report_page".into(),
            "visual".into(),
            "semantic_model".into(),
            "measure".into(),
            "table".into(),
            "column".into(),
            "relationship".into(),
            "warehouse".into(),
        ],
        inspection_capabilities: vec![
            InspectionCapability::ResolveArtifactFromReportUrl,
            InspectionCapability::ResolveArtifactFromWorkspaceAndGuid,
            InspectionCapability::InspectReportMetadata,
            InspectionCapability::InspectSemanticModelMetadata,
            InspectionCapability::InspectMeasureDefinition,
            InspectionCapability::InspectVisualBindingMetadata,
            InspectionCapability::InspectWarehouseMetadata,
        ],
        validation_capabilities: vec![
            ValidationCapability::RunDaxQuery,
            ValidationCapability::RunReadOnlyWarehouseSql,
            ValidationCapability::CompareReportOutputToDaxResult,
            ValidationCapability::CompareDaxResultToWarehouseResult,
        ],
        mutation_capabilities: vec![],
        mutation_mode: MutationMode::ProposalOnly,
        identity_locator_shapes: vec![
            "fabric://workspace/{workspace_id}/report/{report_id}".into(),
            "fabric://workspace/{workspace_id}/report/{report_id}/page/{page_name}".into(),
            "fabric://workspace/{workspace_id}/report/{report_id}/page/{page_name}/visual/{visual_name}".into(),
            "fabric://workspace/{workspace_id}/model/{model_id}".into(),
            "fabric://workspace/{workspace_id}/model/{model_id}/measure/{table}[{measure}]".into(),
            "fabric://workspace/{workspace_id}/model/{model_id}/table/{table}".into(),
            "fabric://workspace/{workspace_id}/model/{model_id}/table/{table}/column/{column}".into(),
            "fabric://workspace/{workspace_id}/model/{model_id}/relationship/{key}".into(),
            "fabric://workspace/{workspace_id}/warehouse/{warehouse_id}".into(),
        ],
        evidence_classes: vec![
            "report_metadata".into(),
            "visual_metadata".into(),
            "semantic_model_metadata".into(),
            "measure_definition".into(),
            "dax_query_result".into(),
            "warehouse_query_result".into(),
            "cross_source_comparison".into(),
        ],
        safety_rules: SafetyRules {
            warehouse_sql: SafetyPolicy::ReadOnlyOnly,
            fabric_mutation: SafetyPolicy::DisallowedInV1,
            cross_workspace_scope_expansion: SafetyPolicy::RequiresUserConfirmation,
            ambiguous_artifact_resolution: SafetyPolicy::RequiresUserChoice,
        },
        freshness_and_drift: vec![
            "report definitions can drift".into(),
            "semantic model definitions can drift".into(),
            "warehouse data can change between validations".into(),
        ],
        auth_requirements: vec![
            AuthRequirement::ProductLogin,
            AuthRequirement::FabricAccessAuth,
        ],
    }
}

// tests at bottom of file (from Step 1)
```

**Step 4: Run test to verify it passes**

Run: `cd spool && cargo test -p spool-fabric -- capability`
Expected: 7 tests PASS

**Step 5: Commit**

```bash
git add spool/spool-fabric/src/capability.rs
git commit -m "feat(spool-fabric): Fabric capability contract declaration per Spec Section 11.4.1"
```

---

## Task 12: MCP Transport Investigation And Documentation

**Files:**

- Create: `spool/spool-fabric/src/mcp_investigation.rs`
- Modify: `spool/spool-fabric/src/lib.rs`

This task documents the MCP transport investigation results as code-level documentation with testable assertions about the current REST coverage assessment.

**Step 1: Write the failing test**

Add to `spool/spool-fabric/src/mcp_investigation.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rest_coverage_assessment_is_complete() {
        let assessment = rest_coverage_assessment();

        // All required operations must be assessed
        assert!(!assessment.operations.is_empty());
        assert!(assessment.operations.len() >= 7);

        // Every operation must have a coverage status
        for op in &assessment.operations {
            assert!(
                !op.operation.is_empty(),
                "operation name must not be empty"
            );
            assert!(
                !op.rest_endpoint.is_empty() || op.coverage == CoverageStatus::NotAvailableViaRest,
                "operation must have a REST endpoint or be marked as not available"
            );
        }
    }

    #[test]
    fn mcp_necessity_assessment_present() {
        let assessment = rest_coverage_assessment();

        // The assessment must include an MCP necessity conclusion
        assert!(
            !assessment.mcp_necessity_conclusion.is_empty(),
            "must include MCP necessity conclusion"
        );
    }

    #[test]
    fn all_coverage_statuses_serialize() {
        let statuses = vec![
            CoverageStatus::FullyCoveredByRest,
            CoverageStatus::PartiallyCoveredByRest,
            CoverageStatus::NotAvailableViaRest,
            CoverageStatus::NeedsInvestigation,
        ];
        for s in statuses {
            let json = serde_json::to_string(&s).unwrap();
            let restored: CoverageStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(restored, s);
        }
    }

    #[test]
    fn assessment_round_trip() {
        let assessment = rest_coverage_assessment();
        let json = serde_json::to_string_pretty(&assessment).unwrap();
        let restored: RestCoverageAssessment = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.operations.len(), assessment.operations.len());
    }

    #[test]
    fn known_rest_gaps_identified() {
        let assessment = rest_coverage_assessment();
        let gaps: Vec<&OperationCoverage> = assessment
            .operations
            .iter()
            .filter(|op| {
                op.coverage == CoverageStatus::PartiallyCoveredByRest
                    || op.coverage == CoverageStatus::NotAvailableViaRest
                    || op.coverage == CoverageStatus::NeedsInvestigation
            })
            .collect();

        // If there are gaps, they should have notes explaining the gap
        for gap in &gaps {
            assert!(
                gap.notes.is_some(),
                "operation '{}' has a REST gap but no explanatory notes",
                gap.operation
            );
        }
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cd spool && cargo test -p spool-fabric -- mcp_investigation`
Expected: FAIL

**Step 3: Write the implementation**

```rust
// spool/spool-fabric/src/mcp_investigation.rs
//!
//! MCP Transport Investigation — Spec Section 11.7
//!
//! This module documents the REST coverage assessment for Fabric operations
//! required by Spool v1. The assessment determines whether MCP is needed
//! for any gaps that REST cannot cover cleanly.
//!
//! Status: Initial assessment based on known Fabric REST API surface.
//! This assessment should be revisited during Plan 3 (Validation Execution Paths)
//! when live DAX and warehouse operations are implemented.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoverageStatus {
    FullyCoveredByRest,
    PartiallyCoveredByRest,
    NotAvailableViaRest,
    NeedsInvestigation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationCoverage {
    pub operation: String,
    pub rest_endpoint: String,
    pub coverage: CoverageStatus,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestCoverageAssessment {
    pub operations: Vec<OperationCoverage>,
    pub mcp_necessity_conclusion: String,
    pub open_questions: Vec<String>,
}

/// Return the current REST coverage assessment for Fabric operations required by Spool v1.
///
/// This is the structured output of the Spec Section 11.7 investigation.
pub fn rest_coverage_assessment() -> RestCoverageAssessment {
    RestCoverageAssessment {
        operations: vec![
            OperationCoverage {
                operation: "resolve_report_by_id".into(),
                rest_endpoint: "GET /v1.0/myorg/groups/{groupId}/reports/{reportId}".into(),
                coverage: CoverageStatus::FullyCoveredByRest,
                notes: None,
            },
            OperationCoverage {
                operation: "list_reports_in_workspace".into(),
                rest_endpoint: "GET /v1.0/myorg/groups/{groupId}/reports".into(),
                coverage: CoverageStatus::FullyCoveredByRest,
                notes: None,
            },
            OperationCoverage {
                operation: "list_report_pages".into(),
                rest_endpoint: "GET /v1.0/myorg/groups/{groupId}/reports/{reportId}/pages".into(),
                coverage: CoverageStatus::FullyCoveredByRest,
                notes: None,
            },
            OperationCoverage {
                operation: "inspect_semantic_model_tables".into(),
                rest_endpoint: "GET /v1.0/myorg/groups/{groupId}/datasets/{datasetId}/tables".into(),
                coverage: CoverageStatus::FullyCoveredByRest,
                notes: None,
            },
            OperationCoverage {
                operation: "inspect_semantic_model_relationships".into(),
                rest_endpoint: "GET /v1.0/myorg/groups/{groupId}/datasets/{datasetId}/relationships".into(),
                coverage: CoverageStatus::FullyCoveredByRest,
                notes: None,
            },
            OperationCoverage {
                operation: "execute_dax_query".into(),
                rest_endpoint: "POST /v1.0/myorg/groups/{groupId}/datasets/{datasetId}/executeQueries".into(),
                coverage: CoverageStatus::FullyCoveredByRest,
                notes: None,
            },
            OperationCoverage {
                operation: "inspect_measure_definitions".into(),
                rest_endpoint: "".into(),
                coverage: CoverageStatus::PartiallyCoveredByRest,
                notes: Some(
                    "Measure expressions are not directly returned by the tables endpoint in all \
                     API versions. The executeQueries endpoint can retrieve definitions via \
                     EVALUATE INFO.MEASURES() DMV. If the tables endpoint does not return \
                     expressions for the target API version, fall back to DMV query. \
                     This gap may motivate MCP if a cleaner semantic-model introspection \
                     surface becomes available."
                        .into(),
                ),
            },
            OperationCoverage {
                operation: "inspect_visual_bindings".into(),
                rest_endpoint: "".into(),
                coverage: CoverageStatus::NeedsInvestigation,
                notes: Some(
                    "Visual-level binding metadata is not exposed by the standard Power BI REST \
                     API. The report definition export (GET /reports/{id}/export) returns a PBIX \
                     file that can be parsed, but this is heavyweight. MCP or the enhanced report \
                     APIs may provide a lighter-weight path. This is the primary open question \
                     for MCP necessity. Plan 3 should investigate whether the Fabric MCP server \
                     exposes visual binding metadata."
                        .into(),
                ),
            },
            OperationCoverage {
                operation: "execute_warehouse_sql".into(),
                rest_endpoint: "".into(),
                coverage: CoverageStatus::NeedsInvestigation,
                notes: Some(
                    "Warehouse SQL execution is not exposed by the Power BI REST API. \
                     Direct T-SQL connectivity via TDS (tabular data stream) is the standard \
                     path. This requires an ODBC/TDS connection rather than REST. \
                     Plan 3 should determine the exact transport."
                        .into(),
                ),
            },
        ],
        mcp_necessity_conclusion:
            "REST covers the majority of required Fabric operations for Spool v1. \
             The two primary gaps are visual binding metadata inspection and warehouse SQL \
             execution. Visual bindings may require MCP or report export parsing. Warehouse \
             SQL requires TDS connectivity regardless of MCP. MCP should not be adopted as a \
             primary transport in Plan 2 but should be evaluated in Plan 3 specifically for \
             the visual binding gap. If MCP provides clean visual introspection that REST \
             does not, it becomes justified for that narrow use case."
                .into(),
        open_questions: vec![
            "Does the Fabric MCP server expose visual binding metadata?".into(),
            "Is the executeQueries DMV path sufficient for measure definition retrieval across all model types?".into(),
            "What is the exact TDS connection path for warehouse SQL execution?".into(),
        ],
    }
}

// tests at bottom of file (from Step 1)
```

**Step 4: Update lib.rs**

```rust
// spool/spool-fabric/src/lib.rs
pub mod auth;
pub mod capability;
pub mod client;
pub mod client_authenticated;
pub mod error;
pub mod mcp_investigation;
pub mod metadata;
pub mod metadata_visuals;
pub mod resolution;
pub mod resolution_api;
pub mod resolution_children;
```

**Step 5: Run test to verify it passes**

Run: `cd spool && cargo test -p spool-fabric -- mcp_investigation`
Expected: 5 tests PASS

**Step 6: Commit**

```bash
git add spool/spool-fabric/src/mcp_investigation.rs spool/spool-fabric/src/lib.rs
git commit -m "feat(spool-fabric): MCP transport investigation with REST coverage assessment per Spec Section 11.7"
```

---

## Task 13: Integration Scenario Tests

**Files:**

- Create: `spool/spool-fabric/tests/integration_scenarios.rs`

**Step 1: Write the integration test file**

This task creates end-to-end scenario tests that exercise the full adapter path: URL parsing, artifact resolution, metadata inspection, child derivation, and capability contract usage — all using fixtures.

```rust
// spool/spool-fabric/tests/integration_scenarios.rs
use spool_fabric::auth::{AccessToken, FixtureTokenProvider};
use spool_fabric::capability::fabric_capability_contract;
use spool_fabric::client::MockFabricHttpClient;
use spool_fabric::client_authenticated::AuthenticatedFabricClient;
use spool_fabric::metadata::FabricMetadataInspector;
use spool_fabric::metadata_visuals::VisualMetadataInspector;
use spool_fabric::resolution::{parse_report_url, resolution_priority};
use spool_fabric::resolution_api::FabricArtifactResolver;
use spool_fabric::resolution_children::{derive_measure, derive_page, derive_visual};
use spool_protocol::artifact::{ArtifactType, ResolutionBasis};
use spool_protocol::capability::{InspectionCapability, ValidationCapability};
use chrono::{Duration, Utc};

fn valid_token() -> AccessToken {
    AccessToken {
        token: "integration_test_token".into(),
        expires_at: Utc::now() + Duration::seconds(3600),
        scopes: vec!["https://analysis.windows.net/powerbi/api/.default".into()],
    }
}

fn build_full_mock() -> MockFabricHttpClient {
    let mut mock = MockFabricHttpClient::new();

    // Report resolution
    mock.register_response(
        "GET",
        "/v1.0/myorg/groups/ws_123/reports/rpt_456",
        200,
        serde_json::json!({
            "id": "rpt_456",
            "name": "Executive Revenue Report",
            "datasetId": "mod_789",
            "webUrl": "https://app.fabric.microsoft.com/groups/ws_123/reports/rpt_456"
        }),
    );

    // Report pages
    mock.register_response(
        "GET",
        "/v1.0/myorg/groups/ws_123/reports/rpt_456/pages",
        200,
        serde_json::json!({
            "value": [
                {"name": "ReportSectionSummary", "displayName": "Summary", "order": 0},
                {"name": "ReportSectionDetail", "displayName": "Detail", "order": 1}
            ]
        }),
    );

    // Visuals
    mock.register_response(
        "GET",
        "/v1.0/myorg/groups/ws_123/reports/rpt_456/pages/ReportSectionSummary/visuals",
        200,
        serde_json::json!({
            "value": [
                {
                    "name": "visual_rev",
                    "title": "Revenue Chart",
                    "type": "barChart",
                    "dataBindings": [
                        {"role": "Values", "table": "Sales", "measure": "Total Revenue"}
                    ],
                    "filters": []
                }
            ]
        }),
    );

    // Semantic model tables
    mock.register_response(
        "GET",
        "/v1.0/myorg/groups/ws_123/datasets/mod_789/tables",
        200,
        serde_json::json!({
            "value": [
                {
                    "name": "Sales",
                    "columns": [
                        {"name": "Amount", "dataType": "Decimal", "isHidden": false}
                    ],
                    "measures": [
                        {"name": "Total Revenue", "expression": "SUM(Sales[Amount])"},
                        {"name": "QoQ Revenue", "expression": "CALCULATE(SUM(Sales[Amount]), DATEADD('Date'[Date], -1, QUARTER))"}
                    ]
                }
            ]
        }),
    );

    // Semantic model relationships
    mock.register_response(
        "GET",
        "/v1.0/myorg/groups/ws_123/datasets/mod_789/relationships",
        200,
        serde_json::json!({"value": []}),
    );

    mock
}

/// Scenario 1: Full report investigation path
///
/// Simulate the adapter path a report investigation task would follow:
/// 1. Parse report URL
/// 2. Resolve report by GUID
/// 3. Inspect report metadata (pages)
/// 4. Derive page artifact
/// 5. Inspect visual bindings
/// 6. Derive visual artifact
#[tokio::test]
async fn scenario_report_investigation_path() {
    let mock = build_full_mock();

    // Step 1: Parse URL
    let parsed = parse_report_url(
        "https://app.fabric.microsoft.com/groups/ws_123/reports/rpt_456/ReportSectionSummary",
    )
    .unwrap();
    assert_eq!(parsed.workspace_id, "ws_123");
    assert_eq!(parsed.report_id, "rpt_456");
    assert_eq!(parsed.page_name, Some("ReportSectionSummary".into()));

    // Step 2: Resolve report
    let resolver = FabricArtifactResolver::new(Box::new(build_full_mock()));
    let report = resolver
        .resolve_by_guid("ws_123", "rpt_456", ArtifactType::Report)
        .await
        .unwrap();
    assert_eq!(report.display_name, "Executive Revenue Report");
    assert_eq!(report.resolution_basis, ResolutionBasis::ExplicitGuid);

    // Step 3: Inspect report pages
    let inspector = FabricMetadataInspector::new(Box::new(build_full_mock()));
    let metadata = inspector.inspect_report("ws_123", "rpt_456").await.unwrap();
    assert_eq!(metadata.pages.len(), 2);

    // Step 4: Derive page
    let page = derive_page(&report, "ReportSectionSummary", "Summary").unwrap();
    assert_eq!(page.artifact_type, ArtifactType::Page);

    // Step 5: Inspect visuals
    let visual_inspector = VisualMetadataInspector::new(Box::new(build_full_mock()));
    let visuals = visual_inspector
        .inspect_visuals("ws_123", "rpt_456", "ReportSectionSummary")
        .await
        .unwrap();
    assert_eq!(visuals.len(), 1);
    assert_eq!(visuals[0].title, "Revenue Chart");

    // Step 6: Derive visual
    let visual = derive_visual(&page, "rpt_456", "visual_rev", "Revenue Chart").unwrap();
    assert_eq!(visual.artifact_type, ArtifactType::Visual);
}

/// Scenario 2: Semantic model investigation path
///
/// 1. Resolve model by GUID
/// 2. Inspect model metadata
/// 3. Derive measure artifact
/// 4. Inspect measure definition
#[tokio::test]
async fn scenario_semantic_model_investigation_path() {
    let mut mock = MockFabricHttpClient::new();
    mock.register_response(
        "GET",
        "/v1.0/myorg/groups/ws_123/datasets/mod_789",
        200,
        serde_json::json!({
            "id": "mod_789",
            "name": "Sales Model"
        }),
    );
    mock.register_response(
        "GET",
        "/v1.0/myorg/groups/ws_123/datasets/mod_789/tables",
        200,
        serde_json::json!({
            "value": [{
                "name": "Sales",
                "columns": [{"name": "Amount", "dataType": "Decimal", "isHidden": false}],
                "measures": [
                    {"name": "QoQ Revenue", "expression": "CALCULATE(SUM(Sales[Amount]), DATEADD('Date'[Date], -1, QUARTER))"}
                ]
            }]
        }),
    );
    mock.register_response(
        "GET",
        "/v1.0/myorg/groups/ws_123/datasets/mod_789/relationships",
        200,
        serde_json::json!({"value": []}),
    );

    // Step 1: Resolve model
    let resolver = FabricArtifactResolver::new(Box::new({
        let mut m = MockFabricHttpClient::new();
        m.register_response(
            "GET",
            "/v1.0/myorg/groups/ws_123/datasets/mod_789",
            200,
            serde_json::json!({"id": "mod_789", "name": "Sales Model"}),
        );
        m
    }));
    let model = resolver
        .resolve_by_guid("ws_123", "mod_789", ArtifactType::SemanticModel)
        .await
        .unwrap();
    assert_eq!(model.display_name, "Sales Model");

    // Step 2: Inspect model
    let inspector = FabricMetadataInspector::new(Box::new(mock));
    let metadata = inspector
        .inspect_semantic_model("ws_123", "mod_789")
        .await
        .unwrap();
    assert_eq!(metadata.tables.len(), 1);

    // Step 3: Derive measure
    let measure = derive_measure(&model, "mod_789", "Sales", "QoQ Revenue").unwrap();
    assert_eq!(measure.artifact_type, ArtifactType::Measure);
    assert!(measure.canonical_locator.0.contains("Sales[QoQ Revenue]"));

    // Step 4: Inspect measure definition
    let inspector2 = FabricMetadataInspector::new(Box::new({
        let mut m = MockFabricHttpClient::new();
        m.register_response(
            "GET",
            "/v1.0/myorg/groups/ws_123/datasets/mod_789/tables",
            200,
            serde_json::json!({
                "value": [{
                    "name": "Sales",
                    "columns": [],
                    "measures": [{"name": "QoQ Revenue", "expression": "CALCULATE(SUM(Sales[Amount]), DATEADD('Date'[Date], -1, QUARTER))"}]
                }]
            }),
        );
        m
    }));
    let definition = inspector2
        .inspect_measure("ws_123", "mod_789", "Sales", "QoQ Revenue")
        .await
        .unwrap();
    assert!(definition.expression.contains("CALCULATE"));
}

/// Scenario 3: Capability contract guards planner decisions
///
/// Verify that the capability contract can be queried to determine
/// whether an operation is valid before attempting it.
#[test]
fn scenario_capability_contract_guards_operations() {
    let contract = fabric_capability_contract();

    // Planner asks: can I inspect report metadata?
    assert!(contract.supports_inspection(&InspectionCapability::InspectReportMetadata));

    // Planner asks: can I run DAX?
    assert!(contract.supports_validation(&ValidationCapability::RunDaxQuery));

    // Planner asks: is warehouse supported?
    assert!(contract.supports_artifact_kind("warehouse"));

    // Planner asks: can I mutate anything?
    assert!(contract.mutation_capabilities.is_empty());

    // Evaluator checks: is the adapter active?
    assert_eq!(
        contract.status,
        spool_protocol::capability::AdapterStatus::Active
    );
}

/// Scenario 4: Resolution priority ordering matches spec
#[test]
fn scenario_resolution_priority_matches_spec() {
    // Spec Section 3.5: explicit GUID > parsed URL > exact API > unique name > derived child
    assert!(
        resolution_priority(&ResolutionBasis::ExplicitGuid)
            > resolution_priority(&ResolutionBasis::ReportUrl)
    );
    assert!(
        resolution_priority(&ResolutionBasis::ReportUrl)
            > resolution_priority(&ResolutionBasis::ExactApiMatch)
    );
    assert!(
        resolution_priority(&ResolutionBasis::ExactApiMatch)
            > resolution_priority(&ResolutionBasis::UniqueNameMatch)
    );
    assert!(
        resolution_priority(&ResolutionBasis::UniqueNameMatch)
            > resolution_priority(&ResolutionBasis::DerivedFromResolvedParent)
    );
}

/// Scenario 5: Authenticated client end-to-end
#[tokio::test]
async fn scenario_authenticated_request_flow() {
    let mut mock = MockFabricHttpClient::new();
    mock.register_response(
        "GET",
        "/v1.0/myorg/groups/ws_123/reports",
        200,
        serde_json::json!({"value": [{"id": "rpt_456", "name": "Test Report", "datasetId": "mod_1", "webUrl": ""}]}),
    );

    let provider = FixtureTokenProvider::new(valid_token());
    let client = AuthenticatedFabricClient::new(Box::new(mock), Box::new(provider));

    let response = client.get("/v1.0/myorg/groups/ws_123/reports").await.unwrap();
    assert_eq!(response.status, 200);

    let body: serde_json::Value = serde_json::from_str(&response.body).unwrap();
    let reports = body["value"].as_array().unwrap();
    assert_eq!(reports.len(), 1);
}
```

**Step 2: Run test to verify it passes**

Run: `cd spool && cargo test --test integration_scenarios`
Expected: 5 tests PASS

Then run the full test suite:

Run: `cd spool && cargo test`
Expected: all tests PASS (protocol + fabric + integration)

**Step 3: Commit**

```bash
git add spool/spool-fabric/tests/integration_scenarios.rs
git commit -m "feat(spool-fabric): integration scenarios proving report investigation, model investigation, capability guarding, resolution priority, and authenticated flow"
```

---

## Summary

| Task | What it proves | Test count |
|------|---------------|------------|
| 1 | Workspace integration + spool-fabric scaffolding | 0 (build check) |
| 2 | Platform capability contract types | 5 |
| 3 | Fabric REST client trait and mock | 4 |
| 4 | Auth types and token management | 9 |
| 5 | Authenticated Fabric client | 3 |
| 6 | Report URL parsing and resolution | 9 |
| 7 | Artifact resolution from workspace+GUID | 6 |
| 8 | Child artifact derivation | 8 |
| 9 | Report and semantic model metadata inspection | 6 |
| 10 | Visual binding metadata inspection | 4 |
| 11 | Fabric capability contract declaration | 7 |
| 12 | MCP transport investigation | 5 |
| 13 | Integration scenarios | 5 |
| **Total** | | **71** |
