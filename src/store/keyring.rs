use ::keyring::Entry;
use super::Store;

/// Desktop platformokon a rendszer kulcstárcáját használja (GNOME Keyring, macOS Keychain, Windows Credential Manager).
pub struct KeyringStore {
    service: &'static str,
    account: &'static str,
}

impl KeyringStore {
    pub fn new(service: &'static str, account: &'static str) -> Self {
        Self { service, account }
    }
}

impl Store for KeyringStore {
    fn save(&self, secret: &str) -> Result<(), String> {
        let entry = Entry::new(self.service, self.account)
            .map_err(|e| format!("Tároló hiba: {:?}", e))?;
        let _ = entry.delete_credential();
        entry.set_password(secret)
            .map_err(|e| format!("Mentési hiba: {:?}", e))
    }

    fn load(&self) -> Result<String, String> {
        let entry = Entry::new(self.service, self.account)
            .map_err(|e| format!("Tároló hiba: {:?}", e))?;
        entry.get_password()
            .map_err(|e| format!("Betöltési hiba: {:?}", e))
    }
}
