use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use js_sys::{Promise, Reflect, Function};
use base64::{Engine, engine::general_purpose::STANDARD};

// ── JS bridge helpers ──

fn get_bridge_fn(method: &str) -> Result<Function, String> {
    let window = web_sys::window().ok_or("No window object")?;
    let bridge = Reflect::get(&window, &JsValue::from_str("__passkey_bridge"))
        .map_err(|_| "Passkey bridge not loaded (missing passkey_bridge.js?)")?;
    let func = Reflect::get(&bridge, &JsValue::from_str(method))
        .map_err(|_| format!("No bridge method: {}", method))?;
    func.dyn_into::<Function>()
        .map_err(|_| format!("{} is not a function", method))
}

async fn await_promise(val: JsValue) -> Result<JsValue, String> {
    let promise: Promise = val.dyn_into()
        .map_err(|_| "Expected a Promise from JS bridge")?;
    JsFuture::from(promise).await
        .map_err(|e| format!("Promise rejected: {:?}", e))
}

// ── Public API ──

/// Result of passkey registration (no PRF).
#[derive(serde::Deserialize)]
pub struct RegisterResult {
    pub success: bool,
    pub error: Option<String>,
}

/// Result of passkey init (register + authenticate).
#[derive(serde::Deserialize)]
pub struct InitResult {
    pub success: bool,
    #[serde(rename = "prfKey")]
    pub prf_key: Option<String>,
    pub error: Option<String>,
}

/// Register a passkey credential (one biometric prompt, no PRF authentication).
/// Call this when enabling biometric; PRF key is obtained lazily later.
pub async fn passkey_register() -> Result<RegisterResult, String> {
    let func = get_bridge_fn("register_only")?;
    let promise_val = func.call0(&JsValue::NULL)
        .map_err(|e| format!("register_only() call error: {:?}", e))?;
    let result = await_promise(promise_val).await?;
    let json_str = result.as_string()
        .ok_or_else(|| "register_only() returned non-string".to_string())?;
    serde_json::from_str(&json_str)
        .map_err(|e| format!("register_only() parse error: {}", e))
}

/// Authenticate with PRF to obtain the encryption key.
/// Credential must already exist (call passkey_register first).
pub async fn passkey_init() -> Result<InitResult, String> {
    let func = get_bridge_fn("init")?;
    let promise_val = func.call0(&JsValue::NULL)
        .map_err(|e| format!("init() call error: {:?}", e))?;
    let result = await_promise(promise_val).await?;
    let json_str = result.as_string()
        .ok_or_else(|| "init() returned non-string".to_string())?;
    serde_json::from_str(&json_str)
        .map_err(|e| format!("init() parse error: {}", e))
}

/// Lightweight passkey verification (no PRF). Returns true if user authenticated.
pub async fn passkey_verify() -> Result<bool, String> {
    let func = get_bridge_fn("verify")?;
    let promise_val = func.call0(&JsValue::NULL)
        .map_err(|e| format!("verify() call error: {:?}", e))?;
    let result = await_promise(promise_val).await?;
    Ok(result.as_bool().unwrap_or(false))
}

/// Encrypt a plaintext string using the PRF-derived key.
/// Returns a base64-encoded ciphertext (IV prepended).
pub async fn passkey_encrypt(plaintext: &str, prf_key: &str) -> Result<String, String> {
    let plaintext_b64 = STANDARD.encode(plaintext.as_bytes());
    let func = get_bridge_fn("encrypt")?;
    let promise_val = func.call2(
        &JsValue::NULL,
        &JsValue::from_str(&plaintext_b64),
        &JsValue::from_str(prf_key),
    ).map_err(|e| format!("encrypt() call error: {:?}", e))?;
    let result = await_promise(promise_val).await?;
    result.as_string()
        .ok_or_else(|| "encrypt() returned non-string".to_string())
}

/// Decrypt a base64-encoded ciphertext using the PRF-derived key.
/// Returns the original plaintext string.
pub async fn passkey_decrypt(ciphertext_b64: &str, prf_key: &str) -> Result<String, String> {
    let func = get_bridge_fn("decrypt")?;
    let promise_val = func.call2(
        &JsValue::NULL,
        &JsValue::from_str(ciphertext_b64),
        &JsValue::from_str(prf_key),
    ).map_err(|e| format!("decrypt() call error: {:?}", e))?;
    let result = await_promise(promise_val).await?;
    let plaintext_b64 = result.as_string()
        .ok_or_else(|| "decrypt() returned non-string".to_string())?;
    let bytes = STANDARD.decode(&plaintext_b64)
        .map_err(|e| format!("Base64 decode error: {}", e))?;
    String::from_utf8(bytes)
        .map_err(|e| format!("UTF-8 decode error: {}", e))
}
