mod clipboard;

use dioxus::prelude::*;
use zeroize::Zeroizing;

use crate::ledger::{Ledger, NetworkEnvironment, StellarLedger};
use crate::store::{Store, KeyringStore};
use clipboard::safe_copy;

pub fn app() -> Element {
    let mut public_key = use_signal(|| String::from("Nincs kulcs betöltve"));
    let mut secret_key_hidden = use_signal(|| None::<Zeroizing<String>>);
    let mut show_secret = use_signal(|| false);
    let clipboard_status = use_signal(|| String::from("Másolás"));
    let mut input_value = use_signal(|| String::new());
    let mut generated_xdr = use_signal(|| String::new());
    let xdr_copy_status = use_signal(|| String::from("XDR Másolása"));
    let mut submission_status = use_signal(|| String::from("Várakozás..."));
    let mut current_network = use_signal(|| NetworkEnvironment::Production);

    use_drop(move || {
        if let Ok(mut cb) = arboard::Clipboard::new() {
            let _ = cb.set_text("".to_string());
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
    });

    let submit_tx_action = move |_| {
        let xdr_to_submit = generated_xdr.read().clone();
        let net_env = *current_network.read();

        if xdr_to_submit.is_empty() {
            submission_status.set("Hiba: Nincs generált XDR!".to_string());
            return;
        }

        spawn(async move {
            submission_status.set("Beküldés folyamatban...".to_string());
            let lgr = StellarLedger::new(net_env);

            match lgr.submit_transaction(&xdr_to_submit).await {
                Ok(msg) => submission_status.set(format!("✅ SIKER! {}", msg)),
                Err(e) => submission_status.set(format!("❌ {}", e)),
            }
        });
    };

    let copy_to_clipboard = move |_| {
        if let Some(secret) = secret_key_hidden.read().as_ref() {
            safe_copy(secret.to_string(), clipboard_status, true);
        }
    };

    let copy_xdr_to_clipboard = move |_| {
        let xdr = generated_xdr.read().clone();
        if !xdr.is_empty() {
            safe_copy(xdr, xdr_copy_status, false);
        }
    };

    let activate_account = move |_| {
        let pubkey = public_key.read().clone();
        let net_env = *current_network.read();

        if pubkey == "Nincs kulcs betöltve" { return; }

        spawn(async move {
            submission_status.set("🚀 Faucet hívása...".to_string());
            let lgr = StellarLedger::new(net_env);

            match lgr.activate_test_account(&pubkey).await {
                Ok(msg) => submission_status.set(format!("✅ {}", msg)),
                Err(e) => submission_status.set(format!("❌ {}", e)),
            }
        });
    };

    let fetch_and_generate = move |_| {
        let secret_str_opt = secret_key_hidden.read().clone();
        let net_env = *current_network.read();

        if secret_str_opt.is_none() {
            submission_status.set("⚠️ Nincs betöltött kulcs!".to_string());
            return;
        }

        let secret_val = secret_str_opt.unwrap().to_string();

        spawn(async move {
            submission_status.set("🔍 Szekvenciaszám lekérése...".to_string());
            let lgr = StellarLedger::new(net_env);
            let net_info = lgr.network_info();

            match lgr.build_self_payment(&secret_val, 100_000_000).await {
                Ok((xdr, seq)) => {
                    generated_xdr.set(xdr);
                    submission_status.set(format!("✅ XDR Kész! [{}] (Seq: {})", net_info.name, seq));
                }
                Err(e) => submission_status.set(format!("❌ {}", e)),
            }
        });
    };

    let generate_key = move |_| {
        let lgr = StellarLedger::new(*current_network.read());
        let kp = lgr.generate_keypair();

        public_key.set(kp.public_key);
        secret_key_hidden.set(Some(Zeroizing::new(kp.secret_key)));
    };

    let import_key = move |_| {
        let raw_input = input_value.read().clone();
        let lgr = StellarLedger::new(*current_network.read());

        if let Some(pub_key_str) = lgr.public_key_from_secret(&raw_input) {
            public_key.set(pub_key_str);
            secret_key_hidden.set(Some(Zeroizing::new(raw_input)));
            input_value.set(String::new());
        }
    };

    let save_action = move |_| {
        if let Some(secret) = secret_key_hidden.read().as_ref() {
            let store = KeyringStore::new("zsozso", "default_account");
            match store.save(secret.as_str()) {
                Ok(_) => println!("✅ Sikeres mentés a rendszer-tárcába!"),
                Err(e) => println!("❌ {}", e),
            }
        } else {
            println!("⚠️ Nincs mit menteni (üres a kulcs)!");
        }
    };

    let load_action = move |_| {
        println!("🔍 Betöltés megkezdése...");
        let store = KeyringStore::new("zsozso", "default_account");
        match store.load() {
            Ok(secret) => {
                let secret: String = secret;  // ← típus megadása
                println!("📥 Kulcs betöltve, hossza: {}", secret.len());
                let lgr = StellarLedger::new(*current_network.read());

                if let Some(pub_key_str) = lgr.public_key_from_secret(&secret) {
                    public_key.set(pub_key_str);
                    secret_key_hidden.set(Some(Zeroizing::new(secret)));
                    println!("✨ UI sikeresen frissítve a betöltött kulccsal.");
                }
            }
            Err(e) => println!("❌ {}", e),
        }
    };
    
    // === Render előkészítés ===
    let net_env = *current_network.read();
    let lgr_for_render = StellarLedger::new(net_env);
    let net_info = lgr_for_render.network_info();
    let is_production = net_env == NetworkEnvironment::Production;
    let has_faucet = net_info.has_faucet;
    let network_btn_style = format!(
        "margin-bottom: 20px; padding: 8px 20px; border: none; border-radius: 4px; font-weight: bold; cursor: pointer; color: white; background: {};",
        if !is_production { "#dc3545" } else { "#17a2b8" }
    );
    let network_btn_label = if !is_production { "🧪 Testnet ⚠️" } else { "Mainnet" };

    rsx! {
        div { style: "padding: 30px; font-family: sans-serif; max-width: 550px; margin: auto;",
            h2 { "Zsozso" }

            // === HÁLÓZAT VÁLTÓ ===
            button {
                style: "{network_btn_style}",
                onclick: move |_| {
                    let next = if *current_network.read() == NetworkEnvironment::Production {
                        NetworkEnvironment::Test
                    } else {
                        NetworkEnvironment::Production
                    };
                    current_network.set(next);
                    generated_xdr.set(String::new());
                },
                "{network_btn_label}"
            }

            // --- CÍM MEGJELENÍTÉSE ---
            div { style: "background: #f8f9fa; padding: 15px; border-radius: 8px; margin-bottom: 20px; border: 1px solid #ddd;",
                p { style: "font-size: 0.8em; color: #666; margin: 0;", "Aktív Cím (Public Key):" }
                code { style: "word-break: break-all; font-weight: bold;", "{public_key}" }
            }

            // --- KULCSKEZELÉS GOMBOK ---
            div { style: "display: flex; gap: 10px; margin-bottom: 20px;",
                button { onclick: generate_key, "✨ Új Kulcs" }
                input {
                    style: "flex-grow: 1; padding: 5px;",
                    r#type: "password",
                    placeholder: "Importálás (S...)",
                    value: "{input_value}",
                    oninput: move |evt| input_value.set(evt.value())
                }
                button { onclick: import_key, "📥 Import" }
            }

            // --- TITKOS KULCS SZEKCIÓ ---
            if let Some(secret) = secret_key_hidden.read().as_ref() {
                div { style: "border: 1px solid #ffeeba; background: #fff3cd; padding: 15px; border-radius: 8px; margin-bottom: 20px;",
                    div { style: "display: flex; gap: 10px; flex-wrap: wrap;",
                        button {
                            onclick: move |_| show_secret.toggle(),
                            if *show_secret.read() { "🙈 Elrejtés" } else { "👁 Felfedés" }
                        }
                        button {
                            style: "background: #28a745; color: white; border: none; padding: 5px 15px; border-radius: 4px;",
                            onclick: copy_to_clipboard,
                            "{clipboard_status}"
                        }
                        if has_faucet {
                            button {
                                style: "background: #17a2b8; color: white; border: none; padding: 5px 15px; border-radius: 4px;",
                                onclick: activate_account,
                                "🚀 Aktiválás (Faucet)"
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

            // --- MENTÉS / BETÖLTÉS ---
            div { style: "display: flex; gap: 10px; margin-top: 15px;",
                button { onclick: save_action, style: "flex: 1;", "💾 Mentés az OS tárcába" }
                button { onclick: load_action, style: "flex: 1;", "🔓 Betöltés" }
            }

            // --- TRANZAKCIÓ GENERÁLÁSA ---
            button {
                style: "margin-top: 30px; width: 100%; padding: 12px; background: #007bff; color: white; border: none; border-radius: 5px; font-weight: bold; cursor: pointer; margin-bottom: 10px;",
                onclick: fetch_and_generate,
                "🛠 Tranzakció XDR Generálása"
            }

            // --- STÁTUSZ ÜZENET ---
            p { style: "text-align: center; font-size: 0.9em; color: #495057; font-style: italic;",
                "{submission_status}"
            }

            // --- GENERÁLT XDR BLOKK ---
            if !generated_xdr.read().is_empty() {
                div { style: "margin-top: 20px; padding: 15px; background: #e9ecef; border-radius: 8px; border: 1px solid #ced4da;",
                    div { style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 10px;",
                        span { style: "font-size: 0.8em; font-weight: bold;", "ALÁÍRT XDR:" }
                        button {
                            style: "font-size: 0.7em; padding: 4px 8px;",
                            onclick: copy_xdr_to_clipboard,
                            "{xdr_copy_status}"
                        }
                    }
                    pre {
                        style: "word-break: break-all; white-space: pre-wrap; font-size: 0.75em; background: white; padding: 10px; border-radius: 4px; border: 1px solid #dee2e6; max-height: 100px; overflow-y: auto;",
                        "{generated_xdr}"
                    }
                    button {
                        style: "width: 100%; margin-top: 15px; padding: 12px; background: #28a745; color: white; border: none; border-radius: 5px; font-weight: bold;",
                        onclick: submit_tx_action,
                        "🚀 Tranzakció BEKÜLDÉSE"
                    }
                }
            }
        }
    }
}