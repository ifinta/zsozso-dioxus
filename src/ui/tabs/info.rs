use dioxus::prelude::*;
use qrcode::{QrCode, render::svg};
use crate::ui::state::WalletState;
use crate::ui::i18n::UiI18n;

/// Ask the service worker for its CACHE_NAME via postMessage.
/// Returns the version string, e.g. "zsozso-v2".
pub async fn get_sw_version() -> Option<String> {
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;

    let val = js_sys::eval(r#"
        (function() {
            var sw = navigator.serviceWorker && navigator.serviceWorker.controller;
            if (!sw) return Promise.resolve('');
            return new Promise(function(resolve) {
                var ch = new MessageChannel();
                ch.port1.onmessage = function(e) {
                    resolve(e.data && e.data.version ? e.data.version : '');
                };
                sw.postMessage({ type: 'GET_VERSION' }, [ch.port2]);
                setTimeout(function() { resolve(''); }, 2000);
            });
        })()
    "#).ok()?;
    let promise: js_sys::Promise = val.dyn_into().ok()?;
    let result = JsFuture::from(promise).await.ok()?;
    let s = result.as_string().unwrap_or_default();
    if s.is_empty() { None } else { Some(s) }
}

pub fn render_info_tab(s: WalletState, i18n: &dyn UiI18n) -> Element {
    let mut version = use_signal(|| None::<String>);

    // Fetch the SW version once when the tab renders
    use_effect(move || {
        spawn(async move {
            if let Some(v) = get_sw_version().await {
                version.set(Some(v));
            }
        });
    });

    let pk = s.public_key.read().clone();

    match pk {
        Some(key) => {
            let qr_svg = QrCode::new(key.as_bytes())
                .map(|code| {
                    code.render::<svg::Color>()
                        .min_dimensions(200, 200)
                        .max_dimensions(280, 280)
                        .quiet_zone(true)
                        .build()
                })
                .unwrap_or_default();

            rsx! {
                if let Some(ver) = version.read().as_ref() {
                    p { style: "margin-top: 12px; font-size: 0.7em; color: #999;",
                        "Version: {ver}"
                    }
                }
                div { style: "text-align: center; margin-top: 30px;",
                    p { style: "font-size: 0.9em; color: #666; margin-bottom: 10px;",
                        "{i18n.info_public_key_label()}"
                    }
                    div { style: "display: inline-block; padding: 12px; background: white; border-radius: 12px; box-shadow: 0 2px 8px rgba(0,0,0,0.1);",
                        dangerous_inner_html: "{qr_svg}"
                    }
                    p { style: "margin-top: 16px; font-family: monospace; font-size: 0.78em; word-break: break-all; padding: 12px; background: #f8f9fa; border-radius: 8px; border: 1px solid #ddd;",
                        "{key}"
                    }
                }
            }
        }
        None => {
            rsx! {
                div { style: "text-align: center; margin-top: 60px; color: #888;",
                    p { style: "font-size: 2em;", "ℹ️" }
                    p { "{i18n.info_no_key()}" }
                    if let Some(ver) = version.read().as_ref() {
                        p { style: "margin-top: 12px; font-size: 0.7em; color: #999;",
                            "Version: {ver}"
                        }
                    }
                }
            }
        }
    }
}
