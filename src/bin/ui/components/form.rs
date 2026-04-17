use dioxus::prelude::*;

use crate::components::status::StatusLine;

#[component]
pub fn LogForm(
    raw_log: String,
    is_loading: bool,
    on_input: EventHandler<String>,
    on_submit: EventHandler<()>,
) -> Element {
    rsx! {
        textarea {
            class: "editor",
            placeholder: "Exemple: log JSON ou syslog a analyser",
            value: raw_log,
            oninput: move |evt| on_input.call(evt.value())
        }

        div { class: "controls",
            button {
                class: "button",
                disabled: is_loading,
                onclick: move |_| on_submit.call(()),
                if is_loading { "Analyse en cours..." } else { "Analyser" }
            }
            StatusLine { is_loading }
        }
    }
}
