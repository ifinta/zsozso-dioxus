use std::collections::HashMap;
use dioxus::prelude::*;
use zeroize::Zeroizing;
use crate::ledger::NetworkEnvironment;
use crate::i18n::Language;
use crate::db::gundb::SeaKeyPair;
use super::status::TxStatus;
use super::tabs::Tab;

/// Read biometric preference from localStorage (synchronous).
fn biometric_enabled_default() -> bool {
    web_sys::window()
        .and_then(|w| w.local_storage().ok())
        .flatten()
        .and_then(|s| s.get_item("zsozso:biometric").ok())
        .flatten()
        .map(|v| v == "true")
        .unwrap_or(false)
}

/// Passkey authentication state machine.
#[derive(Clone, Copy, PartialEq, Default)]
pub enum AuthState {
    #[default]
    Pending,        // gate modal shown, waiting for user to click
    Authenticating, // passkey dialog in progress
    Authenticated,  // success — show app tabs
    Failed,         // terminal — show error modal
}

#[derive(Clone, Copy)]
pub struct WalletState {
    pub language: Signal<Language>,
    pub public_key: Signal<Option<String>>,
    pub secret_key_hidden: Signal<Option<Zeroizing<String>>>,
    pub show_secret: Signal<bool>,
    pub input_value: Signal<String>,
    pub generated_xdr: Signal<String>,
    pub submission_status: Signal<TxStatus>,
    pub current_network: Signal<NetworkEnvironment>,
    pub clipboard_modal_open: Signal<bool>,
    pub active_tab: Signal<Tab>,
    pub auth_state: Signal<AuthState>,
    pub prf_key: Signal<Option<String>>,
    pub ping_status: Signal<Option<String>>,
    /// When Some, shows a modal asking the user to save the secret before switching network.
    /// The value is the target network the user wants to switch to.
    pub network_switch_pending: Signal<Option<NetworkEnvironment>>,
    /// Whether the SEA key generation modal is open.
    pub sea_modal_open: Signal<bool>,
    /// Input value for the SEA secret passphrase (kept only in memory).
    pub sea_modal_input: Signal<Zeroizing<String>>,
    /// Generated SEA key pair (kept only in memory, never persisted).
    pub sea_key_pair: Signal<Option<SeaKeyPair>>,
    /// Whether biometric (passkey) authentication is enabled.
    pub biometric_enabled: Signal<bool>,
    /// Whether the "enable biometric to save" error modal is shown.
    pub biometric_save_modal_open: Signal<bool>,
    /// Current user's nickname (visible to the whole network).
    pub nickname: Signal<String>,
    /// Ancestry chain (parent, grandparent, ...) — up to 6 levels.
    pub network_parents: Signal<Vec<String>>,
    /// Direct workers of this node.
    pub network_workers: Signal<Vec<String>>,
    /// Public key → nickname cache for displayed network nodes.
    pub network_nicknames: Signal<HashMap<String, String>>,
    /// CYF "not yet implemented" modal message (None = hidden).
    pub cyf_modal_message: Signal<Option<String>>,
}

pub fn use_wallet_state() -> WalletState {
    let bio = biometric_enabled_default();
    WalletState {
        language: use_signal(Language::default),
        public_key: use_signal(|| None),
        secret_key_hidden: use_signal(|| None),
        show_secret: use_signal(|| false),
        input_value: use_signal(String::new),
        generated_xdr: use_signal(String::new),
        submission_status: use_signal(|| TxStatus::Waiting),
        current_network: use_signal(|| NetworkEnvironment::Production),
        clipboard_modal_open: use_signal(|| false),
        active_tab: use_signal(Tab::default),
        // If biometric is disabled, skip the auth gate entirely.
        auth_state: use_signal(move || if bio { AuthState::default() } else { AuthState::Authenticated }),
        prf_key: use_signal(|| None),
        ping_status: use_signal(|| None),
        network_switch_pending: use_signal(|| None),
        sea_modal_open: use_signal(|| false),
        sea_modal_input: use_signal(|| Zeroizing::new(String::new())),
        sea_key_pair: use_signal(|| None),
        biometric_enabled: use_signal(move || bio),
        biometric_save_modal_open: use_signal(|| false),
        nickname: use_signal(String::new),
        network_parents: use_signal(Vec::new),
        network_workers: use_signal(Vec::new),
        network_nicknames: use_signal(HashMap::new),
        cyf_modal_message: use_signal(|| None),
    }
}
