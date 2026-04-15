use dioxus::prelude::*;
use log_analizor::analyzer::{AnalysisEvent, analyze_raw_log_stream};

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let mut raw_log = use_signal(String::new);
    let mut output = use_signal(String::new);
    let mut is_loading = use_signal(|| false);
    let mut error_message = use_signal(|| Option::<String>::None);

    rsx! {
        div {
            style: "min-height: 100vh; background: linear-gradient(180deg, #f4f7fb 0%, #e9f1f8 100%); padding: 22px; box-sizing: border-box; font-family: 'IBM Plex Sans', 'Noto Sans', sans-serif; color: #1f2f3b;",
            div {
                style: "max-width: 940px; margin: 0 auto; background: #ffffff; border: 1px solid #d6e2ec; border-radius: 14px; box-shadow: 0 10px 28px rgba(22, 45, 68, 0.08); padding: 18px;",
                h1 {
                    style: "margin: 0 0 8px 0; font-size: 1.35rem;",
                    "Analyseur de logs"
                }
                p {
                    style: "margin: 0 0 14px 0; color: #5d7280;",
                    "Collez un log brut, puis cliquez sur Analyser pour suivre la reponse en streaming."
                }

                textarea {
                    style: "width: 100%; min-height: 230px; resize: vertical; border: 1px solid #d6e2ec; border-radius: 10px; padding: 12px; box-sizing: border-box; font-family: 'JetBrains Mono', 'Fira Code', monospace; font-size: 0.9rem; line-height: 1.45; background: #f8fbff; color: #132533;",
                    placeholder: "Exemple: log JSON ou syslog a analyser",
                    value: raw_log(),
                    oninput: move |evt| raw_log.set(evt.value())
                }

                div {
                    style: "margin-top: 12px; display: flex; align-items: center; gap: 10px;",
                    button {
                        style: "border: none; border-radius: 10px; padding: 10px 16px; font-weight: 600; background: #1f7a8c; color: white; cursor: pointer;",
                        disabled: is_loading(),
                        onclick: move |_| {
                            if raw_log().trim().is_empty() {
                                error_message.set(Some("Veuillez coller un log avant d'analyser.".to_string()));
                                return;
                            }

                            is_loading.set(true);
                            error_message.set(None);
                            output.set(String::new());

                            let raw = raw_log();
                            let mut output_signal = output;
                            let mut loading_signal = is_loading;
                            let mut error_signal = error_message;

                            spawn(async move {
                                let run_result = analyze_raw_log_stream(raw, move |event| match event {
                                    AnalysisEvent::TextDelta(chunk) => {
                                        let mut next = output_signal();
                                        next.push_str(&chunk);
                                        output_signal.set(next);
                                    }
                                    AnalysisEvent::Done { usage_line } => {
                                        let mut next = output_signal();
                                        next.push_str("\n\n");
                                        next.push_str(&usage_line);
                                        output_signal.set(next);
                                    }
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
                        span {
                            style: "font-size: 0.92rem; color: #5d7280;",
                            "Streaming en direct..."
                        }
                    }
                }

                if let Some(err) = error_message() {
                    div {
                        style: "margin-top: 12px; border: 1px solid #f1c4be; border-radius: 10px; background: #fdeceb; color: #9a2f27; padding: 10px 12px;",
                        "Erreur: {err}"
                    }
                }

                pre {
                    style: "margin: 14px 0 0 0; min-height: 190px; border: 1px solid #d6e2ec; border-radius: 10px; background: #f7fbff; padding: 12px; white-space: pre-wrap; word-break: break-word; font-family: 'JetBrains Mono', 'Fira Code', monospace; font-size: 0.9rem; line-height: 1.45;",
                    "{output}"
                }
            }
        }
    }
}
