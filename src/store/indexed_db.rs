use rexie::{Rexie, ObjectStore, TransactionMode};
use wasm_bindgen::JsValue;
use super::Store;
use crate::i18n::Language;
use super::i18n::store_i18n;

pub struct IndexedDbStore {
    key: String,
    language: Language,
}

impl IndexedDbStore {
    pub fn new(service: &str, account: &str, language: Language) -> Self {
        Self {
            key: format!("{}:{}", service, account),
            language,
        }
    }

    async fn get_db(&self) -> Result<Rexie, String> {
        let i18n = store_i18n(self.language);
        Rexie::builder("ZsozsoAuth")
            .version(1)
            .add_object_store(ObjectStore::new("Secrets"))
            .build()
            .await
            .map_err(|e| i18n.storage_error(&format!("IndexedDB init failed: {:?}", e)))
    }
}

impl Store for IndexedDbStore {
    async fn save(&self, secret: &str) -> Result<(), String> {
        let i18n = store_i18n(self.language);
        let db = self.get_db().await?;
        
        let transaction = db.transaction(&["Secrets"], TransactionMode::ReadWrite)
            .map_err(|_| i18n.save_error("Transaction failed"))?;
        
        let store = transaction.store("Secrets")
            .map_err(|_| i18n.save_error("Store not found"))?;

        store.put(&JsValue::from_str(secret), Some(&JsValue::from_str(&self.key)))
            .await
            .map_err(|_| i18n.save_error("Failed to write to IndexedDB"))?;
            
        transaction.done().await.map_err(|_| i18n.save_error("Commit failed"))?;
        Ok(())
    }

    async fn load(&self) -> Result<String, String> {
        let i18n = store_i18n(self.language);
        let db = self.get_db().await?;
        
        let transaction = db.transaction(&["Secrets"], TransactionMode::ReadOnly)
            .map_err(|_| i18n.load_error("Transaction failed"))?;
            
        let store = transaction.store("Secrets")
            .map_err(|_| i18n.load_error("Store not found"))?;

        let value = store.get(JsValue::from_str(&self.key))
            .await
            .map_err(|_| i18n.load_error("Read failed"))?;

        value
            .and_then(|v| v.as_string())
            .ok_or_else(|| i18n.load_error("No secret found in IndexedDB"))
    }
}
