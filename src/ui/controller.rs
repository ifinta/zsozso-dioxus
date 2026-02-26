use dioxus::prelude::*;
use zeroize::Zeroizing;
use super::state::WalletState;
use super::actions::*;
use super::status::TxStatus;
use super::i18n::ui_i18n;
use crate::ledger::{Ledger, NetworkEnvironment, StellarLedger};
use crate::store::Store;
use super::clipboard::{copy_to_clipboard, clear_clipboard};
use super::log;

#[derive(Clone, Copy)]
pub struct AppController {
    s: WalletState,
}

impl AppController {
    pub fn new(state: WalletState) -> Self {
        Self { s: state }
    }

    /// Generate a new keypair and store it in the state
    pub fn generate_key(&self) {
        let (pk, sk) = generate_keypair(*self.s.current_network.read(), *self.s.language.read());
        let mut public_key = self.s.public_key;
        let mut secret_key_hidden = self.s.secret_key_hidden;
        public_key.set(Some(pk));
        secret_key_hidden.set(Some(Zeroizing::new(sk)));
    }

    /// Import a keypair from user input
    pub fn import_key(&self) {
        let raw_input = self.s.input_value.read().clone();
        let net = *self.s.current_network.read();
        let lang = *self.s.language.read();

        if let Some((pub_key_str, secret)) = import_keypair(raw_input, net, lang) {
            let mut public_key = self.s.public_key;
            let mut secret_key_hidden = self.s.secret_key_hidden;
            let mut input_value = self.s.input_value;
            public_key.set(Some(pub_key_str));
            secret_key_hidden.set(Some(Zeroizing::new(secret)));
            input_value.set(String::new());
        }
    }

    /// Securely copy the secret key to clipboard and show modal
    pub fn copy_secret_to_clipboard(&self) {
        if let Some(secret) = self.s.secret_key_hidden.read().as_ref() {
            copy_to_clipboard(secret.as_str());
            let lang = *self.s.language.read();
            let i18n = ui_i18n(lang);
            log(&i18n.copied().to_string());
            let mut modal = self.s.clipboard_modal_open;
            modal.set(true);
        }
    }

    /// Copy the generated XDR to clipboard and show modal
    pub fn copy_xdr_to_clipboard(&self) {
        let xdr = self.s.generated_xdr.read().clone();
        if !xdr.is_empty() {
            copy_to_clipboard(&xdr);
            let lang = *self.s.language.read();
            let i18n = ui_i18n(lang);
            log(&i18n.copied().to_string());
            let mut modal = self.s.clipboard_modal_open;
            modal.set(true);
        }
    }

    /// Dismiss the clipboard modal and clear clipboard content
    pub fn dismiss_clipboard_modal(&self) {
        clear_clipboard();
        let mut modal = self.s.clipboard_modal_open;
        modal.set(false);
        let lang = *self.s.language.read();
        let i18n = ui_i18n(lang);
        log(&i18n.clipboard_cleared().to_string());
    }

    /// Activate a test network account (Faucet call)
    pub fn activate_test_account_action(&self) {
        let pubkey = self.s.public_key.read().clone();
        let net_env = *self.s.current_network.read();
        let lang = *self.s.language.read();
        let mut status = self.s.submission_status;

        spawn(async move {
            status.set(TxStatus::CallingFaucet);
            if let Some(next_status) = activate_test_account(pubkey, net_env, lang).await {
                status.set(next_status);
            }
        });
    }

    /// Generate XDR based on account data
    pub fn fetch_and_generate_xdr_action(&self) {
        let secret_key = self.s.secret_key_hidden.read().as_ref().map(|s| s.to_string());
        let net_env = *self.s.current_network.read();
        let lang = *self.s.language.read();
        let mut status = self.s.submission_status;
        let mut xdr_signal = self.s.generated_xdr;

        spawn(async move {
            status.set(TxStatus::FetchingSequence);
            match fetch_and_generate_xdr(secret_key, net_env, lang).await {
                Ok((xdr, next_status)) => {
                    xdr_signal.set(xdr);
                    status.set(next_status);
                }
                Err(e_status) => status.set(e_status),
            }
        });
    }

    /// Submit a transaction to the network
    pub fn submit_transaction_action(&self) {
        let xdr = self.s.generated_xdr.read().clone();
        let net_env = *self.s.current_network.read();
        let lang = *self.s.language.read();
        let mut status = self.s.submission_status;

        spawn(async move {
            status.set(TxStatus::Submitting);
            status.set(submit_transaction(xdr, net_env, lang).await);
        });
    }

    /// Save key to local store (e.g. file or browser storage)
    pub fn save_to_store(&self) {
        let lang = *self.s.language.read();
        let i18n = ui_i18n(lang);
        
        if let Some(secret) = self.s.secret_key_hidden.read().as_ref() {
            let store = new_store(lang);
            match store.save(secret.as_str()) {
                Ok(_) => log(&i18n.save_success().to_string()),
                Err(e) => log(&i18n.fmt_error(&e)),
            }
        } else {
            log(&i18n.nothing_to_save().to_string());
        }
    }

    /// Load key from local store
    pub fn load_from_store(&self) {
        let lang = *self.s.language.read();
        let net = *self.s.current_network.read();
        let i18n = ui_i18n(lang);
        let mut pk_signal = self.s.public_key;
        let mut sk_signal = self.s.secret_key_hidden;

        log(&i18n.loading_started().to_string());
        let store = new_store(lang);
        
        match store.load() {
            Ok(secret) => {
                log(&i18n.key_loaded_len(secret.len()));
                let lgr = StellarLedger::new(net, lang);
                if let Some(pub_key_str) = lgr.public_key_from_secret(&secret) {
                    pk_signal.set(Some(pub_key_str));
                    sk_signal.set(Some(Zeroizing::new(secret)));
                    log(&i18n.ui_updated_with_key().to_string());
                }
            }
            Err(e) => log(&i18n.fmt_error(&e)),
        }
    }

    pub fn toggle_language(&self) {
        let next = if *self.s.language.read() == crate::i18n::Language::English {
            crate::i18n::Language::Hungarian
        } else {
            crate::i18n::Language::English
        };
        let mut language = self.s.language;
        language.set(next);
    }

    pub fn toggle_network(&self) {
        let current = *self.s.current_network.read();
        let next = if current == NetworkEnvironment::Production {
            NetworkEnvironment::Test
        } else {
            NetworkEnvironment::Production
        };

        let mut current_network = self.s.current_network;
        let mut generated_xdr = self.s.generated_xdr;
        current_network.set(next);
        generated_xdr.set(String::new());
    }
}
