use dioxus::prelude::*;
use super::log;
use crate::i18n::Language;
use super::i18n::ui_i18n;

/// Copy text to clipboard with timed reset.
/// On desktop: uses arboard, clears clipboard after 30s for secrets.
/// On web: uses navigator.clipboard API, clears clipboard after 30s for secrets.

#[cfg(not(target_arch = "wasm32"))]
pub fn safe_copy(text: String, mut copied_signal: Signal<bool>, is_secret: bool) {
    spawn(async move {
        if let Ok(mut cb) = arboard::Clipboard::new() {
            let _ = cb.set_text(text);

            copied_signal.set(true);

            let wait_secs = if is_secret { 30 } else { 10 };
            tokio::time::sleep(std::time::Duration::from_secs(wait_secs)).await;

            if is_secret {
                let _ = cb.set_text("".to_string());
                std::thread::sleep(std::time::Duration::from_millis(500));
                let i18n = ui_i18n(Language::default());
                log(&i18n.clipboard_cleared().to_string());
            }

            copied_signal.set(false);
        }
    });
}

#[cfg(target_arch = "wasm32")]
pub fn safe_copy(text: String, mut copied_signal: Signal<bool>, is_secret: bool) {
    spawn(async move {
        let ok = write_to_web_clipboard(&text).await;
        if ok {
            copied_signal.set(true);

            let wait_secs = if is_secret { 30 } else { 10 };
            gloo_timers::future::sleep(std::time::Duration::from_secs(wait_secs)).await;

            if is_secret {
                let _ = write_to_web_clipboard("").await;
                let i18n = ui_i18n(Language::default());
                log(&i18n.clipboard_cleared().to_string());
            }

            copied_signal.set(false);
        }
    });
}

#[cfg(target_arch = "wasm32")]
async fn write_to_web_clipboard(text: &str) -> bool {
    use wasm_bindgen_futures::JsFuture;

    let Some(window) = web_sys::window() else { return false; };
    let clipboard = window.navigator().clipboard();
    let promise = clipboard.write_text(text);
    JsFuture::from(promise).await.is_ok()
}
