use super::StoreI18n;

pub struct FrenchStore;

impl StoreI18n for FrenchStore {
    fn storage_error(&self, error: &str) -> String { format!("Erreur de stockage : {:?}", error) }
    fn save_error(&self, error: &str) -> String { format!("Erreur d'enregistrement : {:?}", error) }
    fn load_error(&self, error: &str) -> String { format!("Erreur de chargement : {:?}", error) }
}
