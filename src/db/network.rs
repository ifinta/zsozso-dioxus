use crate::i18n::Language;
use super::gundb::{Db, GunDb, GunConfig, GunValue, SeaKeyPair};

fn log(msg: &str) {
    web_sys::console::log_1(&msg.into());
}

/// Extract the message content from a SEA-signed envelope.
///
/// GUN's `putSigned` wraps values as `SEA{"m":"actual","s":"signature"}`.
/// This strips the wrapper and returns just the `"m"` field.
/// If the string is not a SEA envelope, it is returned unchanged.
fn unwrap_sea(s: &str) -> String {
    if let Some(json) = s.strip_prefix("SEA") {
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(json) {
            if let Some(m) = parsed.get("m").and_then(|v| v.as_str()) {
                return m.to_string();
            }
        }
    }
    s.to_string()
}

/// Maximum length for a user nickname displayed on network buttons.
pub const MAX_NICKNAME_LEN: usize = 16;

/// Abstract interface for the Iceberg Protocol network graph.
///
/// Hides the underlying graph database behind domain-specific operations.
/// Each node in the network is identified by its Stellar public key.
/// The graph stores: parent link, worker links, and a human-readable nickname.
#[allow(async_fn_in_trait)]
pub trait NetworkGraph {
    /// Get the direct parent of a node, if any.
    async fn get_parent(&self, node_key: &str) -> Result<Option<String>, String>;

    /// Get the ancestry chain (parent, grandparent, ...) up to `depth` levels.
    async fn get_ancestors(&self, node_key: &str, depth: usize) -> Result<Vec<String>, String>;

    /// Get all worker public keys of a node.
    async fn get_workers(&self, node_key: &str) -> Result<Vec<String>, String>;

    /// Get the nickname of a node.
    async fn get_nickname(&self, node_key: &str) -> Result<Option<String>, String>;

    /// Set the nickname of the current node.
    async fn set_nickname(&self, node_key: &str, nickname: &str) -> Result<(), String>;

    /// Add a worker to the current node and set this node as the worker's parent.
    async fn add_worker(&self, node_key: &str, worker_key: &str) -> Result<(), String>;

    /// Request a modification to a node's record.
    /// The environment will decide whether to grant the modification.
    async fn request_modify(&self, node_key: &str, field: &str, value: &str) -> Result<(), String>;

    /// Store the GUN node address (SEA public key) for a node.
    async fn set_gun_address(&self, node_key: &str, gun_address: &str) -> Result<(), String>;

    /// Get the stored GUN node address (SEA public key) for a node.
    async fn get_gun_address(&self, node_key: &str) -> Result<Option<String>, String>;

    /// Store the GUN relay URL for a node (optional — if the user runs their own relay).
    async fn set_gun_relay_url(&self, node_key: &str, url: &str) -> Result<(), String>;

    /// Get the stored GUN relay URL for a node.
    async fn get_gun_relay_url(&self, node_key: &str) -> Result<Option<String>, String>;
}

/// Concrete implementation backed by the GUN decentralised database.
pub struct GunNetworkGraph {
    db: GunDb,
    sea_pair: Option<SeaKeyPair>,
}

impl GunNetworkGraph {
    pub fn new(language: Language, sea_pair: Option<SeaKeyPair>) -> Self {
        log(&format!("[GunNetworkGraph::new] has_sea_pair={}", sea_pair.is_some()));
        let config = GunConfig {
            peers: vec![],
            local_storage: true,
        };
        Self {
            db: GunDb::new(config, language),
            sea_pair,
        }
    }

    /// Write helper — uses SEA-signed put. Requires a keypair.
    async fn authenticated_put(&self, path: &[&str], value: GunValue) -> Result<(), String> {
        log(&format!("[GunNetworkGraph::authenticated_put] path={:?}, value={:?}, has_pair={}", path, value, self.sea_pair.is_some()));
        match &self.sea_pair {
            Some(pair) => {
                let result = self.db.put_signed(path, value, pair).await;
                log(&format!("[GunNetworkGraph::authenticated_put] result={:?}", result));
                result
            }
            None => {
                log("[GunNetworkGraph::authenticated_put] ERROR: No SEA keypair");
                Err("SEA keypair required for authenticated writes".to_string())
            }
        }
    }
}

impl NetworkGraph for GunNetworkGraph {
    async fn get_parent(&self, node_key: &str) -> Result<Option<String>, String> {
        log(&format!("[NetworkGraph::get_parent] node={}", node_key));
        let result = match self.db.get(&["network", node_key, "parent"]).await? {
            Some(GunValue::Text(s)) if !s.is_empty() => Ok(Some(unwrap_sea(&s))),
            _ => Ok(None),
        };
        log(&format!("[NetworkGraph::get_parent] result={:?}", result));
        result
    }

    async fn get_ancestors(&self, node_key: &str, depth: usize) -> Result<Vec<String>, String> {
        log(&format!("[NetworkGraph::get_ancestors] node={}, depth={}", node_key, depth));
        let mut ancestors = Vec::new();
        let mut current = node_key.to_string();
        for i in 0..depth {
            log(&format!("[NetworkGraph::get_ancestors] step={}, current={}", i, current));
            match self.get_parent(&current).await? {
                Some(parent) => {
                    log(&format!("[NetworkGraph::get_ancestors] found parent={}", parent));
                    ancestors.push(parent.clone());
                    current = parent;
                }
                None => {
                    log(&format!("[NetworkGraph::get_ancestors] no parent at step={}", i));
                    break;
                }
            }
        }
        log(&format!("[NetworkGraph::get_ancestors] total ancestors={}", ancestors.len()));
        Ok(ancestors)
    }

    async fn get_workers(&self, node_key: &str) -> Result<Vec<String>, String> {
        log(&format!("[NetworkGraph::get_workers] node={}", node_key));
        let result = match self.db.get(&["network", node_key, "workers"]).await? {
            Some(GunValue::Node(map)) => {
                let keys: Vec<String> = map.into_keys().collect();
                log(&format!("[NetworkGraph::get_workers] found {} workers", keys.len()));
                Ok(keys)
            }
            other => {
                log(&format!("[NetworkGraph::get_workers] no workers found, raw={:?}", other));
                Ok(vec![])
            }
        };
        result
    }

    async fn get_nickname(&self, node_key: &str) -> Result<Option<String>, String> {
        log(&format!("[NetworkGraph::get_nickname] node={}", node_key));
        let result = match self.db.get(&["network", node_key, "nickname"]).await? {
            Some(GunValue::Text(s)) if !s.is_empty() => {
                let nick = unwrap_sea(&s);
                log(&format!("[NetworkGraph::get_nickname] found nickname={}", nick));
                Ok(Some(nick))
            }
            other => {
                log(&format!("[NetworkGraph::get_nickname] no nickname, raw={:?}", other));
                Ok(None)
            }
        };
        result
    }

    async fn set_nickname(&self, node_key: &str, nickname: &str) -> Result<(), String> {
        log(&format!("[NetworkGraph::set_nickname] node={}, nickname={}", node_key, nickname));
        let trimmed: String = nickname.chars().take(MAX_NICKNAME_LEN).collect();
        log(&format!("[NetworkGraph::set_nickname] trimmed={}", trimmed));
        let result = self.authenticated_put(
            &["network", node_key, "nickname"],
            GunValue::Text(trimmed),
        ).await;
        log(&format!("[NetworkGraph::set_nickname] result={:?}", result));
        result
    }

    async fn add_worker(&self, node_key: &str, worker_key: &str) -> Result<(), String> {
        log(&format!("[NetworkGraph::add_worker] node={}, worker={}", node_key, worker_key));
        log("[NetworkGraph::add_worker] Setting worker entry...");
        self.authenticated_put(
            &["network", node_key, "workers", worker_key],
            GunValue::Bool(true),
        ).await?;
        log("[NetworkGraph::add_worker] Worker entry set. Setting parent link...");
        let result = self.authenticated_put(
            &["network", worker_key, "parent"],
            GunValue::Text(node_key.to_string()),
        ).await;
        log(&format!("[NetworkGraph::add_worker] result={:?}", result));
        result
    }

    async fn request_modify(&self, node_key: &str, field: &str, value: &str) -> Result<(), String> {
        log(&format!("[NetworkGraph::request_modify] node={}, field={}, value={}", node_key, field, value));
        let result = self.authenticated_put(
            &["network", node_key, "requests", field],
            GunValue::Text(value.to_string()),
        ).await;
        log(&format!("[NetworkGraph::request_modify] result={:?}", result));
        result
    }

    async fn set_gun_address(&self, node_key: &str, gun_address: &str) -> Result<(), String> {
        log(&format!("[NetworkGraph::set_gun_address] node={}, gun_address={}", node_key, gun_address));
        let result = self.authenticated_put(
            &["network", node_key, "gun_address"],
            GunValue::Text(gun_address.to_string()),
        ).await;
        log(&format!("[NetworkGraph::set_gun_address] result={:?}", result));
        result
    }

    async fn get_gun_address(&self, node_key: &str) -> Result<Option<String>, String> {
        log(&format!("[NetworkGraph::get_gun_address] node={}", node_key));
        let result = match self.db.get(&["network", node_key, "gun_address"]).await? {
            Some(GunValue::Text(s)) if !s.is_empty() => Ok(Some(unwrap_sea(&s))),
            _ => Ok(None),
        };
        log(&format!("[NetworkGraph::get_gun_address] result={:?}", result));
        result
    }

    async fn set_gun_relay_url(&self, node_key: &str, url: &str) -> Result<(), String> {
        log(&format!("[NetworkGraph::set_gun_relay_url] node={}, url={}", node_key, url));
        let result = self.authenticated_put(
            &["network", node_key, "gun_relay_url"],
            GunValue::Text(url.to_string()),
        ).await;
        log(&format!("[NetworkGraph::set_gun_relay_url] result={:?}", result));
        result
    }

    async fn get_gun_relay_url(&self, node_key: &str) -> Result<Option<String>, String> {
        log(&format!("[NetworkGraph::get_gun_relay_url] node={}", node_key));
        let result = match self.db.get(&["network", node_key, "gun_relay_url"]).await? {
            Some(GunValue::Text(s)) if !s.is_empty() => Ok(Some(unwrap_sea(&s))),
            _ => Ok(None),
        };
        log(&format!("[NetworkGraph::get_gun_relay_url] result={:?}", result));
        result
    }
}
