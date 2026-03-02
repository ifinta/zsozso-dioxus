mod english;
mod french;
mod german;
mod hungarian;
mod spanish;

use crate::i18n::Language;
use english::EnglishStore;
use french::FrenchStore;
use german::GermanStore;
use hungarian::HungarianStore;
use spanish::SpanishStore;

/// Trait for store-related internationalized strings
pub trait StoreI18n {
    fn storage_error(&self, error: &str) -> String;
    fn save_error(&self, error: &str) -> String;
    fn load_error(&self, error: &str) -> String;
}

/// Factory function to get the appropriate StoreI18n implementation
pub fn store_i18n(lang: Language) -> Box<dyn StoreI18n> {
    match lang {
        Language::English => Box::new(EnglishStore),
        Language::French => Box::new(FrenchStore),
        Language::German => Box::new(GermanStore),
        Language::Hungarian => Box::new(HungarianStore),
        Language::Spanish => Box::new(SpanishStore),
    }
}
