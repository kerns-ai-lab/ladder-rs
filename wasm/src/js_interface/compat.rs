//! Browser compatibility interface for JavaScript

use wasm_bindgen::prelude::*;
use js_sys::*;

/// Browser compatibility checker
#[wasm_bindgen(js_name = "BrowserCompat")]
pub struct JsBrowserCompatInterface;

#[wasm_bindgen(js_class = "BrowserCompat")]
impl JsBrowserCompatInterface {
    /// Creates a new compatibility checker
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self
    }
    
    /// Check WebAssembly support
    #[wasm_bindgen(js_name = "supportsWebAssembly")]
    pub fn supports_webassembly(&self) -> bool {
        true  // If we're running, WASM is supported
    }
    
    /// Check SharedArrayBuffer support
    #[wasm_bindgen(js_name = "supportsSharedArrayBuffer")]
    pub fn supports_shared_array_buffer(&self) -> bool {
        js_sys::eval("typeof SharedArrayBuffer !== 'undefined'").unwrap().as_bool().unwrap_or(false)
    }
    
    /// Check BigInt support
    #[wasm_bindgen(js_name = "supportsBigInt")]
    pub fn supports_bigint(&self) -> bool {
        js_sys::eval("typeof BigInt !== 'undefined'").unwrap().as_bool().unwrap_or(false)
    }
    
    /// Get feature matrix
    #[wasm_bindgen(js_name = "getFeatureMatrix")]
    pub fn get_feature_matrix(&self) -> Object {
        let features = Object::new();
        Reflect::set(&features, &"webassembly".into(), &self.supports_webassembly().into()).unwrap();
        Reflect::set(&features, &"sharedArrayBuffer".into(), &self.supports_shared_array_buffer().into()).unwrap();
        Reflect::set(&features, &"bigint".into(), &self.supports_bigint().into()).unwrap();
        features
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_wasm_support() {
        let compat = JsBrowserCompatInterface::new();
        assert!(compat.supports_webassembly());
    }
}