mod gundb;
mod sea;
pub mod i18n;

pub use gundb::GunDb;
pub use gundb::poll_subscription;
pub use sea::{GunSea, Sea, SeaKeyPair};

use std::collections::HashMap;

/// Configuration for connecting to GUN peers.
#[derive(Clone, Debug, Default)]
pub struct GunConfig {
    /// List of relay peer URLs to sync with, e.g. `["http://localhost:8765/gun"]`
    pub peers: Vec<String>,
    /// Whether to persist data locally (localStorage in browser, Radisk on server)
    pub local_storage: bool,
}

/// A single value in the GUN graph.
/// GUN supports: strings, numbers (f64), booleans, null, and nested objects (nodes).
/// Traditional arrays are NOT supported — use `set` instead.
#[derive(Clone, Debug, PartialEq)]
pub enum GunValue {
    Null,
    Bool(bool),
    Number(f64),
    Text(String),
    Node(HashMap<String, GunValue>),
}

impl GunValue {
    pub fn as_text(&self) -> Option<&str> {
        match self {
            GunValue::Text(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_number(&self) -> Option<f64> {
        match self {
            GunValue::Number(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            GunValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_node(&self) -> Option<&HashMap<String, GunValue>> {
        match self {
            GunValue::Node(m) => Some(m),
            _ => None,
        }
    }

    pub fn is_null(&self) -> bool {
        matches!(self, GunValue::Null)
    }
}

/// Abstract interface for a GUN-compatible graph database.
///
/// GUN is a decentralized, real-time, offline-first, graph database.
/// This trait captures its core operations so the rest of the app
/// doesn't depend on the concrete implementation.
///
/// # Core methods (implemented first)
///
/// 1. **`new`**        — Create a GunDb instance with config (peers, options)
/// 2. **`get`**        — Read a node/value once by key path (like `gun.get(key).once()`)
/// 3. **`put`**        — Write/update data at a key path (like `gun.get(key).put(data)`)
/// 4. **`on`**         — Subscribe to real-time changes on a key (like `gun.get(key).on(cb)`)
/// 5. **`off`**        — Unsubscribe from real-time changes (like `gun.get(key).off()`)
#[allow(async_fn_in_trait)]
pub trait Db {
    /// Read a value once from the graph at the given key path.
    ///
    /// The `path` is a slice of keys that drill into the graph,
    /// e.g. `&["users", "alice", "name"]` corresponds to
    /// `gun.get('users').get('alice').get('name').once(cb)`.
    ///
    /// Returns `Ok(None)` if the key does not exist.
    async fn get(&self, path: &[&str]) -> Result<Option<GunValue>, String>;

    /// Write/update data at the given key path.
    ///
    /// The `path` works the same as in `get`.
    /// Like GUN's `.put`, intermediate nodes are created implicitly.
    ///
    /// e.g. `put(&["users", "alice"], value)` corresponds to
    /// `gun.get('users').get('alice').put(data)`.
    async fn put(&self, path: &[&str], value: GunValue) -> Result<(), String>;

    /// Subscribe to real-time changes at the given key path.
    ///
    /// The `callback` is called once immediately with the current value
    /// and then again whenever the data changes (from any peer).
    ///
    /// Returns a subscription ID that can be passed to `off()`.
    async fn on(
        &self,
        path: &[&str],
        callback: Box<dyn Fn(GunValue, String) + Send + 'static>,
    ) -> Result<u64, String>;

    /// Unsubscribe from real-time changes previously registered via `on`.
    fn off(&self, subscription_id: u64) -> Result<(), String>;
}
