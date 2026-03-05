use dioxus::prelude::*;
use super::state::{WalletState, AuthState};
use super::controller::AppController;
use super::i18n::ui_i18n;
use super::tabs::Tab;
use super::tabs::{home, networking, info, settings, log};

pub fn render_app(s: WalletState, ctrl: AppController) -> Element {
    let lang = *s.language.read();
    let i18n = ui_i18n(lang);
    let auth_state = *s.auth_state.read();

    // ── Auth failed: terminal error modal ──
    if auth_state == AuthState::Failed {
        return rsx! {
            div { style: "position: fixed; top: 0; left: 0; width: 100%; height: 100%; background: rgba(0,0,0,0.7); display: flex; align-items: center; justify-content: center; z-index: 3000; font-family: sans-serif;",
                div { style: "background: white; padding: 40px; border-radius: 16px; max-width: 360px; width: 90%; text-align: center; box-shadow: 0 8px 32px rgba(0,0,0,0.3);",
                    h2 { style: "margin: 0 0 12px; color: #dc3545;", "⚠️" }
                    p { style: "margin: 0 0 30px; color: #333; font-size: 1em; font-weight: bold;",
                        "{i18n.auth_failed()}"
                    }
                    button {
                        style: "padding: 14px 48px; background: #dc3545; color: white; border: none; border-radius: 8px; font-weight: bold; cursor: pointer; font-size: 1.1em;",
                        onclick: move |_| {
                            // Close window or blank the page
                            let _ = js_sys::eval("window.close() || (document.body.innerHTML = '')");
                        },
                        "{i18n.btn_exit()}"
                    }
                }
            }
        };
    }

    // ── Gate modal: pending or authenticating ──
    if auth_state != AuthState::Authenticated {
        let is_busy = auth_state == AuthState::Authenticating;
        let btn_label = if is_busy {
            i18n.authenticating()
        } else {
            i18n.btn_next()
        };
        let btn_bg = if is_busy { "#6c757d" } else { "#007bff" };

        return rsx! {
            div { style: "position: fixed; top: 0; left: 0; width: 100%; height: 100%; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 2000; font-family: sans-serif;",
                div { style: "background: white; padding: 40px; border-radius: 16px; max-width: 360px; width: 90%; text-align: center; box-shadow: 0 8px 32px rgba(0,0,0,0.3);",
                    h2 { style: "margin: 0 0 12px; color: #333;", "Zsozso" }
                    p { style: "margin: 0 0 30px; color: #666; font-size: 1em;",
                        "{i18n.gate_title()}"
                    }
                    button {
                        style: "padding: 14px 48px; background: {btn_bg}; color: white; border: none; border-radius: 8px; font-weight: bold; cursor: pointer; font-size: 1.1em;",
                        disabled: is_busy,
                        onclick: move |_| ctrl.start_auth(),
                        "{btn_label}"
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
                    Tab::Info => info::render_info_tab(s, i18n.as_ref()),
                    Tab::Settings => settings::render_settings_tab(s, ctrl, i18n.as_ref()),
                    Tab::Log => log::render_log_tab(i18n.as_ref()),
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

        // Network switch save modal
        if s.network_switch_pending.read().is_some() {
            div { style: "position: fixed; top: 0; left: 0; width: 100%; height: 100%; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 1100;",
                div { style: "background: white; padding: 30px; border-radius: 12px; max-width: 400px; width: 90%; text-align: center; box-shadow: 0 4px 20px rgba(0,0,0,0.3);",
                    p { style: "margin-bottom: 20px; font-size: 1em; color: #333;",
                        "{i18n.network_switch_save_prompt()}"
                    }
                    div { style: "display: flex; flex-direction: column; gap: 10px;",
                        button {
                            style: "padding: 12px 24px; background: #28a745; color: white; border: none; border-radius: 6px; font-weight: bold; cursor: pointer; font-size: 1em;",
                            onclick: move |_| ctrl.confirm_network_switch_save(),
                            "{i18n.btn_save_and_switch()}"
                        }
                        button {
                            style: "padding: 12px 24px; background: #007bff; color: white; border: none; border-radius: 6px; font-weight: bold; cursor: pointer; font-size: 1em;",
                            onclick: move |_| ctrl.confirm_network_switch_and_save(),
                            "{i18n.btn_switch_and_save()}"
                        }
                        button {
                            style: "padding: 12px 24px; background: #ffc107; color: #333; border: none; border-radius: 6px; font-weight: bold; cursor: pointer; font-size: 1em;",
                            onclick: move |_| ctrl.confirm_network_switch_discard(),
                            "{i18n.btn_switch_without_saving()}"
                        }
                        button {
                            style: "padding: 12px 24px; background: #6c757d; color: white; border: none; border-radius: 6px; font-weight: bold; cursor: pointer; font-size: 1em;",
                            onclick: move |_| ctrl.cancel_network_switch(),
                            "{i18n.btn_cancel()}"
                        }
                    }
                }
            }
        }

        // Biometric required to save – error modal
        if *s.biometric_save_modal_open.read() {
            div { style: "position: fixed; top: 0; left: 0; width: 100%; height: 100%; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 1150;",
                div { style: "background: white; padding: 30px; border-radius: 12px; max-width: 400px; width: 90%; text-align: center; box-shadow: 0 4px 20px rgba(0,0,0,0.3);",
                    h3 { style: "margin: 0 0 12px; color: #dc3545;", "\u{26A0}\u{FE0F}" }
                    p { style: "margin: 0 0 20px; color: #333; font-size: 1em;",
                        "{i18n.biometric_required_to_save()}"
                    }
                    button {
                        style: "padding: 12px 24px; background: #007bff; color: white; border: none; border-radius: 6px; font-weight: bold; cursor: pointer; font-size: 1em;",
                        onclick: move |_| ctrl.dismiss_biometric_save_modal(),
                        "{i18n.btn_close()}"
                    }
                }
            }
        }

        // SEA key generation modal
        if *s.sea_modal_open.read() {
            div { style: "position: fixed; top: 0; left: 0; width: 100%; height: 100%; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 1200;",
                div { style: "background: white; padding: 30px; border-radius: 12px; max-width: 400px; width: 90%; text-align: center; box-shadow: 0 4px 20px rgba(0,0,0,0.3);",
                    h3 { style: "margin: 0 0 16px; color: #333;", "{i18n.sea_modal_title()}" }
                    input {
                        style: "width: 100%; padding: 10px; border: 1px solid #ccc; border-radius: 6px; font-size: 1em; margin-bottom: 16px; box-sizing: border-box;",
                        r#type: "password",
                        placeholder: "{i18n.sea_modal_placeholder()}",
                        value: "{s.sea_modal_input.read().as_str()}",
                        oninput: move |evt| {
                            let mut input = s.sea_modal_input;
                            input.set(zeroize::Zeroizing::new(evt.value()));
                        }
                    }
                    div { style: "display: flex; flex-direction: column; gap: 10px;",
                        button {
                            style: "padding: 12px 24px; background: #6f42c1; color: white; border: none; border-radius: 6px; font-weight: bold; cursor: pointer; font-size: 1em;",
                            onclick: move |_| ctrl.generate_sea_keys(),
                            "{i18n.btn_generate_db_keys()}"
                        }
                        button {
                            style: "padding: 12px 24px; background: #6c757d; color: white; border: none; border-radius: 6px; font-weight: bold; cursor: pointer; font-size: 1em;",
                            onclick: move |_| ctrl.close_sea_modal(),
                            "{i18n.btn_close()}"
                        }
                    }
                }
            }
        }
    }
}

fn render_tab_bar(s: WalletState, i18n: &dyn super::i18n::UiI18n) -> Element {
    let active = *s.active_tab.read();

    let tabs: [(Tab, &str, &str); 5] = [
        (Tab::Home, "M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6", i18n.tab_home()),
        (Tab::Networking, "M21 12a9 9 0 01-9 9m9-9a9 9 0 00-9-9m9 9H3m9 9a9 9 0 01-9-9m9 9c1.657 0 3-4.03 3-9s-1.343-9-3-9m0 18c-1.657 0-3-4.03-3-9s1.343-9 3-9", i18n.tab_networking()),
        (Tab::Info, "M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z", i18n.tab_info()),
        (Tab::Settings, "M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.066 2.573c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.573 1.066c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.066-2.573c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z", i18n.tab_settings()),
        (Tab::Log, "M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2m-3 7h3m-3 4h3m-6-4h.01m-.01 4h.01", i18n.tab_log()),
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
