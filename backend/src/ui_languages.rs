pub const SUPPORTED_UI_LANGUAGES: &[&str] = &[
    "en", "de", "es", "fr", "it", "nl", "pt", "sv", "no", "da", "fi", "et", "zh", "ar", "ur",
    "tar", "apw", "lkt", "chr", "apm", "yua", "zap",
];

pub fn normalize_ui_language(value: &str) -> Option<String> {
    let raw = value.trim().to_lowercase();
    if raw.is_empty() {
        return None;
    }
    let base = raw.split('-').next().unwrap_or(&raw);
    if SUPPORTED_UI_LANGUAGES.contains(&base) {
        Some(base.to_string())
    } else {
        None
    }
}
