mod ledger;
mod ui;
mod store;

use arboard::Clipboard;
use dioxus::prelude::*;

fn main() {
    let config = dioxus::desktop::Config::new()
        .with_window(
            dioxus::desktop::WindowBuilder::new()
                .with_always_on_top(false)
                .with_title("Zsozso"),
        )
        .with_menu(None);

    LaunchBuilder::desktop().with_cfg(config).launch(ui::app);

    if let Ok(mut clipboard) = Clipboard::new() {
        let _ = clipboard.set_text("".to_string());
        std::thread::sleep(std::time::Duration::from_millis(500));
        println!("🔐 Vágólap törölve a biztonság érdekében.");
    }
}