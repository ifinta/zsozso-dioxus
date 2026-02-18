use super::StoreI18n;

pub struct EnglishStore;

impl StoreI18n for EnglishStore {
    fn storage_error(&self, error: impl std::fmt::Display) -> String {
        format!("Storage error: {:?}", error)
    }

    fn save_error(&self, error: impl std::fmt::Display) -> String {
        format!("Save error: {:?}", error)
    }

    fn load_error(&self, error: impl std::fmt::Display) -> String {
        format!("Load error: {:?}", error)
    }
}
