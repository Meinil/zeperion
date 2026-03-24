use std::time::{Duration, Instant};

use anyhow::{Context, Result, anyhow};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use sha2::{Digest, Sha256};
use tokio::time::sleep;
use uuid::Uuid;

const MICROSOFT_AUTHORIZE_ENDPOINT: &str =
    "https://login.microsoftonline.com/consumers/oauth2/v2.0/authorize";
const MICROSOFT_DEVICE_CODE_ENDPOINT: &str =
    "https://login.microsoftonline.com/consumers/oauth2/v2.0/devicecode";
const MICROSOFT_TOKEN_ENDPOINT: &str =
    "https://login.microsoftonline.com/consumers/oauth2/v2.0/token";
const XBOX_LIVE_AUTH_ENDPOINT: &str = "https://user.auth.xboxlive.com/user/authenticate";
const XSTS_AUTH_ENDPOINT: &str = "https://xsts.auth.xboxlive.com/xsts/authorize";
const MINECRAFT_LOGIN_WITH_XBOX_ENDPOINT: &str =
    "https://api.minecraftservices.com/authentication/login_with_xbox";
const MINECRAFT_ENTITLEMENTS_ENDPOINT: &str =
    "https://api.minecraftservices.com/entitlements/mcstore";
const MINECRAFT_PROFILE_ENDPOINT: &str = "https://api.minecraftservices.com/minecraft/profile";

const DEFAULT_MICROSOFT_SCOPE: &str = "XboxLive.signin offline_access";

#[derive(Clone, Debug)]
pub struct MicrosoftAuthConfig {
    pub client_id: String,
    pub redirect_uri: Url,
    pub scope: String,
}

impl MicrosoftAuthConfig {
    pub fn new(client_id: impl Into<String>, redirect_uri: Url) -> Self {
        Self {
            client_id: client_id.into(),
            redirect_uri,
            scope: DEFAULT_MICROSOFT_SCOPE.to_string(),
        }
    }

    pub fn with_scope(mut self, scope: impl Into<String>) -> Self {
        self.scope = scope.into();
        self
    }
}

#[derive(Clone, Debug)]
pub struct MicrosoftPkcePair {
    pub verifier: String,
    pub challenge: String,
}

pub fn generate_pkce_pair() -> MicrosoftPkcePair {
    let verifier = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
    let digest = Sha256::digest(verifier.as_bytes());
    let challenge = URL_SAFE_NO_PAD.encode(digest);

    MicrosoftPkcePair {
        verifier,
        challenge,
    }
}

#[derive(Clone, Debug)]
pub struct MicrosoftAuthClient {
    http: Client,
    config: MicrosoftAuthConfig,
}

impl MicrosoftAuthClient {
    pub fn new(config: MicrosoftAuthConfig) -> Self {
        Self {
            http: Client::new(),
            config,
        }
    }

    pub fn with_client(http: Client, config: MicrosoftAuthConfig) -> Self {
        Self { http, config }
    }

    pub fn authorization_url(
        &self,
        pkce: Option<&MicrosoftPkcePair>,
        state: Option<&str>,
    ) -> Result<Url> {
        build_authorization_url(&self.config, pkce, state)
    }

    pub async fn request_device_code(&self) -> Result<MicrosoftDeviceCodeResponse> {
        let response = self
            .http
            .post(MICROSOFT_DEVICE_CODE_ENDPOINT)
            .form(&MicrosoftDeviceCodeRequest::new(
                &self.config.client_id,
                &self.config.scope,
            ))
            .send()
            .await
            .context("failed to send Microsoft device code request")?;

        decode_json_response(response, "Microsoft device code response").await
    }

    pub async fn exchange_authorization_code(
        &self,
        code: &str,
        pkce_verifier: Option<&str>,
    ) -> Result<MicrosoftTokenResponse> {
        let response = self
            .http
            .post(MICROSOFT_TOKEN_ENDPOINT)
            .form(&MicrosoftAuthorizationCodeRequest::new(
                &self.config.client_id,
                &self.config.redirect_uri,
                &self.config.scope,
                code,
                pkce_verifier,
            ))
            .send()
            .await
            .context("failed to send Microsoft authorization code request")?;

        decode_json_response(response, "Microsoft token response").await
    }

    pub async fn exchange_refresh_token(
        &self,
        refresh_token: &str,
    ) -> Result<MicrosoftTokenResponse> {
        let response = self
            .http
            .post(MICROSOFT_TOKEN_ENDPOINT)
            .form(&MicrosoftRefreshTokenRequest::new(
                &self.config.client_id,
                &self.config.scope,
                refresh_token,
            ))
            .send()
            .await
            .context("failed to send Microsoft refresh token request")?;

        decode_json_response(response, "Microsoft token refresh response").await
    }

    pub async fn poll_device_code_token(
        &self,
        device_code: &MicrosoftDeviceCodeResponse,
    ) -> Result<MicrosoftTokenResponse> {
        let deadline = Instant::now() + Duration::from_secs(device_code.expires_in.max(1));
        let mut interval = Duration::from_secs(device_code.interval.max(1));

        loop {
            if Instant::now() >= deadline {
                return Err(anyhow!(
                    "Microsoft device code expired before authorization completed"
                ));
            }

            let response = self
                .http
                .post(MICROSOFT_TOKEN_ENDPOINT)
                .form(&MicrosoftDeviceCodeTokenRequest::new(
                    &self.config.client_id,
                    &device_code.device_code,
                    &self.config.scope,
                ))
                .send()
                .await
                .context("failed to send Microsoft device code polling request")?;

            let status = response.status();
            let bytes = response
                .bytes()
                .await
                .context("failed to read Microsoft device code polling response")?;

            if status.is_success() {
                let token = serde_json::from_slice::<MicrosoftTokenResponse>(&bytes)
                    .context("failed to parse Microsoft token response")?;
                return Ok(token);
            }

            let error = parse_microsoft_error(&bytes).unwrap_or_else(|| body_as_string(&bytes));
            if is_retryable_device_code_error(&error) {
                if error.contains("slow_down") {
                    interval = interval.saturating_add(Duration::from_secs(5));
                }
                sleep(interval).await;
                continue;
            }

            return Err(anyhow!(
                "Microsoft device code authorization failed: {error}"
            ));
        }
    }

    pub async fn authenticate_xbox_live(&self, access_token: &str) -> Result<XboxTokenResponse> {
        let response = self
            .http
            .post(XBOX_LIVE_AUTH_ENDPOINT)
            .json(&XboxLiveAuthRequest::new(access_token))
            .send()
            .await
            .context("failed to send Xbox Live authentication request")?;

        decode_json_response(response, "Xbox Live authentication response").await
    }

    pub async fn authorize_xsts(&self, xbox_token: &str) -> Result<XboxTokenResponse> {
        let response = self
            .http
            .post(XSTS_AUTH_ENDPOINT)
            .json(&XstsAuthRequest::new(xbox_token))
            .send()
            .await
            .context("failed to send XSTS authorization request")?;

        decode_json_response(response, "XSTS authorization response").await
    }

    pub async fn login_with_xbox(
        &self,
        user_hash: &str,
        xsts_token: &str,
    ) -> Result<MinecraftLoginResponse> {
        let response = self
            .http
            .post(MINECRAFT_LOGIN_WITH_XBOX_ENDPOINT)
            .json(&MinecraftLoginWithXboxRequest::new(user_hash, xsts_token))
            .send()
            .await
            .context("failed to send Minecraft login request")?;

        decode_json_response(response, "Minecraft login response").await
    }

    pub async fn fetch_entitlements(
        &self,
        minecraft_access_token: &str,
    ) -> Result<MinecraftEntitlementsResponse> {
        let response = self
            .http
            .get(MINECRAFT_ENTITLEMENTS_ENDPOINT)
            .bearer_auth(minecraft_access_token)
            .send()
            .await
            .context("failed to request Minecraft entitlements")?;

        decode_json_response(response, "Minecraft entitlements response").await
    }

    pub async fn fetch_profile(
        &self,
        minecraft_access_token: &str,
    ) -> Result<MinecraftProfileResponse> {
        let response = self
            .http
            .get(MINECRAFT_PROFILE_ENDPOINT)
            .bearer_auth(minecraft_access_token)
            .send()
            .await
            .context("failed to request Minecraft profile")?;

        decode_json_response(response, "Minecraft profile response").await
    }

    pub async fn authenticate_with_authorization_code(
        &self,
        code: &str,
        pkce_verifier: Option<&str>,
    ) -> Result<MicrosoftMinecraftSession> {
        let microsoft_token = self
            .exchange_authorization_code(code, pkce_verifier)
            .await
            .context("failed to exchange Microsoft authorization code")?;

        self.finish_minecraft_session(microsoft_token).await
    }

    pub async fn authenticate_with_refresh_token(
        &self,
        refresh_token: &str,
    ) -> Result<MicrosoftMinecraftSession> {
        let microsoft_token = self
            .exchange_refresh_token(refresh_token)
            .await
            .context("failed to refresh Microsoft access token")?;

        self.finish_minecraft_session(microsoft_token).await
    }

    pub async fn authenticate_with_device_code(
        &self,
        device_code: &MicrosoftDeviceCodeResponse,
    ) -> Result<MicrosoftMinecraftSession> {
        let microsoft_token = self
            .poll_device_code_token(device_code)
            .await
            .context("failed to complete Microsoft device code flow")?;

        self.finish_minecraft_session(microsoft_token).await
    }

    async fn finish_minecraft_session(
        &self,
        microsoft_token: MicrosoftTokenResponse,
    ) -> Result<MicrosoftMinecraftSession> {
        let xbox_live = self
            .authenticate_xbox_live(&microsoft_token.access_token)
            .await
            .context("failed to authenticate Xbox Live")?;

        let user_hash = xbox_live
            .user_hash()
            .context("Xbox Live response did not include a user hash")?;

        let xsts = self
            .authorize_xsts(&xbox_live.token)
            .await
            .context("failed to authorize XSTS")?;

        let minecraft_login = self
            .login_with_xbox(user_hash, &xsts.token)
            .await
            .context("failed to login with Xbox token")?;

        let entitlements = self
            .fetch_entitlements(&minecraft_login.access_token)
            .await
            .context("failed to fetch Minecraft entitlements")?;

        let profile = self
            .fetch_profile(&minecraft_login.access_token)
            .await
            .context("failed to fetch Minecraft profile")?;

        Ok(MicrosoftMinecraftSession {
            microsoft_token,
            xbox_live,
            xsts,
            minecraft_login,
            entitlements,
            profile,
        })
    }
}

pub fn build_authorization_url(
    config: &MicrosoftAuthConfig,
    pkce: Option<&MicrosoftPkcePair>,
    state: Option<&str>,
) -> Result<Url> {
    let mut url = Url::parse(MICROSOFT_AUTHORIZE_ENDPOINT)
        .context("failed to parse Microsoft authorization endpoint")?;
    {
        let mut pairs = url.query_pairs_mut();
        pairs.append_pair("client_id", &config.client_id);
        pairs.append_pair("response_type", "code");
        pairs.append_pair("response_mode", "query");
        pairs.append_pair("redirect_uri", config.redirect_uri.as_str());
        pairs.append_pair("scope", &config.scope);
        pairs.append_pair("prompt", "select_account");
        if let Some(pkce) = pkce {
            pairs.append_pair("code_challenge_method", "S256");
            pairs.append_pair("code_challenge", &pkce.challenge);
        }
        if let Some(state) = state {
            pairs.append_pair("state", state);
        }
    }
    Ok(url)
}

#[derive(Clone, Debug)]
pub struct MicrosoftMinecraftSession {
    pub microsoft_token: MicrosoftTokenResponse,
    pub xbox_live: XboxTokenResponse,
    pub xsts: XboxTokenResponse,
    pub minecraft_login: MinecraftLoginResponse,
    pub entitlements: MinecraftEntitlementsResponse,
    pub profile: MinecraftProfileResponse,
}

impl MicrosoftMinecraftSession {
    pub fn has_entitlement(&self) -> bool {
        self.entitlements.has_game_entitlement()
    }

    pub fn minecraft_access_token(&self) -> &str {
        &self.minecraft_login.access_token
    }

    pub fn refresh_token(&self) -> Option<&str> {
        self.microsoft_token.refresh_token.as_deref()
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct MicrosoftDeviceCodeResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    #[serde(default)]
    pub verification_uri_complete: Option<String>,
    pub expires_in: u64,
    pub interval: u64,
    pub message: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct MicrosoftTokenResponse {
    pub token_type: String,
    #[serde(default)]
    pub scope: Option<String>,
    pub expires_in: u64,
    pub access_token: String,
    #[serde(default)]
    pub refresh_token: Option<String>,
    #[serde(default)]
    pub id_token: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
struct MicrosoftErrorResponse {
    error: String,
    #[serde(default)]
    error_description: Option<String>,
    #[serde(default)]
    error_codes: Vec<u64>,
    #[serde(default)]
    message: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
struct MicrosoftDeviceCodeRequest<'a> {
    client_id: &'a str,
    scope: &'a str,
}

impl<'a> MicrosoftDeviceCodeRequest<'a> {
    fn new(client_id: &'a str, scope: &'a str) -> Self {
        Self { client_id, scope }
    }
}

#[derive(Clone, Debug, Serialize)]
struct MicrosoftAuthorizationCodeRequest<'a> {
    client_id: &'a str,
    grant_type: &'a str,
    code: &'a str,
    redirect_uri: &'a str,
    scope: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    code_verifier: Option<&'a str>,
}

impl<'a> MicrosoftAuthorizationCodeRequest<'a> {
    fn new(
        client_id: &'a str,
        redirect_uri: &'a Url,
        scope: &'a str,
        code: &'a str,
        code_verifier: Option<&'a str>,
    ) -> Self {
        Self {
            client_id,
            grant_type: "authorization_code",
            code,
            redirect_uri: redirect_uri.as_str(),
            scope,
            code_verifier,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
struct MicrosoftRefreshTokenRequest<'a> {
    client_id: &'a str,
    grant_type: &'a str,
    refresh_token: &'a str,
    scope: &'a str,
}

impl<'a> MicrosoftRefreshTokenRequest<'a> {
    fn new(client_id: &'a str, scope: &'a str, refresh_token: &'a str) -> Self {
        Self {
            client_id,
            grant_type: "refresh_token",
            refresh_token,
            scope,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
struct MicrosoftDeviceCodeTokenRequest<'a> {
    client_id: &'a str,
    grant_type: &'a str,
    device_code: &'a str,
    scope: &'a str,
}

impl<'a> MicrosoftDeviceCodeTokenRequest<'a> {
    fn new(client_id: &'a str, device_code: &'a str, scope: &'a str) -> Self {
        Self {
            client_id,
            grant_type: "urn:ietf:params:oauth:grant-type:device_code",
            device_code,
            scope,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
struct XboxLiveAuthRequest<'a> {
    properties: XboxLiveAuthProperties<'a>,
    relying_party: &'a str,
    token_type: &'a str,
}

impl<'a> XboxLiveAuthRequest<'a> {
    fn new(access_token: &'a str) -> Self {
        Self {
            properties: XboxLiveAuthProperties {
                auth_method: "RPS",
                site_name: "user.auth.xboxlive.com",
                rps_ticket: format!("d={access_token}"),
            },
            relying_party: "http://auth.xboxlive.com",
            token_type: "JWT",
        }
    }
}

#[derive(Clone, Debug, Serialize)]
struct XboxLiveAuthProperties<'a> {
    #[serde(rename = "AuthMethod")]
    auth_method: &'a str,
    #[serde(rename = "SiteName")]
    site_name: &'a str,
    #[serde(rename = "RpsTicket")]
    rps_ticket: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
struct XstsAuthRequest<'a> {
    properties: XstsAuthProperties<'a>,
    relying_party: &'a str,
    token_type: &'a str,
}

impl<'a> XstsAuthRequest<'a> {
    fn new(xbox_token: &'a str) -> Self {
        Self {
            properties: XstsAuthProperties {
                sandbox_id: "RETAIL",
                user_tokens: vec![xbox_token],
            },
            relying_party: "rp://api.minecraftservices.com/",
            token_type: "JWT",
        }
    }
}

#[derive(Clone, Debug, Serialize)]
struct XstsAuthProperties<'a> {
    #[serde(rename = "SandboxId")]
    sandbox_id: &'a str,
    #[serde(rename = "UserTokens")]
    user_tokens: Vec<&'a str>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct MinecraftLoginWithXboxRequest {
    identity_token: String,
}

impl MinecraftLoginWithXboxRequest {
    fn new(user_hash: &str, xsts_token: &str) -> Self {
        Self {
            identity_token: format!("XBL3.0 x={user_hash};{xsts_token}"),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct XboxTokenResponse {
    pub token: String,
    pub display_claims: XboxDisplayClaims,
}

impl XboxTokenResponse {
    pub fn user_hash(&self) -> Option<&str> {
        self.display_claims
            .xui
            .first()
            .map(|item| item.uhs.as_str())
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct XboxDisplayClaims {
    #[serde(rename = "xui")]
    pub xui: Vec<XboxDisplayClaim>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct XboxDisplayClaim {
    #[serde(rename = "uhs")]
    pub uhs: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct MinecraftLoginResponse {
    pub username: Option<String>,
    pub roles: Option<Vec<String>>,
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
}

#[derive(Clone, Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MinecraftEntitlementsResponse {
    #[serde(default)]
    pub items: Vec<MinecraftEntitlement>,
}

impl MinecraftEntitlementsResponse {
    pub fn has_game_entitlement(&self) -> bool {
        !self.items.is_empty()
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MinecraftEntitlement {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub signature: Option<String>,
    #[serde(default)]
    pub signature_public_key: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MinecraftProfileResponse {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub skins: Vec<serde_json::Value>,
    #[serde(default)]
    pub capes: Vec<serde_json::Value>,
}

async fn decode_json_response<T: DeserializeOwned>(
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
        let detail = parse_microsoft_error(&bytes).unwrap_or_else(|| body_as_string(&bytes));
        Err(anyhow!("{context} failed: {detail}"))
    }
}

fn parse_microsoft_error(bytes: &[u8]) -> Option<String> {
    let parsed = serde_json::from_slice::<MicrosoftErrorResponse>(bytes).ok()?;
    let mut parts = vec![parsed.error];
    if let Some(description) = parsed.error_description {
        if !description.is_empty() {
            parts.push(description);
        }
    }
    if let Some(message) = parsed.message {
        if !message.is_empty() {
            parts.push(message);
        }
    }
    if !parsed.error_codes.is_empty() {
        parts.push(format!("codes={:?}", parsed.error_codes));
    }
    Some(parts.join(": "))
}

fn body_as_string(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).trim().to_string()
}

fn is_retryable_device_code_error(error: &str) -> bool {
    error.contains("authorization_pending") || error.contains("slow_down")
}

#[cfg(test)]
mod tests {
    use super::{
        MicrosoftAuthConfig, build_authorization_url, generate_pkce_pair,
        is_retryable_device_code_error,
    };
    use reqwest::Url;

    #[test]
    fn builds_authorization_url_with_expected_query() {
        let config = MicrosoftAuthConfig::new(
            "client-123",
            Url::parse("http://127.0.0.1:7888/callback").unwrap(),
        );
        let pkce = generate_pkce_pair();
        let url = build_authorization_url(&config, Some(&pkce), Some("state-xyz")).unwrap();

        assert_eq!(url.scheme(), "https");
        assert_eq!(url.host_str(), Some("login.microsoftonline.com"));
        assert_eq!(url.path(), "/consumers/oauth2/v2.0/authorize");
        assert!(url.as_str().contains("client_id=client-123"));
        assert!(url.as_str().contains("response_type=code"));
        assert!(url.as_str().contains("response_mode=query"));
        assert!(
            url.as_str()
                .contains("redirect_uri=http%3A%2F%2F127.0.0.1%3A7888%2Fcallback")
        );
        assert!(url.as_str().contains("code_challenge_method=S256"));
        assert!(url.as_str().contains("state=state-xyz"));
    }

    #[test]
    fn pkce_pair_has_a_long_enough_verifier() {
        let pair = generate_pkce_pair();
        assert!(pair.verifier.len() >= 43);
        assert!(!pair.challenge.is_empty());
    }

    #[test]
    fn recognizes_retryable_device_code_errors() {
        assert!(is_retryable_device_code_error("authorization_pending"));
        assert!(is_retryable_device_code_error(
            "slow_down: please wait a bit"
        ));
        assert!(!is_retryable_device_code_error("access_denied"));
    }
}
