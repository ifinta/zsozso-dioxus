use super::UiI18n;

pub struct GermanUi;

impl UiI18n for GermanUi {
    // Initial signal values
    fn no_key_loaded(&self) -> &'static str { "Kein Schlüssel geladen" }
    fn copy_label(&self) -> &'static str { "Kopieren" }
    fn copy_xdr_label(&self) -> &'static str { "XDR kopieren" }
    fn waiting(&self) -> &'static str { "Warten..." }
    
    // Status messages
    fn err_no_generated_xdr(&self) -> &'static str { "Fehler: Kein generiertes XDR!" }
    fn submitting(&self) -> &'static str { "Wird gesendet..." }
    fn calling_faucet(&self) -> &'static str { "🚀 Faucet wird aufgerufen..." }
    fn no_loaded_key(&self) -> &'static str { "⚠️ Kein Schlüssel geladen!" }
    fn fetching_sequence(&self) -> &'static str { "🔍 Sequenznummer wird abgerufen..." }
    
    // Console/println messages
    fn clipboard_cleared(&self) -> &'static str { "🔐 Zwischenablage aus Sicherheitsgründen geleert." }
    fn save_success(&self) -> &'static str { "✅ Erfolgreich in der System-Wallet gespeichert!" }
    fn nothing_to_save(&self) -> &'static str { "⚠️ Nichts zu speichern (Schlüssel ist leer)!" }
    fn loading_started(&self) -> &'static str { "🔍 Laden gestartet..." }
    fn key_loaded_len(&self, len: usize) -> String { format!("📥 Schlüssel geladen, Länge: {}", len) }
    fn ui_updated_with_key(&self) -> &'static str { "✨ Oberfläche erfolgreich mit geladenem Schlüssel aktualisiert." }
    
    // Format helpers
    fn fmt_success(&self, msg: &str) -> String { format!("✅ ERFOLG! {}", msg) }
    fn fmt_error(&self, e: &str) -> String { format!("❌ {}", e) }
    fn fmt_xdr_ready(&self, net: &str, seq: i64) -> String { format!("✅ XDR bereit! [{}] (Seq: {})", net, seq) }
    
    // Button labels
    fn btn_new_key(&self) -> &'static str { "✨ Neuer Schlüssel" }
    fn btn_import(&self) -> &'static str { "📥 Importieren" }
    fn btn_hide_secret(&self) -> &'static str { "🙈 Verbergen" }
    fn btn_reveal_secret(&self) -> &'static str { "👁 Anzeigen" }
    fn btn_activate_faucet(&self) -> &'static str { "🚀 Aktivieren (Faucet)" }
    fn btn_save_to_os(&self) -> &'static str { "💾 In OS-Wallet speichern" }
    fn btn_load(&self) -> &'static str { "🔓 Laden" }
    fn btn_generate_xdr(&self) -> &'static str { "🛠 Transaktions-XDR generieren" }
    fn btn_submit_tx(&self) -> &'static str { "🚀 Transaktion SENDEN" }
    
    // UI labels
    fn lbl_active_address(&self) -> &'static str { "Aktive Adresse (Öffentlicher Schlüssel):" }
    fn lbl_signed_xdr(&self) -> &'static str { "SIGNIERTES XDR:" }
    fn lbl_import_ph(&self) -> &'static str { "Importieren (S...)" }
    
    // Network labels
    fn net_testnet_label(&self) -> &'static str { "🧪 Testnet ⚠️" }
    fn net_mainnet_label(&self) -> &'static str { "Mainnet" }
    
    // Clipboard
    fn copied(&self) -> &'static str { "KOPIERT!" }
    fn clipboard_modal_text(&self) -> &'static str { "Der Inhalt wurde in die Zwischenablage kopiert. Beim Schließen dieses Fensters wird die Zwischenablage aus Sicherheitsgründen geleert." }
    fn btn_clear_clipboard(&self) -> &'static str { "🗑️ Zwischenablage leeren" }

    // Tab labels
    fn tab_home(&self) -> &'static str { "Startseite" }
    fn tab_networking(&self) -> &'static str { "Netzwerk" }
    fn tab_info(&self) -> &'static str { "Info" }
    fn tab_settings(&self) -> &'static str { "Einstellungen" }

    // Start gate modal
    fn gate_title(&self) -> &'static str { "Willkommen bei Zsozso" }
    fn btn_next(&self) -> &'static str { "Weiter" }

    // Passkey authentication
    fn authenticating(&self) -> &'static str { "Authentifizierung..." }
    fn auth_failed(&self) -> &'static str { "Authentifizierung fehlgeschlagen" }
    fn btn_exit(&self) -> &'static str { "Beenden" }
    fn no_prf_key(&self) -> &'static str { "Kein Passkey-Verschlüsselungsschlüssel verfügbar. Bitte zuerst erneut authentifizieren." }

    // Info tab
    fn info_public_key_label(&self) -> &'static str { "Ihr öffentlicher Schlüssel" }
    fn info_no_key(&self) -> &'static str { "Kein Schlüssel geladen. Generieren oder importieren Sie einen unter Einstellungen." }

    // Networking tab / Smart Contract
    fn btn_ping(&self) -> &'static str { "\u{1F3D3} Ping" }
    fn ping_calling(&self) -> &'static str { "\u{1F4E1} Vertrag wird aufgerufen..." }
    fn ping_success(&self, msg: &str) -> String { format!("\u{2705} {}", msg) }
    fn ping_error(&self, e: &str) -> String { format!("\u{274C} {}", e) }
    fn ping_no_key(&self) -> &'static str { "\u{26A0}\u{FE0F} Laden Sie zuerst einen Schlüssel (Tab Einstellungen)." }

    // QR Scanner
    fn btn_scan_qr(&self) -> &'static str { "\u{1F4F7} QR scannen" }
    fn scan_scanning(&self) -> &'static str { "\u{1F4F7} Wird gescannt..." }
    fn scan_success(&self, key: &str) -> String { format!("\u{2705} Gescannt: {}", key) }
    fn scan_error(&self, e: &str) -> String { format!("\u{274C} Scan fehlgeschlagen: {}", e) }

    // Log tab
    fn tab_log(&self) -> &'static str { "Protokoll" }
    fn log_refresh(&self) -> &'static str { "\u{1F504} Aktualisieren" }
    fn log_clear(&self) -> &'static str { "\u{1F5D1} Löschen" }

    // Update toast
    fn toast_update_available(&self) -> &'static str { "\u{1F680} Eine neue Version von Zsozso ist verfügbar!" }
    fn btn_update_now(&self) -> &'static str { "Jetzt aktualisieren" }

    // Info tab – version
    fn info_version(&self, ver: &str) -> String { format!("Version: {}", ver) }
}
