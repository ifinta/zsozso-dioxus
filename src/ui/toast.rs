use dioxus::prelude::*;
use gloo_timers::future::TimeoutFuture;

#[component]
pub fn UpdateNotification() -> Element {
    let mut show_update = use_signal(|| false);

    // Sync with the JavaScript bridge
    use_effect(move || {
        spawn(async move {
            loop {
                // Check if the JS flag is set
                let is_ready = js_sys::eval("window.__ZSOZSO_UPDATE_READY === true")
                    .ok()
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                if is_ready {
                    show_update.set(true);
                    break; // Stop polling once detected
                }
                // Check every 5 seconds if not already detected
                TimeoutFuture::new(5_000).await;
            }
        });
    });

    if !show_update() {
        return rsx! {};
    }

    rsx! {
        div {
            class: "fixed bottom-4 right-4 bg-info text-white p-4 rounded-lg shadow-lg flex flex-col gap-2 z-50",
            style: "background-color: #17a2b8; min-width: 250px;",
            p { "🚀 A new version of Zsozso is available!" }
            button {
                class: "bg-white text-info font-bold py-1 px-3 rounded hover:bg-gray-100",
                style: "color: #17a2b8;",
                onclick: move |_| {
                    // Trigger the reload via JS
                    let _ = js_sys::eval("window.location.reload()");
                },
                "Update Now"
            }
        }
    }
}
