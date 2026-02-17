use arboard::Clipboard;
use dioxus::prelude::*;
use keyring::Entry;
use rand::RngCore;
use zeroize::{Zeroize, Zeroizing};

// Az új típusok a 25.0.0 verzióból
use ed25519_dalek::{Signer, SigningKey};
use stellar_strkey::{ed25519, Strkey};
use stellar_xdr::curr::{
    MuxedAccount, Uint256, Transaction, SequenceNumber, Memo, Operation, 
    OperationBody, PaymentOp, Asset, Preconditions, TransactionExt, VecM,
    TransactionEnvelope, TransactionV1Envelope, DecoratedSignature, Hash,
    Signature, BytesM, SignatureHint, WriteXdr, Limits,
    TransactionSignaturePayload, TransactionSignaturePayloadTaggedTransaction
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

    // Amikor a Dioxus leáll (bezárod az ablakot):
    if let Ok(mut clipboard) = Clipboard::new() {
        let _ = clipboard.set_text("".to_string());
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

    let mut safe_copy = move |text: String, mut status_signal: Signal<String>, is_secret: bool| {
        // 2. Megállítjuk a futó folyamatot
        if let Some(_task) = active_clipboard_task.write().take() {
            // Dioxus 0.6+ alatt a Task-nak nincs stop() metódusa közvetlenül így, 
            // de a szignál felülírása és a spawn kezelése megoldja a leváltást.
        }

        let new_task = spawn(async move {
            if let Ok(mut cb) = arboard::Clipboard::new() {
                let _ = cb.set_text(text);
                
                let original_label = status_signal.peek().clone();
                status_signal.set("MÁSOLVA!".to_string());

                let wait_secs = if is_secret { 30 } else { 10 };
                tokio::time::sleep(std::time::Duration::from_secs(wait_secs)).await;
                
                if is_secret {
                    let _ = cb.set_text("".to_string());
                }
                
                status_signal.set(original_label);
            }
        });

        // Eltároljuk az új task-ot
        active_clipboard_task.set(Some(new_task));
    };

    let submit_tx_action = move |_| {
        let xdr_to_submit = generated_xdr.read().clone();
        
        if xdr_to_submit.is_empty() {
            submission_status.set("Hiba: Nincs generált XDR!".to_string());
            return;
        }

        spawn(async move {
            submission_status.set("Beküldés folyamatban...".to_string());
            
            // FIGYELJ: Az URL-nek pontosan így kell kinéznie, per jel nélkül a végén!
            let url = "https://horizon-testnet.stellar.org/transactions";
            
            let client = reqwest::Client::new();

            // A Stellar Horizon számára a legbiztosabb a form-encoded POST
            // A kulcs "tx", az érték a Base64 XDR
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
                        // Itt kiírjuk a hiba részleteit is a terminálba
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

    // --- AKTIVÁLÓ (Friendbot) ---
    let activate_account = move |_| {
        let pubkey = public_key.read().clone();
        if pubkey == "Nincs kulcs betöltve" { return; }

        spawn(async move {
            submission_status.set("🚀 Friendbot hívása...".to_string());
            let url = format!("https://friendbot.stellar.org/?addr={}", pubkey);
            
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

    // --- XDR GENERÁLÓ (Lekéréssel együtt) ---
    let fetch_and_generate = move |_| {
        let pubkey_str = public_key.read().clone();
        let secret_str_opt = secret_key_hidden.read().clone();

        if pubkey_str == "Nincs kulcs betöltve" { 
            submission_status.set("⚠️ Nincs betöltött kulcs!".to_string());
            return; 
        }

        spawn(async move {
            submission_status.set("🔍 Szekvenciaszám lekérése...".to_string());
            
            let url = format!("https://horizon-testnet.stellar.org/accounts/{}", pubkey_str);
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
                
               // --- Aláírási Logika ---
                if let Ok(Strkey::PrivateKeyEd25519(priv_key)) = Strkey::from_string(&secret_val) {
                    let signing_key = ed25519_dalek::SigningKey::from_bytes(&priv_key.0);
                    let pub_bytes = signing_key.verifying_key().to_bytes();
                    
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
                                    destination: MuxedAccount::Ed25519(Uint256(pub_bytes)), // Önmagának küld
                                    asset: Asset::Native,
                                    amount: 100_000_000, 
                                }),
                            }
                        ]).unwrap(),
                        ext: TransactionExt::V0,
                    };

                    // 1. Hálózat azonosítója (Testnet)
                    let network_passphrase = "Test SDF Network ; September 2015";
                    let network_id = Hash(Sha256::digest(network_passphrase.as_bytes()).into());

                    // 2. Payload összeállítása (ahogy elkezded a kód végén)
                    let payload = TransactionSignaturePayload {
                        network_id,
                        tagged_transaction: TransactionSignaturePayloadTaggedTransaction::Tx(tx.clone()),
                    };

                    // 3. Szerializálás XDR-be
                    let tx_payload_xdr = payload.to_xdr(Limits::none()).unwrap();

                    // 4. FONTOS: A Stellar a payload SHA256 hash-ét íratja alá!
                    let tx_hash = Sha256::digest(&tx_payload_xdr);

                    // . Aláírás a HASH-re (vagy használj sign_prehashed-et, ha a dalek verziód támogatja, 
                    // de a sima sign is jó a hash bájtjaira)
                    let sig_bytes = signing_key.sign(&tx_hash).to_bytes();

                    // 6. Hint generálása (maradhat a régi)
                    let mut hint_bytes = [0u8; 4];
                    hint_bytes.copy_from_slice(&pub_bytes[pub_bytes.len() - 4..]);

                    // 7. Envelope összeállítása
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
                            submission_status.set(format!("✅ XDR Kész! (Seq: {})", next_seq));
                        },
                        Err(e) => submission_status.set(format!("❌ XDR hiba: {:?}", e)),
                    }
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

            // --- TITKOS KULCS SZEKCIÓ (Csak ha van betöltve) ---
            if let Some(secret) = secret_key_hidden.read().as_ref() {
                div { style: "border: 1px solid #ffeeba; background: #fff3cd; padding: 15px; border-radius: 8px; margin-bottom: 20px;",
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
                        // AKTIVÁLÓ GOMB
                        button {
                            style: "background: #17a2b8; color: white; border: none; padding: 5px 15px; border-radius: 4px;",
                            onclick: activate_account,
                            "🚀 Aktiválás (Friendbot)"
                        }
                    }

                    if *show_secret.read() {
                        p { style: "margin-top: 15px; font-family: monospace; word-break: break-all; background: white; padding: 10px;",
                            "{secret.as_str()}"
                        }
                    }
                }
            }

            // --- TRANZAKCIÓ GENERÁLÁSA ---
            button { 
                style: "width: 100%; padding: 12px; background: #007bff; color: white; border: none; border-radius: 5px; font-weight: bold; cursor: pointer; margin-bottom: 10px;",
                onclick: fetch_and_generate, 
                "🛠 Tranzakció XDR Generálása" 
            }

            // --- STÁTUSZ ÜZENET ---
            p { style: "text-align: center; font-size: 0.9em; color: #495057; font-style: italic;", 
                "{submission_status}" 
            }

            // --- GENERÁLT XDR BLOKK (Csak ha elkészült) ---
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

            // --- MENTÉS / BETÖLTÉS ---
            div { style: "display: flex; gap: 10px; margin-top: 30px;",
                button { onclick: save_action, style: "flex: 1;", "💾 Mentés az OS tárcába" }
                button { onclick: load_action, style: "flex: 1;", "🔓 Betöltés" }
            }
        }
    }
}
