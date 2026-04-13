use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::RwLock;

pub mod keys;

pub use keys::I18nKey;

static CURRENT_LOCALE: Lazy<RwLock<String>> = Lazy::new(|| RwLock::new("en".to_string()));

pub static SUPPORTED_LOCALES: &[&str] = &["en", "de", "es", "fr", "ru", "zh"];

pub fn set_locale(locale: &str) {
    let normalized = normalize_locale(locale);
    let mut current = CURRENT_LOCALE.write().unwrap();
    *current = normalized.clone();
    rust_i18n::set_locale(&normalized);
}

pub fn get_locale() -> String {
    CURRENT_LOCALE.read().unwrap().clone()
}

pub fn normalize_locale(locale: &str) -> String {
    let normalized = locale.to_lowercase();
    
    // Check exact match first
    if SUPPORTED_LOCALES.contains(&normalized.as_str()) {
        return normalized;
    }
    
    // Check base language (e.g., "en-US" -> "en")
    let base = normalized.split('-').next().unwrap_or(&normalized);
    if SUPPORTED_LOCALES.contains(&base) {
        return base.to_string();
    }
    
    // Fallback to English
    "en".to_string()
}

pub fn is_supported_locale(locale: &str) -> bool {
    let normalized = locale.to_lowercase();
    SUPPORTED_LOCALES.contains(&normalized.as_str()) || 
    SUPPORTED_LOCALES.contains(&normalized.split('-').next().unwrap_or(""))
}

pub fn get_locale_options() -> Vec<(&'static str, &'static str)> {
    vec![
        ("en", "English"),
        ("de", "Deutsch"),
        ("es", "Español"),
        ("fr", "Français"),
        ("ru", "Русский"),
        ("zh", "简体中文"),
    ]
}

/// Translate a key with optional parameters
pub fn t(key: I18nKey, params: Option<HashMap<&str, String>>) -> String {
    let locale = get_locale();
    let raw = translate_key(&locale, key.as_str());
    
    match params {
        Some(p) => interpolate(&raw, &p),
        None => raw,
    }
}

fn translate_key(locale: &str, key: &str) -> String {
    // Use rust_i18n's t! macro
    let val = rust_i18n::t!(key, locale = locale, fallback = true);
    
    let val_str = val.to_string();
    
    // If not found and not already using fallback, try English
    if val_str == key && locale != "en" {
        rust_i18n::t!(key, locale = "en", fallback = true).to_string()
    } else {
        val_str
    }
}

fn interpolate(template: &str, params: &HashMap<&str, String>) -> String {
    let mut result = template.to_string();
    
    for (key, value) in params {
        let placeholder = format!("{{{}}}", key);
        result = result.replace(&placeholder, value);
    }
    
    result
}

/// Convenience macro for translation with parameters
#[macro_export]
macro_rules! tr {
    ($key:expr) => {
        $crate::i18n::t($key, None)
    };
    ($key:expr, $($param:expr => $value:expr),+ $(,)?) => {
        {
            let mut params = std::collections::HashMap::new();
            $(
                params.insert($param, $value.to_string());
            )+
            $crate::i18n::t($key, Some(params))
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_locale() {
        assert_eq!(normalize_locale("en"), "en");
        assert_eq!(normalize_locale("EN"), "en");
        assert_eq!(normalize_locale("en-US"), "en");
        assert_eq!(normalize_locale("ru-RU"), "ru");
        assert_eq!(normalize_locale("unknown"), "en");
        assert_eq!(normalize_locale("zh-CN"), "zh");
    }

    #[test]
    fn test_is_supported_locale() {
        assert!(is_supported_locale("en"));
        assert!(is_supported_locale("EN"));
        assert!(is_supported_locale("en-US"));
        assert!(is_supported_locale("ru"));
        assert!(!is_supported_locale("unknown"));
        assert!(!is_supported_locale("jp"));
    }

    #[test]
    fn test_set_and_get_locale() {
        set_locale("en");
        assert_eq!(get_locale(), "en");
        
        set_locale("ru");
        assert_eq!(get_locale(), "ru");
        
        // Test fallback
        set_locale("unknown");
        assert_eq!(get_locale(), "en");
    }

    #[test]
    fn test_locale_options() {
        let options = get_locale_options();
        assert_eq!(options.len(), 6);
        assert!(options.iter().any(|(code, _)| *code == "en"));
        assert!(options.iter().any(|(code, _)| *code == "ru"));
    }
}
