use dioxus::prelude::*;

/// Copy text to clipboard with timed reset.
/// If `is_secret` is true, clears the clipboard after 30s; otherwise resets the indicator after 10s.
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
            }

            copied_signal.set(false);
        }
    });
}
