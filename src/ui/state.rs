use dioxus::prelude::*;
use zeroize::Zeroizing;
use crate::ledger::NetworkEnvironment;
use crate::i18n::Language;
use super::status::TxStatus;

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
}

pub fn use_wallet_state() -> WalletState {
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
    }
}
