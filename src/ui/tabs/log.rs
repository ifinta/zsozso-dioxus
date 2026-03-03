use dioxus::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use crate::ui::i18n::UiI18n;

/// Read the in-app log buffer from the JS bridge.
fn read_log_buffer() -> String {
    js_sys::eval("window.__zsozso_log ? window.__zsozso_log.get() : ''")
        .ok()
        .and_then(|v| v.as_string())
        .unwrap_or_default()
}

/// Clear the in-app log buffer.
fn clear_log_buffer() {
    let _ = js_sys::eval("window.__zsozso_log && window.__zsozso_log.clear()");
}

/// Upload the log buffer to the server. Returns a status string.
async fn upload_log_buffer() -> String {
    let promise = match js_sys::eval("window.__zsozso_log && window.__zsozso_log.upload()") {
        Ok(v) => v,
        Err(_) => return "ERR:eval".to_string(),
    };
    // The upload function returns a Promise
    let promise: js_sys::Promise = match promise.dyn_into() {
        Ok(p) => p,
        Err(_) => return "ERR:not_a_promise".to_string(),
    };
    match JsFuture::from(promise).await {
        Ok(val) => val.as_string().unwrap_or_else(|| "ERR:unknown".to_string()),
        Err(e) => format!("ERR:{}", e.as_string().unwrap_or_default()),
    }
}

pub fn render_log_tab(i18n: &dyn UiI18n) -> Element {
    let mut log_text = use_signal(String::new);
    let mut upload_status = use_signal(|| String::new());

    // Load log immediately when this tab is rendered (on mount)
    use_effect(move || {
        log_text.set(read_log_buffer());
    });

    // Pre-compute i18n strings we need inside async closures
    let uploading_msg = i18n.log_uploading().to_string();
    let upload_ok_msg = i18n.log_upload_ok().to_string();
    let upload_empty_msg = i18n.log_upload_empty().to_string();

    rsx! {
        div { style: "display: flex; flex-direction: column; height: 100%;",
            // Title
            h3 { style: "margin: 0 0 10px 0; font-size: 1.1em;", "{i18n.tab_log()}" }

            // Buttons — each full width, stacked vertically
            div { style: "display: flex; flex-direction: column; gap: 8px; margin-bottom: 10px;",
                // Refresh button
                button {
                    style: "width: 100%; padding: 10px 14px; background: #007bff; color: white; border: none; border-radius: 6px; cursor: pointer; font-size: 0.95em; font-weight: bold;",
                    onclick: move |_| {
                        log_text.set(read_log_buffer());
                    },
                    "{i18n.log_refresh()}"
                }
                // Upload button
                button {
                    style: "width: 100%; padding: 10px 14px; background: #17a2b8; color: white; border: none; border-radius: 6px; cursor: pointer; font-size: 0.95em; font-weight: bold;",
                    onclick: {
                        let uploading_msg = uploading_msg.clone();
                        let upload_ok_msg = upload_ok_msg.clone();
                        let upload_empty_msg = upload_empty_msg.clone();
                        move |_| {
                            let uploading_msg = uploading_msg.clone();
                            let upload_ok_msg = upload_ok_msg.clone();
                            let upload_empty_msg = upload_empty_msg.clone();
                            spawn(async move {
                                upload_status.set(uploading_msg);
                                let result = upload_log_buffer().await;
                                match result.as_str() {
                                    "OK" => upload_status.set(upload_ok_msg),
                                    "EMPTY" => upload_status.set(upload_empty_msg),
                                    other => upload_status.set(format!("\u{274C} {}", other)),
                                }
                            });
                        }
                    },
                    "{i18n.log_upload()}"
                }
                // Clear button
                button {
                    style: "width: 100%; padding: 10px 14px; background: #dc3545; color: white; border: none; border-radius: 6px; cursor: pointer; font-size: 0.95em; font-weight: bold;",
                    onclick: move |_| {
                        clear_log_buffer();
                        log_text.set(String::new());
                        upload_status.set(String::new());
                    },
                    "{i18n.log_clear()}"
                }
            }

            // Upload status line (only shown when non-empty)
            if !upload_status.read().is_empty() {
                p {
                    style: "margin: 0 0 8px 0; font-size: 0.85em; color: #6c757d;",
                    "{upload_status.read()}"
                }
            }

            // Log output area
            pre {
                style: "flex: 1; overflow-y: auto; background: #1e1e1e; color: #d4d4d4; padding: 12px; border-radius: 8px; font-size: 0.72em; line-height: 1.5; white-space: pre-wrap; word-break: break-all; margin: 0; font-family: 'Courier New', monospace;",
                {log_text.read().clone()}
            }
        }
    }
}
