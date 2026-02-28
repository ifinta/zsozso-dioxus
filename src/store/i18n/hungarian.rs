use super::StoreI18n;

pub struct HungarianStore;

impl StoreI18n for HungarianStore {
    fn storage_error(&self, error: &str) -> String { format!("Tároló hiba: {:?}", error) }
    fn save_error(&self, error: &str) -> String { format!("Mentési hiba: {:?}", error) }
    fn load_error(&self, error: &str) -> String { format!("Betöltési hiba: {:?}", error) }
}
