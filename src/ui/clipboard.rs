use dioxus::prelude::*;

/// Szöveg másolása a vágólapra, időzített visszaállítással.
/// Ha `is_secret` true, 30 mp után törli a vágólapot; egyébként 10 mp után visszaállítja a feliratot.
pub fn safe_copy(text: String, mut status_signal: Signal<String>, is_secret: bool, copied_label: String) {
    spawn(async move {
        if let Ok(mut cb) = arboard::Clipboard::new() {
            let _ = cb.set_text(text);

            let original_label = status_signal.peek().clone();
            status_signal.set(copied_label);

            let wait_secs = if is_secret { 30 } else { 10 };
            tokio::time::sleep(std::time::Duration::from_secs(wait_secs)).await;

            if is_secret {
                let _ = cb.set_text("".to_string());
                std::thread::sleep(std::time::Duration::from_millis(500));
            }

            status_signal.set(original_label);
        }
    });
}
