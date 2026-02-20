mod clipboard;
pub mod i18n;

use dioxus::prelude::*;
use zeroize::Zeroizing;

use crate::i18n::Language;
use crate::ledger::{Ledger, NetworkEnvironment, StellarLedger};
use crate::store::Store;
use clipboard::safe_copy;
use i18n::ui_i18n;

#[cfg(not(target_arch = "wasm32"))]
use crate::store::KeyringStore;
#[cfg(target_arch = "wasm32")]
use crate::store::LocalStorageStore;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

#[derive(Clone)]
enum TxStatus {
    Waiting,
    Submitting,
    CallingFaucet,
    FetchingSequence,
    NoKey,
    NoXdr,
    XdrReady { net: String, seq: i64 },
    Success(String),
    Error(String),
    FaucetSuccess(String),
}

pub fn app() -> Element {
    let mut language = use_signal(|| Language::default());

    let mut public_key = use_signal(|| None::<String>);
    let mut secret_key_hidden = use_signal(|| None::<Zeroizing<String>>);
    let mut show_secret = use_signal(|| false);
    let clipboard_copied = use_signal(|| false);
    let mut input_value = use_signal(|| String::new());
    let mut generated_xdr = use_signal(|| String::new());
    let xdr_copied = use_signal(|| false);
    let mut submission_status = use_signal(|| TxStatus::Waiting);
    let mut current_network = use_signal(|| NetworkEnvironment::Production);

    #[cfg(not(target_arch = "wasm32"))]
    use_drop(move || {
        if let Ok(mut cb) = arboard::Clipboard::new() {
            let _ = cb.set_text("".to_string());
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
    });

    let submit_tx_action = move |_| {
        let xdr_to_submit = generated_xdr.read().clone();
        let net_env = *current_network.read();
        let lang = *language.read();

        if xdr_to_submit.is_empty() {
            submission_status.set(TxStatus::NoXdr);
            return;
        }

        spawn(async move {
            submission_status.set(TxStatus::Submitting);
            let lgr = StellarLedger::new(net_env, lang);

            match lgr.submit_transaction(&xdr_to_submit).await {
                Ok(msg) => submission_status.set(TxStatus::Success(msg)),
                Err(e) => submission_status.set(TxStatus::Error(e)),
            }
        });
    };

    let copy_to_clipboard = move |_| {
        if let Some(secret) = secret_key_hidden.read().as_ref() {
            safe_copy(secret.to_string(), clipboard_copied, true);
        }
    };

    let copy_xdr_to_clipboard = move |_| {
        let xdr = generated_xdr.read().clone();
        if !xdr.is_empty() {
            safe_copy(xdr, xdr_copied, false);
        }
    };

    let activate_account = move |_| {
        let pubkey_opt = public_key.read().clone();
        let net_env = *current_network.read();
        let lang = *language.read();

        let Some(pubkey) = pubkey_opt else { return; };

        spawn(async move {
            submission_status.set(TxStatus::CallingFaucet);
            let lgr = StellarLedger::new(net_env, lang);

            match lgr.activate_test_account(&pubkey).await {
                Ok(msg) => submission_status.set(TxStatus::FaucetSuccess(msg)),
                Err(e) => submission_status.set(TxStatus::Error(e)),
            }
        });
    };

    let fetch_and_generate = move |_| {
        let secret_str_opt = secret_key_hidden.read().clone();
        let net_env = *current_network.read();
        let lang = *language.read();

        if secret_str_opt.is_none() {
            submission_status.set(TxStatus::NoKey);
            return;
        }

        let secret_val = secret_str_opt.unwrap().to_string();

        spawn(async move {
            submission_status.set(TxStatus::FetchingSequence);
            let lgr = StellarLedger::new(net_env, lang);
            let net_info = lgr.network_info();

            match lgr.build_self_payment(&secret_val, 100_000_000).await {
                Ok((xdr, seq)) => {
                    generated_xdr.set(xdr);
                    submission_status.set(TxStatus::XdrReady { net: net_info.name.to_string(), seq });
                }
                Err(e) => submission_status.set(TxStatus::Error(e)),
            }
        });
    };

    let generate_key = move |_| {
        let lang = *language.read();
        let lgr = StellarLedger::new(*current_network.read(), lang);
        let kp = lgr.generate_keypair();

        public_key.set(Some(kp.public_key));
        secret_key_hidden.set(Some(Zeroizing::new(kp.secret_key)));
    };

    let import_key = move |_| {
        let raw_input = input_value.read().clone();
        let lang = *language.read();
        let lgr = StellarLedger::new(*current_network.read(), lang);

        if let Some(pub_key_str) = lgr.public_key_from_secret(&raw_input) {
            public_key.set(Some(pub_key_str));
            secret_key_hidden.set(Some(Zeroizing::new(raw_input)));
            input_value.set(String::new());
        }
    };

    let save_action = move |_| {
        let lang = *language.read();
        let i18n = ui_i18n(lang);
        if let Some(secret) = secret_key_hidden.read().as_ref() {
            let store = new_store(lang);
            match store.save(secret.as_str()) {
                Ok(_) => log(&i18n.save_success().to_string()),
                Err(e) => log(&i18n.fmt_error(&e)),
            }
        } else {
            log(&i18n.nothing_to_save().to_string());
        }
    };

    let load_action = move |_| {
        let lang = *language.read();
        let i18n = ui_i18n(lang);
        log(&i18n.loading_started().to_string());
        let store = new_store(lang);
        match store.load() {
            Ok(secret) => {
                let secret: String = secret;
                log(&i18n.key_loaded_len(secret.len()));
                let lgr = StellarLedger::new(*current_network.read(), lang);

                if let Some(pub_key_str) = lgr.public_key_from_secret(&secret) {
                    public_key.set(Some(pub_key_str));
                    secret_key_hidden.set(Some(Zeroizing::new(secret)));
                    log(&i18n.ui_updated_with_key().to_string());
                }
            }
            Err(e) => log(&i18n.fmt_error(&e)),
        }
    };
    
    let network_toggle = move |_| {
        let next = if *current_network.read() == NetworkEnvironment::Production {
            NetworkEnvironment::Test
        } else {
            NetworkEnvironment::Production
        };
        current_network.set(next);
        generated_xdr.set(String::new());
    };
    
    let language_toggle = move |_| {
        let current_lang = *language.read();
        let next_lang = if current_lang == Language::English {
            Language::Hungarian
        } else {
            Language::English
        };
        language.set(next_lang);
    };
    
    // === Render preparation ===
    let lang = *language.read();
    let i18n = ui_i18n(lang);

    let status_text = match &*submission_status.read() {
        TxStatus::Waiting => i18n.waiting().to_string(),
        TxStatus::Submitting => i18n.submitting().to_string(),
        TxStatus::CallingFaucet => i18n.calling_faucet().to_string(),
        TxStatus::FetchingSequence => i18n.fetching_sequence().to_string(),
        TxStatus::NoKey => i18n.no_loaded_key().to_string(),
        TxStatus::NoXdr => i18n.err_no_generated_xdr().to_string(),
        TxStatus::XdrReady { net, seq } => i18n.fmt_xdr_ready(net, *seq),
        TxStatus::Success(msg) => i18n.fmt_success(msg),
        TxStatus::Error(e) => i18n.fmt_error(e),
        TxStatus::FaucetSuccess(msg) => format!("✅ {}", msg),
    };

    let pk_display = match &*public_key.read() {
        Some(key) => key.clone(),
        None => i18n.no_key_loaded().to_string(),
    };

    let net_env = *current_network.read();
    let lgr_for_render = StellarLedger::new(net_env, lang);
    let net_info = lgr_for_render.network_info();
    let is_production = net_env == NetworkEnvironment::Production;
    let has_faucet = net_info.has_faucet;
    let network_btn_style = format!(
        "padding: 8px 20px; border: none; border-radius: 4px; font-weight: bold; cursor: pointer; color: white; background: {};",
        if !is_production { "#dc3545" } else { "#17a2b8" }
    );
    let network_btn_label = if !is_production { i18n.net_testnet_label() } else { i18n.net_mainnet_label() };
    let language_btn_style = "padding: 8px 20px; border: none; border-radius: 4px; font-weight: bold; cursor: pointer; color: white; background: #17a2b8;";

    rsx! {
        div { style: "padding: 30px; font-family: sans-serif; max-width: 550px; margin: auto;",
            h2 { "Zsozso" }

            // === NETWORK SWITCHER, LANGUAGE SWITCHER ===
            div { style: "display: flex; gap: 10px; margin-bottom: 20px;",
                // Network switcher button
                button {
                    style: "{network_btn_style}",
                    onclick: network_toggle,
                    "{network_btn_label}"
                }
                
                // Language switcher button
                button {
                    style: "{language_btn_style}",
                    onclick: language_toggle,
                    if *language.read() == Language::English { "Magyar" } else { "English" }
                }
            }

            // --- ADDRESS DISPLAY ---
            div { style: "background: #f8f9fa; padding: 15px; border-radius: 8px; margin-bottom: 20px; border: 1px solid #ddd;",
                p { style: "font-size: 0.8em; color: #666; margin: 0;", "{i18n.lbl_active_address()}" }
                code { style: "word-break: break-all; font-weight: bold;", "{pk_display}" }
            }

            // --- KEY MANAGEMENT BUTTONS ---
            div { style: "display: flex; gap: 10px; margin-bottom: 20px;",
                button { onclick: generate_key, "{i18n.btn_new_key()}" }
                input {
                    style: "flex-grow: 1; padding: 5px;",
                    r#type: "password",
                    placeholder: "{i18n.lbl_import_ph()}",
                    value: "{input_value}",
                    oninput: move |evt| input_value.set(evt.value())
                }
                button { onclick: import_key, "{i18n.btn_import()}" }
            }

            // --- SECRET KEY SECTION ---
            if let Some(secret) = secret_key_hidden.read().as_ref() {
                div { style: "border: 1px solid #ffeeba; background: #fff3cd; padding: 15px; border-radius: 8px; margin-bottom: 20px;",
                    div { style: "display: flex; gap: 10px; flex-wrap: wrap;",
                        button {
                            onclick: move |_| show_secret.toggle(),
                            if *show_secret.read() { "{i18n.btn_hide_secret()}" } else { "{i18n.btn_reveal_secret()}" }
                        }
                        button {
                            style: "background: #28a745; color: white; border: none; padding: 5px 15px; border-radius: 4px;",
                            onclick: copy_to_clipboard,
                            if *clipboard_copied.read() { "{i18n.copied()}" } else { "{i18n.copy_label()}" }
                        }
                        if has_faucet {
                            button {
                                style: "background: #17a2b8; color: white; border: none; padding: 5px 15px; border-radius: 4px;",
                                onclick: activate_account,
                                "{i18n.btn_activate_faucet()}"
                            }
                        }
                    }

                    if *show_secret.read() {
                        p { style: "margin-top: 15px; font-family: monospace; word-break: break-all; background: white; padding: 10px;",
                            "{secret.as_str()}"
                        }
                    }
                }
            }

            // --- SAVE / LOAD ---
            div { style: "display: flex; gap: 10px; margin-top: 15px;",
                button { onclick: save_action, style: "flex: 1;", "{i18n.btn_save_to_os()}" }
                button { onclick: load_action, style: "flex: 1;", "{i18n.btn_load()}" }
            }

            // --- GENERATE TRANSACTION ---
            button {
                style: "margin-top: 30px; width: 100%; padding: 12px; background: #007bff; color: white; border: none; border-radius: 5px; font-weight: bold; cursor: pointer; margin-bottom: 10px;",
                onclick: fetch_and_generate,
                "{i18n.btn_generate_xdr()}"
            }

            // --- STATUS MESSAGE ---
            p { style: "text-align: center; font-size: 0.9em; color: #495057; font-style: italic;",
                "{status_text}"
            }

            // --- GENERATED XDR BLOCK ---
            if !generated_xdr.read().is_empty() {
                div { style: "margin-top: 20px; padding: 15px; background: #e9ecef; border-radius: 8px; border: 1px solid #ced4da;",
                    div { style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 10px;",
                        span { style: "font-size: 0.8em; font-weight: bold;", "{i18n.lbl_signed_xdr()}" }
                        button {
                            style: "font-size: 0.7em; padding: 4px 8px;",
                            onclick: copy_xdr_to_clipboard,
                            if *xdr_copied.read() { "{i18n.copied()}" } else { "{i18n.copy_xdr_label()}" }
                        }
                    }
                    pre {
                        style: "word-break: break-all; white-space: pre-wrap; font-size: 0.75em; background: white; padding: 10px; border-radius: 4px; border: 1px solid #dee2e6; max-height: 100px; overflow-y: auto;",
                        "{generated_xdr}"
                    }
                    button {
                        style: "width: 100%; margin-top: 15px; padding: 12px; background: #28a745; color: white; border: none; border-radius: 5px; font-weight: bold;",
                        onclick: submit_tx_action,
                        "{i18n.btn_submit_tx()}"
                    }
                }
            }
        }
    }
}

/// Create the platform-appropriate Store implementation.
#[cfg(not(target_arch = "wasm32"))]
fn new_store(lang: crate::i18n::Language) -> KeyringStore {
    KeyringStore::new("zsozso", "default_account", lang)
}

#[cfg(target_arch = "wasm32")]
fn new_store(lang: crate::i18n::Language) -> LocalStorageStore {
    LocalStorageStore::new("zsozso", "default_account", lang)
}

/// Cross-platform logging: println on desktop, console.log on web.
#[cfg(not(target_arch = "wasm32"))]
fn log(msg: &str) {
    println!("{}", msg);
}

#[cfg(target_arch = "wasm32")]
fn log(msg: &str) {
    web_sys::console::log_1(&msg.into());
}
