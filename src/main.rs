use arboard::Clipboard;
use dioxus::prelude::*;
use keyring::Entry;
use rand::RngCore;
use std::str::FromStr;
use std::time::Duration;
use zeroize::{Zeroize, Zeroizing};

use stellar_base::{
    amount::{Amount, Stroops},
    asset::Asset,
    crypto::{KeyPair, MuxedAccount},
    memo::Memo,
    network::Network,
    operations::Operation,
    operations::PaymentOperation,
    transaction::{Transaction, TransactionBuilder},
    xdr::{PaymentOp, XDRSerialize},
};

fn main() {
    // In your main function, you can tweak the desktop configuration
    let config = dioxus::desktop::Config::new()
        .with_window(
            dioxus::desktop::WindowBuilder::new()
                .with_always_on_top(false) // Force it to behave
                .with_title("Zsozso"),
        )
        .with_menu(None); // Removes the default menu bar

    LaunchBuilder::desktop().with_cfg(config).launch(app);
    //launch(app);
}

fn save_to_secure_storage(secret: &str) -> keyring::Result<()> {
    // Az "entry" azonosítja az alkalmazást és a fiókot
    let entry = Entry::new("my_stellar_app", "default_account")?;
    let _ = entry.delete_credential();
    entry.set_password(secret)?;
    Ok(())
}

fn load_from_secure_storage() -> keyring::Result<String> {
    let entry = Entry::new("my_stellar_app", "default_account")?;
    entry.get_password()
}

fn app() -> Element {
    let mut public_key = use_signal(|| String::from("Nincs kulcs betöltve"));
    let mut secret_key_hidden = use_signal(|| None::<Zeroizing<String>>);
    let mut show_secret = use_signal(|| false);
    let mut clipboard_status = use_signal(|| String::from("Másolás"));
    let mut input_value = use_signal(|| String::new());

    // --- Generálás ---
    let generate_key = move |_| {
        let mut entropy = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut entropy);

        if let Ok(sk) = KeyPair::from_seed_bytes(&entropy) {
            public_key.set(sk.public_key().account_id());
            secret_key_hidden.set(Some(Zeroizing::new(
                sk.secret_key().secret_seed().to_string(),
            )));
        }
        entropy.zeroize();
        show_secret.set(false);
    };

    // --- Importálás ---
    let import_key = move |_| {
        let raw_input = input_value.read().clone();
        if raw_input.starts_with('S') && raw_input.len() == 56 {
            if let Ok(sk) = KeyPair::from_str(&raw_input) {
                public_key.set(sk.public_key().account_id());
                secret_key_hidden.set(Some(Zeroizing::new(raw_input)));
                input_value.set(String::new());
                show_secret.set(false);
            }
        }
    };

    // --- Biztonságos Másolás ---
    let copy_to_clipboard = move |_| {
        if let Some(secret) = secret_key_hidden.read().as_ref() {
            let secret_to_copy = secret.to_string();
            if let Ok(mut clipboard) = Clipboard::new() {
                let _ = clipboard.set_text(secret_to_copy);
                clipboard_status.set("MÁSOLVA (30mp)".to_string());

                spawn(async move {
                    tokio::time::sleep(Duration::from_secs(30)).await;
                    if let Ok(mut cb) = Clipboard::new() {
                        if let Ok(content) = cb.get_text() {
                            if content.starts_with('S') && content.len() == 56 {
                                let _ = cb.set_text("".to_string());
                            }
                        }
                    }
                    clipboard_status.set("Másolás".to_string());
                });
            }
        }
    };

    let save_action = move |_| {
        if let Some(secret) = secret_key_hidden.read().as_ref() {
            match save_to_secure_storage(secret.as_str()) {
                Ok(_) => println!("✅ Sikeres mentés a rendszer-tárcába!"),
                Err(e) => println!("❌ Mentési hiba: {:?}", e),
            }
        } else {
            println!("⚠️ Nincs mit menteni (üres a kulcs)!");
        }
    };

    let load_action = move |_| {
        println!("🔍 Betöltés megkezdése...");
        match load_from_secure_storage() {
            Ok(secret) => {
                println!("📥 Kulcs betöltve, hossza: {}", secret.len());
                // Itt fontos a helyes metódus: from_str vagy from_secret_seed
                if let Ok(sk) = KeyPair::from_str(&secret) {
                    public_key.set(sk.public_key().account_id());
                    secret_key_hidden.set(Some(Zeroizing::new(secret)));
                    println!("✨ UI sikeresen frissítve a betöltött kulccsal.");
                }
            }
            Err(e) => println!("❌ Betöltési hiba: {:?}", e),
        }
    };

    rsx! {
        div { style: "padding: 30px; font-family: sans-serif; max-width: 550px; margin: auto;",
            h2 { "Zsozso" }

            div { style: "background: #f8f9fa; padding: 15px; border-radius: 8px; margin-bottom: 20px; border: 1px solid #ddd;",
                p { style: "font-size: 0.8em; color: #666; margin: 0;", "Aktív Cím (Public Key):" }
                code { style: "word-break: break-all; font-weight: bold;", "{public_key}" }
            }

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

            if let Some(secret) = secret_key_hidden.read().as_ref() {
                div { style: "border: 1px solid #ffeeba; background: #fff3cd; padding: 15px; border-radius: 8px;",
                    div { style: "display: flex; gap: 10px;",
                        button {
                            onclick: move |_| show_secret.toggle(),
                            if *show_secret.read() { "🙈 Elrejtés" } else { "👁 Felfedés" }
                        }
                        button {
                            style: "background: #28a745; color: white; border: none; padding: 5px 15px; border-radius: 4px;",
                            onclick: copy_to_clipboard,
                            "{clipboard_status}"
                        }
                    }

                    if *show_secret.read() {
                        p { style: "margin-top: 15px; font-family: monospace; word-break: break-all; background: white; padding: 10px;",
                            "{secret.as_str()}"
                        }
                    }
                }
            }

            div { style: "display: flex; gap: 10px;",
                button { onclick: save_action, "💾 Mentés az OS tárcába" }
                button { onclick: load_action, "🔓 Betöltés (Biometria/Pass)" }
            }

            hr { margin: "20px 0" }
        }
    }
}
