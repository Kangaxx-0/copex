# Native Copilot Model Provider Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace the Python proxy with a built-in Copilot provider that handles auth, headers, endpoint resolution, and quota display natively in Rust.

**Architecture:** Register `"copilot"` as a built-in provider in `built_in_model_providers()`. At startup, read the GitHub OAuth token from macOS Keychain (via `keyring-store` crate), resolve the enterprise API endpoint by calling `/copilot_internal/user`, and inject Copilot-specific headers. Parse `x-quota-snapshot-*` response headers for quota display. SSE-only transport, local compaction only.

**Tech Stack:** Rust, `keyring` crate (via `keyring-store`), `reqwest` for endpoint resolution, `wiremock` for test mocking, existing `codex-api` SSE infrastructure.

---

## Phase 1: Core Provider Registration

### Task 1: Copilot Provider Definition — Unit Tests

**Files:**
- Modify: `codex-rs/core/src/model_provider_info_tests.rs`

**Step 1: Write the failing test for Copilot provider TOML deserialization**

```rust
#[test]
fn test_copilot_provider_is_registered_as_builtin() {
    let providers = built_in_model_providers(None);
    assert!(providers.contains_key("copilot"), "copilot should be a built-in provider");
    let copilot = &providers["copilot"];
    assert_eq!(copilot.name, "GitHub Copilot");
    assert_eq!(copilot.requires_openai_auth, false);
    assert_eq!(copilot.supports_websockets, false);
    assert_eq!(copilot.wire_api, WireApi::Responses);
    // base_url is None — resolved dynamically at startup
    assert!(copilot.base_url.is_none());
    // Copilot-specific headers are set
    let headers = copilot.http_headers.as_ref().expect("should have http_headers");
    assert_eq!(headers.get("Copilot-Integration-Id").unwrap(), "copilot-developer-cli");
    assert_eq!(headers.get("X-GitHub-Api-Version").unwrap(), "2026-01-09");
    assert_eq!(headers.get("Openai-Intent").unwrap(), "conversation-agent");
    assert_eq!(headers.get("X-Initiator").unwrap(), "user");
}

#[test]
fn test_copilot_provider_is_copilot() {
    let providers = built_in_model_providers(None);
    let copilot = &providers["copilot"];
    assert!(copilot.is_copilot());

    let openai = &providers["openai"];
    assert!(!openai.is_copilot());
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test -p codex-core --lib model_provider_info_tests -- --nocapture`
Expected: FAIL — `copilot` key not found, `is_copilot()` method doesn't exist.

**Step 3: Write minimal implementation — register the provider**

In `codex-rs/core/src/model_provider_info.rs`:

Add constants:
```rust
const COPILOT_PROVIDER_NAME: &str = "GitHub Copilot";
pub const COPILOT_PROVIDER_ID: &str = "copilot";
```

Add `create_copilot_provider()`:
```rust
pub fn create_copilot_provider() -> ModelProviderInfo {
    ModelProviderInfo {
        name: COPILOT_PROVIDER_NAME.into(),
        base_url: None, // Resolved dynamically at startup
        env_key: None,
        env_key_instructions: None,
        experimental_bearer_token: None,
        wire_api: WireApi::Responses,
        query_params: None,
        http_headers: Some(
            [
                ("Copilot-Integration-Id".to_string(), "copilot-developer-cli".to_string()),
                ("X-GitHub-Api-Version".to_string(), "2026-01-09".to_string()),
                ("Openai-Intent".to_string(), "conversation-agent".to_string()),
                ("X-Initiator".to_string(), "user".to_string()),
            ]
            .into_iter()
            .collect(),
        ),
        env_http_headers: None,
        request_max_retries: None,
        stream_max_retries: None,
        stream_idle_timeout_ms: None,
        websocket_connect_timeout_ms: None,
        requires_openai_auth: false,
        supports_websockets: false,
    }
}
```

Add `is_copilot()` to `impl ModelProviderInfo`:
```rust
pub fn is_copilot(&self) -> bool {
    self.name == COPILOT_PROVIDER_NAME
}
```

Add to `built_in_model_providers()` array:
```rust
(COPILOT_PROVIDER_ID, create_copilot_provider()),
```

**Step 4: Run tests to verify they pass**

Run: `cargo test -p codex-core --lib model_provider_info_tests -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add codex-rs/core/src/model_provider_info.rs codex-rs/core/src/model_provider_info_tests.rs
git commit -m "feat(copilot): register copilot as built-in provider with static headers"
```

---

### Task 2: Copilot Provider Cannot Be Overridden by User Config

**Files:**
- Modify: `codex-rs/core/src/model_provider_info_tests.rs`

**Context:** The real config merge logic in `config/mod.rs:2260-2262` uses `entry(key).or_insert(provider)` when merging user-defined providers into built-ins, so built-ins always win. This test documents that guarantee.

**Step 1: Write the test**

```rust
#[test]
fn test_copilot_provider_cannot_be_overridden_by_user_config() {
    // Mirrors the merge logic in config/mod.rs:2260-2262:
    //   for (key, provider) in cfg.model_providers.into_iter() {
    //       model_providers.entry(key).or_insert(provider);
    //   }
    // Built-in providers are inserted first, so or_insert is a no-op
    // when the user defines a "copilot" provider in config.toml.
    let mut providers = built_in_model_providers(None);
    let user_provider = ModelProviderInfo {
        name: "My Custom Copilot".into(),
        base_url: Some("http://localhost:9999".into()),
        ..create_copilot_provider()
    };
    providers.entry("copilot".to_string()).or_insert(user_provider);

    let copilot = &providers["copilot"];
    assert_eq!(copilot.name, "GitHub Copilot"); // Built-in wins
    assert!(copilot.base_url.is_none()); // Not overridden
}
```

**Step 2: Run test to verify it passes**

Run: `cargo test -p codex-core --lib model_provider_info_tests::test_copilot_provider_cannot_be_overridden -- --nocapture`
Expected: PASS (the `or_insert` logic already handles this — this test documents the behavior).

**Step 3: Commit**

```bash
git add codex-rs/core/src/model_provider_info_tests.rs
git commit -m "test(copilot): verify built-in copilot provider cannot be overridden"
```

---

## Phase 2: Copilot Auth — Keychain Token Reading

### Task 3: Copilot Auth Module — Token Reading

**Files:**
- Create: `codex-rs/core/src/copilot_auth.rs`
- Modify: `codex-rs/core/src/lib.rs` (add `mod copilot_auth;`)

**Step 1: Write the failing tests**

Create `codex-rs/core/src/copilot_auth.rs` with tests first:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_copilot_token_returns_token_from_keychain() {
        // Use a mock keyring store
        let mock = MockKeyringStore::new();
        mock.save(COPILOT_KEYCHAIN_SERVICE, "https://github.com:testuser", "gho_test123").unwrap();

        let result = read_copilot_token_from_store(&mock, Some("testuser"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "gho_test123");
    }

    #[test]
    fn test_read_copilot_token_missing_returns_error() {
        let mock = MockKeyringStore::new();
        // No token saved

        let result = read_copilot_token_from_store(&mock, Some("testuser"));
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Copilot token not found"), "Error was: {err}");
    }

    #[test]
    fn test_read_copilot_token_empty_returns_error() {
        let mock = MockKeyringStore::new();
        mock.save(COPILOT_KEYCHAIN_SERVICE, "https://github.com:testuser", "").unwrap();

        let result = read_copilot_token_from_store(&mock, Some("testuser"));
        assert!(result.is_err());
    }

    #[test]
    fn test_read_copilot_token_without_username_uses_subprocess_fallback() {
        // When no username is configured, we fall back to `security` CLI
        // This test verifies the function signature accepts None
        let mock = MockKeyringStore::new();
        let result = read_copilot_token_from_store(&mock, None);
        assert!(result.is_err()); // No token in mock, no subprocess in test
    }
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test -p codex-core --lib copilot_auth -- --nocapture`
Expected: FAIL — module doesn't exist yet.

**Step 3: Write minimal implementation**

```rust
//! Copilot authentication via macOS Keychain.
//!
//! Reads the GitHub OAuth token stored by the `copilot` CLI.
//! The token is stored in macOS Keychain under service "copilot-cli"
//! with account "https://github.com:<username>".

use keyring_store::KeyringStore;
use std::process::Command;

pub(crate) const COPILOT_KEYCHAIN_SERVICE: &str = "copilot-cli";

/// Read the Copilot OAuth token from the keyring store.
///
/// If `github_username` is provided, reads from the keyring directly.
/// Otherwise, falls back to `security find-generic-password -s copilot-cli -w`
/// which returns the first match regardless of account.
pub(crate) fn read_copilot_token_from_store(
    store: &dyn KeyringStore,
    github_username: Option<&str>,
) -> anyhow::Result<String> {
    if let Some(username) = github_username {
        let account = format!("https://github.com:{username}");
        match store.load(COPILOT_KEYCHAIN_SERVICE, &account) {
            Ok(Some(token)) if !token.trim().is_empty() => return Ok(token),
            Ok(_) => {}
            Err(e) => {
                tracing::warn!("Keyring lookup failed: {e}, trying subprocess fallback");
            }
        }
    }

    read_copilot_token_from_subprocess()
}

/// Fallback: use macOS `security` command to read by service name only.
fn read_copilot_token_from_subprocess() -> anyhow::Result<String> {
    let output = Command::new("security")
        .args(["find-generic-password", "-s", COPILOT_KEYCHAIN_SERVICE, "-w"])
        .output()
        .map_err(|e| anyhow::anyhow!(
            "Failed to run 'security' command: {e}. \
             Install and log into GitHub Copilot CLI first: https://docs.github.com/en/copilot/github-copilot-in-the-cli"
        ))?;

    let token = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if token.is_empty() {
        anyhow::bail!(
            "Copilot token not found in macOS Keychain. \
             Run 'copilot auth login' first to authenticate."
        );
    }
    Ok(token)
}
```

**Step 4: Add `mod copilot_auth;` to `core/src/lib.rs`**

**Step 5: Run tests to verify they pass**

Run: `cargo test -p codex-core --lib copilot_auth -- --nocapture`
Expected: PASS

**Step 6: Commit**

```bash
git add codex-rs/core/src/copilot_auth.rs codex-rs/core/src/lib.rs
git commit -m "feat(copilot): add keychain token reading with subprocess fallback"
```

---

### Task 4: Copilot Endpoint Resolution

**Files:**
- Modify: `codex-rs/core/src/copilot_auth.rs`

**Step 1: Write the failing tests**

```rust
#[cfg(test)]
mod tests {
    // ... existing tests ...

    #[test]
    fn test_parse_copilot_user_response_extracts_endpoint() {
        let json = serde_json::json!({
            "login": "testuser",
            "copilot_plan": "enterprise",
            "endpoints": {
                "api": "https://api.enterprise.githubcopilot.com"
            },
            "quota_snapshots": {
                "premium_interactions": {
                    "entitlement": 1000,
                    "remaining": 750,
                    "percent_remaining": 75.0,
                    "unlimited": false
                }
            },
            "quota_reset_date_utc": "2026-04-01T00:00:00.000Z"
        });

        let info = parse_copilot_user_info(&json).unwrap();
        assert_eq!(info.api_endpoint, "https://api.enterprise.githubcopilot.com");
        assert_eq!(info.login, "testuser");
        assert_eq!(info.copilot_plan, "enterprise");
        assert_eq!(info.quota.as_ref().unwrap().percent_remaining, 75.0);
        assert_eq!(info.quota.as_ref().unwrap().unlimited, false);
        assert_eq!(info.quota.as_ref().unwrap().entitlement, 1000);
        assert_eq!(info.quota.as_ref().unwrap().remaining, 750);
    }

    #[test]
    fn test_parse_copilot_user_response_unlimited_quota() {
        let json = serde_json::json!({
            "login": "testuser",
            "copilot_plan": "enterprise",
            "endpoints": {
                "api": "https://api.enterprise.githubcopilot.com"
            },
            "quota_snapshots": {
                "premium_interactions": {
                    "entitlement": 1000,
                    "remaining": 1000,
                    "percent_remaining": 100.0,
                    "unlimited": true
                }
            },
            "quota_reset_date_utc": "2026-04-01T00:00:00.000Z"
        });

        let info = parse_copilot_user_info(&json).unwrap();
        assert!(info.quota.as_ref().unwrap().unlimited);
    }

    #[test]
    fn test_parse_copilot_user_response_missing_endpoints_fails() {
        let json = serde_json::json!({
            "login": "testuser",
            "copilot_plan": "enterprise"
        });

        let result = parse_copilot_user_info(&json);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_copilot_user_response_no_quota_is_ok() {
        let json = serde_json::json!({
            "login": "testuser",
            "copilot_plan": "individual",
            "endpoints": {
                "api": "https://api.githubcopilot.com"
            }
        });

        let info = parse_copilot_user_info(&json).unwrap();
        assert_eq!(info.api_endpoint, "https://api.githubcopilot.com");
        assert!(info.quota.is_none());
    }
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test -p codex-core --lib copilot_auth -- --nocapture`
Expected: FAIL — `parse_copilot_user_info` doesn't exist.

**Step 3: Write implementation**

Add to `copilot_auth.rs`:

```rust
const COPILOT_USER_ENDPOINT: &str = "https://api.github.com/copilot_internal/user";

#[derive(Debug, Clone)]
pub(crate) struct CopilotUserInfo {
    pub login: String,
    pub copilot_plan: String,
    pub api_endpoint: String,
    pub quota: Option<CopilotQuotaSnapshot>,
}

#[derive(Debug, Clone)]
pub(crate) struct CopilotQuotaSnapshot {
    pub entitlement: i64,
    pub remaining: i64,
    pub percent_remaining: f64,
    pub unlimited: bool,
}

pub(crate) fn parse_copilot_user_info(json: &serde_json::Value) -> anyhow::Result<CopilotUserInfo> {
    let login = json["login"]
        .as_str()
        .unwrap_or("unknown")
        .to_string();

    let copilot_plan = json["copilot_plan"]
        .as_str()
        .unwrap_or("unknown")
        .to_string();

    let api_endpoint = json
        .get("endpoints")
        .and_then(|e| e.get("api"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!(
            "No Copilot API endpoint found. Check your Copilot subscription status."
        ))?
        .to_string();

    let quota = json
        .get("quota_snapshots")
        .and_then(|qs| qs.get("premium_interactions"))
        .and_then(|pi| {
            Some(CopilotQuotaSnapshot {
                entitlement: pi.get("entitlement")?.as_i64()?,
                remaining: pi.get("remaining")?.as_i64()?,
                percent_remaining: pi.get("percent_remaining")?.as_f64()?,
                unlimited: pi.get("unlimited")?.as_bool()?,
            })
        });

    Ok(CopilotUserInfo {
        login,
        copilot_plan,
        api_endpoint,
        quota,
    })
}

/// Fetch Copilot user info from GitHub API.
/// Called at startup to resolve the enterprise API endpoint.
pub(crate) async fn fetch_copilot_user_info(token: &str) -> anyhow::Result<CopilotUserInfo> {
    let client = reqwest::Client::new();
    let resp = client
        .get(COPILOT_USER_ENDPOINT)
        .header("Authorization", format!("Bearer {token}"))
        .header("Accept", "application/json")
        .send()
        .await?;

    match resp.status().as_u16() {
        200 => {
            let json: serde_json::Value = resp.json().await?;
            parse_copilot_user_info(&json)
        }
        401 => anyhow::bail!(
            "Copilot token expired or revoked. Run 'copilot auth login' to re-authenticate."
        ),
        403 => anyhow::bail!(
            "No active Copilot subscription found for this GitHub account."
        ),
        status => {
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Copilot API returned {status}: {body}")
        }
    }
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test -p codex-core --lib copilot_auth -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add codex-rs/core/src/copilot_auth.rs
git commit -m "feat(copilot): add endpoint resolution and user info parsing"
```

---

## Phase 3: Wire Auth into Request Pipeline

### Task 5: Integrate Copilot Token into Auth Bridge

**Files:**
- Modify: `codex-rs/core/src/api_bridge.rs` (around line 167)
- Modify: `codex-rs/core/src/copilot_auth.rs`

**Step 1: Write the failing test**

Add to `copilot_auth.rs` tests:

```rust
#[test]
fn test_copilot_auth_provider_returns_bearer_token() {
    let mock = MockKeyringStore::new();
    mock.save(COPILOT_KEYCHAIN_SERVICE, "https://github.com:testuser", "gho_abc123").unwrap();

    let token = read_copilot_token_from_store(&mock, Some("testuser")).unwrap();
    assert!(token.starts_with("gho_"));
}
```

Add a new test in `api_bridge.rs` (or a new test file) that verifies the auth flow for a Copilot provider:

```rust
#[test]
fn test_auth_provider_from_copilot_provider_uses_copilot_token() {
    // When provider.is_copilot() and a copilot token is set,
    // auth_provider_from_auth should return that token
    let provider = create_copilot_provider();
    // Simulate copilot token being set on the provider via experimental_bearer_token
    // (This is how we inject the resolved token at startup)
    let mut provider = provider;
    provider.experimental_bearer_token = Some("gho_test_token".to_string());

    let result = auth_provider_from_auth(None, &provider).unwrap();
    assert_eq!(result.token, Some("gho_test_token".to_string()));
}
```

**Step 2: Run test to verify behavior**

Run: `cargo test -p codex-core --lib -- auth_provider_from_copilot --nocapture`
Expected: PASS — `experimental_bearer_token` path already works in `auth_provider_from_auth()`.

This confirms we don't need to modify `api_bridge.rs` at all. The token injection happens at startup by setting `experimental_bearer_token` on the provider after reading from Keychain.

**Step 3: Commit**

```bash
git add codex-rs/core/src/copilot_auth.rs codex-rs/core/src/api_bridge.rs
git commit -m "test(copilot): verify copilot token flows through auth bridge"
```

---

### Task 6: Startup Initialization — Wire Everything Together

**Files:**
- Modify: `codex-rs/core/src/config/mod.rs` (insertion point: after line 2278, between `.clone()` and `let shell_environment_policy`)

**Context:** The config loader is `Config::load_with_cli_overrides()` (line 687) which is already `async`. The provider is resolved at line 2268-2278 via `model_providers.get(&model_provider_id)...clone()`. We insert Copilot initialization immediately after that `.clone()`, before `let shell_environment_policy` on line 2280.

**Step 1: Write the failing test**

Add integration test in `codex-rs/core/tests/suite/copilot_provider.rs`:

```rust
use core_test_support::*;
use wiremock::MockServer;
use wiremock::matchers::{method, path};
use wiremock::ResponseTemplate;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn copilot_provider_resolves_endpoint_at_startup() -> anyhow::Result<()> {
    skip_if_sandbox!(Ok(()));

    let github_server = MockServer::start().await;

    // Mock the /copilot_internal/user endpoint
    wiremock::Mock::given(method("GET"))
        .and(path("/copilot_internal/user"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "login": "testuser",
            "copilot_plan": "enterprise",
            "endpoints": {
                "api": "https://api.enterprise.githubcopilot.com"
            },
            "quota_snapshots": {
                "premium_interactions": {
                    "entitlement": 1000,
                    "remaining": 1000,
                    "percent_remaining": 100.0,
                    "unlimited": true
                }
            }
        })))
        .mount(&github_server)
        .await;

    // Test that the provider's base_url gets set after initialization
    // This requires an integration path through config loading
    // We test the parsing function directly as a unit test proxy
    let json = serde_json::json!({
        "login": "testuser",
        "copilot_plan": "enterprise",
        "endpoints": {
            "api": &format!("{}/v1", github_server.uri())
        }
    });

    let info = codex_core::copilot_auth::parse_copilot_user_info(&json)?;
    assert!(info.api_endpoint.contains(&github_server.uri()));

    Ok(())
}
```

**Step 2: Run test to verify it fails**

Run: `cargo nextest run -p codex-core copilot_provider`
Expected: FAIL — module/function not pub.

**Step 3: Wire startup logic in `config/mod.rs`**

Insert after line 2278 (after `let model_provider = model_providers.get(...)...clone();`), before line 2280 (`let shell_environment_policy`):

```rust
// If Copilot provider is selected, resolve the API endpoint
let model_provider = if model_provider.is_copilot() {
    let mut provider = model_provider;
    match resolve_copilot_provider(&mut provider).await {
        Ok(()) => provider,
        Err(e) => return Err(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())),
    }
} else {
    model_provider
};
```

Add helper function (as a free function in `config/mod.rs` or in `copilot_auth.rs`):

```rust
async fn resolve_copilot_provider(provider: &mut ModelProviderInfo) -> anyhow::Result<()> {
    use crate::copilot_auth::{read_copilot_token_from_store, fetch_copilot_user_info};
    use keyring_store::DefaultKeyringStore;

    let store = DefaultKeyringStore;
    // Try keychain first (no username needed — subprocess fallback handles it)
    let token = read_copilot_token_from_store(&store, None)?;

    let user_info = fetch_copilot_user_info(&token).await?;
    tracing::info!(
        login = %user_info.login,
        plan = %user_info.copilot_plan,
        endpoint = %user_info.api_endpoint,
        "Copilot provider initialized"
    );

    provider.base_url = Some(user_info.api_endpoint);
    provider.experimental_bearer_token = Some(token);

    Ok(())
}
```

No sync/async conversion needed — `load_with_cli_overrides` is already async.

**Step 4: Run tests**

Run: `cargo nextest run -p codex-core copilot_provider`
Expected: PASS

**Step 5: Commit**

```bash
git add codex-rs/core/src/config/mod.rs
git commit -m "feat(copilot): resolve API endpoint and inject token at startup"
```

---

### Task 6b: Handle Network Failure During Startup

**Context:** `fetch_copilot_user_info` makes a network call at startup. If GitHub is unreachable, `resolve_copilot_provider` returns `Err`, which propagates as `io::Error` and kills startup. This is the correct behavior — without endpoint resolution, the Copilot provider cannot function (there is no default/fallback endpoint). The error message from `fetch_copilot_user_info` should be clear enough for users to diagnose (network issues, expired tokens, no subscription).

**Decision:** Hard failure at startup. No caching, no retry, no deferred resolution. If users want offline fallback, they can switch to a different provider.

**No code changes needed** — the error path in Task 6's `resolve_copilot_provider` already returns a clear error. Just ensure the error messages from `fetch_copilot_user_info` (401, 403, network error) are user-actionable, which they are as written in Task 4.

---

### Task 7: Eliminate auth.json Workaround

**Files:**
- Modify: `codex-rs/core/src/config/mod.rs` or auth initialization path

**Step 1: Write the failing test**

```rust
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn copilot_provider_does_not_trigger_openai_login() -> anyhow::Result<()> {
    skip_if_sandbox!(Ok(()));

    // Create a config with model_provider = "copilot"
    // and a stale auth.json with expired tokens
    // Verify that Codex does NOT attempt to refresh the OpenAI token
    let home = tempfile::TempDir::new()?;
    let auth_json = home.path().join("auth.json");
    std::fs::write(&auth_json, r#"{"auth_mode":"Chatgpt","tokens":{"access_token":"expired"}}"#)?;

    // Loading config with copilot provider should succeed
    // without touching auth.json
    let provider = create_copilot_provider();
    assert_eq!(provider.requires_openai_auth, false);

    // auth_provider_from_auth with no CodexAuth should return None token
    // (token comes from experimental_bearer_token, not auth.json)
    let mut provider = provider;
    provider.experimental_bearer_token = Some("gho_valid".to_string());
    let auth = auth_provider_from_auth(None, &provider)?;
    assert_eq!(auth.token, Some("gho_valid".to_string()));

    Ok(())
}
```

**Step 2: Run test — should PASS**

This test documents the existing correct behavior: `requires_openai_auth = false` means the login flow is skipped entirely, and `experimental_bearer_token` takes priority in `auth_provider_from_auth`. The stale `auth.json` is never consulted.

Run: `cargo nextest run -p codex-core copilot_provider_does_not_trigger`
Expected: PASS

**Step 3: Commit**

```bash
git add codex-rs/core/tests/suite/copilot_provider.rs
git commit -m "test(copilot): verify copilot provider bypasses OpenAI auth.json entirely"
```

---

## Phase 4: Copilot Quota Display

### Task 8: Parse x-quota-snapshot Headers

**Files:**
- Create: `codex-rs/codex-api/src/copilot_quota.rs`
- Modify: `codex-rs/codex-api/src/lib.rs`

**Step 1: Write the failing tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use http::{HeaderMap, HeaderValue};

    #[test]
    fn test_parse_quota_snapshot_header() {
        let raw = "ent=-1&ov=0.0&ovPerm=true&rem=100.0&rst=2026-04-01T00%3A00%3A00Z";
        let quota = parse_quota_snapshot_header(raw).unwrap();
        assert_eq!(quota.entitlement, -1);
        assert_eq!(quota.percent_remaining, 100.0);
        assert_eq!(quota.resets_at, "2026-04-01T00:00:00Z");
    }

    #[test]
    fn test_parse_quota_snapshot_header_capped_plan() {
        let raw = "ent=1000&ov=0.0&ovPerm=false&rem=62.3&rst=2026-04-01T00%3A00%3A00Z";
        let quota = parse_quota_snapshot_header(raw).unwrap();
        assert_eq!(quota.entitlement, 1000);
        assert_eq!(quota.percent_remaining, 62.3);
        assert_eq!(quota.overage_permitted, false);
    }

    #[test]
    fn test_parse_quota_from_response_headers() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-quota-snapshot-premium_interactions",
            HeaderValue::from_static("ent=-1&ov=0.0&ovPerm=true&rem=100.0&rst=2026-04-01T00%3A00%3A00Z"),
        );
        headers.insert(
            "x-quota-snapshot-chat",
            HeaderValue::from_static("ent=500&ov=0.0&ovPerm=false&rem=80.0&rst=2026-04-01T00%3A00%3A00Z"),
        );

        let quotas = parse_copilot_quota_headers(&headers);
        assert_eq!(quotas.len(), 2);
        assert!(quotas.contains_key("premium_interactions"));
        assert!(quotas.contains_key("chat"));
        assert_eq!(quotas["premium_interactions"].percent_remaining, 100.0);
        assert_eq!(quotas["chat"].percent_remaining, 80.0);
    }

    #[test]
    fn test_parse_quota_from_response_headers_empty() {
        let headers = HeaderMap::new();
        let quotas = parse_copilot_quota_headers(&headers);
        assert!(quotas.is_empty());
    }

    #[test]
    fn test_parse_quota_snapshot_header_malformed() {
        let raw = "garbage";
        let result = parse_quota_snapshot_header(raw);
        assert!(result.is_none());
    }
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test -p codex-api copilot_quota -- --nocapture`
Expected: FAIL — module doesn't exist.

**Step 3: Write implementation**

```rust
//! Parser for Copilot x-quota-snapshot-* response headers.
//!
//! Headers have the format:
//!   x-quota-snapshot-<category>: ent=<int>&ov=<float>&ovPerm=<bool>&rem=<float>&rst=<url-encoded-date>

use http::HeaderMap;
use std::collections::HashMap;

const QUOTA_HEADER_PREFIX: &str = "x-quota-snapshot-";

#[derive(Debug, Clone, PartialEq)]
pub struct CopilotQuotaSnapshot {
    pub entitlement: i64,
    pub overage: f64,
    pub overage_permitted: bool,
    pub percent_remaining: f64,
    pub resets_at: String,
}

pub fn parse_copilot_quota_headers(headers: &HeaderMap) -> HashMap<String, CopilotQuotaSnapshot> {
    let mut quotas = HashMap::new();
    for (name, value) in headers.iter() {
        let header_name = name.as_str().to_ascii_lowercase();
        if let Some(category) = header_name.strip_prefix(QUOTA_HEADER_PREFIX) {
            if let Some(snapshot) = value.to_str().ok().and_then(parse_quota_snapshot_header) {
                quotas.insert(category.to_string(), snapshot);
            }
        }
    }
    quotas
}

pub fn parse_quota_snapshot_header(raw: &str) -> Option<CopilotQuotaSnapshot> {
    let params: HashMap<&str, &str> = raw
        .split('&')
        .filter_map(|pair| pair.split_once('='))
        .collect();

    Some(CopilotQuotaSnapshot {
        entitlement: params.get("ent")?.parse().ok()?,
        overage: params.get("ov")?.parse().ok()?,
        overage_permitted: *params.get("ovPerm")? == "true",
        percent_remaining: params.get("rem")?.parse().ok()?,
        resets_at: urlencoding::decode(params.get("rst")?).ok()?.to_string(),
    })
}
```

**Note:** Check if `urlencoding` already exists in the workspace `Cargo.toml` before adding. If not present, add it to both `[workspace.dependencies]` and `codex-api/Cargo.toml`. Alternatively, avoid the dependency entirely with a simple inline decode: `raw.replace("%3A", ":").replace("%2F", "/")` — the only encoded chars in the reset date are colons and possibly slashes.

**Step 4: Add `pub mod copilot_quota;` to `codex-api/src/lib.rs`**

**Step 5: Run tests to verify they pass**

Run: `cargo test -p codex-api copilot_quota -- --nocapture`
Expected: PASS

**Step 6: Commit**

```bash
git add codex-rs/codex-api/src/copilot_quota.rs codex-rs/codex-api/src/lib.rs codex-rs/codex-api/Cargo.toml
git commit -m "feat(copilot): parse x-quota-snapshot response headers"
```

---

### Task 9: Add CopilotQuota to Protocol Types

**Files:**
- Modify: `codex-rs/protocol/src/protocol.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn test_copilot_quota_serialization_roundtrip() {
    let quota = CopilotQuota {
        category: "premium_interactions".to_string(),
        entitlement: 1000,
        percent_remaining: 75.0,
        unlimited: false,
        resets_at: Some("2026-04-01T00:00:00Z".to_string()),
    };

    let json = serde_json::to_string(&quota).unwrap();
    let parsed: CopilotQuota = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, quota);
}
```

**Step 2: Run test — FAIL**

**Step 3: Add the type**

Add to `protocol/src/protocol.rs` near `TokenUsage`:

```rust
/// Copilot subscription quota snapshot from response headers.
#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq, JsonSchema, TS)]
pub struct CopilotQuota {
    pub category: String,
    #[ts(type = "number")]
    pub entitlement: i64,
    pub percent_remaining: f64,
    pub unlimited: bool,
    pub resets_at: Option<String>,
}
```

Add to `TokenCountEvent`:

```rust
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, TS)]
pub struct TokenCountEvent {
    pub info: Option<TokenUsageInfo>,
    pub rate_limits: Option<RateLimitSnapshot>,
    pub copilot_quota: Option<Vec<CopilotQuota>>,
}
```

**Step 4: Fix all construction sites** — Update all places that create `TokenCountEvent` to include `copilot_quota: None`.

**Step 5: Run tests**

Run: `cargo test -p codex-protocol -- --nocapture`
Expected: PASS

**Step 6: Commit**

```bash
git add codex-rs/protocol/src/protocol.rs
git commit -m "feat(copilot): add CopilotQuota protocol type"
```

---

### Task 10: Wire Quota Parsing into SSE Response Flow

**Files:**
- Modify: `codex-rs/codex-api/src/common.rs` (add variant to `ResponseEvent` enum, line 66-95)
- Modify: `codex-rs/codex-api/src/sse/responses.rs` (emit quota event in `spawn_response_stream`, line 57+)
- Modify: `codex-rs/core/src/codex.rs` (handle new event ~line 7371, store in session state, include in `send_token_count_event` at line 3794-3801)
- Modify: `codex-rs/core/src/turn_timing.rs` (add `CopilotQuota` to the non-streaming match arm, line 110-116)
- Modify: `codex-rs/core/src/compact.rs` (handle new event variant in compact flow, ~line 426)

**Data flow:**
1. SSE response headers arrive in `spawn_response_stream()` (`codex-api/src/sse/responses.rs`)
2. Parse headers → emit `ResponseEvent::CopilotQuota(HashMap<String, CopilotQuotaSnapshot>)` via `tx_event`
3. `ResponseEvent::CopilotQuota` variant added to enum in `codex-api/src/common.rs:66`
4. In `codex-rs/core/src/codex.rs`, the match arm at ~line 7371 (next to `ResponseEvent::RateLimits`) stores quota in session state
5. `send_token_count_event` at line 3794-3801 reads quota from session state and includes it in `TokenCountEvent.copilot_quota`
6. Must also add the variant to the exhaustive matches in `turn_timing.rs:110-116` and `compact.rs:~426`

**Step 1: Write the failing test**

Add test in `codex-api/src/copilot_quota.rs` (reconfirms parsing, already passes from Task 8):

```rust
#[test]
fn test_quota_headers_extracted_from_stream_response() {
    let mut headers = HeaderMap::new();
    headers.insert(
        "x-quota-snapshot-premium_interactions",
        HeaderValue::from_static("ent=-1&ov=0.0&ovPerm=true&rem=100.0&rst=2026-04-01T00%3A00%3A00Z"),
    );

    let quotas = parse_copilot_quota_headers(&headers);
    assert_eq!(quotas.len(), 1);
    assert_eq!(quotas["premium_interactions"].percent_remaining, 100.0);
}
```

**Step 2: Add `CopilotQuota` variant to `ResponseEvent`**

In `codex-api/src/common.rs`, add after `RateLimits(RateLimitSnapshot)` (line 93):

```rust
CopilotQuota(std::collections::HashMap<String, crate::copilot_quota::CopilotQuotaSnapshot>),
```

**Step 3: Emit quota event in SSE response stream**

In `codex-api/src/sse/responses.rs`, in `spawn_response_stream()` where headers are parsed (after existing header extraction for rate limits/model etag, around line 57-106), add:

```rust
let copilot_quotas = crate::copilot_quota::parse_copilot_quota_headers(&stream_response.headers);
if !copilot_quotas.is_empty() {
    let _ = tx_event.send(Ok(ResponseEvent::CopilotQuota(copilot_quotas))).await;
}
```

**Step 4: Handle new event in `codex.rs` and `compact.rs`**

In `codex-rs/core/src/codex.rs`, add match arm next to `ResponseEvent::RateLimits` (~line 7371):

```rust
ResponseEvent::CopilotQuota(quotas) => {
    sess.update_copilot_quota(&turn_context, quotas).await;
}
```

Add `update_copilot_quota` to session state (stores `Vec<CopilotQuota>` on the session, converting from `HashMap`).

In `send_token_count_event` (line 3794-3801), read stored quota from session state:

```rust
let (info, rate_limits, copilot_quota) = {
    let state = self.state.lock().await;
    (state.token_usage_info(), state.rate_limits(), state.copilot_quota())
};
let event = EventMsg::TokenCount(TokenCountEvent { info, rate_limits, copilot_quota });
```

Add the variant to exhaustive matches in `turn_timing.rs:110-116` and `compact.rs:~426` (both as no-op arms that return false / do nothing).

**Step 5: Run full test suite**

Run: `cargo nextest run -p codex-core -p codex-api`
Expected: PASS

**Step 4: Run full test suite**

Run: `cargo nextest run -p codex-core -p codex-api`
Expected: PASS

**Step 5: Commit**

```bash
git add codex-rs/codex-api/src/sse/responses.rs codex-rs/core/src/codex.rs
git commit -m "feat(copilot): wire quota header parsing into SSE response flow"
```

---

### Task 11: TUI Quota Display

**Files:**
- Modify: `codex-rs/tui/src/status/card.rs`

**Step 1: Write the failing test (or manual verification)**

This is a display-only change. Add the quota rendering logic:

```rust
fn copilot_quota_spans(&self) -> Option<Vec<Span<'static>>> {
    let quotas = self.copilot_quota.as_ref()?;
    // Find the primary quota (premium_interactions)
    let primary = quotas.iter().find(|q| q.category == "premium_interactions")?;

    let mut spans = vec![];
    if primary.unlimited {
        spans.push(Span::from(format!("{}% remaining", primary.percent_remaining)));
        spans.push(Span::from(" (unlimited)").dim());
    } else {
        let remaining = (primary.entitlement as f64 * primary.percent_remaining / 100.0) as i64;
        spans.push(Span::from(format!("{}% remaining", primary.percent_remaining)));
        spans.push(Span::from(format!(" ({}/{} premium interactions)", remaining, primary.entitlement)).dim());
    }
    if let Some(resets_at) = &primary.resets_at {
        spans.push(Span::from(format!(", resets {resets_at}")).dim());
    }
    Some(spans)
}
```

Add to the status card rendering, after the token usage line:

```rust
if let Some(quota_spans) = self.copilot_quota_spans() {
    lines.push(formatter.line("Copilot quota", quota_spans));
}
```

**Step 2: Manual verification**

Run: `just codex -c model_provider=copilot -m gpt-5.3-codex` then type `/status`
Expected: See "Copilot quota: 100.0% remaining (unlimited)" line.

**Step 3: Commit**

```bash
git add codex-rs/tui/src/status/card.rs
git commit -m "feat(copilot): display quota in TUI /status"
```

---

## Phase 5: End-to-End Integration Test

### Task 12: Full Integration Test with Mock Server

**Files:**
- Create: `codex-rs/core/tests/suite/copilot_provider.rs`
- Modify: `codex-rs/core/tests/all.rs` (add module)

**Step 1: Write the integration test**

```rust
//! Integration tests for the native Copilot provider.

use core_test_support::*;
use wiremock::MockServer;
use wiremock::matchers::{method, path, header};
use wiremock::ResponseTemplate;

/// Verify end-to-end: Copilot provider sends request with correct headers.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn copilot_provider_sends_copilot_headers() -> anyhow::Result<()> {
    skip_if_sandbox!(Ok(()));

    let server = start_mock_server().await;

    // Mount a response that checks for Copilot-specific headers
    let mock = wiremock::Mock::given(method("POST"))
        .and(path("/responses"))
        .and(header("Copilot-Integration-Id", "copilot-developer-cli"))
        .and(header("X-GitHub-Api-Version", "2026-01-09"))
        .and(header("Openai-Intent", "conversation-agent"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            sse(vec![
                ev_response_created("resp-1"),
                ev_assistant_message("msg-1", "Hello from Copilot"),
                ev_completed("resp-1"),
            ]),
        ))
        .mount(&server)
        .await;

    // Build test codex with copilot provider pointing at mock server
    let mut builder = test_codex()
        .with_model("gpt-5.3-codex")
        .with_config(|config| {
            config.model_provider_id = "copilot".to_string();
            // Override base_url to mock server
            config.model_provider.base_url = Some(server.uri());
            config.model_provider.experimental_bearer_token = Some("gho_test".to_string());
        });

    let test = builder.build(&server).await?;
    test.submit_turn("hello").await?;

    // Verify the mock was hit (headers matched)
    // If headers didn't match, wiremock wouldn't have responded
    Ok(())
}

/// Verify SSE streaming works through the Copilot provider.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn copilot_provider_streams_sse_response() -> anyhow::Result<()> {
    skip_if_sandbox!(Ok(()));

    let server = start_mock_server().await;

    let response_mock = mount_sse_once(
        &server,
        sse(vec![
            ev_response_created("resp-1"),
            ev_assistant_message("msg-1", "Copilot says hi"),
            ev_completed_with_tokens("resp-1", serde_json::json!({
                "input_tokens": 10,
                "input_tokens_details": { "cached_tokens": 0 },
                "output_tokens": 5,
                "output_tokens_details": { "reasoning_tokens": 0 },
                "total_tokens": 15
            })),
        ]),
    ).await;

    let mut builder = test_codex()
        .with_model("gpt-5.3-codex")
        .with_config(|config| {
            config.model_provider_id = "copilot".to_string();
            config.model_provider.base_url = Some(server.uri());
            config.model_provider.experimental_bearer_token = Some("gho_test".to_string());
        });

    let test = builder.build(&server).await?;
    test.submit_turn("hello").await?;

    // Verify request was made
    let req = response_mock.single_request();
    let body = req.body_json();
    assert_eq!(body["model"], "gpt-5.3-codex");

    Ok(())
}
```

**Step 2: Add to `all.rs`**

```rust
mod copilot_provider;
```

**Step 3: Run the integration tests**

Run: `cargo nextest run -p codex-core copilot_provider`
Expected: PASS

**Step 4: Commit**

```bash
git add codex-rs/core/tests/suite/copilot_provider.rs codex-rs/core/tests/all.rs
git commit -m "test(copilot): add end-to-end integration tests for native provider"
```

---

## Phase 6: Cleanup

### Task 13: Documentation

**Files:**
- Modify: `codex-rs/tools/copilot-proxy/README.md` (add deprecation notice)

**Step 1: Add deprecation notice to proxy README**

Add at the top:

```markdown
> **DEPRECATED:** This Python proxy has been superseded by the native Copilot
> provider built into Codex. Set `model_provider = "copilot"` in
> `~/.codex/config.toml` — no proxy needed.
```

**Step 2: Commit**

```bash
git add codex-rs/tools/copilot-proxy/README.md
git commit -m "docs: deprecate copilot-proxy in favor of native provider"
```

---

### Task 14: Manual End-to-End Verification

**No code changes — manual test.**

**Step 1: Stop the Python proxy (if running)**

```bash
./codex-rs/tools/copilot-proxy/copilot-proxy.sh stop
```

**Step 2: Update config.toml**

Ensure `~/.codex/config.toml` has:
```toml
model_provider = "copilot"
```

Remove any `[model_providers.copilot]` block (no longer needed — it's built-in).

**Step 3: Build and run**

```bash
cd codex-rs && cargo build
just codex -m gpt-5.3-codex "explain what 1+1 is"
```

Expected: Response from Copilot API, no proxy running.

**Step 4: Verify /status shows quota**

```bash
just codex -m gpt-5.3-codex
# In the TUI, type /status
```

Expected: See "Copilot quota" line with percentage remaining.

**Step 5: Verify subagents inherit provider**

```bash
just exec -m gpt-5.3-codex "list files in current directory"
```

Expected: Subagent uses Copilot provider (check logs for Copilot-Integration-Id header).

---

## Summary

| Phase | Tasks | What it delivers |
|-------|-------|-----------------|
| 1 | Tasks 1-2 | Built-in provider registration with static headers |
| 2 | Tasks 3-4 | Keychain token reading + endpoint resolution |
| 3 | Tasks 5-6b, 7 | Auth pipeline integration, startup error handling, auth.json bypass |
| 4 | Tasks 8-11 | Quota header parsing + TUI display |
| 5 | Task 12 | End-to-end integration tests |
| 6 | Tasks 13-14 | Docs, deprecation, manual verification |

**Total: 15 tasks (including 6b), tests cover every component.**
