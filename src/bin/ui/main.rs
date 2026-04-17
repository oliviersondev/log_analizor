mod components;
mod model;

use dioxus::prelude::*;
use log_analizor::analyzer::Analyzer;
use log_analizor::app::runner;

use components::form::LogForm;
use components::output::OutputPanel;
use model::{UiAction, UiState, validate_raw_log};

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
    let mut ui_state = use_signal(UiState::default);
    let analyzer = use_signal(|| Analyzer::from_env().map_err(|err| err.to_string()));

    rsx! {
        style { "{APP_CSS}" }
        div { class: "app-shell",
            div { class: "card",
                h1 { class: "title", "Analyseur de logs" }
                p { class: "subtitle", "Collez un log brut, puis cliquez sur Analyser pour suivre la reponse en streaming." }

                LogForm {
                    raw_log: ui_state().raw_log.clone(),
                    is_loading: ui_state().is_loading,
                    on_input: move |value: String| {
                        ui_state.with_mut(|state| state.apply(UiAction::RawLogChanged(value)));
                    },
                    on_submit: move |_| {
                        let raw = match validate_raw_log(&ui_state().raw_log) {
                            Ok(value) => value,
                            Err(message) => {
                                ui_state.with_mut(|state| state.apply(UiAction::SubmitFailed(message)));
                                return;
                            }
                        };

                        let analyzer_instance = match analyzer().clone() {
                            Ok(instance) => instance,
                            Err(message) => {
                                ui_state.with_mut(|state| state.apply(UiAction::SubmitFailed(message)));
                                return;
                            }
                        };

                        ui_state.with_mut(|state| {
                            state.apply(UiAction::SubmitStarted {
                                raw_log: raw.clone(),
                            })
                        });

                        let mut state_signal = ui_state;
                        spawn(async move {
                            let run_result = runner::run_raw_log_stream(
                                &analyzer_instance,
                                raw,
                                move |event| {
                                    state_signal.with_mut(|state| {
                                        state.apply(UiAction::StreamEvent(event))
                                    });
                                },
                            )
                            .await;

                            if let Err(err) = run_result {
                                state_signal.with_mut(|state| {
                                    state.apply(UiAction::SubmitFailed(err.to_string()))
                                });
                            }

                            state_signal.with_mut(|state| state.apply(UiAction::SubmitFinished));
                        });
                    }
                }

                OutputPanel {
                    output: ui_state().output.clone(),
                    error_message: ui_state().error_message.clone(),
                }
            }
        }
    }
}
