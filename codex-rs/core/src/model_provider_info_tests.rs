use super::*;
use pretty_assertions::assert_eq;

#[test]
fn test_copilot_provider_is_registered_as_builtin() {
    let providers = built_in_model_providers(None);
    assert!(
        providers.contains_key("copilot"),
        "copilot should be a built-in provider"
    );
    let copilot = &providers["copilot"];
    assert_eq!(copilot.name, "GitHub Copilot");
    assert_eq!(copilot.requires_openai_auth, false);
    assert_eq!(copilot.supports_websockets, false);
    assert_eq!(copilot.wire_api, WireApi::Responses);
    // base_url is None — resolved dynamically at startup
    assert!(copilot.base_url.is_none());
    // Copilot-specific headers are set
    let headers = copilot
        .http_headers
        .as_ref()
        .expect("should have http_headers");
    assert_eq!(
        headers.get("Copilot-Integration-Id").unwrap(),
        "copilot-developer-cli"
    );
    assert_eq!(
        headers.get("X-GitHub-Api-Version").unwrap(),
        "2026-01-09"
    );
    assert_eq!(
        headers.get("Openai-Intent").unwrap(),
        "conversation-agent"
    );
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

#[test]
fn test_deserialize_ollama_model_provider_toml() {
    let azure_provider_toml = r#"
name = "Ollama"
base_url = "http://localhost:11434/v1"
        "#;
    let expected_provider = ModelProviderInfo {
        name: "Ollama".into(),
        base_url: Some("http://localhost:11434/v1".into()),
        env_key: None,
        env_key_instructions: None,
        experimental_bearer_token: None,
        wire_api: WireApi::Responses,
        query_params: None,
        http_headers: None,
        env_http_headers: None,
        request_max_retries: None,
        stream_max_retries: None,
        stream_idle_timeout_ms: None,
        websocket_connect_timeout_ms: None,
        requires_openai_auth: false,
        supports_websockets: false,
    };

    let provider: ModelProviderInfo = toml::from_str(azure_provider_toml).unwrap();
    assert_eq!(expected_provider, provider);
}

#[test]
fn test_deserialize_azure_model_provider_toml() {
    let azure_provider_toml = r#"
name = "Azure"
base_url = "https://xxxxx.openai.azure.com/openai"
env_key = "AZURE_OPENAI_API_KEY"
query_params = { api-version = "2025-04-01-preview" }
        "#;
    let expected_provider = ModelProviderInfo {
        name: "Azure".into(),
        base_url: Some("https://xxxxx.openai.azure.com/openai".into()),
        env_key: Some("AZURE_OPENAI_API_KEY".into()),
        env_key_instructions: None,
        experimental_bearer_token: None,
        wire_api: WireApi::Responses,
        query_params: Some(maplit::hashmap! {
            "api-version".to_string() => "2025-04-01-preview".to_string(),
        }),
        http_headers: None,
        env_http_headers: None,
        request_max_retries: None,
        stream_max_retries: None,
        stream_idle_timeout_ms: None,
        websocket_connect_timeout_ms: None,
        requires_openai_auth: false,
        supports_websockets: false,
    };

    let provider: ModelProviderInfo = toml::from_str(azure_provider_toml).unwrap();
    assert_eq!(expected_provider, provider);
}

#[test]
fn test_deserialize_example_model_provider_toml() {
    let azure_provider_toml = r#"
name = "Example"
base_url = "https://example.com"
env_key = "API_KEY"
http_headers = { "X-Example-Header" = "example-value" }
env_http_headers = { "X-Example-Env-Header" = "EXAMPLE_ENV_VAR" }
        "#;
    let expected_provider = ModelProviderInfo {
        name: "Example".into(),
        base_url: Some("https://example.com".into()),
        env_key: Some("API_KEY".into()),
        env_key_instructions: None,
        experimental_bearer_token: None,
        wire_api: WireApi::Responses,
        query_params: None,
        http_headers: Some(maplit::hashmap! {
            "X-Example-Header".to_string() => "example-value".to_string(),
        }),
        env_http_headers: Some(maplit::hashmap! {
            "X-Example-Env-Header".to_string() => "EXAMPLE_ENV_VAR".to_string(),
        }),
        request_max_retries: None,
        stream_max_retries: None,
        stream_idle_timeout_ms: None,
        websocket_connect_timeout_ms: None,
        requires_openai_auth: false,
        supports_websockets: false,
    };

    let provider: ModelProviderInfo = toml::from_str(azure_provider_toml).unwrap();
    assert_eq!(expected_provider, provider);
}

#[test]
fn test_deserialize_chat_wire_api_shows_helpful_error() {
    let provider_toml = r#"
name = "OpenAI using Chat Completions"
base_url = "https://api.openai.com/v1"
env_key = "OPENAI_API_KEY"
wire_api = "chat"
        "#;

    let err = toml::from_str::<ModelProviderInfo>(provider_toml).unwrap_err();
    assert!(err.to_string().contains(CHAT_WIRE_API_REMOVED_ERROR));
}

#[test]
fn test_deserialize_websocket_connect_timeout() {
    let provider_toml = r#"
name = "OpenAI"
base_url = "https://api.openai.com/v1"
websocket_connect_timeout_ms = 15000
supports_websockets = true
        "#;

    let provider: ModelProviderInfo = toml::from_str(provider_toml).unwrap();
    assert_eq!(provider.websocket_connect_timeout_ms, Some(15_000));
}

#[test]
fn test_copilot_provider_cannot_be_overridden_by_user_config() {
    // Mirrors the merge logic in config/mod.rs:
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
    providers
        .entry("copilot".to_string())
        .or_insert(user_provider);

    let copilot = &providers["copilot"];
    assert_eq!(copilot.name, "GitHub Copilot"); // Built-in wins
    assert!(copilot.base_url.is_none()); // Not overridden
}
