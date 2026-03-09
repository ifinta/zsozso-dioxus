use crate::i18n::Language;
use super::gundb::{Db, GunDb, GunConfig, GunValue, SeaKeyPair};

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
}

/// Concrete implementation backed by the GUN decentralised database.
pub struct GunNetworkGraph {
    db: GunDb,
    sea_pair: Option<SeaKeyPair>,
}

impl GunNetworkGraph {
    pub fn new(language: Language, sea_pair: Option<SeaKeyPair>) -> Self {
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
        match &self.sea_pair {
            Some(pair) => self.db.put_signed(path, value, pair).await,
            None => Err("SEA keypair required for authenticated writes".to_string()),
        }
    }
}

impl NetworkGraph for GunNetworkGraph {
    async fn get_parent(&self, node_key: &str) -> Result<Option<String>, String> {
        match self.db.get(&["network", node_key, "parent"]).await? {
            Some(GunValue::Text(s)) if !s.is_empty() => Ok(Some(s)),
            _ => Ok(None),
        }
    }

    async fn get_ancestors(&self, node_key: &str, depth: usize) -> Result<Vec<String>, String> {
        let mut ancestors = Vec::new();
        let mut current = node_key.to_string();
        for _ in 0..depth {
            match self.get_parent(&current).await? {
                Some(parent) => {
                    ancestors.push(parent.clone());
                    current = parent;
                }
                None => break,
            }
        }
        Ok(ancestors)
    }

    async fn get_workers(&self, node_key: &str) -> Result<Vec<String>, String> {
        match self.db.get(&["network", node_key, "workers"]).await? {
            Some(GunValue::Node(map)) => {
                Ok(map.into_keys().collect())
            }
            _ => Ok(vec![]),
        }
    }

    async fn get_nickname(&self, node_key: &str) -> Result<Option<String>, String> {
        match self.db.get(&["network", node_key, "nickname"]).await? {
            Some(GunValue::Text(s)) if !s.is_empty() => Ok(Some(s)),
            _ => Ok(None),
        }
    }

    async fn set_nickname(&self, node_key: &str, nickname: &str) -> Result<(), String> {
        let trimmed: String = nickname.chars().take(MAX_NICKNAME_LEN).collect();
        self.authenticated_put(
            &["network", node_key, "nickname"],
            GunValue::Text(trimmed),
        ).await
    }

    async fn add_worker(&self, node_key: &str, worker_key: &str) -> Result<(), String> {
        self.authenticated_put(
            &["network", node_key, "workers", worker_key],
            GunValue::Bool(true),
        ).await?;
        self.authenticated_put(
            &["network", worker_key, "parent"],
            GunValue::Text(node_key.to_string()),
        ).await
    }

    async fn request_modify(&self, node_key: &str, field: &str, value: &str) -> Result<(), String> {
        self.authenticated_put(
            &["network", node_key, "requests", field],
            GunValue::Text(value.to_string()),
        ).await
    }
}
