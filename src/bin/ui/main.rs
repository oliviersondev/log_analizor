mod state;

use dioxus::prelude::*;
use log_analizor::analyzer::analyze_raw_log_stream;
use state::{append_stream_event, validate_raw_log};

const APP_CSS: &str = include_str!("../../../assets/ui.css");

fn main() {
    dioxus::LaunchBuilder::desktop()
        .with_cfg(
            dioxus::desktop::Config::new().with_window(
                dioxus::desktop::tao::window::WindowBuilder::new()
                    .with_title("Analyseur de logs")
                    .with_inner_size(dioxus::desktop::tao::dpi::LogicalSize::new(980.0, 760.0))
                    .with_min_inner_size(dioxus::desktop::tao::dpi::LogicalSize::new(720.0, 560.0)),
            ),
        )
        .launch(App);
}

#[component]
fn App() -> Element {
    let mut raw_log = use_signal(String::new);
    let mut output = use_signal(|| String::with_capacity(4 * 1024));
    let mut is_loading = use_signal(|| false);
    let mut error_message = use_signal(|| Option::<String>::None);

    rsx! {
        style { "{APP_CSS}" }
        div { class: "app-shell",
            div { class: "card",
                h1 { class: "title", "Analyseur de logs" }
                p { class: "subtitle", "Collez un log brut, puis cliquez sur Analyser pour suivre la reponse en streaming." }

                textarea {
                    class: "editor",
                    placeholder: "Exemple: log JSON ou syslog a analyser",
                    value: raw_log(),
                    oninput: move |evt| raw_log.set(evt.value())
                }

                div { class: "controls",
                    button {
                        class: "button",
                        disabled: is_loading(),
                        onclick: move |_| {
                            let raw = match validate_raw_log(&raw_log()) {
                                Ok(value) => value,
                                Err(message) => {
                                    error_message.set(Some(message));
                                    return;
                                }
                            };

                            is_loading.set(true);
                            error_message.set(None);
                            output.with_mut(|out| out.clear());

                            let mut output_signal = output;
                            let mut loading_signal = is_loading;
                            let mut error_signal = error_message;

                            spawn(async move {
                                let run_result = analyze_raw_log_stream(raw, move |event| {
                                    output_signal.with_mut(|out| append_stream_event(out, event));
                                })
                                .await;

                                if let Err(err) = run_result {
                                    error_signal.set(Some(err.to_string()));
                                }

                                loading_signal.set(false);
                            });
                        },
                        if is_loading() { "Analyse en cours..." } else { "Analyser" }
                    }
                    if is_loading() {
                        span { class: "status", "Streaming en direct..." }
                    }
                }

                if let Some(err) = error_message() {
                    div { class: "error", "Erreur: {err}" }
                }

                pre { class: "output", "{output}" }
            }
        }
    }
}
