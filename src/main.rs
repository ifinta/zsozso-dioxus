use arboard::Clipboard;
use dioxus::prelude::*;
use keyring::Entry;
use rand::RngCore;
use std::time::Duration;
use zeroize::{Zeroize, Zeroizing};

// Az új típusok a 25.0.0 verzióból
use ed25519_dalek::{Signer, SigningKey};
use stellar_strkey::{ed25519, Strkey};
use stellar_xdr::curr::{
    MuxedAccount, Uint256, Transaction, SequenceNumber, Memo, Operation, 
    OperationBody, PaymentOp, Asset, Preconditions, TransactionExt, VecM,
    TransactionEnvelope, TransactionV1Envelope, DecoratedSignature, 
    Signature, EnvelopeType, BytesM, SignatureHint, WriteXdr, Limits
};
use sha2::{Sha256, Digest};
use serde::Deserialize;

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
    let mut clipboard_status = use_signal(|| String::from("Másolás"));
    let mut input_value = use_signal(|| String::new());
    let mut generated_xdr = use_signal(|| String::new());
    let mut xdr_copy_status = use_signal(|| String::from("XDR Másolása"));
    let mut submission_status = use_signal(|| String::from("Várakozás..."));

    let submit_tx_action = move |_| {
        let xdr_to_submit = generated_xdr.read().clone();
        
        if xdr_to_submit.is_empty() {
            submission_status.set("Hiba: Nincs generált XDR!".to_string());
            return;
        }

        spawn(async move {
            submission_status.set("Beküldés folyamatban...".to_string());
            
            // A Horizon API URL-je (Testnet)
            let url = "https://horizon-testnet.stellar.org";
            
            // A paraméter formátuma: tx=BASE64_XDR
            let params = [("tx", xdr_to_submit)];
            let client = reqwest::Client::new();

            match client.post(url).form(&params).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        submission_status.set("✅ SIKER! Tranzakció elfogadva.".to_string());
                    } else {
                        let error_text = response.text().await.unwrap_or_default();
                        println!("Horizon hiba: {}", error_text);
                        submission_status.set("❌ Hiba: A hálózat elutasította.".to_string());
                    }
                }
                Err(e) => {
                    submission_status.set(format!("❌ Hálózati hiba: {}", e));
                }
            }
        });
    };

    let copy_xdr_to_clipboard = move |_| {
        let xdr_text = generated_xdr.read().clone();
        if !xdr_text.is_empty() {
            if let Ok(mut clipboard) = Clipboard::new() {
                let _ = clipboard.set_text(xdr_text);
                xdr_copy_status.set("MÁSOLVA!".to_string());

                // Visszaállítás 5 másodperc után
                spawn(async move {
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    xdr_copy_status.set("XDR Másolása".to_string());
                });
            }
        }
    };

    // // --- Biztonságos Másolás ---
    // let copy_to_clipboard = move |_| {
    //     if let Some(secret) = secret_key_hidden.read().as_ref() {
    //         let secret_to_copy = secret.to_string();
    //         if let Ok(mut clipboard) = Clipboard::new() {
    //             let _ = clipboard.set_text(secret_to_copy);
    //             clipboard_status.set("MÁSOLVA (30mp)".to_string());

    //             spawn(async move {
    //                 tokio::time::sleep(Duration::from_secs(30)).await;
    //                 if let Ok(mut cb) = Clipboard::new() {
    //                     if let Ok(content) = cb.get_text() {
    //                         if content.starts_with('S') && content.len() == 56 {
    //                             let _ = cb.set_text("".to_string());
    //                         }
    //                     }
    //                 }
    //                 clipboard_status.set("Másolás".to_string());
    //             });
    //         }
    //     }
    // };

    // let fetch_sequence_and_create_xdr = move |_| {
    //     let pubkey_str = public_key.read().clone();
    //     let secret_str_opt = secret_key_hidden.read().clone();

    //     if pubkey_str == "Nincs kulcs betöltve" { return; }

    //     spawn(async move {
    //         submission_status.set("Szekvenciaszám lekérése...".to_string());
            
    //         let url = format!("https://horizon-testnet.stellar.org{}", pubkey_str);
    //         let client = reqwest::Client::new();

    //         match client.get(url).send().await {
    //             Ok(resp) if resp.status().is_success() => {
    //                 if let Ok(account_data) = resp.json::<HorizonAccount>().await {
    //                     let current_seq: i64 = account_data.sequence.parse().unwrap_or(0);
    //                     let next_seq = current_seq + 1;
                        
    //                     // Most már generálhatjuk az XDR-t a friss szekvenciával
    //                     if let Some(secret_str) = secret_str_opt.as_ref() {
    //                         create_signed_transaction(secret_str, next_seq);
    //                     }
    //                 }
    //             },
    //             _ => submission_status.set("❌ Hiba: Nem található a fiók (nincs aktiválva?)".to_string()),
    //         }
    //     });
    // };

    // // Ez a függvény a szekvencia birtokában állítja össze a végleges XDR-t
    // fn create_signed_transaction(secret_str: &str, next_seq: i64) {
    //     // ... Itt jön az előzőleg megírt Transaction envelope-os kód ...
    //     if let Ok(Strkey::PrivateKeyEd25519(priv_key)) = Strkey::from_string(secret_str.as_str()) {
            
    //         let signing_key = ed25519_dalek::SigningKey::from_bytes(&priv_key.0);
    //         let pub_bytes = signing_key.verifying_key().to_bytes();
    //         let source_pubkey = Uint256(pub_bytes);

    //         let tx = Transaction {
    //             source_account: MuxedAccount::Ed25519(source_pubkey),
    //             fee: 100,
    //             seq_num: SequenceNumber(next_seq),
    //             cond: Preconditions::None,
    //             memo: Memo::None,
    //             operations: VecM::try_from(vec![
    //                 Operation {
    //                     source_account: None,
    //                     body: OperationBody::Payment(PaymentOp {
    //                         destination: MuxedAccount::Ed25519(Uint256(pub_bytes)),
    //                         asset: Asset::Native,
    //                         amount: 100_000_000, 
    //                     }),
    //                 }
    //             ]).unwrap(),
    //             ext: TransactionExt::V0,
    //         };

    //         // --- ALÁÍRÁS JAVÍTVA ---
            
    //         // 1. Network ID kiszámítása (SHA256 a passphrase-re)
    //         let network_passphrase = "Test SDF Network ; September 2015";
    //         let network_id = Sha256::digest(network_passphrase.as_bytes());

    //         // 2. Aláírandó payload összeállítása: NetworkID + EnvelopeType + TransactionXDR
    //         let mut sig_payload = Vec::new();
    //         sig_payload.extend_from_slice(&network_id);
    //         sig_payload.extend_from_slice(&(EnvelopeType::Tx as i32).to_be_bytes()); // 4 bájtos BigEndian típus
    //         sig_payload.extend_from_slice(&tx.to_xdr(Limits::none()).unwrap());
            
    //         // 3. Aláírás és típuskonverzió (BytesM<64>)
    //         let signature_bytes = signing_key.sign(&sig_payload).to_bytes();
    //         let signature = Signature(BytesM::try_from(signature_bytes).unwrap());

    //         // 4. Hint (Pubkey utolsó 4 bájtja)
    //         let mut hint_bytes = [0u8; 4];
    //         hint_bytes.copy_from_slice(&pub_bytes[28..]);

    //         let decorated_signature = DecoratedSignature {
    //             hint: SignatureHint(hint_bytes),
    //             signature,
    //         };

    //         // 5. Boríték és Base64
    //         let envelope = TransactionEnvelope::Tx(TransactionV1Envelope {
    //             tx,
    //             signatures: VecM::try_from(vec![decorated_signature]).unwrap(),
    //         });

    //         if let Ok(xdr_string) = envelope.to_xdr_base64(Limits::none()) {
    //             generated_xdr.set(xdr_string);
    //             submission_status.set(format!("XDR kész (Seq: {})", next_seq));
    //         }
    //     }
    // }

    let activate_account = move |_| {
        let pubkey = public_key.read().clone();
        spawn(async move {
            submission_status.set("Friendbot hívása...".to_string());
            let url = format!("https://friendbot.stellar.org{}", pubkey);
            if let Ok(_) = reqwest::get(url).await {
                submission_status.set("✅ Fiók aktiválva! (10,000 XLM)".to_string());
            }
        });
    };

    // A szekvencia lekérő és generáló gomb logic az app() { ... } -en belül
    let fetch_and_generate = move |_| {
        let pubkey_str = public_key.read().clone();
        let secret_str_opt = secret_key_hidden.read().clone();

        if pubkey_str == "Nincs kulcs betöltve" { return; }

        spawn(async move {
            submission_status.set("Szekvenciaszám lekérése...".to_string());
            
            let url = format!("https://horizon-testnet.stellar.org{}", pubkey_str);
            let client = reqwest::Client::new();

            if let Ok(resp) = client.get(url).send().await {
                if resp.status().is_success() {
                    if let Ok(account_data) = resp.json::<HorizonAccount>().await {
                        let current_seq: i64 = account_data.sequence.parse().unwrap_or(0);
                        let next_seq = current_seq + 1;
                        
                        if let Some(secret_str) = secret_str_opt.as_ref() {
                            // Itt hívjuk meg a generáló logikát
                            // A secret_str-t String-ként adjuk át
                            let secret_val = secret_str.to_string();
                            
                            // Kriptográfiai rész
                            if let Ok(Strkey::PrivateKeyEd25519(priv_key)) = Strkey::from_string(&secret_val) {
                                let signing_key = ed25519_dalek::SigningKey::from_bytes(&priv_key.0);
                                let pub_bytes = signing_key.verifying_key().to_bytes();
                                
                                // Tranzakció összeállítása a kapott next_seq-vel
                                let tx = Transaction {
                                    source_account: MuxedAccount::Ed25519(Uint256(pub_bytes)),
                                    fee: 100,
                                    seq_num: SequenceNumber(next_seq),
                                    cond: Preconditions::None,
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

                                // Aláírás (Network ID, Hash, Envelope...)
                                let network_passphrase = "Test SDF Network ; September 2015";
                                let network_id = Sha256::digest(network_passphrase.as_bytes());
                                let mut sig_payload = Vec::new();
                                sig_payload.extend_from_slice(&network_id);
                                sig_payload.extend_from_slice(&(EnvelopeType::Tx as i32).to_be_bytes());
                                sig_payload.extend_from_slice(&tx.to_xdr(Limits::none()).unwrap());
                                
                                let sig_bytes = signing_key.sign(&sig_payload).to_bytes();
                                let mut hint = [0u8; 4];
                                hint.copy_from_slice(&pub_bytes[28..]);

                                let envelope = TransactionEnvelope::Tx(TransactionV1Envelope {
                                    tx,
                                    signatures: VecM::try_from(vec![
                                        DecoratedSignature {
                                            hint: SignatureHint(hint),
                                            signature: Signature(BytesM::try_from(sig_bytes).unwrap()),
                                        }
                                    ]).unwrap(),
                                });

                                if let Ok(xdr) = envelope.to_xdr_base64(Limits::none()) {
                                    generated_xdr.set(xdr);
                                    submission_status.set(format!("XDR Kész! (Seq: {})", next_seq));
                                }
                            }
                        }
                    }
                } else {
                    submission_status.set("❌ Hiba: Fiók nem található (aktiváld Friendbottal!)".to_string());
                }
            }
        });
    };

    let generate_key = move |_| {
        let mut seed_bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut seed_bytes);

        // 1. Kiszámoljuk a valódi publikus kulcsot a dalek segítségével
        let signing_key = SigningKey::from_bytes(&seed_bytes);
        let verifying_key = signing_key.verifying_key();
        let pub_bytes = verifying_key.to_bytes();

        // 2. Kódoljuk Stellar formátumra
        let secret_str = Strkey::PrivateKeyEd25519(ed25519::PrivateKey(seed_bytes)).to_string();
        let pub_key_str = Strkey::PublicKeyEd25519(ed25519::PublicKey(pub_bytes)).to_string();

        public_key.set(pub_key_str);
        secret_key_hidden.set(Some(Zeroizing::new(secret_str)));
        seed_bytes.zeroize();  
    };

    let import_key = move |_| {
        let raw_input = input_value.read().clone();
        if let Ok(Strkey::PrivateKeyEd25519(priv_key)) = Strkey::from_string(&raw_input) {
            // A betöltött S... bájtokból (priv_key.0) kiszámoljuk a hozzá tartozó G...-t
            let signing_key = SigningKey::from_bytes(&priv_key.0);
            let pub_bytes = signing_key.verifying_key().to_bytes();
        
            let pub_key_str = Strkey::PublicKeyEd25519(ed25519::PublicKey(pub_bytes)).to_string();
        
            public_key.set(pub_key_str);
            secret_key_hidden.set(Some(Zeroizing::new(raw_input)));
            input_value.set(String::new());
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
                if let Ok(Strkey::PrivateKeyEd25519(priv_key)) = Strkey::from_string(&secret) {
                    // 1. Kinyerjük a nyers bájtokat a tuple struct-ból (.0)
                    let seed_bytes: [u8; 32] = priv_key.0;

                    // 2. Létrehozzuk a dalek SigningKey-t (ez felel meg a KeyPair-nek)
                    let signing_key = SigningKey::from_bytes(&seed_bytes);

                    // 3. Ha szükséged van a G... címre is:
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

            div { style: "display: flex; gap: 10px; margin-top: 20px;",
                button { onclick: activate_account, "💾 Account aktiválása" }
                button { onclick: save_action, "💾 Mentés az OS tárcába" }
                button { onclick: load_action, "🔓 Betöltés (Biometria/Pass)" }
            }

            // Generáló gomb
            button { 
                style: "width: 100%; margin-top: 20px; padding: 10px; background: #007bff; color: white; border: none; border-radius: 5px; cursor: pointer;",
                onclick: fetch_and_generate, 
                "🛠 Tranzakció XDR Generálása" 
            }

            if !generated_xdr.read().is_empty() {
                div { style: "margin-top: 20px; padding: 15px; background: #e9ecef; border-radius: 8px;",

                    div { style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 10px;",
                        span { style: "font-size: 0.8em; font-weight: bold; color: #495057;", "GENERÁLT XDR (Base64):" }
                        button { 
                            style: "font-size: 0.7em; padding: 4px 8px; cursor: pointer;",
                            onclick: copy_xdr_to_clipboard,
                            "{xdr_copy_status}"
                        }
                    }
                    pre { 
                        style: "word-break: break-all; white-space: pre-wrap; font-size: 0.75em; background: white; padding: 10px; border-radius: 4px; border: 1px solid #dee2e6; max-height: 150px; overflow-y: auto;",
                        "{generated_xdr}" 
                    }
                    p { style: "font-size: 0.7em; color: #6c757d; margin-top: 5px;", 
                        "Tipp: Ezt az XDR-t beillesztheted a [Stellar Laboratory](https://laboratory.stellar.org) oldalán." 
                    }

                    button { 
                        style: "width: 100%; margin-top: 15px; padding: 12px; background: #28a745; color: white; border: none; border-radius: 5px; font-weight: bold; cursor: pointer;",
                        onclick: submit_tx_action, 
                        "🚀 Tranzakció Beküldése a Testnetre" 
                    }
                    
                    p { style: "margin-top: 10px; font-size: 0.9em; text-align: center; color: #495057;",
                        "{submission_status}"
                    }
                }
            }
        }
    }
}
