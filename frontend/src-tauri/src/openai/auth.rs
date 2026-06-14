use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Runtime};
use url::Url;

use crate::{database::repositories::setting::SettingsRepository, state::AppState};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OpenAIAuthMode {
    Disabled,
    ApiKey,
    OauthPkce,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OpenAIOAuthPkceConfig {
    pub client_id: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub redirect_uri: String,
    #[serde(default)]
    pub scopes: Vec<String>,
    #[serde(default)]
    pub issuer: Option<String>,
    #[serde(default)]
    pub audience: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OpenAIAuthConfig {
    pub mode: OpenAIAuthMode,
    #[serde(default)]
    pub oauth_pkce: Option<OpenAIOAuthPkceConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OpenAIAuthStatus {
    pub mode: OpenAIAuthMode,
    pub configured: bool,
    pub api_key_present: bool,
    pub oauth_pkce_configured: bool,
    pub can_authenticate_requests: bool,
    pub requires_user_action: bool,
    pub source: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oauth_pkce: Option<OpenAIOAuthPkceConfig>,
}

fn is_present(value: Option<&str>) -> bool {
    value.map(|v| !v.trim().is_empty()).unwrap_or(false)
}

fn parse_openai_auth_config(json: Option<String>) -> Result<Option<OpenAIAuthConfig>, String> {
    json.map(|raw| {
        serde_json::from_str::<OpenAIAuthConfig>(&raw)
            .map_err(|e| format!("Invalid OpenAI auth configuration JSON: {}", e))
    })
    .transpose()
}

fn validate_url_field(label: &str, value: &str) -> Result<(), String> {
    let parsed = Url::parse(value).map_err(|e| format!("{} must be a valid URL: {}", label, e))?;
    match parsed.scheme() {
        "https" => Ok(()),
        "http"
            if parsed.host_str() == Some("localhost") || parsed.host_str() == Some("127.0.0.1") =>
        {
            Ok(())
        }
        "http" => Err(format!(
            "{} must use https unless it targets localhost",
            label
        )),
        _ => Err(format!("{} must use http or https", label)),
    }
}

fn normalize_oauth_pkce_config(
    config: OpenAIOAuthPkceConfig,
) -> Result<OpenAIOAuthPkceConfig, String> {
    let client_id = config.client_id.trim();
    if client_id.is_empty() {
        return Err("OAuth client ID is required for oauth_pkce mode".to_string());
    }

    let authorization_endpoint = config.authorization_endpoint.trim();
    let token_endpoint = config.token_endpoint.trim();
    let redirect_uri = config.redirect_uri.trim();
    validate_url_field("Authorization endpoint", authorization_endpoint)?;
    validate_url_field("Token endpoint", token_endpoint)?;
    validate_url_field("Redirect URI", redirect_uri)?;

    let scopes = config
        .scopes
        .into_iter()
        .map(|scope| scope.trim().to_string())
        .filter(|scope| !scope.is_empty())
        .collect::<Vec<_>>();

    Ok(OpenAIOAuthPkceConfig {
        client_id: client_id.to_string(),
        authorization_endpoint: authorization_endpoint.to_string(),
        token_endpoint: token_endpoint.to_string(),
        redirect_uri: redirect_uri.to_string(),
        scopes,
        issuer: config
            .issuer
            .and_then(|issuer| (!issuer.trim().is_empty()).then(|| issuer.trim().to_string())),
        audience: config.audience.and_then(|audience| {
            (!audience.trim().is_empty()).then(|| audience.trim().to_string())
        }),
    })
}

fn normalize_auth_config(config: OpenAIAuthConfig) -> Result<OpenAIAuthConfig, String> {
    match config.mode {
        OpenAIAuthMode::Disabled => Ok(OpenAIAuthConfig {
            mode: OpenAIAuthMode::Disabled,
            oauth_pkce: None,
        }),
        OpenAIAuthMode::ApiKey => Ok(OpenAIAuthConfig {
            mode: OpenAIAuthMode::ApiKey,
            oauth_pkce: None,
        }),
        OpenAIAuthMode::OauthPkce => {
            let oauth_pkce = config.oauth_pkce.ok_or_else(|| {
                "OAuth PKCE configuration is required for oauth_pkce mode".to_string()
            })?;

            Ok(OpenAIAuthConfig {
                mode: OpenAIAuthMode::OauthPkce,
                oauth_pkce: Some(normalize_oauth_pkce_config(oauth_pkce)?),
            })
        }
    }
}

fn build_openai_auth_status(
    stored_config: Option<OpenAIAuthConfig>,
    legacy_api_key: Option<&str>,
) -> OpenAIAuthStatus {
    let api_key_present = is_present(legacy_api_key);

    match stored_config {
        Some(config) => match config.mode {
            OpenAIAuthMode::Disabled => OpenAIAuthStatus {
                mode: OpenAIAuthMode::Disabled,
                configured: false,
                api_key_present,
                oauth_pkce_configured: false,
                can_authenticate_requests: false,
                requires_user_action: true,
                source: "openai_auth_config".to_string(),
                message: "OpenAI auth is disabled in auth-mode configuration".to_string(),
                oauth_pkce: None,
            },
            OpenAIAuthMode::ApiKey => OpenAIAuthStatus {
                mode: OpenAIAuthMode::ApiKey,
                configured: api_key_present,
                api_key_present,
                oauth_pkce_configured: false,
                can_authenticate_requests: api_key_present,
                requires_user_action: !api_key_present,
                source: "openai_auth_config".to_string(),
                message: if api_key_present {
                    "OpenAI API key auth is configured through the legacy settings path".to_string()
                } else {
                    "OpenAI API key auth is selected, but no API key is stored".to_string()
                },
                oauth_pkce: None,
            },
            OpenAIAuthMode::OauthPkce => {
                let oauth_pkce_configured = config.oauth_pkce.is_some();
                OpenAIAuthStatus {
                    mode: OpenAIAuthMode::OauthPkce,
                    configured: oauth_pkce_configured,
                    api_key_present,
                    oauth_pkce_configured,
                    can_authenticate_requests: false,
                    requires_user_action: true,
                    source: "openai_auth_config".to_string(),
                    message: if oauth_pkce_configured {
                        "OpenAI OAuth PKCE metadata is configured, but token exchange is not implemented"
                            .to_string()
                    } else {
                        "OpenAI OAuth PKCE mode is selected, but metadata is incomplete".to_string()
                    },
                    oauth_pkce: config.oauth_pkce,
                }
            }
        },
        None if api_key_present => OpenAIAuthStatus {
            mode: OpenAIAuthMode::ApiKey,
            configured: true,
            api_key_present,
            oauth_pkce_configured: false,
            can_authenticate_requests: true,
            requires_user_action: false,
            source: "legacy_api_key".to_string(),
            message: "OpenAI API key auth is configured through the legacy settings path"
                .to_string(),
            oauth_pkce: None,
        },
        None => OpenAIAuthStatus {
            mode: OpenAIAuthMode::Disabled,
            configured: false,
            api_key_present: false,
            oauth_pkce_configured: false,
            can_authenticate_requests: false,
            requires_user_action: true,
            source: "not_configured".to_string(),
            message: "OpenAI auth is not configured".to_string(),
            oauth_pkce: None,
        },
    }
}

/// Reports the configured OpenAI auth mode without returning secrets.
#[tauri::command]
pub async fn api_get_openai_auth_status<R: Runtime>(
    _app: AppHandle<R>,
    state: tauri::State<'_, AppState>,
) -> Result<OpenAIAuthStatus, String> {
    let pool = state.db_manager.pool();
    let stored_config = parse_openai_auth_config(
        SettingsRepository::get_openai_auth_config(pool)
            .await
            .map_err(|e| format!("Failed to read OpenAI auth configuration: {}", e))?,
    )?;
    let api_key = SettingsRepository::get_api_key(pool, "openai")
        .await
        .map_err(|e| format!("Failed to read OpenAI API key status: {}", e))?;

    Ok(build_openai_auth_status(stored_config, api_key.as_deref()))
}

/// Saves OpenAI auth-mode metadata. API keys still use the existing settings path.
/// OAuth client secrets and tokens are intentionally not accepted or stored here.
#[tauri::command]
pub async fn api_save_openai_auth_config<R: Runtime>(
    _app: AppHandle<R>,
    state: tauri::State<'_, AppState>,
    config: OpenAIAuthConfig,
) -> Result<OpenAIAuthStatus, String> {
    let config = normalize_auth_config(config)?;
    let config_json = serde_json::to_string(&config)
        .map_err(|e| format!("Failed to serialize OpenAI auth configuration: {}", e))?;
    let pool = state.db_manager.pool();

    SettingsRepository::save_openai_auth_config(pool, &config_json)
        .await
        .map_err(|e| format!("Failed to save OpenAI auth configuration: {}", e))?;

    let api_key = SettingsRepository::get_api_key(pool, "openai")
        .await
        .map_err(|e| format!("Failed to read OpenAI API key status: {}", e))?;

    Ok(build_openai_auth_status(Some(config), api_key.as_deref()))
}

/// Clears only the auth-mode metadata. Existing legacy OpenAI API keys are not removed.
#[tauri::command]
pub async fn api_clear_openai_auth_config<R: Runtime>(
    _app: AppHandle<R>,
    state: tauri::State<'_, AppState>,
) -> Result<OpenAIAuthStatus, String> {
    let pool = state.db_manager.pool();
    SettingsRepository::clear_openai_auth_config(pool)
        .await
        .map_err(|e| format!("Failed to clear OpenAI auth configuration: {}", e))?;

    let api_key = SettingsRepository::get_api_key(pool, "openai")
        .await
        .map_err(|e| format!("Failed to read OpenAI API key status: {}", e))?;

    Ok(build_openai_auth_status(None, api_key.as_deref()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn oauth_config() -> OpenAIOAuthPkceConfig {
        OpenAIOAuthPkceConfig {
            client_id: " client-123 ".to_string(),
            authorization_endpoint: "https://auth.example.test/oauth/authorize ".to_string(),
            token_endpoint: "https://auth.example.test/oauth/token".to_string(),
            redirect_uri: "http://127.0.0.1:38451/openai/oauth/callback".to_string(),
            scopes: vec![" openai ".to_string(), "".to_string()],
            issuer: Some(" ".to_string()),
            audience: Some(" api ".to_string()),
        }
    }

    #[test]
    fn legacy_api_key_reports_api_key_mode_without_stored_config() {
        let status = build_openai_auth_status(None, Some("sk-test"));

        assert_eq!(status.mode, OpenAIAuthMode::ApiKey);
        assert!(status.configured);
        assert!(status.can_authenticate_requests);
        assert_eq!(status.source, "legacy_api_key");
    }

    #[test]
    fn disabled_mode_overrides_legacy_key_in_status() {
        let status = build_openai_auth_status(
            Some(OpenAIAuthConfig {
                mode: OpenAIAuthMode::Disabled,
                oauth_pkce: None,
            }),
            Some("sk-test"),
        );

        assert_eq!(status.mode, OpenAIAuthMode::Disabled);
        assert!(!status.configured);
        assert!(status.api_key_present);
        assert!(!status.can_authenticate_requests);
    }

    #[test]
    fn oauth_pkce_metadata_is_not_reported_as_request_ready() {
        let status = build_openai_auth_status(
            Some(OpenAIAuthConfig {
                mode: OpenAIAuthMode::OauthPkce,
                oauth_pkce: Some(oauth_config()),
            }),
            None,
        );

        assert_eq!(status.mode, OpenAIAuthMode::OauthPkce);
        assert!(status.configured);
        assert!(status.oauth_pkce_configured);
        assert!(!status.can_authenticate_requests);
    }

    #[test]
    fn oauth_pkce_normalization_trims_public_metadata() {
        let config = normalize_auth_config(OpenAIAuthConfig {
            mode: OpenAIAuthMode::OauthPkce,
            oauth_pkce: Some(oauth_config()),
        })
        .expect("valid oauth config");

        let oauth = config.oauth_pkce.expect("oauth config");
        assert_eq!(oauth.client_id, "client-123");
        assert_eq!(oauth.scopes, vec!["openai"]);
        assert_eq!(oauth.issuer, None);
        assert_eq!(oauth.audience.as_deref(), Some("api"));
    }

    #[test]
    fn oauth_pkce_requires_https_for_non_localhost_endpoints() {
        let mut config = oauth_config();
        config.authorization_endpoint = "http://auth.example.test/oauth/authorize".to_string();

        let error = normalize_auth_config(OpenAIAuthConfig {
            mode: OpenAIAuthMode::OauthPkce,
            oauth_pkce: Some(config),
        })
        .expect_err("non-localhost http endpoint should fail");

        assert!(error.contains("must use https"));
    }
}
