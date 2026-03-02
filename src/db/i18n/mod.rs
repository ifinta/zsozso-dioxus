mod english;
mod french;
mod german;
mod hungarian;
mod spanish;

use crate::i18n::Language;
use english::EnglishDb;
use french::FrenchDb;
use german::GermanDb;
use hungarian::HungarianDb;
use spanish::SpanishDb;

/// Trait for db-related internationalized strings
pub trait DbI18n {
    fn read_error(&self, error: &str) -> String;
    fn write_error(&self, error: &str) -> String;
    fn subscribe_error(&self, error: &str) -> String;
    fn connection_error(&self, error: &str) -> String;
    fn sea_error(&self, error: &str) -> String;
}

/// Factory function to get the appropriate DbI18n implementation
pub fn db_i18n(lang: Language) -> Box<dyn DbI18n> {
    match lang {
        Language::English => Box::new(EnglishDb),
        Language::French => Box::new(FrenchDb),
        Language::German => Box::new(GermanDb),
        Language::Hungarian => Box::new(HungarianDb),
        Language::Spanish => Box::new(SpanishDb),
    }
}
