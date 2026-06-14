# OpenAI Auth Modes

This fork keeps the existing OpenAI API-key path and adds a backend scaffold for future OAuth PKCE support.

## Current Status

### API key

Status: working legacy path.

The existing `openai` provider continues to use the `openaiApiKey` value in the local settings database. Summary generation still authenticates OpenAI requests with a bearer API key.

Related existing commands:

- `api_save_model_config`
- `api_get_model_config`
- `api_get_api_key`
- `get_openai_models`

### OAuth PKCE

Status: scaffolded only. Not production-ready.

The backend now has safe auth-mode metadata types and Tauri commands for `oauth_pkce`, but it does not perform browser login, PKCE verifier/challenge generation, token exchange, refresh, token storage, or authenticated OpenAI API calls with OAuth tokens.

The scaffold is intentionally provider-neutral and PKCE-oriented until official OpenAI OAuth application details are available for this product. Before enabling OAuth, the product still needs:

- Official OpenAI OAuth app/client details for this desktop app.
- The allowed redirect URI shape for the desktop callback.
- The authorization endpoint, token endpoint, scopes, and any audience/issuer requirements.
- A secure local token storage design.
- Request-path changes so summary/model calls can use OAuth access tokens.

No OAuth client secret is stored in source code or accepted by the scaffolded commands.

### Disabled or not configured

Status: supported as a status/config mode.

When no OpenAI auth metadata and no legacy API key are present, `api_get_openai_auth_status` reports `disabled` with `configured: false`.

## Backend Commands

### `api_get_openai_auth_status`

Returns auth status without returning secrets.

Important fields:

- `mode`: `disabled`, `api_key`, or `oauth_pkce`.
- `configured`: whether the selected mode has enough metadata or key material recorded.
- `apiKeyPresent`: whether the legacy OpenAI API key exists.
- `oauthPkceConfigured`: whether OAuth PKCE metadata exists.
- `canAuthenticateRequests`: currently true only for API-key auth with a stored key.
- `oauthPkce`: safe OAuth metadata, when configured.

### `api_save_openai_auth_config`

Stores auth-mode metadata in the settings database column `openAIAuthConfig`.

For `api_key`, this command stores only the selected mode. API keys still use the existing API-key settings path.

For `oauth_pkce`, this command stores only public/client metadata:

- `clientId`
- `authorizationEndpoint`
- `tokenEndpoint`
- `redirectUri`
- `scopes`
- optional `issuer`
- optional `audience`

### `api_clear_openai_auth_config`

Clears only `openAIAuthConfig`. It does not remove the legacy OpenAI API key.

## Validation Notes

The scaffold has Rust unit tests for mode resolution, disabled status, API-key compatibility, OAuth metadata normalization, and URL validation.

OAuth cannot be validated end to end yet because this change does not implement OpenAI OAuth login or token exchange.
