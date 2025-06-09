//! Event system interface for JavaScript

use wasm_bindgen::prelude::*;
use js_sys::*;
use std::collections::HashMap;

/// Event emitter interface for JavaScript
#[wasm_bindgen(js_name = "EventEmitter")]
pub struct JsEventEmitterInterface {
    listeners: HashMap<String, Vec<Function>>,
}

#[wasm_bindgen(js_class = "EventEmitter")]
impl JsEventEmitterInterface {
    /// Creates a new event emitter
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            listeners: HashMap::new(),
        }
    }
    
    /// Add event listener
    pub fn on(&mut self, event: &str, callback: &Function) {
        self.listeners
            .entry(event.to_string())
            .or_insert_with(Vec::new)
            .push(callback.clone());
    }
    
    /// Remove event listener
    pub fn off(&mut self, event: &str, callback: &Function) {
        if let Some(listeners) = self.listeners.get_mut(event) {
            listeners.retain(|listener| listener != callback);
        }
    }
    
    /// Emit event
    pub fn emit(&self, event: &str, data: JsValue) {
        if let Some(listeners) = self.listeners.get(event) {
            for listener in listeners {
                let _ = listener.call1(&JsValue::null(), &data);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_event_emitter_creation() {
        let emitter = JsEventEmitterInterface::new();
        assert_eq!(emitter.listeners.len(), 0);
    }
}