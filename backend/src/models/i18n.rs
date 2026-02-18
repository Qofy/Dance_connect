use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TranslationOverrides {
    pub language: String,
    pub entries: HashMap<String, String>,
    pub updated_at: String,
    pub updated_by: Option<String>,
}

impl TranslationOverrides {
    pub fn empty(language: &str, updated_at: String) -> Self {
        Self {
            language: language.to_string(),
            entries: HashMap::new(),
            updated_at,
            updated_by: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MissingTranslationStat {
    pub language: String,
    pub key: String,
    pub count: u64,
    pub last_seen: String,
    pub last_context: Option<String>,
}

impl MissingTranslationStat {
    pub fn new(language: String, key: String, context: Option<String>, now: String) -> Self {
        Self {
            language,
            key,
            count: 1,
            last_seen: now,
            last_context: context,
        }
    }

    pub fn bump(&mut self, context: Option<String>, now: String) {
        self.count = self.count.saturating_add(1);
        self.last_seen = now;
        if context.is_some() {
            self.last_context = context;
        }
    }
}
