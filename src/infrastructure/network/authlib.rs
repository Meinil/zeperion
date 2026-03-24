use anyhow::{Context, Result, anyhow};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};

const AUTHLIB_API_SUFFIX: &str = "api/yggdrasil/";
const AUTHLIB_AUTHENTICATE_SUFFIX: &str = "authserver/authenticate";
const AUTHLIB_REFRESH_SUFFIX: &str = "authserver/refresh";

#[derive(Clone, Debug)]
pub struct AuthlibClient {
    http: Client,
}

impl AuthlibClient {
    pub fn new() -> Self {
        Self {
            http: Client::new(),
        }
    }

    pub fn with_client(http: Client) -> Self {
        Self { http }
    }

    pub fn normalize_server_root_url(input: &str) -> Result<Url> {
        normalize_server_root_url(input)
    }

    pub fn authlib_api_base_url(input: &str) -> Result<Url> {
        authlib_api_base_url(input)
    }

    pub async fn fetch_metadata(&self, server_url: &str) -> Result<AuthlibServerMetadata> {
        let root_url = normalize_server_root_url(server_url)?;
        let api_base_url = root_url
            .join(AUTHLIB_API_SUFFIX)
            .context("failed to build Authlib API base URL")?;

        let response = self
            .http
            .get(api_base_url.clone())
            .send()
            .await
            .context("failed to fetch Authlib metadata")?;

        let status = response.status();
        let bytes = response
            .bytes()
            .await
            .context("failed to read Authlib metadata body")?;

        if !status.is_success() {
            return Err(anyhow!(
                "Authlib metadata request failed: {}",
                body_as_string(&bytes)
            ));
        }

        let raw_json = serde_json::from_slice::<serde_json::Value>(&bytes)
            .context("failed to parse Authlib metadata JSON")?;
        let prefetched_base64 = STANDARD.encode(&bytes);

        Ok(AuthlibServerMetadata {
            server_root_url: root_url,
            api_base_url,
            raw_json,
            prefetched_base64,
        })
    }

    pub async fn authenticate(
        &self,
        server_url: &str,
        username: &str,
        password: &str,
        client_token: Option<&str>,
        request_user: bool,
    ) -> Result<AuthlibAuthenticateResponse> {
        let api_base_url = authlib_api_base_url(server_url)?;
        let response = self
            .http
            .post(api_base_url.join(AUTHLIB_AUTHENTICATE_SUFFIX)?)
            .json(&AuthlibAuthenticateRequest::new(
                username,
                password,
                client_token,
                request_user,
            ))
            .send()
            .await
            .context("failed to send Authlib authenticate request")?;

        decode_authlib_response(response, "Authlib authenticate response").await
    }

    pub async fn refresh(
        &self,
        server_url: &str,
        access_token: &str,
        client_token: &str,
        selected_profile: &AuthlibProfile,
        request_user: bool,
    ) -> Result<AuthlibRefreshResponse> {
        let api_base_url = authlib_api_base_url(server_url)?;
        let response = self
            .http
            .post(api_base_url.join(AUTHLIB_REFRESH_SUFFIX)?)
            .json(&AuthlibRefreshRequest::new(
                access_token,
                client_token,
                selected_profile,
                request_user,
            ))
            .send()
            .await
            .context("failed to send Authlib refresh request")?;

        decode_authlib_response(response, "Authlib refresh response").await
    }
}

impl Default for AuthlibClient {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug)]
pub struct AuthlibServerMetadata {
    pub server_root_url: Url,
    pub api_base_url: Url,
    pub raw_json: serde_json::Value,
    pub prefetched_base64: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthlibProfile {
    pub id: String,
    pub name: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthlibUserProperty {
    pub name: String,
    pub value: String,
    #[serde(default)]
    pub signature: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthlibUser {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub properties: Vec<AuthlibUserProperty>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthlibAuthenticateResponse {
    pub access_token: String,
    pub client_token: String,
    #[serde(default)]
    pub available_profiles: Vec<AuthlibProfile>,
    #[serde(default)]
    pub selected_profile: Option<AuthlibProfile>,
    #[serde(default)]
    pub user: Option<AuthlibUser>,
}

impl AuthlibAuthenticateResponse {
    pub fn resolved_profile(&self) -> Option<&AuthlibProfile> {
        self.selected_profile
            .as_ref()
            .or_else(|| self.available_profiles.first())
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthlibRefreshResponse {
    pub access_token: String,
    pub client_token: String,
    #[serde(default)]
    pub selected_profile: Option<AuthlibProfile>,
    #[serde(default)]
    pub available_profiles: Vec<AuthlibProfile>,
    #[serde(default)]
    pub user: Option<AuthlibUser>,
}

impl AuthlibRefreshResponse {
    pub fn resolved_profile(&self) -> Option<&AuthlibProfile> {
        self.selected_profile
            .as_ref()
            .or_else(|| self.available_profiles.first())
    }
}

#[derive(Clone, Debug, Serialize)]
struct AuthlibAgent {
    name: &'static str,
    version: u8,
}

impl Default for AuthlibAgent {
    fn default() -> Self {
        Self {
            name: "Minecraft",
            version: 1,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
struct AuthlibAuthenticateRequest<'a> {
    agent: AuthlibAgent,
    username: &'a str,
    password: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    client_token: Option<&'a str>,
    request_user: bool,
}

impl<'a> AuthlibAuthenticateRequest<'a> {
    fn new(
        username: &'a str,
        password: &'a str,
        client_token: Option<&'a str>,
        request_user: bool,
    ) -> Self {
        Self {
            agent: AuthlibAgent::default(),
            username,
            password,
            client_token,
            request_user,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
struct AuthlibRefreshRequest<'a> {
    access_token: &'a str,
    client_token: &'a str,
    selected_profile: &'a AuthlibProfile,
    request_user: bool,
}

impl<'a> AuthlibRefreshRequest<'a> {
    fn new(
        access_token: &'a str,
        client_token: &'a str,
        selected_profile: &'a AuthlibProfile,
        request_user: bool,
    ) -> Self {
        Self {
            access_token,
            client_token,
            selected_profile,
            request_user,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AuthlibErrorResponse {
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    error_message: Option<String>,
    #[serde(default)]
    cause: Option<String>,
}

async fn decode_authlib_response<T: serde::de::DeserializeOwned>(
    response: reqwest::Response,
    context: &str,
) -> Result<T> {
    let status = response.status();
    let bytes = response
        .bytes()
        .await
        .with_context(|| format!("failed to read {context} body"))?;

    if status.is_success() {
        serde_json::from_slice::<T>(&bytes).with_context(|| format!("failed to parse {context}"))
    } else {
        let detail = parse_authlib_error(&bytes).unwrap_or_else(|| body_as_string(&bytes));
        Err(anyhow!("{context} failed: {detail}"))
    }
}

fn parse_authlib_error(bytes: &[u8]) -> Option<String> {
    let parsed = serde_json::from_slice::<AuthlibErrorResponse>(bytes).ok()?;
    let mut parts = Vec::new();
    if let Some(error) = parsed.error {
        if !error.is_empty() {
            parts.push(error);
        }
    }
    if let Some(message) = parsed.error_message {
        if !message.is_empty() {
            parts.push(message);
        }
    }
    if let Some(cause) = parsed.cause {
        if !cause.is_empty() {
            parts.push(cause);
        }
    }
    if parts.is_empty() {
        None
    } else {
        Some(parts.join(": "))
    }
}

fn body_as_string(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).trim().to_string()
}

fn normalize_server_root_url(input: &str) -> Result<Url> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(anyhow!("Authlib server URL cannot be empty"));
    }

    let candidate = if trimmed.contains("://") {
        trimmed.to_string()
    } else {
        format!("https://{trimmed}")
    };

    let mut url = Url::parse(&candidate).context("invalid Authlib server URL")?;
    url.set_query(None);
    url.set_fragment(None);

    let path = url.path().trim_end_matches('/');
    let path = path.strip_suffix("/api/yggdrasil").unwrap_or(path);
    let normalized_path = if path.is_empty() {
        "/".to_string()
    } else if path.ends_with('/') {
        path.to_string()
    } else {
        format!("{path}/")
    };
    url.set_path(&normalized_path);

    Ok(url)
}

fn authlib_api_base_url(input: &str) -> Result<Url> {
    let root = normalize_server_root_url(input)?;
    root.join(AUTHLIB_API_SUFFIX)
        .context("failed to build Authlib API base URL")
}

#[cfg(test)]
mod tests {
    use super::{AuthlibClient, authlib_api_base_url, normalize_server_root_url};

    #[test]
    fn normalizes_server_root_without_scheme() {
        let url = normalize_server_root_url("auth.example.com").unwrap();
        assert_eq!(url.as_str(), "https://auth.example.com/");
    }

    #[test]
    fn strips_existing_yggdrasil_suffix() {
        let url = normalize_server_root_url("https://auth.example.com/api/yggdrasil/").unwrap();
        assert_eq!(url.as_str(), "https://auth.example.com/");
    }

    #[test]
    fn builds_api_base_url_from_root() {
        let url = authlib_api_base_url("https://auth.example.com").unwrap();
        assert_eq!(url.as_str(), "https://auth.example.com/api/yggdrasil/");
    }

    #[test]
    fn keeps_nested_server_paths_intact() {
        let url = authlib_api_base_url("https://auth.example.com/custom/").unwrap();
        assert_eq!(
            url.as_str(),
            "https://auth.example.com/custom/api/yggdrasil/"
        );
    }

    #[test]
    fn client_helpers_delegate_to_free_functions() {
        assert_eq!(
            AuthlibClient::normalize_server_root_url("auth.example.com")
                .unwrap()
                .as_str(),
            "https://auth.example.com/"
        );
        assert_eq!(
            AuthlibClient::authlib_api_base_url("https://auth.example.com")
                .unwrap()
                .as_str(),
            "https://auth.example.com/api/yggdrasil/"
        );
    }
}
