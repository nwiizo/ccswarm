//! Internationalization (i18n) support for Piece/Movement workflows.
//!
//! Enables localized prompts, messages, and output for different languages.
//! Inspired by takt's multilingual support.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::debug;

/// Supported languages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    #[default]
    En,
    Ja,
    Zh,
    Ko,
    Es,
    Fr,
    De,
    Pt,
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::En => write!(f, "en"),
            Self::Ja => write!(f, "ja"),
            Self::Zh => write!(f, "zh"),
            Self::Ko => write!(f, "ko"),
            Self::Es => write!(f, "es"),
            Self::Fr => write!(f, "fr"),
            Self::De => write!(f, "de"),
            Self::Pt => write!(f, "pt"),
        }
    }
}

impl Language {
    /// Parse from string
    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "en" | "english" => Some(Self::En),
            "ja" | "japanese" => Some(Self::Ja),
            "zh" | "chinese" => Some(Self::Zh),
            "ko" | "korean" => Some(Self::Ko),
            "es" | "spanish" => Some(Self::Es),
            "fr" | "french" => Some(Self::Fr),
            "de" | "german" => Some(Self::De),
            "pt" | "portuguese" => Some(Self::Pt),
            _ => None,
        }
    }

    /// All available languages
    pub fn all() -> Vec<Self> {
        vec![
            Self::En,
            Self::Ja,
            Self::Zh,
            Self::Ko,
            Self::Es,
            Self::Fr,
            Self::De,
            Self::Pt,
        ]
    }
}

/// A localized string bundle for a specific context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocaleBundle {
    /// Language for this bundle
    pub language: Language,
    /// Key-value pairs of localized strings
    pub strings: HashMap<String, String>,
}

/// i18n manager for workflow localization
#[derive(Debug, Clone, Default)]
pub struct I18nManager {
    /// Locale bundles by language
    bundles: HashMap<Language, LocaleBundle>,
    /// Current active language
    active_language: Language,
    /// Fallback language
    fallback_language: Language,
}

impl I18nManager {
    pub fn new() -> Self {
        let mut manager = Self {
            bundles: HashMap::new(),
            active_language: Language::En,
            fallback_language: Language::En,
        };
        // Load built-in English bundle
        manager.load_builtin_en();
        manager
    }

    /// Set the active language
    pub fn set_language(&mut self, lang: Language) {
        self.active_language = lang;
        debug!("Active language set to: {}", lang);
    }

    /// Get the active language
    pub fn active_language(&self) -> Language {
        self.active_language
    }

    /// Register a locale bundle
    pub fn register_bundle(&mut self, bundle: LocaleBundle) {
        self.bundles.insert(bundle.language, bundle);
    }

    /// Get a localized string by key. Returns the key itself if not found.
    pub fn get<'a>(&'a self, key: &'a str) -> &'a str {
        // Try active language first
        if let Some(bundle) = self.bundles.get(&self.active_language)
            && let Some(value) = bundle.strings.get(key)
        {
            return value;
        }

        // Fall back to fallback language
        if self.active_language != self.fallback_language
            && let Some(bundle) = self.bundles.get(&self.fallback_language)
            && let Some(value) = bundle.strings.get(key)
        {
            return value;
        }

        // Return the key itself as last resort
        key
    }

    /// Get a localized string with variable substitution.
    /// Variables in the format `{name}` are replaced.
    pub fn format(&self, key: &str, vars: &HashMap<String, String>) -> String {
        let mut result = self.get(key).to_string();
        for (name, value) in vars {
            result = result.replace(&format!("{{{}}}", name), value);
        }
        result
    }

    /// Generate a language instruction for the AI agent
    pub fn agent_language_instruction(&self) -> String {
        match self.active_language {
            Language::En => "Respond in English.".to_string(),
            Language::Ja => "日本語で回答してください。".to_string(),
            Language::Zh => "请用中文回答。".to_string(),
            Language::Ko => "한국어로 답변해 주세요.".to_string(),
            Language::Es => "Responde en español.".to_string(),
            Language::Fr => "Répondez en français.".to_string(),
            Language::De => "Antworten Sie auf Deutsch.".to_string(),
            Language::Pt => "Responda em português.".to_string(),
        }
    }

    /// Load built-in English strings
    fn load_builtin_en(&mut self) {
        let mut strings = HashMap::new();

        // Workflow messages
        strings.insert(
            "workflow.starting".to_string(),
            "Starting workflow: {name}".to_string(),
        );
        strings.insert(
            "workflow.completed".to_string(),
            "Workflow completed successfully".to_string(),
        );
        strings.insert(
            "workflow.failed".to_string(),
            "Workflow failed: {error}".to_string(),
        );
        strings.insert(
            "workflow.aborted".to_string(),
            "Workflow aborted: exceeded maximum movements".to_string(),
        );

        // Movement messages
        strings.insert(
            "movement.executing".to_string(),
            "Executing movement: {name} (#{count})".to_string(),
        );
        strings.insert(
            "movement.completed".to_string(),
            "Movement '{name}' completed".to_string(),
        );
        strings.insert(
            "movement.failed".to_string(),
            "Movement '{name}' failed: {error}".to_string(),
        );

        // Interactive messages
        strings.insert(
            "interactive.welcome".to_string(),
            "Welcome to ccswarm interactive mode".to_string(),
        );
        strings.insert(
            "interactive.select_piece".to_string(),
            "Select a piece to execute:".to_string(),
        );
        strings.insert(
            "interactive.select_mode".to_string(),
            "Select interactive mode:".to_string(),
        );
        strings.insert(
            "interactive.ready".to_string(),
            "Ready to execute. Type /go to start.".to_string(),
        );

        // Watch messages
        strings.insert(
            "watch.started".to_string(),
            "Watch mode started. Monitoring for changes...".to_string(),
        );
        strings.insert(
            "watch.change_detected".to_string(),
            "Change detected: {count} file(s) modified".to_string(),
        );
        strings.insert("watch.paused".to_string(), "Watch mode paused".to_string());

        // Permission messages
        strings.insert(
            "permission.denied".to_string(),
            "Permission denied: {action} requires {level} access".to_string(),
        );

        self.bundles.insert(
            Language::En,
            LocaleBundle {
                language: Language::En,
                strings,
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_display() {
        assert_eq!(Language::En.to_string(), "en");
        assert_eq!(Language::Ja.to_string(), "ja");
    }

    #[test]
    fn test_language_parse() {
        assert_eq!(Language::from_str_opt("en"), Some(Language::En));
        assert_eq!(Language::from_str_opt("japanese"), Some(Language::Ja));
        assert_eq!(Language::from_str_opt("unknown"), None);
    }

    #[test]
    fn test_get_string() {
        let manager = I18nManager::new();
        assert_eq!(
            manager.get("workflow.completed"),
            "Workflow completed successfully"
        );
    }

    #[test]
    fn test_format_with_vars() {
        let manager = I18nManager::new();
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "my-workflow".to_string());

        let result = manager.format("workflow.starting", &vars);
        assert_eq!(result, "Starting workflow: my-workflow");
    }

    #[test]
    fn test_missing_key_returns_key() {
        let manager = I18nManager::new();
        assert_eq!(manager.get("nonexistent.key"), "nonexistent.key");
    }

    #[test]
    fn test_agent_language_instruction() {
        let mut manager = I18nManager::new();

        manager.set_language(Language::En);
        assert!(manager.agent_language_instruction().contains("English"));

        manager.set_language(Language::Ja);
        assert!(manager.agent_language_instruction().contains("日本語"));
    }

    #[test]
    fn test_custom_bundle() {
        let mut manager = I18nManager::new();

        let mut strings = HashMap::new();
        strings.insert(
            "workflow.completed".to_string(),
            "ワークフローが完了しました".to_string(),
        );

        manager.register_bundle(LocaleBundle {
            language: Language::Ja,
            strings,
        });

        manager.set_language(Language::Ja);
        assert_eq!(
            manager.get("workflow.completed"),
            "ワークフローが完了しました"
        );

        // Falls back to English for missing keys
        assert_eq!(
            manager.get("workflow.aborted"),
            "Workflow aborted: exceeded maximum movements"
        );
    }

    #[test]
    fn test_all_languages() {
        let langs = Language::all();
        assert_eq!(langs.len(), 8);
    }
}
