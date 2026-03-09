use super::UiI18n;

pub struct HungarianUi;

impl UiI18n for HungarianUi {
    // Initial signal values
    fn no_key_loaded(&self) -> &'static str { "Nincs kulcs betöltve" }
    fn copy_label(&self) -> &'static str { "Másolás" }
    fn copy_xdr_label(&self) -> &'static str { "XDR Másolása" }
    fn waiting(&self) -> &'static str { "Várakozás..." }
    
    // Status messages
    fn err_no_generated_xdr(&self) -> &'static str { "Hiba: Nincs generált XDR!" }
    fn submitting(&self) -> &'static str { "Beküldés folyamatban..." }
    fn calling_faucet(&self) -> &'static str { "🚀 Faucet hívása..." }
    fn no_loaded_key(&self) -> &'static str { "⚠️ Nincs betöltött kulcs!" }
    fn fetching_sequence(&self) -> &'static str { "🔍 Szekvenciaszám lekérése..." }
    
    // Console/println messages
    fn clipboard_cleared(&self) -> &'static str { "🔐 Vágólap törölve a biztonság érdekében." }
    fn save_success(&self) -> &'static str { "✅ Sikeres mentés a rendszer-tárcába!" }
    fn nothing_to_save(&self) -> &'static str { "⚠️ Nincs mit menteni (üres a kulcs)!" }
    fn loading_started(&self) -> &'static str { "🔍 Betöltés megkezdése..." }
    fn key_loaded_len(&self, len: usize) -> String { format!("📥 Kulcs betöltve, hossza: {}", len) }
    fn ui_updated_with_key(&self) -> &'static str { "✨ UI sikeresen frissítve a betöltött kulccsal." }
    
    // Format helpers
    fn fmt_success(&self, msg: &str) -> String { format!("✅ SIKER! {}", msg) }
    fn fmt_error(&self, e: &str) -> String { format!("❌ {}", e) }
    fn fmt_xdr_ready(&self, net: &str, seq: i64) -> String { format!("✅ XDR Kész! [{}] (Seq: {})", net, seq) }
    
    // Button labels
    fn btn_new_key(&self) -> &'static str { "✨ Új Kulcs" }
    fn btn_import(&self) -> &'static str { "📥 Import" }
    fn btn_hide_secret(&self) -> &'static str { "🙈 Elrejtés" }
    fn btn_reveal_secret(&self) -> &'static str { "👁 Felfedés" }
    fn btn_activate_faucet(&self) -> &'static str { "🚀 Aktiválás (Faucet)" }
    fn btn_save_to_os(&self) -> &'static str { "💾 Mentés az OS tárcába" }
    fn btn_load(&self) -> &'static str { "🔓 Betöltés" }
    fn btn_generate_xdr(&self) -> &'static str { "🛠 Tranzakció XDR Generálása" }
    fn btn_submit_tx(&self) -> &'static str { "🚀 Tranzakció BEKÜLDÉSE" }
    
    // UI labels
    fn lbl_active_address(&self) -> &'static str { "Aktív Cím (Public Key):" }
    fn lbl_signed_xdr(&self) -> &'static str { "ALÁÍRT XDR:" }
    fn lbl_import_ph(&self) -> &'static str { "Importálás (S...)" }
    
    // Network labels
    fn net_testnet_label(&self) -> &'static str { "🧪 Testnet ⚠️" }
    fn net_mainnet_label(&self) -> &'static str { "Mainnet" }
    
    // Clipboard
    fn copied(&self) -> &'static str { "MÁSOLVA!" }
    fn clipboard_modal_text(&self) -> &'static str { "A tartalom a vágólapra került. Amikor bezárod ezt az ablakot, a vágólap tartalma biztonsági okokból törlődik." }
    fn btn_clear_clipboard(&self) -> &'static str { "🗑️ Törlöm a vágólap tartalmát" }

    // Tab labels
    fn tab_cyf(&self) -> &'static str { "CYF" }
    fn tab_networking(&self) -> &'static str { "Hálózat" }
    fn tab_info(&self) -> &'static str { "Infó" }
    fn tab_settings(&self) -> &'static str { "Beállítások" }

    // Start gate modal
    fn gate_title(&self) -> &'static str { "Üdvözöl a Zsozso" }
    fn btn_next(&self) -> &'static str { "Tovább" }

    // Passkey authentication
    fn authenticating(&self) -> &'static str { "Hitelesítés..." }
    fn auth_failed(&self) -> &'static str { "A hitelesítés sikertelen" }
    fn btn_exit(&self) -> &'static str { "Kilépés" }
    fn no_prf_key(&self) -> &'static str { "Nincs passkey titkosítási kulcs. Először hitelesítsd magad újra." }

    // Info tab
    fn info_public_key_label(&self) -> &'static str { "Publikus Kulcsod" }
    fn info_no_key(&self) -> &'static str { "Nincs betöltött kulcs. Generálj vagy importálj egyet a Beállításokban." }

    // Networking tab / Smart Contract
    fn btn_ping(&self) -> &'static str { "\u{1F3D3} Ping" }
    fn ping_calling(&self) -> &'static str { "\u{1F4E1} Szerződés hívása..." }
    fn ping_success(&self, msg: &str) -> String { format!("\u{2705} {}", msg) }
    fn ping_error(&self, e: &str) -> String { format!("\u{274C} {}", e) }
    fn ping_no_key(&self) -> &'static str { "\u{26A0}\u{FE0F} Előbb tölts be egy kulcsot (Beállítások fül)." }

    // QR Scanner
    fn btn_scan_qr(&self) -> &'static str { "\u{1F4F7} QR Szkennelés" }
    fn scan_scanning(&self) -> &'static str { "\u{1F4F7} Szkennelés..." }
    fn scan_success(&self, key: &str) -> String { format!("\u{2705} Beolvasva: {}", key) }
    fn scan_error(&self, e: &str) -> String { format!("\u{274C} Szkennelés sikertelen: {}", e) }

    // Log tab
    fn tab_log(&self) -> &'static str { "Napló" }
    fn log_refresh(&self) -> &'static str { "\u{1F504} Frissítés" }
    fn log_clear(&self) -> &'static str { "\u{1F5D1} Törlés" }
    fn log_upload(&self) -> &'static str { "\u{2B06}\u{FE0F} Feltöltés" }
    fn log_uploading(&self) -> &'static str { "Feltöltés..." }
    fn log_upload_ok(&self) -> &'static str { "\u{2705} Napló feltöltve" }
    fn log_upload_fail(&self, e: &str) -> String { format!("\u{274C} Feltöltés sikertelen: {}", e) }
    fn log_upload_empty(&self) -> &'static str { "\u{26A0}\u{FE0F} A napló üres" }

    // Update toast
    fn toast_update_available(&self) -> &'static str { "\u{1F680} A Zsozso új verziója elérhető!" }
    fn btn_update_now(&self) -> &'static str { "Frissítés most" }

    // Info tab – version
    fn info_version(&self, ver: &str) -> String { format!("Verzió: {}", ver) }

    // Network switch modal
    fn network_switch_save_prompt(&self) -> &'static str { "Van betöltött titkos kulcsod. Szeretnéd elmenteni a hálózatváltás előtt?" }
    fn btn_save_and_switch(&self) -> &'static str { "\u{1F4BE} Mentés és váltás" }
    fn btn_switch_and_save(&self) -> &'static str { "\u{1F504} Váltás és mentés" }
    fn btn_switch_without_saving(&self) -> &'static str { "Váltás mentés nélkül" }
    fn btn_cancel(&self) -> &'static str { "Mégse" }

    // SEA key generation modal
    fn btn_generate_db_secret(&self) -> &'static str { "\u{1F511} DB Titok Generálása" }
    fn sea_modal_title(&self) -> &'static str { "GunDB SEA Kulcsok Generálása" }
    fn sea_modal_placeholder(&self) -> &'static str { "Írd be a titkos jelszót..." }
    fn btn_generate_db_keys(&self) -> &'static str { "\u{1F511} DB Kulcsok Generálása" }
    fn sea_generating(&self) -> &'static str { "\u{1F504} Kulcsok generálása..." }
    fn sea_keys_generated(&self) -> &'static str { "\u{2705} SEA kulcsok generálva és betöltve a memóriába." }
    fn sea_generation_error(&self, e: &str) -> String { format!("\u{274C} Kulcsgenerálás sikertelen: {}", e) }
    fn btn_close(&self) -> &'static str { "Bezárás" }

    // Biometric identification toggle
    fn lbl_biometric(&self) -> &'static str { "Biometrikus azonosítás" }
    fn lbl_biometric_desc(&self) -> &'static str { "Biometrikus hitelesítés használata a tárca védelméhez" }
    fn biometric_required_to_save(&self) -> &'static str { "Kérjük, először kapcsold be a Biometrikus azonosítást a Beállításokban a titok mentése előtt." }

    // Nickname (Settings)
    fn lbl_nickname(&self) -> &'static str { "Becenév" }
    fn lbl_nickname_ph(&self) -> &'static str { "Írd be a beceneved..." }
    fn btn_save_nickname(&self) -> &'static str { "\u{1F4BE} Mentés" }
    fn nickname_saved(&self) -> &'static str { "\u{2705} Becenév mentve!" }
    fn nickname_save_error(&self, e: &str) -> String { format!("\u{274C} Becenév mentése sikertelen: {}", e) }

    // Network tab – hierarchy
    fn lbl_parents(&self) -> &'static str { "Szülők" }
    fn lbl_workers(&self) -> &'static str { "Munkatársak" }
    fn btn_new_worker(&self) -> &'static str { "\u{2795} Új" }
    fn network_no_key(&self) -> &'static str { "\u{26A0}\u{FE0F} Előbb tölts be egy kulcsot (Beállítások fül)." }
    fn network_add_worker_success(&self, key: &str) -> String { format!("\u{2705} Munkatárs hozzáadva: {}", key) }
    fn network_add_worker_error(&self, e: &str) -> String { format!("\u{274C} Munkatárs hozzáadása sikertelen: {}", e) }

    // CYF tab
    fn btn_burn(&self) -> &'static str { "\u{1F525} Égetés" }
    fn btn_mint(&self) -> &'static str { "\u{1FA99} Kibocsátás" }
    fn btn_ok(&self) -> &'static str { "OK" }
    fn cyf_not_implemented(&self, fn_name: &str) -> String { format!("A(z) {} funkció még nincs implementálva.", fn_name) }

    // GUN node address
    fn lbl_gun_address(&self) -> &'static str { "GUN csomópont cím" }
    fn lbl_gun_relay_url(&self) -> &'static str { "GUN Relay URL" }
    fn lbl_gun_relay_ph(&self) -> &'static str { "https://your-server.com/gun" }
    fn btn_save_gun_relay(&self) -> &'static str { "Mentés" }
}
