use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const STEVE_DEFAULT_REFERENCE: &str = "steve-default";
const STEVE_BOLD_REFERENCE: &str = "steve-bold";
const OFFLINE_SKIN_PRESETS: [&str; 18] = [
    "alex-slim",
    "ari-slim",
    "efe-slim",
    "kai-slim",
    "makena-slim",
    "noor-slim",
    "steve-slim",
    "sunny-slim",
    "zuri-slim",
    "alex-bold",
    "ari-bold",
    "efe-bold",
    "kai-bold",
    "makena-bold",
    "noor-bold",
    "steve-bold",
    "sunny-bold",
    "zuri-bold",
];

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum LoginType {
    Offline,
    Microsoft,
    ThirdParty,
}

impl LoginType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Offline => "offline",
            Self::Microsoft => "microsoft",
            Self::ThirdParty => "third_party",
        }
    }

    pub fn from_str(value: &str) -> Self {
        match value {
            "microsoft" => Self::Microsoft,
            "third_party" => Self::ThirdParty,
            _ => Self::Offline,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum OAuthFlow {
    AuthorizationCode,
    DeviceCode,
}

impl OAuthFlow {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::AuthorizationCode => "authorization_code",
            Self::DeviceCode => "device_code",
        }
    }

    pub fn from_optional(value: Option<&str>) -> Option<Self> {
        match value {
            Some("authorization_code") => Some(Self::AuthorizationCode),
            Some("device_code") => Some(Self::DeviceCode),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum SkinMode {
    Default,
    OfflinePreset,
    PremiumLookup,
    ThirdPartyProfile,
}

impl SkinMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::OfflinePreset => "offline_preset",
            Self::PremiumLookup => "premium_lookup",
            Self::ThirdPartyProfile => "third_party_profile",
        }
    }

    pub fn from_optional(value: Option<&str>) -> Option<Self> {
        match value {
            Some("default") => Some(Self::Default),
            Some("offline_preset") => Some(Self::OfflinePreset),
            Some("premium_lookup") => Some(Self::PremiumLookup),
            Some("third_party_profile") => Some(Self::ThirdPartyProfile),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Account {
    pub id: String,
    pub username: String,
    pub uuid: String,
    pub login_type: LoginType,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub client_token: Option<String>,
    pub auth_server_url: Option<String>,
    pub authlib_metadata_b64: Option<String>,
    pub oauth_flow: Option<OAuthFlow>,
    pub skin_mode: Option<SkinMode>,
    pub skin_reference: Option<String>,
    pub avatar_url: Option<String>,
    pub selected: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_validated_at: Option<DateTime<Utc>>,
}

impl Account {
    pub fn offline(username: String) -> Self {
        Self::offline_with_options(username, None, Some("random".to_string()))
    }

    pub fn offline_with_options(
        username: String,
        provided_uuid: Option<String>,
        skin_reference: Option<String>,
    ) -> Self {
        let reference = skin_reference
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| STEVE_DEFAULT_REFERENCE.to_string());
        let skin_mode = if reference == STEVE_DEFAULT_REFERENCE {
            Some(SkinMode::Default)
        } else {
            Some(SkinMode::OfflinePreset)
        };
        let uuid = provided_uuid
            .as_deref()
            .and_then(normalize_uuid)
            .unwrap_or_else(|| generate_offline_uuid_for_skin_reference(&reference));
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            username,
            uuid,
            login_type: LoginType::Offline,
            access_token: None,
            refresh_token: None,
            client_token: None,
            auth_server_url: None,
            authlib_metadata_b64: None,
            oauth_flow: None,
            skin_mode,
            skin_reference: Some(reference),
            avatar_url: None,
            selected: false,
            created_at: now,
            updated_at: now,
            last_validated_at: None,
        }
    }

    pub fn microsoft(
        username: String,
        uuid: String,
        access_token: String,
        refresh_token: String,
        oauth_flow: OAuthFlow,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            username,
            uuid: normalize_uuid(&uuid).unwrap_or_else(|| Uuid::new_v4().simple().to_string()),
            login_type: LoginType::Microsoft,
            access_token: Some(access_token),
            refresh_token: Some(refresh_token),
            client_token: None,
            auth_server_url: None,
            authlib_metadata_b64: None,
            oauth_flow: Some(oauth_flow),
            skin_mode: None,
            skin_reference: None,
            avatar_url: None,
            selected: false,
            created_at: now,
            updated_at: now,
            last_validated_at: Some(now),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn third_party(
        username: String,
        uuid: String,
        access_token: String,
        client_token: String,
        auth_server_url: String,
        authlib_metadata_b64: Option<String>,
        skin_reference: Option<String>,
        avatar_url: Option<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            username,
            uuid: normalize_uuid(&uuid).unwrap_or_else(|| Uuid::new_v4().simple().to_string()),
            login_type: LoginType::ThirdParty,
            access_token: Some(access_token),
            refresh_token: None,
            client_token: Some(client_token),
            auth_server_url: Some(auth_server_url),
            authlib_metadata_b64,
            oauth_flow: None,
            skin_mode: Some(SkinMode::ThirdPartyProfile),
            skin_reference,
            avatar_url,
            selected: false,
            created_at: now,
            updated_at: now,
            last_validated_at: Some(now),
        }
    }

    pub fn launch_user_type(&self) -> &'static str {
        match self.login_type {
            LoginType::Offline => "Legacy",
            LoginType::Microsoft | LoginType::ThirdParty => "msa",
        }
    }

    pub fn launch_access_token(&self) -> String {
        match self.login_type {
            LoginType::Offline => self.uuid.clone(),
            LoginType::Microsoft | LoginType::ThirdParty => self
                .access_token
                .clone()
                .unwrap_or_else(|| self.uuid.clone()),
        }
    }

    pub fn skin_mode_str(&self) -> Option<&'static str> {
        self.skin_mode.as_ref().map(SkinMode::as_str)
    }

    pub fn oauth_flow_str(&self) -> Option<&'static str> {
        self.oauth_flow.as_ref().map(OAuthFlow::as_str)
    }
}

pub fn offline_skin_presets() -> &'static [&'static str] {
    &OFFLINE_SKIN_PRESETS
}

pub fn default_offline_skin_reference() -> &'static str {
    STEVE_DEFAULT_REFERENCE
}

pub fn normalize_uuid(value: &str) -> Option<String> {
    let normalized = value.trim().replace('-', "").to_lowercase();
    if normalized.len() == 32 && normalized.chars().all(|ch| ch.is_ascii_hexdigit()) {
        Some(normalized)
    } else {
        None
    }
}

pub fn offline_skin_slot_for_uuid(uuid: &str) -> usize {
    let digest = md5::compute(normalize_uuid(uuid).unwrap_or_else(|| uuid.to_lowercase()));
    let value = u32::from_be_bytes([digest.0[0], digest.0[1], digest.0[2], digest.0[3]]);
    (value as usize) % OFFLINE_SKIN_PRESETS.len()
}

pub fn offline_skin_reference_for_uuid(uuid: &str) -> &'static str {
    OFFLINE_SKIN_PRESETS[offline_skin_slot_for_uuid(uuid)]
}

pub fn generate_offline_uuid_for_skin_reference(reference: &str) -> String {
    let target_reference = if reference == STEVE_DEFAULT_REFERENCE {
        STEVE_BOLD_REFERENCE
    } else {
        reference
    };
    let Some(target_index) = OFFLINE_SKIN_PRESETS
        .iter()
        .position(|value| *value == target_reference)
    else {
        return Uuid::new_v4().simple().to_string();
    };

    for _ in 0..4_096 {
        let candidate = Uuid::new_v4().simple().to_string();
        if offline_skin_slot_for_uuid(&candidate) == target_index {
            return candidate;
        }
    }

    Uuid::new_v4().simple().to_string()
}

pub fn generate_unique_offline_uuid_for_skin_reference<F>(
    reference: &str,
    mut is_taken: F,
) -> String
where
    F: FnMut(&str) -> bool,
{
    for _ in 0..4_096 {
        let candidate = generate_offline_uuid_for_skin_reference(reference);
        if !is_taken(&candidate) {
            return candidate;
        }
    }

    loop {
        let candidate = Uuid::new_v4().simple().to_string();
        if !is_taken(&candidate) {
            return candidate;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        generate_offline_uuid_for_skin_reference, generate_unique_offline_uuid_for_skin_reference,
        normalize_uuid, offline_skin_reference_for_uuid,
    };

    #[test]
    fn normalizes_uuid_strings() {
        assert_eq!(
            normalize_uuid("550E8400-E29B-41D4-A716-446655440000"),
            Some("550e8400e29b41d4a716446655440000".to_string())
        );
    }

    #[test]
    fn default_offline_account_prefers_random_reference() {
        let account = super::Account::offline("player".to_string());
        assert_eq!(account.skin_reference.as_deref(), Some("random"));
    }

    #[test]
    fn generates_uuid_for_requested_skin_reference() {
        let uuid = generate_offline_uuid_for_skin_reference("steve-bold");
        assert_eq!(offline_skin_reference_for_uuid(&uuid), "steve-bold");
    }

    #[test]
    fn regenerates_until_uuid_is_unique() {
        let uuid = generate_unique_offline_uuid_for_skin_reference("steve-bold", |candidate| {
            candidate == "taken"
        });
        assert_ne!(uuid, "taken");
    }
}
