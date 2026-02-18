use super::UiI18n;

pub struct HungarianUi;

impl UiI18n for HungarianUi {
    // Initial signal values
    fn no_key_loaded(&self) -> &'static str {
        "Nincs kulcs betöltve"
    }

    fn copy_label(&self) -> &'static str {
        "Másolás"
    }

    fn copy_xdr_label(&self) -> &'static str {
        "XDR Másolása"
    }

    fn waiting(&self) -> &'static str {
        "Várakozás..."
    }
    
    // Status messages
    fn err_no_generated_xdr(&self) -> &'static str {
        "Hiba: Nincs generált XDR!"
    }

    fn submitting(&self) -> &'static str {
        "Beküldés folyamatban..."
    }

    fn calling_faucet(&self) -> &'static str {
        "🚀 Faucet hívása..."
    }

    fn no_loaded_key(&self) -> &'static str {
        "⚠️ Nincs betöltött kulcs!"
    }

    fn fetching_sequence(&self) -> &'static str {
        "🔍 Szekvenciaszám lekérése..."
    }
    
    // Console/println messages
    fn clipboard_cleared(&self) -> &'static str {
        "🔐 Vágólap törölve a biztonság érdekében."
    }

    fn save_success(&self) -> &'static str {
        "✅ Sikeres mentés a rendszer-tárcába!"
    }

    fn nothing_to_save(&self) -> &'static str {
        "⚠️ Nincs mit menteni (üres a kulcs)!"
    }

    fn loading_started(&self) -> &'static str {
        "🔍 Betöltés megkezdése..."
    }

    fn key_loaded_len(&self, len: usize) -> String {
        format!("📥 Kulcs betöltve, hossza: {}", len)
    }

    fn ui_updated_with_key(&self) -> &'static str {
        "✨ UI sikeresen frissítve a betöltött kulccsal."
    }
    
    // Format helpers
    fn fmt_success(&self, msg: impl std::fmt::Display) -> String {
        format!("✅ SIKER! {}", msg)
    }

    fn fmt_error(&self, e: impl std::fmt::Display) -> String {
        format!("❌ {}", e)
    }

    fn fmt_xdr_ready(&self, net: &str, seq: i64) -> String {
        format!("✅ XDR Kész! [{}] (Seq: {})", net, seq)
    }
    
    // Button labels
    fn btn_new_key(&self) -> &'static str {
        "✨ Új Kulcs"
    }

    fn btn_import(&self) -> &'static str {
        "📥 Import"
    }

    fn btn_hide_secret(&self) -> &'static str {
        "🙈 Elrejtés"
    }

    fn btn_reveal_secret(&self) -> &'static str {
        "👁 Felfedés"
    }

    fn btn_activate_faucet(&self) -> &'static str {
        "🚀 Aktiválás (Faucet)"
    }

    fn btn_save_to_os(&self) -> &'static str {
        "💾 Mentés az OS tárcába"
    }

    fn btn_load(&self) -> &'static str {
        "🔓 Betöltés"
    }

    fn btn_generate_xdr(&self) -> &'static str {
        "🛠 Tranzakció XDR Generálása"
    }

    fn btn_submit_tx(&self) -> &'static str {
        "🚀 Tranzakció BEKÜLDÉSE"
    }
    
    // UI labels
    fn lbl_active_address(&self) -> &'static str {
        "Aktív Cím (Public Key):"
    }

    fn lbl_signed_xdr(&self) -> &'static str {
        "ALÁÍRT XDR:"
    }

    fn lbl_import_ph(&self) -> &'static str {
        "Importálás (S...)"
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
        "MÁSOLVA!"
    }
}
