use dioxus::prelude::*;
use zeroize::Zeroizing;
use super::state::{WalletState, AuthState};
use super::actions::*;
use super::status::TxStatus;
use super::i18n::ui_i18n;
use crate::ledger::{Ledger, NetworkEnvironment, StellarLedger};
use crate::store::Store;
use crate::store::passkey;
use crate::db::gundb::{GunSea, Sea};
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

    /// Enable biometric identification.
    /// Initialises the passkey to obtain the PRF key. Once enabled, cannot be turned off.
    pub fn toggle_biometric(&self) {
        // Only allow turning ON (switch is disabled when already on)
        if *self.s.biometric_enabled.read() {
            return;
        }

        let mut biometric = self.s.biometric_enabled;
        let mut prf_key_signal = self.s.prf_key;

        spawn(async move {
            if is_localhost() {
                biometric.set(true);
                write_biometric_pref(true);
                return;
            }
            match passkey::passkey_init().await {
                Ok(result) if result.success => {
                    prf_key_signal.set(result.prf_key);
                    biometric.set(true);
                    write_biometric_pref(true);
                }
                _ => {
                    log("Failed to initialize biometric authentication");
                }
            }
        });
    }

    /// Dismiss the biometric save error modal.
    pub fn dismiss_biometric_save_modal(&self) {
        let mut modal = self.s.biometric_save_modal_open;
        modal.set(false);
    }

    /// Reveal the secret key after passkey verification.
    /// On localhost or when biometric is disabled, the passkey check is skipped.
    pub fn reveal_secret(&self) {
        let mut show_secret = self.s.show_secret;
        let biometric_on = *self.s.biometric_enabled.read();

        spawn(async move {
            if is_localhost() || !biometric_on {
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

    /// Save key to local store — encrypted with passkey if PRF available and biometric is on.
    /// The stored format is 'tn:secret' for testnet or 'mn:secret' for mainnet.
    pub fn save_to_store(&self) {
        let lang = *self.s.language.read();
        let net = *self.s.current_network.read();
        let i18n = ui_i18n(lang);
        let biometric_on = *self.s.biometric_enabled.read();

        // Refuse to save when biometric is off (except localhost dev)
        if !biometric_on && !is_localhost() {
            let mut modal = self.s.biometric_save_modal_open;
            modal.set(true);
            return;
        }
        
        if let Some(secret) = self.s.secret_key_hidden.read().as_ref() {
            let store = new_store(lang);
            let secret = secret.clone();
            let prf_key = self.s.prf_key.read().clone();
            spawn(async move {
                let prefix = match net {
                    NetworkEnvironment::Test => "tn:",
                    NetworkEnvironment::Production => "mn:",
                };
                let prefixed_secret = format!("{}{}", prefix, secret.as_str());
                let data_to_save = if is_localhost() || !biometric_on {
                    prefixed_secret
                } else {
                    let prf = match &prf_key {
                        Some(key) => key.clone(),
                        None => {
                            log(&i18n.fmt_error(i18n.no_prf_key()));
                            return;
                        }
                    };
                    match passkey::passkey_encrypt(&prefixed_secret, &prf).await {
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

    /// Load key from local store — decrypted with passkey if PRF available and biometric is on.
    /// Parses the 'tn:' / 'mn:' prefix to restore the correct network.
    pub fn load_from_store(&self) {
        let lang = *self.s.language.read();
        let i18n = ui_i18n(lang);
        let mut pk_signal = self.s.public_key;
        let mut sk_signal = self.s.secret_key_hidden;
        let mut net_signal = self.s.current_network;
        let prf_key = self.s.prf_key.read().clone();
        let biometric_on = *self.s.biometric_enabled.read();

        log(&i18n.loading_started().to_string());
        let store = new_store(lang);
        
        spawn(async move {
            match store.load().await {
                Ok(stored_data) => {
                    let decrypted = if is_localhost() || !biometric_on {
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
                            Ok(d) => d,
                            Err(e) => {
                                log(&i18n.fmt_error(&e));
                                return;
                            }
                        }
                    };

                    // Parse network prefix
                    let (net, secret) = if let Some(rest) = decrypted.strip_prefix("tn:") {
                        (NetworkEnvironment::Test, rest.to_string())
                    } else if let Some(rest) = decrypted.strip_prefix("mn:") {
                        (NetworkEnvironment::Production, rest.to_string())
                    } else {
                        // Legacy data without prefix — default to current network
                        (*net_signal.read(), decrypted)
                    };

                    log(&i18n.key_loaded_len(secret.len()));
                    let lgr = StellarLedger::new(net, lang);
                    if let Some(pub_key_str) = lgr.public_key_from_secret(&secret) {
                        net_signal.set(net);
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

        // If there is a secret in memory, ask the user whether to save first
        let has_secret = self.s.secret_key_hidden.read().is_some();
        if has_secret {
            let mut pending = self.s.network_switch_pending;
            pending.set(Some(next));
        } else {
            self.do_switch_network(next);
        }
    }

    /// Actually perform the network switch (clears XDR).
    fn do_switch_network(&self, next: NetworkEnvironment) {
        let mut current_network = self.s.current_network;
        let mut generated_xdr = self.s.generated_xdr;
        current_network.set(next);
        generated_xdr.set(String::new());
    }

    /// User confirmed: save the secret with the current network, then switch.
    pub fn confirm_network_switch_save(&self) {
        // Save first (uses current network for the prefix)
        self.save_to_store();

        // Now switch to the pending network
        if let Some(next) = *self.s.network_switch_pending.read() {
            self.do_switch_network(next);
        }
        let mut pending = self.s.network_switch_pending;
        pending.set(None);
    }

    /// User confirmed: switch to the new network first, then save with the new prefix.
    pub fn confirm_network_switch_and_save(&self) {
        if let Some(next) = *self.s.network_switch_pending.read() {
            self.do_switch_network(next);
        }
        let mut pending = self.s.network_switch_pending;
        pending.set(None);

        // Save after switching (uses new network for the prefix)
        self.save_to_store();
    }

    /// User declined saving: just switch network without saving.
    pub fn confirm_network_switch_discard(&self) {
        if let Some(next) = *self.s.network_switch_pending.read() {
            self.do_switch_network(next);
        }
        let mut pending = self.s.network_switch_pending;
        pending.set(None);
    }

    /// User cancelled the network switch entirely.
    pub fn cancel_network_switch(&self) {
        let mut pending = self.s.network_switch_pending;
        pending.set(None);
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

    /// Open the SEA key generation modal.
    pub fn open_sea_modal(&self) {
        let mut open = self.s.sea_modal_open;
        open.set(true);
    }

    /// Close the SEA key generation modal and zeroize the input.
    pub fn close_sea_modal(&self) {
        let mut open = self.s.sea_modal_open;
        let mut input = self.s.sea_modal_input;
        open.set(false);
        input.set(Zeroizing::new(String::new()));
    }

    /// Generate a SEA key pair from the passphrase entered in the modal.
    /// The passphrase is zeroized after use; the keys live only in memory.
    pub fn generate_sea_keys(&self) {
        let lang = *self.s.language.read();
        let i18n = ui_i18n(lang);

        let passphrase = self.s.sea_modal_input.read().clone();
        if passphrase.is_empty() {
            return;
        }

        let mut key_pair_signal = self.s.sea_key_pair;
        let mut modal_open = self.s.sea_modal_open;
        let mut modal_input = self.s.sea_modal_input;

        spawn(async move {
            let sea = GunSea::new(lang);
            match sea.pair_from_seed(&passphrase).await {
                Ok(pair) => {
                    key_pair_signal.set(Some(pair));
                    let i18n = ui_i18n(lang);
                    log(&i18n.sea_keys_generated().to_string());
                }
                Err(e) => {
                    let i18n = ui_i18n(lang);
                    log(&i18n.sea_generation_error(&e));
                }
            }
            // Zeroize the passphrase input and close the modal
            modal_input.set(Zeroizing::new(String::new()));
            modal_open.set(false);
        });
    }
}

/// Returns true when the app is served from localhost (development).
fn is_localhost() -> bool {
    web_sys::window()
        .and_then(|w| w.location().hostname().ok())
        .is_some_and(|h| h == "localhost" || h == "127.0.0.1" || h == "::1")
}

/// Write biometric preference to localStorage.
fn write_biometric_pref(enabled: bool) {
    if let Some(storage) = web_sys::window()
        .and_then(|w| w.local_storage().ok())
        .flatten()
    {
        let _ = storage.set_item("zsozso:biometric", if enabled { "true" } else { "false" });
    }
}
