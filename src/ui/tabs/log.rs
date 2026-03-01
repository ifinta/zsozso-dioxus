use dioxus::prelude::*;
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

pub fn render_log_tab(i18n: &dyn UiI18n) -> Element {
    let mut log_text = use_signal(String::new);

    // Auto-refresh the log every 2 seconds
    use_future(move || async move {
        loop {
            log_text.set(read_log_buffer());
            gloo_timers::future::sleep(std::time::Duration::from_secs(2)).await;
        }
    });

    rsx! {
        div { style: "display: flex; flex-direction: column; height: 100%;",
            // Header row with title and buttons
            div { style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 10px;",
                h3 { style: "margin: 0; font-size: 1.1em;", "{i18n.tab_log()}" }
                div { style: "display: flex; gap: 8px;",
                    button {
                        style: "padding: 6px 14px; background: #007bff; color: white; border: none; border-radius: 6px; cursor: pointer; font-size: 0.85em;",
                        onclick: move |_| {
                            log_text.set(read_log_buffer());
                        },
                        "{i18n.log_refresh()}"
                    }
                    button {
                        style: "padding: 6px 14px; background: #dc3545; color: white; border: none; border-radius: 6px; cursor: pointer; font-size: 0.85em;",
                        onclick: move |_| {
                            clear_log_buffer();
                            log_text.set(String::new());
                        },
                        "{i18n.log_clear()}"
                    }
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
