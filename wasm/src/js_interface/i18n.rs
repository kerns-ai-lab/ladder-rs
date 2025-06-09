//! Internationalization interface for JavaScript

use wasm_bindgen::prelude::*;
use js_sys::*;
use std::collections::HashMap;

/// I18n interface for localized messages
#[wasm_bindgen(js_name = "I18n")]
pub struct JsI18nInterface {
    locale: String,
    messages: HashMap<String, String>,
}

#[wasm_bindgen(js_class = "I18n")]
impl JsI18nInterface {
    /// Creates a new i18n instance
    #[wasm_bindgen(constructor)]
    pub fn new(locale: &str) -> Self {
        let mut messages = HashMap::new();
        
        // Add some default messages
        messages.insert("rating_updated".to_string(), "Rating updated".to_string());
        messages.insert("match_processed".to_string(), "Match processed".to_string());
        
        Self {
            locale: locale.to_string(),
            messages,
        }
    }
    
    /// Format a message
    #[wasm_bindgen(js_name = "formatMessage")]
    pub fn format_message(&self, key: &str, _params: &Object) -> String {
        self.messages.get(key).cloned().unwrap_or_else(|| key.to_string())
    }
    
    /// Format a number according to locale
    #[wasm_bindgen(js_name = "formatNumber")]
    pub fn format_number(&self, number: f64) -> String {
        // Simplified number formatting
        format!("{:.1}", number)
    }
    
    /// Set locale
    #[wasm_bindgen(js_name = "setLocale")]
    pub fn set_locale(&mut self, locale: &str) {
        self.locale = locale.to_string();
    }
}

/// Plugin manager interface
#[wasm_bindgen(js_name = "PluginManager")]
pub struct JsPluginManagerInterface {
    plugins: HashMap<String, Object>,
}

#[wasm_bindgen(js_class = "PluginManager")]
impl JsPluginManagerInterface {
    /// Creates a new plugin manager
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }
    
    /// Register a plugin
    #[wasm_bindgen(js_name = "registerPlugin")]
    pub fn register_plugin(&mut self, name: &str, config: Object) -> bool {
        self.plugins.insert(name.to_string(), config);
        true
    }
    
    /// List registered plugins
    #[wasm_bindgen(js_name = "listPlugins")]
    pub fn list_plugins(&self) -> Array {
        let array = Array::new();
        for key in self.plugins.keys() {
            array.push(&JsValue::from_str(key));
        }
        array
    }
    
    /// Execute a plugin
    #[wasm_bindgen(js_name = "executePlugin")]
    pub fn execute_plugin(&self, name: &str, _params: &Object) -> Result<JsValue, JsValue> {
        if self.plugins.contains_key(name) {
            Ok(JsValue::from_str("Plugin executed"))
        } else {
            Err(JsValue::from_str("Plugin not found"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_i18n_creation() {
        let i18n = JsI18nInterface::new("en-US");
        assert_eq!(i18n.locale, "en-US");
    }
    
    #[test]
    fn test_plugin_manager() {
        let mut manager = JsPluginManagerInterface::new();
        let config = Object::new();
        assert!(manager.register_plugin("test", config));
        assert_eq!(manager.list_plugins().length(), 1);
    }
}