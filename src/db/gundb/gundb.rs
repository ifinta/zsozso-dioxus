use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;

use crate::i18n::Language;
use super::{Db, GunConfig, GunValue};
use super::i18n::{DbI18n, db_i18n};
use super::sea::SeaKeyPair;

fn log(msg: &str) {
    web_sys::console::log_1(&msg.into());
}

/// Get a reference to `window.__gun_bridge`.
fn gun_bridge() -> Result<JsValue, String> {
    let window = web_sys::window().ok_or("no window")?;
    js_sys::Reflect::get(&window, &JsValue::from_str("__gun_bridge"))
        .map_err(|_| "no __gun_bridge".to_string())
}

/// Get a function from the gun bridge object by name.
fn bridge_fn(bridge: &JsValue, name: &str) -> Result<js_sys::Function, String> {
    let val = js_sys::Reflect::get(bridge, &JsValue::from_str(name))
        .map_err(|_| format!("no {} on __gun_bridge", name))?;
    Ok(val.unchecked_into())
}

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
        log(&format!("[GunDb::new] peers={:?}, local_storage={}", config.peers, config.local_storage));
        let peers_json = serde_json::to_string(&config.peers).unwrap_or_else(|_| "[]".into());

        if let Ok(bridge) = gun_bridge() {
            if let Ok(init_fn) = bridge_fn(&bridge, "init") {
                let result = init_fn.call1(&bridge, &JsValue::from_str(&peers_json));
                log(&format!("[GunDb::new] init result: {:?}", result.is_ok()));
            } else {
                log("[GunDb::new] ERROR: init function not found on bridge");
            }
        } else {
            log("[GunDb::new] ERROR: __gun_bridge not found");
        }

        Self {
            config,
            language,
            next_sub_id: AtomicU64::new(1),
        }
    }

    /// Add a peer relay URL to the live GUN instance.
    pub fn add_peer(&self, peer_url: &str) {
        log(&format!("[GunDb::add_peer] url={}", peer_url));
        if let Ok(bridge) = gun_bridge() {
            if let Ok(add_peer_fn) = bridge_fn(&bridge, "addPeer") {
                let _ = add_peer_fn.call1(&bridge, &JsValue::from_str(peer_url));
            } else {
                log("[GunDb::add_peer] ERROR: addPeer not found on bridge");
            }
        }
    }
}

// ===========================================================================
// WASM implementation — delegates to gun_bridge.js
// ===========================================================================
impl Db for GunDb {
    async fn get(&self, path: &[&str]) -> Result<Option<GunValue>, String> {
        log(&format!("[GunDb::get] path={:?}", path));
        let i18n = db_i18n(self.language);
        let path_json = serde_json::to_string(path)
            .map_err(|e| i18n.read_error(&e.to_string()))?;

        let bridge = gun_bridge().map_err(|e| i18n.read_error(&e))?;
        let get_fn = bridge_fn(&bridge, "get").map_err(|e| i18n.read_error(&e))?;

        log(&format!("[GunDb::get] Calling JS get({})", path_json));
        let promise_val = get_fn.call1(&bridge, &JsValue::from_str(&path_json))
            .map_err(|e| {
                log(&format!("[GunDb::get] ERROR: call failed: {:?}", e));
                i18n.read_error("call failed")
            })?;

        let promise: js_sys::Promise = promise_val.unchecked_into();
        let result = JsFuture::from(promise).await
            .map_err(|_| {
                log("[GunDb::get] ERROR: Promise rejected");
                i18n.read_error("Promise rejected")
            })?;

        let json_str = result.as_string()
            .unwrap_or_else(|| "null".into());

        log(&format!("[GunDb::get] raw result JSON: {}", &json_str[..json_str.len().min(200)]));
        let gun_value = json_to_gun_value(&json_str);
        log(&format!("[GunDb::get] parsed GunValue: {:?}", gun_value));
        Ok(gun_value)
    }

    async fn put(&self, path: &[&str], value: GunValue) -> Result<(), String> {
        log(&format!("[GunDb::put] path={:?}, value={:?}", path, value));
        let i18n = db_i18n(self.language);
        let path_json = serde_json::to_string(path)
            .map_err(|e| i18n.write_error(&e.to_string()))?;
        let value_json = gun_value_to_json(&value);
        log(&format!("[GunDb::put] path_json={}, value_json={}", path_json, value_json));

        let bridge = gun_bridge().map_err(|e| i18n.write_error(&e))?;
        let put_fn = bridge_fn(&bridge, "put").map_err(|e| i18n.write_error(&e))?;

        log("[GunDb::put] Calling JS put...");
        let promise_val = put_fn.call2(
            &bridge,
            &JsValue::from_str(&path_json),
            &JsValue::from_str(&value_json),
        ).map_err(|e| {
            log(&format!("[GunDb::put] ERROR: call failed: {:?}", e));
            i18n.write_error("call failed")
        })?;

        let promise: js_sys::Promise = promise_val.unchecked_into();
        let result = JsFuture::from(promise).await
            .map_err(|_| {
                log("[GunDb::put] ERROR: Promise rejected");
                i18n.write_error("Promise rejected")
            })?;

        let ack = result.as_string().unwrap_or_default();
        log(&format!("[GunDb::put] ack={}", ack));
        if ack.starts_with("err:") {
            log(&format!("[GunDb::put] ERROR: {}", &ack[4..]));
            Err(i18n.write_error(&ack[4..]))
        } else {
            log("[GunDb::put] Success");
            Ok(())
        }
    }

    async fn on(
        &self,
        path: &[&str],
        _callback: Box<dyn Fn(GunValue, String) + Send + 'static>,
    ) -> Result<u64, String> {
        log(&format!("[GunDb::on] path={:?}", path));
        let i18n = db_i18n(self.language);
        let path_json = serde_json::to_string(path)
            .map_err(|e| i18n.subscribe_error(&e.to_string()))?;

        let bridge = gun_bridge().map_err(|e| i18n.subscribe_error(&e))?;
        let on_fn = bridge_fn(&bridge, "on").map_err(|e| i18n.subscribe_error(&e))?;

        log(&format!("[GunDb::on] Calling JS on({})", path_json));
        let result = on_fn.call1(&bridge, &JsValue::from_str(&path_json))
            .map_err(|_| {
                log("[GunDb::on] ERROR: call failed");
                i18n.subscribe_error("call failed")
            })?;

        let sub_id = result.as_f64()
            .map(|n| n as u64)
            .unwrap_or_else(|| self.next_sub_id.fetch_add(1, Ordering::Relaxed));

        log(&format!("[GunDb::on] subscription id={}", sub_id));
        Ok(sub_id)
    }

    fn off(&self, subscription_id: u64) -> Result<(), String> {
        log(&format!("[GunDb::off] subscription_id={}", subscription_id));
        let i18n = db_i18n(self.language);

        let bridge = gun_bridge().map_err(|e| i18n.subscribe_error(&e))?;
        let off_fn = bridge_fn(&bridge, "off").map_err(|e| i18n.subscribe_error(&e))?;

        off_fn.call1(&bridge, &JsValue::from_f64(subscription_id as f64))
            .map_err(|_| {
                log("[GunDb::off] ERROR: call failed");
                i18n.subscribe_error("call failed")
            })?;
        log("[GunDb::off] Done");
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
        log(&format!("[GunDb::put_signed] path={:?}, value={:?}, pub_key={}", path, value, &sea_pair.pub_key));
        let i18n = db_i18n(self.language);
        let path_json = serde_json::to_string(path)
            .map_err(|e| i18n.write_error(&e.to_string()))?;
        let value_json = gun_value_to_json(&value);
        let pair_json = sea_pair.to_json();
        log(&format!("[GunDb::put_signed] path_json={}, value_json={}", path_json, value_json));
        log(&format!("[GunDb::put_signed] pair_json length={}", pair_json.len()));

        let bridge = gun_bridge().map_err(|e| i18n.write_error(&e))?;
        let put_signed_fn = bridge_fn(&bridge, "putSigned").map_err(|e| i18n.write_error(&e))?;

        log("[GunDb::put_signed] Calling JS putSigned...");
        let promise_val = put_signed_fn.call3(
            &bridge,
            &JsValue::from_str(&path_json),
            &JsValue::from_str(&value_json),
            &JsValue::from_str(&pair_json),
        ).map_err(|e| {
            log(&format!("[GunDb::put_signed] ERROR: call failed: {:?}", e));
            i18n.write_error("call failed")
        })?;

        let promise: js_sys::Promise = promise_val.unchecked_into();
        let result = JsFuture::from(promise).await
            .map_err(|_| {
                log("[GunDb::put_signed] ERROR: Promise rejected");
                i18n.write_error("Promise rejected")
            })?;

        let ack = result.as_string().unwrap_or_default();
        log(&format!("[GunDb::put_signed] ack={}", ack));
        if ack.starts_with("err:") {
            log(&format!("[GunDb::put_signed] ERROR: {}", &ack[4..]));
            Err(i18n.write_error(&ack[4..]))
        } else {
            log("[GunDb::put_signed] Success");
            Ok(())
        }
    }
}

/// Poll pending subscription updates from the JS side.
/// Call this from an async loop (e.g. inside a Dioxus `use_future`).
///
/// Returns a vec of (data, key) pairs that arrived since the last poll.
pub fn poll_subscription(sub_id: u64) -> Vec<(GunValue, String)> {
    let Ok(bridge) = gun_bridge() else {
        log("[poll_subscription] no bridge");
        return vec![];
    };
    let Ok(poll_fn) = bridge_fn(&bridge, "poll") else {
        log("[poll_subscription] no poll fn");
        return vec![];
    };

    let Ok(result) = poll_fn.call1(&bridge, &JsValue::from_f64(sub_id as f64)) else {
        log("[poll_subscription] call failed");
        return vec![];
    };
    let json_str = result.as_string().unwrap_or_else(|| "[]".into());
    if json_str != "[]" {
        log(&format!("[poll_subscription] sub_id={}, raw: {}", sub_id, &json_str[..json_str.len().min(200)]));
    }

    let Ok(arr) = serde_json::from_str::<Vec<serde_json::Value>>(&json_str) else {
        log("[poll_subscription] JSON parse failed");
        return vec![];
    };

    let items: Vec<_> = arr.iter().filter_map(|entry| {
        let data_val = entry.get("data")?;
        let key = entry.get("key")?.as_str()?.to_string();
        let gun_val = serde_json_to_gun_value(data_val);
        Some((gun_val, key))
    }).collect();
    if !items.is_empty() {
        log(&format!("[poll_subscription] returning {} items", items.len()));
    }
    items
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
