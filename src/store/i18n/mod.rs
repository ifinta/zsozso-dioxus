mod english;
mod hungarian;

use crate::i18n::Language;
use english::EnglishStore;
use hungarian::HungarianStore;

/// Trait for store-related internationalized strings
pub trait StoreI18n {
    fn storage_error(&self, error: impl std::fmt::Display) -> String;
    fn save_error(&self, error: impl std::fmt::Display) -> String;
    fn load_error(&self, error: impl std::fmt::Display) -> String;
}

/// Factory function to get the appropriate StoreI18n implementation
pub fn store_i18n(lang: Language) -> Box<dyn StoreI18n> {
    match lang {
        Language::English => Box::new(EnglishStore),
        Language::Hungarian => Box::new(HungarianStore),
    }
}
