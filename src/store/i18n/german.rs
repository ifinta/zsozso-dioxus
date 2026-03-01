use super::StoreI18n;

pub struct GermanStore;

impl StoreI18n for GermanStore {
    fn storage_error(&self, error: &str) -> String { format!("Speicherfehler: {:?}", error) }
    fn save_error(&self, error: &str) -> String { format!("Fehler beim Speichern: {:?}", error) }
    fn load_error(&self, error: &str) -> String { format!("Fehler beim Laden: {:?}", error) }
}
