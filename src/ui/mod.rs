mod clipboard;
pub mod actions;
pub mod i18n;
pub mod status;
pub mod view;
pub mod state;
pub mod controller;

use dioxus::prelude::*;
use state::use_wallet_state;
use controller::AppController;

pub fn app() -> Element {
    let state = use_wallet_state();
    let ctrl = AppController::new(state);

    // Cross-platform vágólap tisztítás (maradhat itt vagy mehet hook-ba)
    #[cfg(not(target_arch = "wasm32"))]
    use_drop(move || {
        if let Ok(mut cb) = arboard::Clipboard::new() {
            let _ = cb.set_text("".to_string());
        }
    });

    // A view-nak átadjuk a controllert és a state-et
    // Az app_view!() makró vagy függvény most már ezekből dolgozik
    view::render_app(state, ctrl)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn log(msg: &str) { println!("{}", msg); }
#[cfg(target_arch = "wasm32")]
pub fn log(msg: &str) { web_sys::console::log_1(&msg.into()); }
