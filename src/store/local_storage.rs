use super::Store;
use crate::i18n::Language;
use super::i18n::store_i18n;

/// On web platforms, uses the browser's localStorage API.
/// The secret is stored under a namespaced key: "{service}:{account}".
///
/// Security note: localStorage is accessible to any JS running on the same origin.
/// This is comparable to how most browser-based wallets store secrets client-side.
/// The main threat is XSS — ensure the app does not load untrusted third-party scripts.
pub struct LocalStorageStore {
    key: String,
    language: Language,
}

impl LocalStorageStore {
    pub fn new(service: &str, account: &str, language: Language) -> Self {
        Self {
            key: format!("{}:{}", service, account),
            language,
        }
    }

    fn local_storage(&self) -> Result<web_sys::Storage, String> {
        let i18n = store_i18n(self.language);
        let window = web_sys::window()
            .ok_or_else(|| i18n.storage_error("No window object available"))?;
        window.local_storage()
            .map_err(|_| i18n.storage_error("localStorage access denied"))?
            .ok_or_else(|| i18n.storage_error("localStorage not available"))
    }
}

impl Store for LocalStorageStore {
    fn save(&self, secret: &str) -> Result<(), String> {
        let i18n = store_i18n(self.language);
        let storage = self.local_storage()?;
        storage.set_item(&self.key, secret)
            .map_err(|_| i18n.save_error("Failed to write to localStorage"))
    }

    fn load(&self) -> Result<String, String> {
        let i18n = store_i18n(self.language);
        let storage = self.local_storage()?;
        storage.get_item(&self.key)
            .map_err(|_| i18n.load_error("Failed to read from localStorage"))?
            .ok_or_else(|| i18n.load_error("No secret found in localStorage"))
    }
}
