use dioxus::prelude::*;
use qrcode::{QrCode, render::svg};
use crate::ui::state::WalletState;
use crate::ui::controller::AppController;
use crate::ui::i18n::UiI18n;
use crate::ui::status::status_text;
use crate::i18n::Language;
use crate::ledger::{Ledger, NetworkEnvironment, StellarLedger};

pub fn render_settings_tab(s: WalletState, ctrl: AppController, i18n: &dyn UiI18n) -> Element {
    let lang = *s.language.read();
    let net_env = *s.current_network.read();
    let is_production = net_env == NetworkEnvironment::Production;

    let lgr = StellarLedger::new(net_env, lang);
    let has_faucet = lgr.network_info().has_faucet;

    let pk_display = match &*s.public_key.read() {
        Some(key) => key.clone(),
        None => i18n.no_key_loaded().to_string(),
    };

    let status_display = status_text(&s.submission_status.read(), i18n);

    let network_btn_style = format!(
        "padding: 8px 20px; border: none; border-radius: 4px; font-weight: bold; cursor: pointer; color: white; background: {};",
        if !is_production { "#dc3545" } else { "#17a2b8" }
    );
    let networork_btn_label = if !is_production { i18n.net_testnet_label() } else { i18n.net_mainnet_label() };

    let lang_value = match lang {
        Language::English => "en",
        Language::Hungarian => "hu",
        Language::French => "fr",
        Language::German => "de",
        Language::Spanish => "es",
    };

    rsx! {
        // Header: Network & Language
        div { style: "display: flex; gap: 10px; margin-bottom: 20px; align-items: center;",
            button {
                style: "{network_btn_style}",
                onclick: move |_| ctrl.toggle_network(),
                "{networork_btn_label}"
            }
            select {
                style: "padding: 8px 12px; border: 1px solid #17a2b8; border-radius: 4px; font-weight: bold; cursor: pointer; color: #17a2b8; background: white; font-size: 0.95em;",
                value: "{lang_value}",
                onchange: move |evt| ctrl.set_language(&evt.value()),
                option { value: "en", selected: lang == Language::English, "English" }
                option { value: "hu", selected: lang == Language::Hungarian, "Magyar" }
                option { value: "fr", selected: lang == Language::French, "Français" }
                option { value: "de", selected: lang == Language::German, "Deutsch" }
                option { value: "es", selected: lang == Language::Spanish, "Español" }
            }
        }

        // Active Address Display
        div { style: "background: #f8f9fa; padding: 15px; border-radius: 8px; margin-bottom: 20px; border: 1px solid #ddd;",
            p { style: "font-size: 0.8em; color: #666; margin: 0;", "{i18n.lbl_active_address()}" }
            code { style: "word-break: break-all; font-weight: bold;", "{pk_display}" }
        }

        // Nickname
        div { style: "display: flex; gap: 6px; margin-bottom: 20px; align-items: center;",
            input {
                style: "flex-grow: 1; min-width: 0; padding: 8px; border: 1px solid #ddd; border-radius: 4px;",
                r#type: "text",
                maxlength: "16",
                placeholder: "{i18n.lbl_nickname_ph()}",
                value: "{s.nickname}",
                oninput: move |evt| {
                    let mut nickname = s.nickname;
                    nickname.set(evt.value());
                }
            }
            button {
                style: "padding: 8px 16px; background: #28a745; color: white; border: none; border-radius: 4px; font-weight: bold; cursor: pointer; white-space: nowrap;",
                onclick: move |_| ctrl.save_nickname_action(),
                "{i18n.btn_save_nickname()}"
            }
        }

        // Biometric Identification Toggle
        {
            let biometric_on = *s.biometric_enabled.read();
            let track_bg = if biometric_on { "#28a745" } else { "#ccc" };
            let thumb_left = if biometric_on { "24px" } else { "2px" };
            let track_extra = if biometric_on { "opacity: 0.6; cursor: not-allowed;" } else { "cursor: pointer;" };
            let track_style = format!(
                "position: relative; width: 50px; height: 28px; background: {}; border-radius: 28px; transition: background 0.3s; flex-shrink: 0; {}",
                track_bg, track_extra
            );
            rsx! {
                div { style: "display: flex; align-items: center; justify-content: space-between; padding: 12px 15px; background: #f8f9fa; border-radius: 8px; margin-bottom: 20px; border: 1px solid #ddd;",
                    div { style: "flex: 1; margin-right: 12px;",
                        p { style: "font-weight: bold; margin: 0; font-size: 0.95em;", "{i18n.lbl_biometric()}" }
                        p { style: "font-size: 0.8em; color: #666; margin: 4px 0 0;", "{i18n.lbl_biometric_desc()}" }
                    }
                    div {
                        style: "{track_style}",
                        onclick: move |_| {
                            if !biometric_on {
                                ctrl.toggle_biometric();
                            }
                        },
                        div {
                            style: "position: absolute; top: 3px; left: {thumb_left}; width: 22px; height: 22px; background: white; border-radius: 50%; transition: left 0.3s; box-shadow: 0 1px 3px rgba(0,0,0,0.3);"
                        }
                    }
                }
            }
        }

        // Key Input & Generation
        div { style: "display: flex; gap: 6px; margin-bottom: 20px; align-items: center;",
            button { onclick: move |_| ctrl.generate_key(), "{i18n.btn_new_key()}" }
            button {
                style: "padding: 5px 10px; background: #6f42c1; color: white; border: none; border-radius: 4px; cursor: pointer; white-space: nowrap;",
                onclick: move |_| {
                    let mut input_value = s.input_value;
                    spawn(async move {
                        match crate::ui::qr_scanner::scan_qr().await {
                            Ok(text) => input_value.set(text),
                            Err(e) => {
                                if e != "cancelled" {
                                    crate::ui::log(&format!("QR scan error: {}", e));
                                }
                            }
                        }
                    });
                },
                "QR"
            }
            input {
                style: "flex-grow: 1; min-width: 0; padding: 5px;",
                r#type: "password",
                placeholder: "{i18n.lbl_import_ph()}",
                value: "{s.input_value}",
                oninput: move |evt| {
                    let mut input_value = s.input_value;
                    input_value.set(evt.value())
                }
            }
            button { onclick: move |_| ctrl.import_key(), "{i18n.btn_import()}" }
        }

        // Secret Key Section (Yellow Box)
        if let Some(secret) = s.secret_key_hidden.read().as_ref() {
            div { style: "border: 1px solid #ffeeba; background: #fff3cd; padding: 15px; border-radius: 8px; margin-bottom: 20px;",
                div { style: "display: flex; gap: 10px; flex-wrap: wrap;",
                    button {
                        onclick: move |_| {
                            if *s.show_secret.read() {
                                // Hiding — no auth needed
                                let mut show_secret = s.show_secret;
                                show_secret.set(false);
                            } else {
                                // Revealing — require passkey verification
                                ctrl.reveal_secret();
                            }
                        },
                        if *s.show_secret.read() { "{i18n.btn_hide_secret()}" } else { "{i18n.btn_reveal_secret()}" }
                    }
                    button {
                        style: "background: #28a745; color: white; border: none; padding: 5px 15px; border-radius: 4px;",
                        onclick: move |_| ctrl.copy_secret_to_clipboard(),
                        "{i18n.copy_label()}"
                    }
                    if has_faucet {
                        button {
                            style: "background: #17a2b8; color: white; border: none; padding: 5px 15px; border-radius: 4px;",
                            onclick: move |_| ctrl.activate_test_account_action(),
                            "{i18n.btn_activate_faucet()}"
                        }
                    }
                }

                if *s.show_secret.read() {
                    div { style: "text-align: center; margin-top: 15px;",
                        {
                            let qr_svg = QrCode::new(secret.as_str().as_bytes())
                                .map(|code| {
                                    code.render::<svg::Color>()
                                        .min_dimensions(200, 200)
                                        .max_dimensions(280, 280)
                                        .quiet_zone(true)
                                        .build()
                                })
                                .unwrap_or_default();
                            rsx! {
                                div { style: "display: inline-block; padding: 12px; background: white; border-radius: 12px; box-shadow: 0 2px 8px rgba(0,0,0,0.1);",
                                    dangerous_inner_html: "{qr_svg}"
                                }
                            }
                        }
                    }
                    p { style: "margin-top: 15px; font-family: monospace; word-break: break-all; background: white; padding: 10px;",
                        "{secret.as_str()}"
                    }
                }
            }
        }

        // Persistence
        div { style: "display: flex; gap: 10px; margin-top: 15px;",
            button { onclick: move |_| ctrl.save_to_store(), style: "flex: 1;", "{i18n.btn_save_to_os()}" }
            button { onclick: move |_| ctrl.load_from_store(), style: "flex: 1;", "{i18n.btn_load()}" }
        }

        // GunDB SEA key generation
        div { style: "margin-top: 15px;",
            button {
                style: "width: 100%; padding: 10px; background: #6f42c1; color: white; border: none; border-radius: 5px; font-weight: bold; cursor: pointer;",
                onclick: move |_| ctrl.open_sea_modal(),
                "{i18n.btn_generate_db_secret()}"
            }
            if s.sea_key_pair.read().is_some() {
                p { style: "text-align: center; font-size: 0.8em; color: #28a745; margin-top: 5px;",
                    "{i18n.sea_keys_generated()}"
                }
            }
        }

        // XDR Generator Button
        button {
            style: "margin-top: 30px; width: 100%; padding: 12px; background: #007bff; color: white; border: none; border-radius: 5px; font-weight: bold; cursor: pointer; margin-bottom: 10px;",
            onclick: move |_| ctrl.fetch_and_generate_xdr_action(),
            "{i18n.btn_generate_xdr()}"
        }

        p { style: "text-align: center; font-size: 0.9em; color: #495057; font-style: italic;",
            "{status_display}"
        }

        // Signed XDR Result Box
        if !s.generated_xdr.read().is_empty() {
            div { style: "margin-top: 20px; padding: 15px; background: #e9ecef; border-radius: 8px; border: 1px solid #ced4da;",
                div { style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 10px;",
                    span { style: "font-size: 0.8em; font-weight: bold;", "{i18n.lbl_signed_xdr()}" }
                    button {
                        style: "font-size: 0.7em; padding: 4px 8px;",
                        onclick: move |_| ctrl.copy_xdr_to_clipboard(),
                        "{i18n.copy_xdr_label()}"
                    }
                }
                pre {
                    style: "word-break: break-all; white-space: pre-wrap; font-size: 0.75em; background: white; padding: 10px; border-radius: 4px; border: 1px solid #dee2e6; max-height: 100px; overflow-y: auto;",
                    "{s.generated_xdr}"
                }
                button {
                    style: "width: 100%; margin-top: 15px; padding: 12px; background: #28a745; color: white; border: none; border-radius: 5px; font-weight: bold;",
                    onclick: move |_| ctrl.submit_transaction_action(),
                    "{i18n.btn_submit_tx()}"
                }
            }
        }
    }
}
