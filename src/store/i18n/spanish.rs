use super::StoreI18n;

pub struct SpanishStore;

impl StoreI18n for SpanishStore {
    fn storage_error(&self, error: &str) -> String { format!("Error de almacenamiento: {:?}", error) }
    fn save_error(&self, error: &str) -> String { format!("Error al guardar: {:?}", error) }
    fn load_error(&self, error: &str) -> String { format!("Error al cargar: {:?}", error) }
}
