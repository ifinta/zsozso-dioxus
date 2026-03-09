use crate::i18n::Language;
use super::i18n::{DbI18n, db_i18n};

use serde::{Deserialize, Serialize};

fn log(msg: &str) {
    web_sys::console::log_1(&msg.into());
}

/// A SEA key pair — the four keys returned by `SEA.pair()`.
///
/// * `pub_key` / `priv_key`  — ECDSA (signing)
/// * `epub` / `epriv`        — ECDH  (encryption / shared secret)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SeaKeyPair {
    /// ECDSA public key (used for `verify`)
    #[serde(rename = "pub")]
    pub pub_key: String,
    /// ECDSA private key (used for `sign`)
    #[serde(rename = "priv")]
    pub priv_key: String,
    /// ECDH public key (shared with others for `secret`)
    pub epub: String,
    /// ECDH private key (used locally for `secret` + `encrypt`/`decrypt`)
    pub epriv: String,
}

impl SeaKeyPair {
    /// Serialise to JSON (for passing to the JS bridge).
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}

/// Abstract interface for GUN SEA (Security, Encryption, Authorization).
///
/// # Methods (maps 1-to-1 to `Gun.SEA.*`)
///
/// | Rust                | JS SEA              | Purpose                          |
/// |---------------------|---------------------|----------------------------------|
/// | `pair()`            | `SEA.pair()`        | Generate random key pair         |
/// | `pair_from_seed(s)` | `SEA.pair(seed)`    | Derive deterministic key pair    |
/// | `sign(data, pair)`  | `SEA.sign()`        | Sign data                        |
/// | `verify(msg, pub)`  | `SEA.verify()`      | Verify signed message            |
/// | `encrypt(data, ..)` | `SEA.encrypt()`     | Encrypt with pair or passphrase  |
/// | `decrypt(msg, ..)`  | `SEA.decrypt()`     | Decrypt                          |
/// | `work(data, salt)`  | `SEA.work()`        | PBKDF2 proof-of-work / hash      |
/// | `secret(epub, pair)`| `SEA.secret()`      | ECDH shared secret               |
#[allow(async_fn_in_trait)]
pub trait Sea {
    /// Generate a new random cryptographic key pair (ECDSA + ECDH).
    async fn pair(&self) -> Result<SeaKeyPair, String>;

    /// Derive a deterministic key pair from a seed / passphrase string.
    /// Equivalent to `SEA.pair("some-seed-string")` in JS.
    async fn pair_from_seed(&self, seed: &str) -> Result<SeaKeyPair, String>;

    /// Sign `data` with the given key pair.
    /// Returns the signed message string.
    async fn sign(&self, data: &str, pair: &SeaKeyPair) -> Result<String, String>;

    /// Verify a signed message against a public key.
    /// Returns the original data if valid; `Err` if verification fails.
    async fn verify(&self, message: &str, pub_key: &str) -> Result<String, String>;

    /// Encrypt `data` using a key pair (uses `pair.epriv`) or a passphrase.
    /// Returns the encrypted ciphertext string.
    async fn encrypt(&self, data: &str, pair_or_passphrase: &str) -> Result<String, String>;

    /// Decrypt a ciphertext using a key pair or passphrase.
    /// Returns the decrypted plaintext.
    async fn decrypt(&self, message: &str, pair_or_passphrase: &str) -> Result<String, String>;

    /// Proof-of-work / PBKDF2 key derivation.
    /// `salt` can be a JSON key pair or a passphrase string.
    /// Returns the derived hash.
    async fn work(&self, data: &str, salt: &str) -> Result<String, String>;

    /// ECDH shared secret derivation.
    /// `other_epub` is the other party's public encryption key.
    /// Returns the shared secret string.
    async fn secret(&self, other_epub: &str, my_pair: &SeaKeyPair) -> Result<String, String>;
}

/// Concrete SEA implementation backed by the JS `sea_bridge.js`.
pub struct GunSea {
    language: Language,
}

/// Get a reference to `window.__sea_bridge`.
fn sea_bridge() -> Result<wasm_bindgen::JsValue, String> {
    let window = web_sys::window().ok_or("no window")?;
    js_sys::Reflect::get(&window, &wasm_bindgen::JsValue::from_str("__sea_bridge"))
        .map_err(|_| "no __sea_bridge".to_string())
}

/// Get a function from the sea bridge by name.
fn sea_fn(bridge: &wasm_bindgen::JsValue, name: &str) -> Result<js_sys::Function, String> {
    use wasm_bindgen::JsCast;
    let val = js_sys::Reflect::get(bridge, &wasm_bindgen::JsValue::from_str(name))
        .map_err(|_| format!("no {} on __sea_bridge", name))?;
    Ok(val.unchecked_into())
}

/// Call a sea bridge function with arguments, await the Promise, return the string result.
async fn call_sea_fn(name: &str, args: &[&str]) -> Result<String, String> {
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;
    use wasm_bindgen::JsValue;

    log(&format!("[GunSea::{}] calling with {} args", name, args.len()));
    let bridge = sea_bridge()?;
    let func = sea_fn(&bridge, name)?;

    let promise_val = match args.len() {
        0 => func.call0(&bridge),
        1 => func.call1(&bridge, &JsValue::from_str(args[0])),
        2 => func.call2(&bridge, &JsValue::from_str(args[0]), &JsValue::from_str(args[1])),
        _ => return Err(format!("too many args for {}", name)),
    }.map_err(|e| {
        log(&format!("[GunSea::{}] call failed: {:?}", name, e));
        format!("call failed: {:?}", e)
    })?;

    let promise: js_sys::Promise = promise_val.unchecked_into();
    let result = JsFuture::from(promise).await
        .map_err(|e| {
            log(&format!("[GunSea::{}] Promise rejected: {:?}", name, e));
            format!("Promise rejected: {:?}", e)
        })?;
    let s = result.as_string().unwrap_or_default();
    log(&format!("[GunSea::{}] result length={}", name, s.len()));
    Ok(s)
}

impl GunSea {
    pub fn new(language: Language) -> Self {
        log("[GunSea::new] language initialized");
        Self { language }
    }
}

impl Sea for GunSea {
    async fn pair(&self) -> Result<SeaKeyPair, String> {
        log("[GunSea::pair] Generating random SEA key pair");
        let i18n = db_i18n(self.language);
        let json = call_sea_fn("pair", &[]).await
            .map_err(|e| { log(&format!("[GunSea::pair] ERROR: {}", e)); i18n.sea_error(&e) })?;
        log(&format!("[GunSea::pair] Got JSON response, length={}", json.len()));
        let pair = serde_json::from_str::<SeaKeyPair>(&json)
            .map_err(|e| { log(&format!("[GunSea::pair] Parse error: {}", e)); i18n.sea_error(&e.to_string()) })?;
        log(&format!("[GunSea::pair] Success, pub_key={}", &pair.pub_key));
        Ok(pair)
    }

    async fn pair_from_seed(&self, seed: &str) -> Result<SeaKeyPair, String> {
        log(&format!("[GunSea::pair_from_seed] seed length={}", seed.len()));
        let i18n = db_i18n(self.language);
        log("[GunSea::pair_from_seed] Calling JS...");
        let json = call_sea_fn("pairFromSeed", &[seed]).await
            .map_err(|e| { log(&format!("[GunSea::pair_from_seed] ERROR: {}", e)); i18n.sea_error(&e) })?;
        log(&format!("[GunSea::pair_from_seed] Got JSON, length={}", json.len()));
        let pair = serde_json::from_str::<SeaKeyPair>(&json)
            .map_err(|e| { log(&format!("[GunSea::pair_from_seed] Parse error: {}", e)); i18n.sea_error(&e.to_string()) })?;
        log(&format!("[GunSea::pair_from_seed] Success, pub_key={}", &pair.pub_key));
        Ok(pair)
    }

    async fn sign(&self, data: &str, pair: &SeaKeyPair) -> Result<String, String> {
        log(&format!("[GunSea::sign] data length={}, pub_key={}", data.len(), &pair.pub_key));
        let i18n = db_i18n(self.language);
        let pair_json = pair.to_json();
        let result = call_sea_fn("sign", &[data, &pair_json]).await
            .map_err(|e| { log(&format!("[GunSea::sign] ERROR: {}", e)); i18n.sea_error(&e) })?;
        if result.is_empty() {
            log("[GunSea::sign] ERROR: sign returned undefined");
            Err(i18n.sea_error("sign returned undefined"))
        } else {
            log(&format!("[GunSea::sign] Success, result length={}", result.len()));
            Ok(result)
        }
    }

    async fn verify(&self, message: &str, pub_key: &str) -> Result<String, String> {
        log(&format!("[GunSea::verify] message length={}, pub_key={}", message.len(), pub_key));
        let i18n = db_i18n(self.language);
        let result = call_sea_fn("verify", &[message, pub_key]).await
            .map_err(|e| { log(&format!("[GunSea::verify] ERROR: {}", e)); i18n.sea_error(&e) })?;
        if result.is_empty() {
            log("[GunSea::verify] ERROR: verification failed");
            Err(i18n.sea_error("verification failed"))
        } else {
            log(&format!("[GunSea::verify] Success, result length={}", result.len()));
            Ok(result)
        }
    }

    async fn encrypt(&self, data: &str, pair_or_passphrase: &str) -> Result<String, String> {
        log(&format!("[GunSea::encrypt] data length={}", data.len()));
        let i18n = db_i18n(self.language);
        let result = call_sea_fn("encrypt", &[data, pair_or_passphrase]).await
            .map_err(|e| { log(&format!("[GunSea::encrypt] ERROR: {}", e)); i18n.sea_error(&e) })?;
        if result.is_empty() {
            log("[GunSea::encrypt] ERROR: encrypt returned undefined");
            Err(i18n.sea_error("encrypt returned undefined"))
        } else {
            log(&format!("[GunSea::encrypt] Success, result length={}", result.len()));
            Ok(result)
        }
    }

    async fn decrypt(&self, message: &str, pair_or_passphrase: &str) -> Result<String, String> {
        log(&format!("[GunSea::decrypt] message length={}", message.len()));
        let i18n = db_i18n(self.language);
        let result = call_sea_fn("decrypt", &[message, pair_or_passphrase]).await
            .map_err(|e| { log(&format!("[GunSea::decrypt] ERROR: {}", e)); i18n.sea_error(&e) })?;
        if result.is_empty() {
            log("[GunSea::decrypt] ERROR: decrypt returned undefined");
            Err(i18n.sea_error("decrypt returned undefined"))
        } else {
            log(&format!("[GunSea::decrypt] Success, result length={}", result.len()));
            Ok(result)
        }
    }

    async fn work(&self, data: &str, salt: &str) -> Result<String, String> {
        log(&format!("[GunSea::work] data length={}, salt length={}", data.len(), salt.len()));
        let i18n = db_i18n(self.language);
        let result = call_sea_fn("work", &[data, salt]).await
            .map_err(|e| { log(&format!("[GunSea::work] ERROR: {}", e)); i18n.sea_error(&e) })?;
        if result.is_empty() {
            log("[GunSea::work] ERROR: work returned undefined");
            Err(i18n.sea_error("work returned undefined"))
        } else {
            log(&format!("[GunSea::work] Success, result length={}", result.len()));
            Ok(result)
        }
    }

    async fn secret(&self, other_epub: &str, my_pair: &SeaKeyPair) -> Result<String, String> {
        log(&format!("[GunSea::secret] other_epub length={}, my pub_key={}", other_epub.len(), &my_pair.pub_key));
        let i18n = db_i18n(self.language);
        let pair_json = my_pair.to_json();
        let result = call_sea_fn("secret", &[other_epub, &pair_json]).await
            .map_err(|e| { log(&format!("[GunSea::secret] ERROR: {}", e)); i18n.sea_error(&e) })?;
        if result.is_empty() {
            log("[GunSea::secret] ERROR: secret returned undefined");
            Err(i18n.sea_error("secret returned undefined"))
        } else {
            log(&format!("[GunSea::secret] Success, result length={}", result.len()));
            Ok(result)
        }
    }
}
