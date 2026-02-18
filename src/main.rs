use arboard::Clipboard;
use dioxus::prelude::*;
use keyring::Entry;
use rand::RngCore;
use zeroize::{Zeroize, Zeroizing};

use ed25519_dalek::{Signer, SigningKey};
use stellar_strkey::{ed25519, Strkey};
use stellar_xdr::curr::{
    MuxedAccount, Uint256, Transaction, SequenceNumber, Memo, Operation, 
    OperationBody, PaymentOp, Asset, Preconditions, TransactionExt, VecM,
    TransactionEnvelope, TransactionV1Envelope, DecoratedSignature, Hash,
    Signature, BytesM, SignatureHint, WriteXdr, Limits, TimeBounds, TimePoint,
    TransactionSignaturePayload, TransactionSignaturePayloadTaggedTransaction
};
use sha2::{Sha256, Digest};
use serde::Deserialize;

// === Stellar Hálózat Konfiguráció ===

#[derive(Clone, Copy, PartialEq)]
enum Network {
    Testnet,
    Mainnet,
}

struct NetworkConfig {
    name: &'static str,
    horizon_url: &'static str,
    passphrase: &'static str,
    friendbot_url: Option<&'static str>,
}

fn network_config(network: Network) -> NetworkConfig {
    match network {
        Network::Testnet => NetworkConfig {
            name: "TESTNET",
            horizon_url: "https://horizon-testnet.stellar.org",
            passphrase: "Test SDF Network ; September 2015",
            friendbot_url: Some("https://friendbot.stellar.org"),
        },
        Network::Mainnet => NetworkConfig {
            name: "MAINNET ⚠️",
            horizon_url: "https://horizon.stellar.org",
            passphrase: "Public Global Stellar Network ; September 2015",
            friendbot_url: None,
        },
    }
}

#[derive(Deserialize)]
struct HorizonAccount {
    sequence: String,
}

fn main() {
    let config = dioxus::desktop::Config::new()
        .with_window(
            dioxus::desktop::WindowBuilder::new()
                .with_always_on_top(false)
                .with_title("Zsozso"),
        )
        .with_menu(None);

    LaunchBuilder::desktop().with_cfg(config).launch(app);

    if let Ok(mut clipboard) = Clipboard::new() {
        let _ = clipboard.set_text("".to_string());
        std::thread::sleep(std::time::Duration::from_millis(500));
        println!("🔐 Vágólap törölve a biztonság érdekében.");
    }
}

fn save_to_secure_storage(secret: &str) -> keyring::Result<()> {
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
    let clipboard_status = use_signal(|| String::from("Másolás"));
    let mut input_value = use_signal(|| String::new());
    let mut generated_xdr = use_signal(|| String::new());
    let xdr_copy_status = use_signal(|| String::from("XDR Másolása"));
    let mut submission_status = use_signal(|| String::from("Várakozás..."));
    let mut active_clipboard_task = use_signal(|| None);
    let mut current_network = use_signal(|| Network::Testnet);

    let mut safe_copy = move |text: String, mut status_signal: Signal<String>, is_secret: bool| {
        if let Some(_task) = active_clipboard_task.write().take() {}

        let new_task = spawn(async move {
            if let Ok(mut cb) = arboard::Clipboard::new() {
                let _ = cb.set_text(text);

                let original_label = status_signal.peek().clone();
                status_signal.set("MÁSOLVA!".to_string());

                let wait_secs = if is_secret { 30 } else { 10 };
                tokio::time::sleep(std::time::Duration::from_secs(wait_secs)).await;

                if is_secret {
                    let _ = cb.set_text("".to_string());
                    std::thread::sleep(std::time::Duration::from_millis(500));
                }

                status_signal.set(original_label);
            }
        });

        active_clipboard_task.set(Some(new_task));
    };

    use_drop(move || {
        if let Ok(mut cb) = arboard::Clipboard::new() {
            let _ = cb.set_text("".to_string());
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
    });

    let submit_tx_action = move |_| {
        let xdr_to_submit = generated_xdr.read().clone();
        let net = network_config(*current_network.read());

        if xdr_to_submit.is_empty() {
            submission_status.set("Hiba: Nincs generált XDR!".to_string());
            return;
        }

        spawn(async move {
            submission_status.set("Beküldés folyamatban...".to_string());

            let url = format!("{}/transactions", net.horizon_url);
            let client = reqwest::Client::new();
            let params = [("tx", xdr_to_submit)];

            match client.post(url)
                .header("Content-Type", "application/x-www-form-urlencoded")
                .form(&params)
                .send()
                .await
            {
                Ok(response) => {
                    let status = response.status();
                    let body = response.text().await.unwrap_or_default();

                    if status.is_success() {
                        submission_status.set("✅ SIKER! Tranzakció elfogadva.".to_string());
                    } else {
                        println!("Horizon hiba ({}): {}", status, body);
                        submission_status.set(format!("❌ Hiba: {}", status));
                    }
                }
                Err(e) => {
                    submission_status.set(format!("❌ Hálózati hiba: {}", e));
                }
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
        let net = network_config(*current_network.read());

        if pubkey == "Nincs kulcs betöltve" { return; }

        let friendbot_base = match net.friendbot_url {
            Some(url) => url.to_string(),
            None => {
                submission_status.set("⚠️ Mainnet-en nincs Friendbot!".to_string());
                return;
            }
        };

        spawn(async move {
            submission_status.set("🚀 Friendbot hívása...".to_string());
            let url = format!("{}/?addr={}", friendbot_base, pubkey);

            match reqwest::get(url).await {
                Ok(resp) if resp.status().is_success() => {
                    submission_status.set("✅ Fiók aktiválva! (10,000 XLM)".to_string());
                },
                Ok(resp) => {
                    submission_status.set(format!("❌ Friendbot hiba: {}", resp.status()));
                },
                Err(e) => {
                    submission_status.set(format!("❌ Hálózati hiba: {}", e));
                }
            }
        });
    };

    let fetch_and_generate = move |_| {
        let pubkey_str = public_key.read().clone();
        let secret_str_opt = secret_key_hidden.read().clone();
        let net = network_config(*current_network.read());

        if pubkey_str == "Nincs kulcs betöltve" {
            submission_status.set("⚠️ Nincs betöltött kulcs!".to_string());
            return;
        }

        spawn(async move {
            submission_status.set("🔍 Szekvenciaszám lekérése...".to_string());

            let url = format!("{}/accounts/{}", net.horizon_url, pubkey_str);
            let client = reqwest::Client::new();

            let response = match client.get(url).send().await {
                Ok(r) => r,
                Err(e) => {
                    submission_status.set(format!("❌ Horizon nem elérhető: {}", e));
                    return;
                }
            };

            if !response.status().is_success() {
                submission_status.set("❌ Fiók nem található! Előbb aktiváld!".to_string());
                return;
            }

            let account_data = match response.json::<HorizonAccount>().await {
                Ok(data) => data,
                Err(e) => {
                    submission_status.set(format!("❌ JSON hiba: {}", e));
                    return;
                }
            };

            let current_seq: i64 = account_data.sequence.parse().unwrap_or(0);
            let next_seq = current_seq + 1;

            if let Some(secret_str) = secret_str_opt.as_ref() {
                let secret_val = secret_str.to_string();

                if let Ok(Strkey::PrivateKeyEd25519(priv_key)) = Strkey::from_string(&secret_val) {
                    let signing_key = ed25519_dalek::SigningKey::from_bytes(&priv_key.0);
                    let pub_bytes = signing_key.verifying_key().to_bytes();

                    let current_unix_time = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs();

                    let time_bounds = TimeBounds {
                        min_time: TimePoint(0),
                        max_time: TimePoint(current_unix_time + 300),
                    };

                    let tx = Transaction {
                        source_account: MuxedAccount::Ed25519(Uint256(pub_bytes)),
                        fee: 100,
                        seq_num: SequenceNumber(next_seq),
                        cond: Preconditions::Time(time_bounds),
                        memo: Memo::None,
                        operations: VecM::try_from(vec![
                            Operation {
                                source_account: None,
                                body: OperationBody::Payment(PaymentOp {
                                    destination: MuxedAccount::Ed25519(Uint256(pub_bytes)),
                                    asset: Asset::Native,
                                    amount: 100_000_000,
                                }),
                            }
                        ]).unwrap(),
                        ext: TransactionExt::V0,
                    };

                    let network_id = Hash(Sha256::digest(net.passphrase.as_bytes()).into());

                    let payload = TransactionSignaturePayload {
                        network_id,
                        tagged_transaction: TransactionSignaturePayloadTaggedTransaction::Tx(tx.clone()),
                    };

                    let tx_payload_xdr = payload.to_xdr(Limits::none()).unwrap();
                    let tx_hash = Sha256::digest(&tx_payload_xdr);
                    let sig_bytes = signing_key.sign(&tx_hash).to_bytes();

                    let mut hint_bytes = [0u8; 4];
                    hint_bytes.copy_from_slice(&pub_bytes[pub_bytes.len() - 4..]);

                    let envelope = TransactionEnvelope::Tx(TransactionV1Envelope {
                        tx: tx.clone(),
                        signatures: VecM::try_from(vec![
                            DecoratedSignature {
                                hint: SignatureHint(hint_bytes),
                                signature: Signature(BytesM::try_from(sig_bytes).unwrap()),
                            }
                        ]).unwrap(),
                    });

                    match envelope.to_xdr_base64(Limits::none()) {
                        Ok(xdr) => {
                            generated_xdr.set(xdr);
                            submission_status.set(format!("✅ XDR Kész! [{}] (Seq: {})", net.name, next_seq));
                        },
                        Err(e) => submission_status.set(format!("❌ XDR hiba: {:?}", e)),
                    }
                }
            }
        });
    };

    let generate_key = move |_| {
        let mut seed_bytes = [0u8; 32];
        rand::rng().fill_bytes(&mut seed_bytes);

        let signing_key = SigningKey::from_bytes(&seed_bytes);
        let verifying_key = signing_key.verifying_key();
        let pub_bytes = verifying_key.to_bytes();

        let secret_str = Strkey::PrivateKeyEd25519(ed25519::PrivateKey(seed_bytes)).to_string();
        let pub_key_str = Strkey::PublicKeyEd25519(ed25519::PublicKey(pub_bytes)).to_string();

        public_key.set(pub_key_str);
        secret_key_hidden.set(Some(Zeroizing::new(secret_str)));
        seed_bytes.zeroize();
    };

    let import_key = move |_| {
        let raw_input = input_value.read().clone();
        if let Ok(Strkey::PrivateKeyEd25519(priv_key)) = Strkey::from_string(&raw_input) {
            let signing_key = SigningKey::from_bytes(&priv_key.0);
            let pub_bytes = signing_key.verifying_key().to_bytes();

            let pub_key_str = Strkey::PublicKeyEd25519(ed25519::PublicKey(pub_bytes)).to_string();

            public_key.set(pub_key_str);
            secret_key_hidden.set(Some(Zeroizing::new(raw_input)));
            input_value.set(String::new());
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
                if let Ok(Strkey::PrivateKeyEd25519(priv_key)) = Strkey::from_string(&secret) {
                    let seed_bytes: [u8; 32] = priv_key.0;
                    let signing_key = SigningKey::from_bytes(&seed_bytes);
                    let pub_bytes = signing_key.verifying_key().to_bytes();
                    let public_key_str = Strkey::PublicKeyEd25519(ed25519::PublicKey(pub_bytes)).to_string();
                    public_key.set(public_key_str);
                    secret_key_hidden.set(Some(Zeroizing::new(secret)));
                    println!("✨ UI sikeresen frissítve a betöltött kulccsal.");
                }
            }
            Err(e) => println!("❌ Betöltési hiba: {:?}", e),
        }
    };

    // === Render előkészítés ===
    let is_testnet = *current_network.read() == Network::Testnet;
    let has_friendbot = network_config(*current_network.read()).friendbot_url.is_some();
    let network_btn_style = format!(
        "margin-bottom: 20px; padding: 8px 20px; border: none; border-radius: 4px; font-weight: bold; cursor: pointer; color: white; background: {};",
        if is_testnet { "#dc3545" } else { "#17a2b8" }
    );
    let network_btn_label = if is_testnet { "🧪 Testnet ⚠️" } else { "Mainnet" };

    rsx! {
        div { style: "padding: 30px; font-family: sans-serif; max-width: 550px; margin: auto;",
            h2 { "Zsozso" }

            // === HÁLÓZAT VÁLTÓ ===
            button {
                style: "{network_btn_style}",
                onclick: move |_| {
                    let next = if *current_network.read() == Network::Testnet {
                        Network::Mainnet
                    } else {
                        Network::Testnet
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
                        if has_friendbot {
                            button {
                                style: "background: #17a2b8; color: white; border: none; padding: 5px 15px; border-radius: 4px;",
                                onclick: activate_account,
                                "🚀 Aktiválás (Friendbot)"
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