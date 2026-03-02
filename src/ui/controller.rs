use dioxus::prelude::*;
use zeroize::Zeroizing;
use super::state::{WalletState, AuthState};
use super::actions::*;
use super::status::TxStatus;
use super::i18n::ui_i18n;
use crate::ledger::{Ledger, NetworkEnvironment, StellarLedger};
use crate::store::Store;
use crate::store::passkey;
use super::clipboard::{copy_to_clipboard, clear_clipboard};
use super::log;
use crate::ledger::sc::SmartContract;

#[derive(Clone, Copy)]
pub struct AppController {
    s: WalletState,
}

impl AppController {
    pub fn new(state: WalletState) -> Self {
        Self { s: state }
    }

    /// Start passkey authentication (gate modal button).
    /// On localhost the passkey check is skipped for easier testing.
    pub fn start_auth(&self) {
        let mut auth_state = self.s.auth_state;
        let mut prf_key_signal = self.s.prf_key;

        auth_state.set(AuthState::Authenticating);

        spawn(async move {
            if is_localhost() {
                auth_state.set(AuthState::Authenticated);
                return;
            }
            match passkey::passkey_init().await {
                Ok(result) if result.success => {
                    prf_key_signal.set(result.prf_key);
                    auth_state.set(AuthState::Authenticated);
                }
                _ => {
                    auth_state.set(AuthState::Failed);
                }
            }
        });
    }

    /// Reveal the secret key after passkey verification.
    /// On localhost the passkey check is skipped for easier testing.
    pub fn reveal_secret(&self) {
        let mut show_secret = self.s.show_secret;

        spawn(async move {
            if is_localhost() {
                show_secret.set(true);
                return;
            }
            match passkey::passkey_verify().await {
                Ok(true) => show_secret.set(true),
                _ => { /* auth failed or cancelled — don't reveal */ }
            }
        });
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

    /// Save key to local store — encrypted with passkey if PRF available
    pub fn save_to_store(&self) {
        let lang = *self.s.language.read();
        let i18n = ui_i18n(lang);
        
        if let Some(secret) = self.s.secret_key_hidden.read().as_ref() {
            let store = new_store(lang);
            let secret = secret.clone();
            let prf_key = self.s.prf_key.read().clone();
            spawn(async move {
                let data_to_save = if is_localhost() {
                    secret.to_string()
                } else {
                    let prf = match &prf_key {
                        Some(key) => key.clone(),
                        None => {
                            log(&i18n.fmt_error(i18n.no_prf_key()));
                            return;
                        }
                    };
                    match passkey::passkey_encrypt(secret.as_str(), &prf).await {
                        Ok(encrypted) => encrypted,
                        Err(e) => {
                            log(&i18n.fmt_error(&e));
                            return;
                        }
                    }
                };
                match store.save(&data_to_save).await {
                    Ok(_) => log(&i18n.save_success().to_string()),
                    Err(e) => log(&i18n.fmt_error(&e)),
                }
            });
        } else {
            log(&i18n.nothing_to_save().to_string());
        }
    }

    /// Load key from local store — decrypted with passkey if PRF available
    pub fn load_from_store(&self) {
        let lang = *self.s.language.read();
        let net = *self.s.current_network.read();
        let i18n = ui_i18n(lang);
        let mut pk_signal = self.s.public_key;
        let mut sk_signal = self.s.secret_key_hidden;
        let prf_key = self.s.prf_key.read().clone();

        log(&i18n.loading_started().to_string());
        let store = new_store(lang);
        
        spawn(async move {
            match store.load().await {
                Ok(stored_data) => {
                    let secret = if is_localhost() {
                        stored_data
                    } else {
                        let prf = match &prf_key {
                            Some(key) => key.clone(),
                            None => {
                                log(&i18n.fmt_error(i18n.no_prf_key()));
                                return;
                            }
                        };
                        match passkey::passkey_decrypt(&stored_data, &prf).await {
                            Ok(decrypted) => decrypted,
                            Err(e) => {
                                log(&i18n.fmt_error(&e));
                                return;
                            }
                        }
                    };
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
        });
    }

    pub fn set_language(&self, code: &str) {
        use crate::i18n::Language;
        let lang = match code {
            "hu" => Language::Hungarian,
            "fr" => Language::French,
            "de" => Language::German,
            "es" => Language::Spanish,
            _ => Language::English,
        };
        let mut language = self.s.language;
        language.set(lang);
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

    /// Open camera QR scanner and log the scanned public key to the console.
    pub fn scan_qr_action(&self) {
        let lang = *self.s.language.read();
        let i18n = ui_i18n(lang);
        let mut ping_status = self.s.ping_status;

        ping_status.set(Some(i18n.scan_scanning().to_string()));

        spawn(async move {
            match super::qr_scanner::scan_qr().await {
                Ok(key) => {
                    log(&format!("Scanned public key: {}", key));
                    let i18n = ui_i18n(lang);
                    ping_status.set(Some(i18n.scan_success(&key)));
                }
                Err(e) => {
                    if e == "cancelled" {
                        ping_status.set(None);
                    } else {
                        let i18n = ui_i18n(lang);
                        ping_status.set(Some(i18n.scan_error(&e)));
                    }
                }
            }
        });
    }

    /// Call the zsozso-sc ping() contract function using the stored secret key
    pub fn ping_contract_action(&self) {
        let lang = *self.s.language.read();
        let i18n = ui_i18n(lang);
        let mut ping_status = self.s.ping_status;

        // Use the in-memory secret key if available
        let secret = match self.s.secret_key_hidden.read().as_ref() {
            Some(sk) => sk.as_str().to_string(),
            None => {
                ping_status.set(Some(i18n.ping_no_key().to_string()));
                return;
            }
        };

        let net_env = *self.s.current_network.read();

        ping_status.set(Some(i18n.ping_calling().to_string()));

        spawn(async move {
            let sc = crate::ledger::sc::zsozso_sc::ZsozsoSc::new(net_env, lang);
            match sc.ping(&secret).await {
                Ok(msg) => {
                    let i18n = ui_i18n(lang);
                    ping_status.set(Some(i18n.ping_success(&msg)));
                }
                Err(e) => {
                    let i18n = ui_i18n(lang);
                    ping_status.set(Some(i18n.ping_error(&e)));
                }
            }
        });
    }
}

/// Returns true when the app is served from localhost (development).
fn is_localhost() -> bool {
    web_sys::window()
        .and_then(|w| w.location().hostname().ok())
        .is_some_and(|h| h == "localhost" || h == "127.0.0.1" || h == "::1")
}
