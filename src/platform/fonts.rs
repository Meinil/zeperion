use std::{collections::BTreeSet, sync::OnceLock};

use font_kit::source::SystemSource;

pub fn system_font_families() -> Vec<String> {
    static CACHE: OnceLock<Vec<String>> = OnceLock::new();

    CACHE
        .get_or_init(|| {
            let source = SystemSource::new();
            let mut families = BTreeSet::new();

            if let Ok(all) = source.all_families() {
                for family in all {
                    let trimmed = family.trim();
                    if !trimmed.is_empty() {
                        families.insert(trimmed.to_string());
                    }
                }
            }

            if families.is_empty() {
                families.insert("Nunito".to_string());
                families.insert("Segoe UI".to_string());
                families.insert("PingFang SC".to_string());
            }

            families.into_iter().collect()
        })
        .clone()
}
