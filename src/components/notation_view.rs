//! Pinta la notacion parseada como badges de colores.

use tokon_tech_log::notation;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct NotationViewProps {
    pub value: String,
}

#[function_component(NotationView)]
pub fn notation_view(props: &NotationViewProps) -> Html {
    let groups = notation::parse(&props.value);

    if groups.is_empty() {
        return html! { <span class="notation-empty">{ "Sin notación" }</span> };
    }

    html! {
        <span class="notation-row">
            { for groups.iter().map(|group| html! {
                <span class={classes!("notation-group", group.es_texto().then_some("notation-group-text"))}>
                    { for group.alternatives.iter().enumerate().map(|(ai, alt)| html! {
                        <span class="notation-alt">
                            { for alt.atoms.iter().enumerate().map(|(si, atom)| html! {
                                <span class="notation-atom">
                                    if let Some(nota) = &atom.note {
                                        <span class="badge-note">{ nota }</span>
                                    }
                                    if atom.aerial {
                                        <span class="aerial-mark" title="En el aire (salto)">{ "j." }</span>
                                    }
                                    if !atom.directions.is_empty() {
                                        <span class="dir-run">{ &atom.directions }</span>
                                    }
                                    if let Some(style) = atom.button {
                                        <span
                                            class="btn-badge"
                                            style={format!("background: {}; color: {}", style.bg, style.fg)}
                                        >
                                            { style.code }
                                        </span>
                                    }
                                    if let Some(text) = &atom.fallback {
                                        <span class="badge-plain">{ text }</span>
                                    }
                                    if si + 1 < alt.atoms.len() {
                                        <span class="joiner">{ "+" }</span>
                                    }
                                </span>
                            }) }
                            if ai + 1 < group.alternatives.len() {
                                <span class="joiner">{ "/" }</span>
                            }
                        </span>
                    }) }
                </span>
            }) }
        </span>
    }
}
