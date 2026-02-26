mod clipboard;
pub mod actions;
pub mod i18n;
pub mod status;
pub mod tabs;
pub mod view;
pub mod state;
pub mod controller;

use dioxus::prelude::*;
use state::use_wallet_state;
use controller::AppController;

pub fn app() -> Element {
    let state = use_wallet_state();
    let ctrl = AppController::new(state);

    // Desktop: clear clipboard when the component is dropped (window closing)
    #[cfg(not(target_arch = "wasm32"))]
    use_drop(move || {
        clipboard::clear_clipboard();
    });

    // Web: clear clipboard when the tab/browser is closed
    #[cfg(target_arch = "wasm32")]
    use_hook(|| {
        clipboard::register_beforeunload_cleanup();
    });

    // Pass the controller and state to the view
    view::render_app(state, ctrl)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn log(msg: &str) { println!("{}", msg); }
#[cfg(target_arch = "wasm32")]
pub fn log(msg: &str) { web_sys::console::log_1(&msg.into()); }
