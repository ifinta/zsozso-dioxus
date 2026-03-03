use crate::i18n::Language;
use super::i18n::{DbI18n, db_i18n};

use serde::{Deserialize, Serialize};

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

impl GunSea {
    pub fn new(language: Language) -> Self {
        Self { language }
    }

    /// Helper: evaluate a JS expression that returns a Promise<string>,
    /// await it, and return the resulting string.
    async fn eval_promise(js_code: &str) -> Result<String, String> {
        use wasm_bindgen::JsCast;
        use wasm_bindgen_futures::JsFuture;

        let val = js_sys::eval(js_code)
            .map_err(|e| format!("eval failed: {:?}", e))?;
        let promise: js_sys::Promise = val.unchecked_into();
        let result = JsFuture::from(promise).await
            .map_err(|e| format!("Promise rejected: {:?}", e))?;
        Ok(result.as_string().unwrap_or_default())
    }
}

impl Sea for GunSea {
    async fn pair(&self) -> Result<SeaKeyPair, String> {
        let i18n = db_i18n(self.language);
        let json = Self::eval_promise("window.__sea_bridge.pair()").await
            .map_err(|e| i18n.sea_error(&e))?;
        serde_json::from_str::<SeaKeyPair>(&json)
            .map_err(|e| i18n.sea_error(&e.to_string()))
    }

    async fn pair_from_seed(&self, seed: &str) -> Result<SeaKeyPair, String> {
        let i18n = db_i18n(self.language);
        let escaped_seed = seed.replace('\\', "\\\\").replace('\'', "\\'");
        let js = format!("window.__sea_bridge.pairFromSeed('{}')", escaped_seed);
        let json = Self::eval_promise(&js).await
            .map_err(|e| i18n.sea_error(&e))?;
        serde_json::from_str::<SeaKeyPair>(&json)
            .map_err(|e| i18n.sea_error(&e.to_string()))
    }

    async fn sign(&self, data: &str, pair: &SeaKeyPair) -> Result<String, String> {
        let i18n = db_i18n(self.language);
        let escaped_data = data.replace('\\', "\\\\").replace('\'', "\\'");
        let pair_json = pair.to_json().replace('\\', "\\\\").replace('\'', "\\'");
        let js = format!(
            "window.__sea_bridge.sign('{}', '{}')",
            escaped_data, pair_json
        );
        let result = Self::eval_promise(&js).await
            .map_err(|e| i18n.sea_error(&e))?;
        if result.is_empty() {
            Err(i18n.sea_error("sign returned undefined"))
        } else {
            Ok(result)
        }
    }

    async fn verify(&self, message: &str, pub_key: &str) -> Result<String, String> {
        let i18n = db_i18n(self.language);
        let escaped_msg = message.replace('\\', "\\\\").replace('\'', "\\'");
        let escaped_pub = pub_key.replace('\\', "\\\\").replace('\'', "\\'");
        let js = format!(
            "window.__sea_bridge.verify('{}', '{}')",
            escaped_msg, escaped_pub
        );
        let result = Self::eval_promise(&js).await
            .map_err(|e| i18n.sea_error(&e))?;
        if result.is_empty() {
            Err(i18n.sea_error("verification failed"))
        } else {
            Ok(result)
        }
    }

    async fn encrypt(&self, data: &str, pair_or_passphrase: &str) -> Result<String, String> {
        let i18n = db_i18n(self.language);
        let escaped_data = data.replace('\\', "\\\\").replace('\'', "\\'");
        let escaped_key = pair_or_passphrase.replace('\\', "\\\\").replace('\'', "\\'");
        let js = format!(
            "window.__sea_bridge.encrypt('{}', '{}')",
            escaped_data, escaped_key
        );
        let result = Self::eval_promise(&js).await
            .map_err(|e| i18n.sea_error(&e))?;
        if result.is_empty() {
            Err(i18n.sea_error("encrypt returned undefined"))
        } else {
            Ok(result)
        }
    }

    async fn decrypt(&self, message: &str, pair_or_passphrase: &str) -> Result<String, String> {
        let i18n = db_i18n(self.language);
        let escaped_msg = message.replace('\\', "\\\\").replace('\'', "\\'");
        let escaped_key = pair_or_passphrase.replace('\\', "\\\\").replace('\'', "\\'");
        let js = format!(
            "window.__sea_bridge.decrypt('{}', '{}')",
            escaped_msg, escaped_key
        );
        let result = Self::eval_promise(&js).await
            .map_err(|e| i18n.sea_error(&e))?;
        if result.is_empty() {
            Err(i18n.sea_error("decrypt returned undefined"))
        } else {
            Ok(result)
        }
    }

    async fn work(&self, data: &str, salt: &str) -> Result<String, String> {
        let i18n = db_i18n(self.language);
        let escaped_data = data.replace('\\', "\\\\").replace('\'', "\\'");
        let escaped_salt = salt.replace('\\', "\\\\").replace('\'', "\\'");
        let js = format!(
            "window.__sea_bridge.work('{}', '{}')",
            escaped_data, escaped_salt
        );
        let result = Self::eval_promise(&js).await
            .map_err(|e| i18n.sea_error(&e))?;
        if result.is_empty() {
            Err(i18n.sea_error("work returned undefined"))
        } else {
            Ok(result)
        }
    }

    async fn secret(&self, other_epub: &str, my_pair: &SeaKeyPair) -> Result<String, String> {
        let i18n = db_i18n(self.language);
        let escaped_epub = other_epub.replace('\\', "\\\\").replace('\'', "\\'");
        let pair_json = my_pair.to_json().replace('\\', "\\\\").replace('\'', "\\'");
        let js = format!(
            "window.__sea_bridge.secret('{}', '{}')",
            escaped_epub, pair_json
        );
        let result = Self::eval_promise(&js).await
            .map_err(|e| i18n.sea_error(&e))?;
        if result.is_empty() {
            Err(i18n.sea_error("secret returned undefined"))
        } else {
            Ok(result)
        }
    }
}
