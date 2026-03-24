use serde::{Deserialize, Serialize};

pub const OFFICIAL_MIRROR_URL: &str = "https://piston-meta.mojang.com/";
pub const BMCLAPI_MIRROR_URL: &str = "https://bmclapi2.bangbang93.com/";
pub const LEGACY_BMCLAPI_DOC_URL: &str = "https://bmclapidoc.bangbang93.com/";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum MirrorType {
    OfficialCompatible,
    BmclApiCompatible,
}

impl MirrorType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OfficialCompatible => "official_compatible",
            Self::BmclApiCompatible => "bmclapi_compatible",
        }
    }

    pub fn from_str(value: &str) -> Self {
        match value {
            "bmclapi_compatible" => Self::BmclApiCompatible,
            _ => Self::OfficialCompatible,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MirrorSource {
    pub id: String,
    pub name: String,
    pub base_url: String,
    pub is_builtin: bool,
    pub mirror_type: MirrorType,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AppSettings {
    pub language: String,
    pub theme_color: String,
    pub font_family: String,
    pub font_size: u16,
    pub game_memory_mb: u32,
    pub resolution_width: u16,
    pub resolution_height: u16,
    pub selected_mirror: String,
    pub rotate_backgrounds: bool,
    pub microsoft_client_id: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            language: "zh-CN".to_string(),
            theme_color: "#F97316".to_string(),
            font_family: "Nunito".to_string(),
            font_size: 15,
            game_memory_mb: 4096,
            resolution_width: 1280,
            resolution_height: 720,
            selected_mirror: OFFICIAL_MIRROR_URL.to_string(),
            rotate_backgrounds: false,
            microsoft_client_id: String::new(),
        }
    }
}

pub fn normalize_selected_mirror_url(value: &str) -> String {
    match value.trim() {
        LEGACY_BMCLAPI_DOC_URL => BMCLAPI_MIRROR_URL.to_string(),
        "" => OFFICIAL_MIRROR_URL.to_string(),
        other => {
            if other.ends_with('/') {
                other.to_string()
            } else {
                format!("{other}/")
            }
        }
    }
}
