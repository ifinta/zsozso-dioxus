use super::UiI18n;

pub struct FrenchUi;

impl UiI18n for FrenchUi {
    // Initial signal values
    fn no_key_loaded(&self) -> &'static str { "Aucune clé chargée" }
    fn copy_label(&self) -> &'static str { "Copier" }
    fn copy_xdr_label(&self) -> &'static str { "Copier XDR" }
    fn waiting(&self) -> &'static str { "En attente..." }
    
    // Status messages
    fn err_no_generated_xdr(&self) -> &'static str { "Erreur : Aucun XDR généré !" }
    fn submitting(&self) -> &'static str { "Envoi en cours..." }
    fn calling_faucet(&self) -> &'static str { "🚀 Appel du faucet..." }
    fn no_loaded_key(&self) -> &'static str { "⚠️ Aucune clé chargée !" }
    fn fetching_sequence(&self) -> &'static str { "🔍 Récupération du numéro de séquence..." }
    
    // Console/println messages
    fn clipboard_cleared(&self) -> &'static str { "🔐 Presse-papiers vidé par mesure de sécurité." }
    fn save_success(&self) -> &'static str { "✅ Sauvegarde réussie dans le portefeuille système !" }
    fn nothing_to_save(&self) -> &'static str { "⚠️ Rien à sauvegarder (la clé est vide) !" }
    fn loading_started(&self) -> &'static str { "🔍 Chargement en cours..." }
    fn key_loaded_len(&self, len: usize) -> String { format!("📥 Clé chargée, longueur : {}", len) }
    fn ui_updated_with_key(&self) -> &'static str { "✨ Interface mise à jour avec la clé chargée." }
    
    // Format helpers
    fn fmt_success(&self, msg: &str) -> String { format!("✅ SUCCÈS ! {}", msg) }
    fn fmt_error(&self, e: &str) -> String { format!("❌ {}", e) }
    fn fmt_xdr_ready(&self, net: &str, seq: i64) -> String { format!("✅ XDR Prêt ! [{}] (Séq : {})", net, seq) }
    
    // Button labels
    fn btn_new_key(&self) -> &'static str { "✨ Nouvelle Clé" }
    fn btn_import(&self) -> &'static str { "📥 Importer" }
    fn btn_hide_secret(&self) -> &'static str { "🙈 Masquer" }
    fn btn_reveal_secret(&self) -> &'static str { "👁 Révéler" }
    fn btn_activate_faucet(&self) -> &'static str { "🚀 Activer (Faucet)" }
    fn btn_save_to_os(&self) -> &'static str { "💾 Sauvegarder dans le portefeuille OS" }
    fn btn_load(&self) -> &'static str { "🔓 Charger" }
    fn btn_generate_xdr(&self) -> &'static str { "🛠 Générer le XDR de transaction" }
    fn btn_submit_tx(&self) -> &'static str { "🚀 ENVOYER la Transaction" }
    
    // UI labels
    fn lbl_active_address(&self) -> &'static str { "Adresse Active (Clé Publique) :" }
    fn lbl_signed_xdr(&self) -> &'static str { "XDR SIGNÉ :" }
    fn lbl_import_ph(&self) -> &'static str { "Importer (S...)" }
    
    // Network labels
    fn net_testnet_label(&self) -> &'static str { "🧪 Testnet ⚠️" }
    fn net_mainnet_label(&self) -> &'static str { "Mainnet" }
    
    // Clipboard
    fn copied(&self) -> &'static str { "COPIÉ !" }
    fn clipboard_modal_text(&self) -> &'static str { "Le contenu a été copié dans le presse-papiers. Lorsque vous fermerez cette fenêtre, le presse-papiers sera vidé par mesure de sécurité." }
    fn btn_clear_clipboard(&self) -> &'static str { "🗑️ Vider le presse-papiers" }

    // Tab labels
    fn tab_home(&self) -> &'static str { "Accueil" }
    fn tab_networking(&self) -> &'static str { "Réseau" }
    fn tab_info(&self) -> &'static str { "Info" }
    fn tab_settings(&self) -> &'static str { "Paramètres" }

    // Start gate modal
    fn gate_title(&self) -> &'static str { "Bienvenue sur Zsozso" }
    fn btn_next(&self) -> &'static str { "Suivant" }

    // Passkey authentication
    fn authenticating(&self) -> &'static str { "Authentification..." }
    fn auth_failed(&self) -> &'static str { "Échec de l'authentification" }
    fn btn_exit(&self) -> &'static str { "Quitter" }
    fn no_prf_key(&self) -> &'static str { "Aucune clé de chiffrement passkey disponible. Réauthentifiez-vous d'abord." }

    // Info tab
    fn info_public_key_label(&self) -> &'static str { "Votre Clé Publique" }
    fn info_no_key(&self) -> &'static str { "Aucune clé chargée. Générez ou importez-en une dans les Paramètres." }

    // Networking tab / Smart Contract
    fn btn_ping(&self) -> &'static str { "\u{1F3D3} Ping" }
    fn ping_calling(&self) -> &'static str { "\u{1F4E1} Appel du contrat..." }
    fn ping_success(&self, msg: &str) -> String { format!("\u{2705} {}", msg) }
    fn ping_error(&self, e: &str) -> String { format!("\u{274C} {}", e) }
    fn ping_no_key(&self) -> &'static str { "\u{26A0}\u{FE0F} Chargez d'abord une clé (onglet Paramètres)." }

    // QR Scanner
    fn btn_scan_qr(&self) -> &'static str { "\u{1F4F7} Scanner QR" }
    fn scan_scanning(&self) -> &'static str { "\u{1F4F7} Scan en cours..." }
    fn scan_success(&self, key: &str) -> String { format!("\u{2705} Scanné : {}", key) }
    fn scan_error(&self, e: &str) -> String { format!("\u{274C} Échec du scan : {}", e) }

    // Log tab
    fn tab_log(&self) -> &'static str { "Journal" }
    fn log_refresh(&self) -> &'static str { "\u{1F504} Rafraîchir" }
    fn log_clear(&self) -> &'static str { "\u{1F5D1} Effacer" }

    // Update toast
    fn toast_update_available(&self) -> &'static str { "\u{1F680} Une nouvelle version de Zsozso est disponible\u{00A0}!" }
    fn btn_update_now(&self) -> &'static str { "Mettre à jour" }

    // Info tab – version
    fn info_version(&self, ver: &str) -> String { format!("Version\u{00A0}: {}", ver) }
}
