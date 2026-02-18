use ::keyring::Entry;
use super::Store;
use crate::i18n::Language;
use super::i18n::{StoreI18n, store_i18n};

/// Desktop platformokon a rendszer kulcstárcáját használja (GNOME Keyring, macOS Keychain, Windows Credential Manager).
pub struct KeyringStore {
    service: &'static str,
    account: &'static str,
    language: Language,
}

impl KeyringStore {
    pub fn new(service: &'static str, account: &'static str, language: Language) -> Self {
        Self { service, account, language }
    }
}

impl Store for KeyringStore {
    fn save(&self, secret: &str) -> Result<(), String> {
        let i18n = store_i18n(self.language);
        let entry = Entry::new(self.service, self.account)
            .map_err(|e| i18n.storage_error(&e.to_string()))?;
        let _ = entry.delete_credential();
        entry.set_password(secret)
            .map_err(|e| i18n.save_error(&e.to_string()))
    }

    fn load(&self) -> Result<String, String> {
        let i18n = store_i18n(self.language);
        let entry = Entry::new(self.service, self.account)
            .map_err(|e| i18n.storage_error(&e.to_string()))?;
        entry.get_password()
            .map_err(|e| i18n.load_error(&e.to_string()))
    }
}
