use super::UiI18n;

pub struct SpanishUi;

impl UiI18n for SpanishUi {
    // Initial signal values
    fn no_key_loaded(&self) -> &'static str { "Ninguna clave cargada" }
    fn copy_label(&self) -> &'static str { "Copiar" }
    fn copy_xdr_label(&self) -> &'static str { "Copiar XDR" }
    fn waiting(&self) -> &'static str { "Esperando..." }
    
    // Status messages
    fn err_no_generated_xdr(&self) -> &'static str { "Error: ¡No hay XDR generado!" }
    fn submitting(&self) -> &'static str { "Enviando..." }
    fn calling_faucet(&self) -> &'static str { "🚀 Llamando al faucet..." }
    fn no_loaded_key(&self) -> &'static str { "⚠️ ¡No hay clave cargada!" }
    fn fetching_sequence(&self) -> &'static str { "🔍 Obteniendo número de secuencia..." }
    
    // Console/println messages
    fn clipboard_cleared(&self) -> &'static str { "🔐 Portapapeles limpiado por seguridad." }
    fn save_success(&self) -> &'static str { "✅ ¡Guardado exitosamente en la billetera del sistema!" }
    fn nothing_to_save(&self) -> &'static str { "⚠️ ¡Nada que guardar (la clave está vacía)!" }
    fn loading_started(&self) -> &'static str { "🔍 Carga iniciada..." }
    fn key_loaded_len(&self, len: usize) -> String { format!("📥 Clave cargada, longitud: {}", len) }
    fn ui_updated_with_key(&self) -> &'static str { "✨ Interfaz actualizada con la clave cargada." }
    
    // Format helpers
    fn fmt_success(&self, msg: &str) -> String { format!("✅ ¡ÉXITO! {}", msg) }
    fn fmt_error(&self, e: &str) -> String { format!("❌ {}", e) }
    fn fmt_xdr_ready(&self, net: &str, seq: i64) -> String { format!("✅ ¡XDR listo! [{}] (Sec: {})", net, seq) }
    
    // Button labels
    fn btn_new_key(&self) -> &'static str { "✨ Nueva clave" }
    fn btn_import(&self) -> &'static str { "📥 Importar" }
    fn btn_hide_secret(&self) -> &'static str { "🙈 Ocultar" }
    fn btn_reveal_secret(&self) -> &'static str { "👁 Revelar" }
    fn btn_activate_faucet(&self) -> &'static str { "🚀 Activar (Faucet)" }
    fn btn_save_to_os(&self) -> &'static str { "💾 Guardar en billetera del SO" }
    fn btn_load(&self) -> &'static str { "🔓 Cargar" }
    fn btn_generate_xdr(&self) -> &'static str { "🛠 Generar transacción XDR" }
    fn btn_submit_tx(&self) -> &'static str { "🚀 ENVIAR Transacción" }
    
    // UI labels
    fn lbl_active_address(&self) -> &'static str { "Dirección activa (clave pública):" }
    fn lbl_signed_xdr(&self) -> &'static str { "XDR FIRMADO:" }
    fn lbl_import_ph(&self) -> &'static str { "Importar (S...)" }
    
    // Network labels
    fn net_testnet_label(&self) -> &'static str { "🧪 Testnet ⚠️" }
    fn net_mainnet_label(&self) -> &'static str { "Mainnet" }
    
    // Clipboard
    fn copied(&self) -> &'static str { "¡COPIADO!" }
    fn clipboard_modal_text(&self) -> &'static str { "El contenido se ha copiado al portapapeles. Al cerrar este diálogo, el portapapeles se limpiará por seguridad." }
    fn btn_clear_clipboard(&self) -> &'static str { "🗑️ Limpiar portapapeles" }

    // Tab labels
    fn tab_home(&self) -> &'static str { "Inicio" }
    fn tab_networking(&self) -> &'static str { "Red" }
    fn tab_info(&self) -> &'static str { "Info" }
    fn tab_settings(&self) -> &'static str { "Ajustes" }

    // Start gate modal
    fn gate_title(&self) -> &'static str { "Bienvenido a Zsozso" }
    fn btn_next(&self) -> &'static str { "Siguiente" }

    // Passkey authentication
    fn authenticating(&self) -> &'static str { "Autenticando..." }
    fn auth_failed(&self) -> &'static str { "Autenticación fallida" }
    fn btn_exit(&self) -> &'static str { "Salir ahora" }
    fn no_prf_key(&self) -> &'static str { "No hay clave de cifrado de passkey disponible. Vuelva a autenticarse." }

    // Info tab
    fn info_public_key_label(&self) -> &'static str { "Tu clave pública" }
    fn info_no_key(&self) -> &'static str { "No hay clave cargada. Genera o importa una en Ajustes." }

    // Networking tab / Smart Contract
    fn btn_ping(&self) -> &'static str { "\u{1F3D3} Ping" }
    fn ping_calling(&self) -> &'static str { "\u{1F4E1} Llamando al contrato..." }
    fn ping_success(&self, msg: &str) -> String { format!("\u{2705} {}", msg) }
    fn ping_error(&self, e: &str) -> String { format!("\u{274C} {}", e) }
    fn ping_no_key(&self) -> &'static str { "\u{26A0}\u{FE0F} Cargue una clave primero (pestaña Ajustes)." }

    // QR Scanner
    fn btn_scan_qr(&self) -> &'static str { "\u{1F4F7} Escanear QR" }
    fn scan_scanning(&self) -> &'static str { "\u{1F4F7} Escaneando..." }
    fn scan_success(&self, key: &str) -> String { format!("\u{2705} Escaneado: {}", key) }
    fn scan_error(&self, e: &str) -> String { format!("\u{274C} Error al escanear: {}", e) }

    // Log tab
    fn tab_log(&self) -> &'static str { "Registro" }
    fn log_refresh(&self) -> &'static str { "\u{1F504} Actualizar" }
    fn log_clear(&self) -> &'static str { "\u{1F5D1} Limpiar" }
    fn log_upload(&self) -> &'static str { "\u{2B06}\u{FE0F} Subir" }
    fn log_uploading(&self) -> &'static str { "Subiendo..." }
    fn log_upload_ok(&self) -> &'static str { "\u{2705} Registro subido" }
    fn log_upload_fail(&self, e: &str) -> String { format!("\u{274C} Error al subir: {}", e) }
    fn log_upload_empty(&self) -> &'static str { "\u{26A0}\u{FE0F} El registro está vacío" }

    // Update toast
    fn toast_update_available(&self) -> &'static str { "\u{1F680} ¡Una nueva versión de Zsozso está disponible!" }
    fn btn_update_now(&self) -> &'static str { "Actualizar ahora" }

    // Info tab – version
    fn info_version(&self, ver: &str) -> String { format!("Versión: {}", ver) }
}
