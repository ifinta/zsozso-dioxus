mod i18n;
mod ledger;
mod ui;
mod store;

use dioxus::prelude::*;

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    use arboard::Clipboard;
    use i18n::Language;
    use ui::i18n::ui_i18n;

    let config = dioxus::desktop::Config::new()
        .with_window(
            dioxus::desktop::WindowBuilder::new()
                .with_always_on_top(false)
                .with_title("Zsozso"),
        )
        .with_menu(None);

    LaunchBuilder::desktop().with_cfg(config).launch(ui::app);

    let lang = Language::default();
    let i18n = ui_i18n(lang);
    if let Ok(mut clipboard) = Clipboard::new() {
        let _ = clipboard.set_text("".to_string());
        std::thread::sleep(std::time::Duration::from_millis(500));
        println!("{}", i18n.clipboard_cleared());
    }
}

#[cfg(target_arch = "wasm32")]
fn main() {
    LaunchBuilder::web().launch(ui::app);
}