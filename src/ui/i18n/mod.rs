mod english;
mod hungarian;

use crate::i18n::Language;
use english::EnglishUi;
use hungarian::HungarianUi;

/// Trait for UI-related internationalized strings
pub trait UiI18n {
    // Initial signal values
    fn no_key_loaded(&self) -> &'static str;
    fn copy_label(&self) -> &'static str;
    fn copy_xdr_label(&self) -> &'static str;
    fn waiting(&self) -> &'static str;
    
    // Status messages
    fn err_no_generated_xdr(&self) -> &'static str;
    fn submitting(&self) -> &'static str;
    fn calling_faucet(&self) -> &'static str;
    fn no_loaded_key(&self) -> &'static str;
    fn fetching_sequence(&self) -> &'static str;
    
    // Console/println messages
    fn clipboard_cleared(&self) -> &'static str;
    fn save_success(&self) -> &'static str;
    fn nothing_to_save(&self) -> &'static str;
    fn loading_started(&self) -> &'static str;
    fn key_loaded_len(&self, len: usize) -> String;
    fn ui_updated_with_key(&self) -> &'static str;
    
    // Format helpers
    fn fmt_success(&self, msg: &str) -> String;
    fn fmt_error(&self, e: &str) -> String;
    fn fmt_xdr_ready(&self, net: &str, seq: i64) -> String;
    
    // Button labels
    fn btn_new_key(&self) -> &'static str;
    fn btn_import(&self) -> &'static str;
    fn btn_hide_secret(&self) -> &'static str;
    fn btn_reveal_secret(&self) -> &'static str;
    fn btn_activate_faucet(&self) -> &'static str;
    fn btn_save_to_os(&self) -> &'static str;
    fn btn_load(&self) -> &'static str;
    fn btn_generate_xdr(&self) -> &'static str;
    fn btn_submit_tx(&self) -> &'static str;
    
    // UI labels
    fn lbl_active_address(&self) -> &'static str;
    fn lbl_signed_xdr(&self) -> &'static str;
    fn lbl_import_ph(&self) -> &'static str;
    
    // Network labels
    fn net_testnet_label(&self) -> &'static str;
    fn net_mainnet_label(&self) -> &'static str;
    
    // Clipboard
    fn copied(&self) -> &'static str;
}

/// Factory function to get the appropriate UiI18n implementation
pub fn ui_i18n(lang: Language) -> Box<dyn UiI18n> {
    match lang {
        Language::English => Box::new(EnglishUi),
        Language::Hungarian => Box::new(HungarianUi),
    }
}
