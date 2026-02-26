use super::UiI18n;

pub struct EnglishUi;

impl UiI18n for EnglishUi {
    // Initial signal values
    fn no_key_loaded(&self) -> &'static str {
        "No key loaded"
    }

    fn copy_label(&self) -> &'static str {
        "Copy"
    }

    fn copy_xdr_label(&self) -> &'static str {
        "Copy XDR"
    }

    fn waiting(&self) -> &'static str {
        "Waiting..."
    }
    
    // Status messages
    fn err_no_generated_xdr(&self) -> &'static str {
        "Error: No generated XDR!"
    }

    fn submitting(&self) -> &'static str {
        "Submitting..."
    }

    fn calling_faucet(&self) -> &'static str {
        "🚀 Calling faucet..."
    }

    fn no_loaded_key(&self) -> &'static str {
        "⚠️ No loaded key!"
    }

    fn fetching_sequence(&self) -> &'static str {
        "🔍 Fetching sequence number..."
    }
    
    // Console/println messages
    fn clipboard_cleared(&self) -> &'static str {
        "🔐 Clipboard cleared for security."
    }

    fn save_success(&self) -> &'static str {
        "✅ Successfully saved to system wallet!"
    }

    fn nothing_to_save(&self) -> &'static str {
        "⚠️ Nothing to save (key is empty)!"
    }

    fn loading_started(&self) -> &'static str {
        "🔍 Loading started..."
    }

    fn key_loaded_len(&self, len: usize) -> String {
        format!("📥 Key loaded, length: {}", len)
    }

    fn ui_updated_with_key(&self) -> &'static str {
        "✨ UI successfully updated with loaded key."
    }
    
    // Format helpers
    fn fmt_success(&self, msg: &str) -> String {
        format!("✅ SUCCESS! {}", msg)
    }

    fn fmt_error(&self, e: &str) -> String {
        format!("❌ {}", e)
    }

    fn fmt_xdr_ready(&self, net: &str, seq: i64) -> String {
        format!("✅ XDR Ready! [{}] (Seq: {})", net, seq)
    }
    
    // Button labels
    fn btn_new_key(&self) -> &'static str {
        "✨ New Key"
    }

    fn btn_import(&self) -> &'static str {
        "📥 Import"
    }

    fn btn_hide_secret(&self) -> &'static str {
        "🙈 Hide"
    }

    fn btn_reveal_secret(&self) -> &'static str {
        "👁 Reveal"
    }

    fn btn_activate_faucet(&self) -> &'static str {
        "🚀 Activate (Faucet)"
    }

    fn btn_save_to_os(&self) -> &'static str {
        "💾 Save to OS Wallet"
    }

    fn btn_load(&self) -> &'static str {
        "🔓 Load"
    }

    fn btn_generate_xdr(&self) -> &'static str {
        "🛠 Generate Transaction XDR"
    }

    fn btn_submit_tx(&self) -> &'static str {
        "🚀 SUBMIT Transaction"
    }
    
    // UI labels
    fn lbl_active_address(&self) -> &'static str {
        "Active Address (Public Key):"
    }

    fn lbl_signed_xdr(&self) -> &'static str {
        "SIGNED XDR:"
    }

    fn lbl_import_ph(&self) -> &'static str {
        "Import (S...)"
    }
    
    // Network labels
    fn net_testnet_label(&self) -> &'static str {
        "🧪 Testnet ⚠️"
    }

    fn net_mainnet_label(&self) -> &'static str {
        "Mainnet"
    }
    
    // Clipboard
    fn copied(&self) -> &'static str {
        "COPIED!"
    }

    fn clipboard_modal_text(&self) -> &'static str {
        "The content has been copied to the clipboard. When you close this dialog, the clipboard will be cleared for security."
    }

    fn btn_clear_clipboard(&self) -> &'static str {
        "🗑️ Clear clipboard content"
    }
}
