mod english;
mod hungarian;

use crate::i18n::Language;
use english::EnglishSc;
use hungarian::HungarianSc;

/// Trait for smart-contract–related internationalized strings
pub trait ScI18n {
    fn rpc_unreachable(&self, error: &str) -> String;
    fn simulation_failed(&self, detail: &str) -> String;
    fn tx_submission_failed(&self, detail: &str) -> String;
    fn tx_pending(&self) -> &'static str;
    fn tx_success(&self) -> &'static str;
    fn tx_failed(&self, detail: &str) -> String;
    fn tx_not_found(&self) -> &'static str;
    fn invalid_response(&self, detail: &str) -> String;
    fn contract_error(&self, detail: &str) -> String;
}

/// Factory function to get the appropriate ScI18n implementation
pub fn sc_i18n(lang: Language) -> Box<dyn ScI18n> {
    match lang {
        Language::English => Box::new(EnglishSc),
        Language::Hungarian => Box::new(HungarianSc),
    }
}
