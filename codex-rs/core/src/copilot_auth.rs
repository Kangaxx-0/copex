//! Copilot authentication via macOS Keychain.
//!
//! Reads the GitHub OAuth token stored by the `copilot` CLI.
//! The token is stored in macOS Keychain under service "copilot-cli"
//! with account "https://github.com:<username>".

use codex_keyring_store::KeyringStore;

pub(crate) const COPILOT_KEYCHAIN_SERVICE: &str = "copilot-cli";
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
        .ok_or_else(|| {
            anyhow::anyhow!(
                "No Copilot API endpoint found. Check your Copilot subscription status."
            )
        })?
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
    fetch_copilot_user_info_from(COPILOT_USER_ENDPOINT, token).await
}

/// Fetch Copilot user info from a given URL (allows test override).
pub(crate) async fn fetch_copilot_user_info_from(
    url: &str,
    token: &str,
) -> anyhow::Result<CopilotUserInfo> {
    let client = reqwest::Client::builder()
        .user_agent("copex")
        .build()?;
    let resp = client
        .get(url)
        .header("Authorization", format!("Bearer {token}"))
        .header("Accept", "application/json")
        .send()
        .await?;

    let status = resp.status().as_u16();
    tracing::debug!(status, url, "Copilot user info response");
    match status {
        200 => {
            let json: serde_json::Value = resp.json().await?;
            parse_copilot_user_info(&json)
        }
        401 => anyhow::bail!(
            "Copilot token expired or revoked. Run 'copilot auth login' to re-authenticate."
        ),
        403 => {
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!(
                "No active Copilot subscription found for this GitHub account. (HTTP 403: {body})"
            )
        }
        other => {
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Copilot API returned {other}: {body}")
        }
    }
}

/// Read the Copilot OAuth token from the keyring store.
///
/// If `github_username` is provided, reads from the keyring directly.
/// Otherwise, falls back to `security find-generic-password -s copilot-cli -w`
/// which returns the first match regardless of account.
pub(crate) fn read_copilot_token_from_store(
    store: &dyn KeyringStore,
    github_username: Option<&str>,
) -> anyhow::Result<String> {
    read_copilot_token_impl(store, github_username, true)
}

fn read_copilot_token_impl(
    store: &dyn KeyringStore,
    github_username: Option<&str>,
    allow_subprocess_fallback: bool,
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

    if allow_subprocess_fallback {
        read_copilot_token_from_subprocess()
    } else {
        anyhow::bail!(
            "Copilot token not found in keyring for the given account."
        )
    }
}

/// Fallback: use macOS `security` command to read by service name only.
fn read_copilot_token_from_subprocess() -> anyhow::Result<String> {
    let output = std::process::Command::new("security")
        .args(["find-generic-password", "-s", COPILOT_KEYCHAIN_SERVICE, "-w"])
        .output()
        .map_err(|e| {
            anyhow::anyhow!(
                "Failed to run 'security' command: {e}. \
                 Install and log into GitHub Copilot CLI first: \
                 https://docs.github.com/en/copilot/github-copilot-in-the-cli"
            )
        })?;

    let token = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if token.is_empty() {
        anyhow::bail!(
            "Copilot token not found in macOS Keychain. \
             Run 'copilot auth login' first to authenticate."
        );
    }
    Ok(token)
}

#[cfg(test)]
mod tests {
    use super::*;
    use codex_keyring_store::tests::MockKeyringStore;

    /// Helper: call read_copilot_token_impl without subprocess fallback
    /// so tests don't hit the real macOS Keychain.
    fn read_token_no_fallback(
        store: &dyn KeyringStore,
        username: Option<&str>,
    ) -> anyhow::Result<String> {
        read_copilot_token_impl(store, username, false)
    }

    #[test]
    fn test_read_copilot_token_returns_token_from_keychain() {
        let mock = MockKeyringStore::default();
        let account = "https://github.com:testuser";
        mock.save(COPILOT_KEYCHAIN_SERVICE, account, "gho_test123")
            .unwrap();

        let result = read_token_no_fallback(&mock, Some("testuser"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "gho_test123");
    }

    #[test]
    fn test_read_copilot_token_missing_returns_error() {
        let mock = MockKeyringStore::default();
        let result = read_token_no_fallback(&mock, Some("testuser"));
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("Copilot token not found"),
            "Error was: {err}"
        );
    }

    #[test]
    fn test_read_copilot_token_empty_returns_error() {
        let mock = MockKeyringStore::default();
        let account = "https://github.com:testuser";
        mock.save(COPILOT_KEYCHAIN_SERVICE, account, "").unwrap();

        let result = read_token_no_fallback(&mock, Some("testuser"));
        assert!(result.is_err());
    }

    #[test]
    fn test_read_copilot_token_without_username_returns_error_without_fallback() {
        let mock = MockKeyringStore::default();
        let result = read_token_no_fallback(&mock, None);
        assert!(result.is_err());
    }

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
        let quota = info.quota.as_ref().unwrap();
        assert_eq!(quota.percent_remaining, 75.0);
        assert!(!quota.unlimited);
        assert_eq!(quota.entitlement, 1000);
        assert_eq!(quota.remaining, 750);
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
            }
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
    fn test_copilot_auth_provider_returns_bearer_token() {
        use crate::api_bridge::auth_provider_from_auth;
        use crate::model_provider_info::create_copilot_provider;

        let mut provider = create_copilot_provider();
        provider.experimental_bearer_token = Some("gho_test_token".to_string());

        let result = auth_provider_from_auth(None, &provider).unwrap();
        // token is private, but auth_header_attached() confirms it was set
        assert!(result.auth_header_attached());
    }

    #[test]
    fn test_copilot_provider_without_token_has_no_auth() {
        use crate::api_bridge::auth_provider_from_auth;
        use crate::model_provider_info::create_copilot_provider;

        let provider = create_copilot_provider();
        let result = auth_provider_from_auth(None, &provider).unwrap();
        assert!(!result.auth_header_attached());
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

    #[test]
    fn test_copilot_provider_does_not_require_openai_auth() {
        use crate::model_provider_info::create_copilot_provider;

        let provider = create_copilot_provider();
        // requires_openai_auth = false means auth.json / login flow is never triggered
        assert!(!provider.requires_openai_auth);
        // Token comes from experimental_bearer_token, not env_key
        assert!(provider.env_key.is_none());
    }
}
