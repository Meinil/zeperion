use std::{collections::HashMap, fs, path::PathBuf};

use anyhow::Result;

#[derive(Clone, Debug)]
pub struct I18n {
    bundles: HashMap<String, HashMap<String, String>>,
    fallback_locale: String,
}

impl I18n {
    pub fn load(default_locale: &str) -> Result<Self> {
        let mut bundles = HashMap::new();
        for locale in ["zh-CN", "en-US"] {
            let path = PathBuf::from("locales").join(format!("{locale}.json"));
            let content = fs::read_to_string(path)?;
            let bundle: HashMap<String, String> = serde_json::from_str(&content)?;
            bundles.insert(locale.to_string(), bundle);
        }

        Ok(Self {
            bundles,
            fallback_locale: default_locale.to_string(),
        })
    }

    pub fn translate(&self, locale: &str, key: &str) -> String {
        self.bundles
            .get(locale)
            .and_then(|bundle| bundle.get(key))
            .or_else(|| {
                self.bundles
                    .get(&self.fallback_locale)
                    .and_then(|bundle| bundle.get(key))
            })
            .cloned()
            .unwrap_or_else(|| key.to_string())
    }
}
