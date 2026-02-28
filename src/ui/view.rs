use dioxus::prelude::*;
use super::state::WalletState;
use super::controller::AppController;
use super::i18n::ui_i18n;
use super::tabs::Tab;
use super::tabs::{home, networking, info, settings};

pub fn render_app(s: WalletState, ctrl: AppController) -> Element {
    let lang = *s.language.read();
    let i18n = ui_i18n(lang);
    let passed_gate = *s.passed_gate.read();

    // Start gate modal — shown before app content
    if !passed_gate {
        return rsx! {
            div { style: "position: fixed; top: 0; left: 0; width: 100%; height: 100%; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 2000; font-family: sans-serif;",
                div { style: "background: white; padding: 40px; border-radius: 16px; max-width: 360px; width: 90%; text-align: center; box-shadow: 0 8px 32px rgba(0,0,0,0.3);",
                    h2 { style: "margin: 0 0 12px; color: #333;", "Zsozso" }
                    p { style: "margin: 0 0 30px; color: #666; font-size: 1em;",
                        "{i18n.gate_title()}"
                    }
                    button {
                        style: "padding: 14px 48px; background: #007bff; color: white; border: none; border-radius: 8px; font-weight: bold; cursor: pointer; font-size: 1.1em;",
                        onclick: move |_| {
                            let mut gate = s.passed_gate;
                            gate.set(true);
                        },
                        "{i18n.btn_next()}"
                    }
                }
            }
        };
    }

    let active = *s.active_tab.read();

    rsx! {
        div { style: "display: flex; flex-direction: column; height: 100vh; max-width: 550px; margin: auto; font-family: sans-serif;",
            // Header
            div { style: "padding: 15px 30px 0;",
                h2 { style: "margin: 0;", "Zsozso" }
            }

            // Tab content (scrollable area)
            div { style: "flex: 1; overflow-y: auto; padding: 20px 30px 90px;",
                match active {
                    Tab::Home => home::render_home_tab(i18n.as_ref()),
                    Tab::Networking => networking::render_networking_tab(s, ctrl, i18n.as_ref()),
                    Tab::Info => info::render_info_tab(i18n.as_ref()),
                    Tab::Settings => settings::render_settings_tab(s, ctrl, i18n.as_ref()),
                }
            }

            // Bottom tab bar
            {render_tab_bar(s, i18n.as_ref())}
        }

        // Clipboard modal overlay
        if *s.clipboard_modal_open.read() {
            div { style: "position: fixed; top: 0; left: 0; width: 100%; height: 100%; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 1000;",
                div { style: "background: white; padding: 30px; border-radius: 12px; max-width: 400px; text-align: center; box-shadow: 0 4px 20px rgba(0,0,0,0.3);",
                    p { style: "margin-bottom: 20px; font-size: 1em; color: #333;",
                        "{i18n.clipboard_modal_text()}"
                    }
                    button {
                        style: "padding: 12px 24px; background: #dc3545; color: white; border: none; border-radius: 6px; font-weight: bold; cursor: pointer; font-size: 1em;",
                        onclick: move |_| ctrl.dismiss_clipboard_modal(),
                        "{i18n.btn_clear_clipboard()}"
                    }
                }
            }
        }
    }
}

fn render_tab_bar(s: WalletState, i18n: &dyn super::i18n::UiI18n) -> Element {
    let active = *s.active_tab.read();

    let tabs: [(Tab, &str, &str); 4] = [
        (Tab::Home, "M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6", i18n.tab_home()),
        (Tab::Networking, "M21 12a9 9 0 01-9 9m9-9a9 9 0 00-9-9m9 9H3m9 9a9 9 0 01-9-9m9 9c1.657 0 3-4.03 3-9s-1.343-9-3-9m0 18c-1.657 0-3-4.03-3-9s1.343-9 3-9", i18n.tab_networking()),
        (Tab::Info, "M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z", i18n.tab_info()),
        (Tab::Settings, "M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.066 2.573c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.573 1.066c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.066-2.573c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z", i18n.tab_settings()),
    ];

    rsx! {
        div { style: "position: fixed; bottom: 0; left: 0; right: 0; max-width: 550px; margin: auto; background: white; border-top: 1px solid #ddd; display: flex; justify-content: space-around; padding: 6px 0; z-index: 500;",
            for (tab, path, label) in tabs {
                {
                    let is_active = active == tab;
                    let color = if is_active { "#007bff" } else { "#999" };
                    let font_weight = if is_active { "bold" } else { "normal" };
                    rsx! {
                        button {
                            key: "{label}",
                            style: "flex: 1; display: flex; flex-direction: column; align-items: center; gap: 2px; background: none; border: none; cursor: pointer; padding: 4px 0; color: {color};",
                            onclick: move |_| {
                                let mut active_tab = s.active_tab;
                                active_tab.set(tab);
                            },
                            svg {
                                width: "24",
                                height: "24",
                                view_box: "0 0 24 24",
                                fill: "none",
                                stroke: "{color}",
                                stroke_width: "2",
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                path { d: "{path}" }
                            }
                            span { style: "font-size: 0.65em; font-weight: {font_weight};",
                                "{label}"
                            }
                        }
                    }
                }
            }
        }
    }
}
