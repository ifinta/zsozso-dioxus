mod english;
mod hungarian;

use crate::i18n::Language;
use english::EnglishLedger;
use hungarian::HungarianLedger;

/// Trait for ledger-related internationalized strings
pub trait LedgerI18n {
    fn faucet_unavailable(&self) -> &'static str;
    fn account_activated(&self) -> &'static str;
    fn faucet_error(&self, status: impl std::fmt::Display) -> String;
    fn network_error(&self, error: impl std::fmt::Display) -> String;
    fn invalid_secret_key(&self) -> &'static str;
    fn horizon_unreachable(&self, error: impl std::fmt::Display) -> String;
    fn account_not_found(&self) -> &'static str;
    fn json_error(&self, error: impl std::fmt::Display) -> String;
    fn xdr_serial_error(&self, error: impl std::fmt::Display) -> String;
    fn xdr_error(&self, error: impl std::fmt::Display) -> String;
    fn tx_accepted(&self) -> &'static str;
    fn error(&self, status: impl std::fmt::Display) -> String;
}

/// Factory function to get the appropriate LedgerI18n implementation
pub fn ledger_i18n(lang: Language) -> Box<dyn LedgerI18n> {
    match lang {
        Language::English => Box::new(EnglishLedger),
        Language::Hungarian => Box::new(HungarianLedger),
    }
}
