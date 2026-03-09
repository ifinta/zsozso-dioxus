use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::i18n::Language;
use super::{Db, GunConfig, GunValue};
use super::i18n::{DbI18n, db_i18n};
use super::sea::SeaKeyPair;

/// GUN-compatible graph database.
///
/// Delegates every operation to the real GUN.js
/// library running in the browser (via `window.__gun_bridge`).
pub struct GunDb {
    config: GunConfig,
    language: Language,
    next_sub_id: AtomicU64,
}

// ---------------------------------------------------------------------------
// Constructor (shared)
// ---------------------------------------------------------------------------
impl GunDb {
    /// Create a new GunDb instance.
    ///
    /// On WASM this also calls `window.__gun_bridge.init(peers)` so that the
    /// JS GUN instance is ready before the first `get`/`put`.
    pub fn new(config: GunConfig, language: Language) -> Self {
        // Serialise peer list and call JS init
        let peers_json = serde_json::to_string(&config.peers).unwrap_or_else(|_| "[]".into());
        let init_js = format!("window.__gun_bridge.init('{}')", peers_json);
        let _ = js_sys::eval(&init_js);

        Self {
            config,
            language,
            next_sub_id: AtomicU64::new(1),
        }
    }
}

// ===========================================================================
// WASM implementation — delegates to gun_bridge.js
// ===========================================================================
impl Db for GunDb {
    async fn get(&self, path: &[&str]) -> Result<Option<GunValue>, String> {
        use wasm_bindgen::JsCast;
        use wasm_bindgen_futures::JsFuture;

        let i18n = db_i18n(self.language);
        let path_json = serde_json::to_string(path)
            .map_err(|e| i18n.read_error(&e.to_string()))?;

        // window.__gun_bridge.get(pathJson) returns a Promise<string>
        let js_code = format!("window.__gun_bridge.get('{}')", path_json);
        let promise = js_sys::eval(&js_code)
            .map_err(|_| i18n.read_error("eval failed"))?;

        let promise = js_sys::Promise::from(promise.unchecked_into::<js_sys::Promise>());
        let result = JsFuture::from(promise).await
            .map_err(|_| i18n.read_error("Promise rejected"))?;

        let json_str = result.as_string()
            .unwrap_or_else(|| "null".into());

        Ok(json_to_gun_value(&json_str))
    }

    async fn put(&self, path: &[&str], value: GunValue) -> Result<(), String> {
        use wasm_bindgen::JsCast;
        use wasm_bindgen_futures::JsFuture;

        let i18n = db_i18n(self.language);
        let path_json = serde_json::to_string(path)
            .map_err(|e| i18n.write_error(&e.to_string()))?;
        let value_json = gun_value_to_json(&value);

        let js_code = format!(
            "window.__gun_bridge.put('{}', '{}')",
            path_json,
            value_json.replace('\'', "\\'")
        );
        let promise = js_sys::eval(&js_code)
            .map_err(|_| i18n.write_error("eval failed"))?;

        let promise = js_sys::Promise::from(promise.unchecked_into::<js_sys::Promise>());
        let result = JsFuture::from(promise).await
            .map_err(|_| i18n.write_error("Promise rejected"))?;

        let ack = result.as_string().unwrap_or_default();
        if ack.starts_with("err:") {
            Err(i18n.write_error(&ack[4..]))
        } else {
            Ok(())
        }
    }

    async fn on(
        &self,
        path: &[&str],
        _callback: Box<dyn Fn(GunValue, String) + Send + 'static>,
    ) -> Result<u64, String> {
        let i18n = db_i18n(self.language);
        let path_json = serde_json::to_string(path)
            .map_err(|e| i18n.subscribe_error(&e.to_string()))?;

        // Register JS-side subscription (returns numeric id)
        let js_code = format!("window.__gun_bridge.on('{}')", path_json);
        let result = js_sys::eval(&js_code)
            .map_err(|_| i18n.subscribe_error("eval failed"))?;

        let sub_id = result.as_f64()
            .map(|n| n as u64)
            .unwrap_or_else(|| self.next_sub_id.fetch_add(1, Ordering::Relaxed));

        // NOTE: The Rust callback is stored but must be driven by polling
        // gun_bridge.poll(subId) from an async loop (see `poll_subscription`).
        // A future iteration can wire this up with `setInterval` → wasm callback.

        Ok(sub_id)
    }

    fn off(&self, subscription_id: u64) -> Result<(), String> {
        let i18n = db_i18n(self.language);
        let js_code = format!("window.__gun_bridge.off({})", subscription_id);
        js_sys::eval(&js_code)
            .map_err(|_| i18n.subscribe_error("eval failed"))?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// SEA-authenticated writes
// ---------------------------------------------------------------------------
impl GunDb {
    /// Write a value signed with a SEA key pair.
    ///
    /// Calls `window.__gun_bridge.putSigned(path, value, pair)` which signs
    /// the value with `Gun.SEA.sign` before storing it.
    pub async fn put_signed(
        &self,
        path: &[&str],
        value: GunValue,
        sea_pair: &SeaKeyPair,
    ) -> Result<(), String> {
        use wasm_bindgen::JsCast;
        use wasm_bindgen_futures::JsFuture;

        let i18n = db_i18n(self.language);
        let path_json = serde_json::to_string(path)
            .map_err(|e| i18n.write_error(&e.to_string()))?;
        let value_json = gun_value_to_json(&value);
        let pair_json = sea_pair.to_json();

        let js_code = format!(
            "window.__gun_bridge.putSigned('{}', '{}', '{}')",
            path_json,
            value_json.replace('\'', "\\'"),
            pair_json.replace('\'', "\\'"),
        );
        let promise = js_sys::eval(&js_code)
            .map_err(|_| i18n.write_error("eval failed"))?;

        let promise = js_sys::Promise::from(promise.unchecked_into::<js_sys::Promise>());
        let result = JsFuture::from(promise).await
            .map_err(|_| i18n.write_error("Promise rejected"))?;

        let ack = result.as_string().unwrap_or_default();
        if ack.starts_with("err:") {
            Err(i18n.write_error(&ack[4..]))
        } else {
            Ok(())
        }
    }
}

/// Poll pending subscription updates from the JS side.
/// Call this from an async loop (e.g. inside a Dioxus `use_future`).
///
/// Returns a vec of (data, key) pairs that arrived since the last poll.
pub fn poll_subscription(sub_id: u64) -> Vec<(GunValue, String)> {
    let js_code = format!("window.__gun_bridge.poll({})", sub_id);
    let Ok(result) = js_sys::eval(&js_code) else { return vec![] };
    let json_str = result.as_string().unwrap_or_else(|| "[]".into());

    let Ok(arr) = serde_json::from_str::<Vec<serde_json::Value>>(&json_str) else { return vec![] };

    arr.iter().filter_map(|entry| {
        let data_val = entry.get("data")?;
        let key = entry.get("key")?.as_str()?.to_string();
        let gun_val = serde_json_to_gun_value(data_val);
        Some((gun_val, key))
    }).collect()
}

// ===========================================================================
// Desktop (non-WASM) implementation — local in-memory graph (unchanged)
// ===========================================================================
// JSON ↔ GunValue conversions
// ===========================================================================

/// Convert a JSON string (from JS bridge) into a GunValue.
/// Returns `None` for the JSON literal `"null"` or unparseable input.
fn json_to_gun_value(json: &str) -> Option<GunValue> {
    let v: serde_json::Value = serde_json::from_str(json).ok()?;
    if v.is_null() { return None; }
    Some(serde_json_to_gun_value(&v))
}

/// Recursive conversion from serde_json::Value → GunValue.
fn serde_json_to_gun_value(v: &serde_json::Value) -> GunValue {
    match v {
        serde_json::Value::Null => GunValue::Null,
        serde_json::Value::Bool(b) => GunValue::Bool(*b),
        serde_json::Value::Number(n) => GunValue::Number(n.as_f64().unwrap_or(0.0)),
        serde_json::Value::String(s) => GunValue::Text(s.clone()),
        serde_json::Value::Array(_) => {
            // GUN doesn't support arrays; treat as null
            GunValue::Null
        }
        serde_json::Value::Object(map) => {
            let mut result = HashMap::new();
            for (k, val) in map {
                // Skip GUN metadata field "_"
                if k == "_" { continue; }
                result.insert(k.clone(), serde_json_to_gun_value(val));
            }
            GunValue::Node(result)
        }
    }
}

/// Convert a GunValue into a JSON string suitable for the JS bridge.
fn gun_value_to_json(value: &GunValue) -> String {
    match value {
        GunValue::Null => "null".into(),
        GunValue::Bool(b) => if *b { "true".into() } else { "false".into() },
        GunValue::Number(n) => format!("{}", n),
        GunValue::Text(s) => serde_json::to_string(s).unwrap_or_else(|_| "null".into()),
        GunValue::Node(map) => {
            let serde_map: serde_json::Map<String, serde_json::Value> = map.iter()
                .map(|(k, v)| (k.clone(), gun_value_to_serde(v)))
                .collect();
            serde_json::to_string(&serde_json::Value::Object(serde_map))
                .unwrap_or_else(|_| "null".into())
        }
    }
}

/// Convert GunValue → serde_json::Value (for serialisation).
fn gun_value_to_serde(value: &GunValue) -> serde_json::Value {
    match value {
        GunValue::Null => serde_json::Value::Null,
        GunValue::Bool(b) => serde_json::Value::Bool(*b),
        GunValue::Number(n) => serde_json::json!(*n),
        GunValue::Text(s) => serde_json::Value::String(s.clone()),
        GunValue::Node(map) => {
            let serde_map: serde_json::Map<String, serde_json::Value> = map.iter()
                .map(|(k, v)| (k.clone(), gun_value_to_serde(v)))
                .collect();
            serde_json::Value::Object(serde_map)
        }
    }
}
