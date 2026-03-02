mod english;
mod french;
mod german;
mod hungarian;
mod spanish;

use crate::i18n::Language;
use english::EnglishUi;
use french::FrenchUi;
use german::GermanUi;
use hungarian::HungarianUi;
use spanish::SpanishUi;

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
    fn clipboard_modal_text(&self) -> &'static str;
    fn btn_clear_clipboard(&self) -> &'static str;

    // Tab labels
    fn tab_home(&self) -> &'static str;
    fn tab_networking(&self) -> &'static str;
    fn tab_info(&self) -> &'static str;
    fn tab_settings(&self) -> &'static str;

    // Start gate modal
    fn gate_title(&self) -> &'static str;
    fn btn_next(&self) -> &'static str;

    // Passkey authentication
    fn authenticating(&self) -> &'static str;
    fn auth_failed(&self) -> &'static str;
    fn btn_exit(&self) -> &'static str;
    fn no_prf_key(&self) -> &'static str;

    // Info tab
    fn info_public_key_label(&self) -> &'static str;
    fn info_no_key(&self) -> &'static str;

    // Networking tab / Smart Contract
    fn btn_ping(&self) -> &'static str;
    fn ping_calling(&self) -> &'static str;
    fn ping_success(&self, msg: &str) -> String;
    fn ping_error(&self, e: &str) -> String;
    fn ping_no_key(&self) -> &'static str;

    // QR Scanner
    fn btn_scan_qr(&self) -> &'static str;
    fn scan_scanning(&self) -> &'static str;
    fn scan_success(&self, key: &str) -> String;
    fn scan_error(&self, e: &str) -> String;

    // Log tab
    fn tab_log(&self) -> &'static str;
    fn log_refresh(&self) -> &'static str;
    fn log_clear(&self) -> &'static str;

    // Update toast
    fn toast_update_available(&self) -> &'static str;
    fn btn_update_now(&self) -> &'static str;

    // Info tab – version
    fn info_version(&self, ver: &str) -> String;
}

/// Factory function to get the appropriate UiI18n implementation
pub fn ui_i18n(lang: Language) -> Box<dyn UiI18n> {
    match lang {
        Language::English => Box::new(EnglishUi),
        Language::French => Box::new(FrenchUi),
        Language::German => Box::new(GermanUi),
        Language::Hungarian => Box::new(HungarianUi),
        Language::Spanish => Box::new(SpanishUi),
    }
}
