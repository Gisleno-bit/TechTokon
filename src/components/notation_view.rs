//! Pinta la notacion parseada: botones de color, flechas, etiquetas.

use tokon_tech_log::notation::{self, Piece};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct NotationViewProps {
    pub value: String,
}

fn pintar_pieza(pieza: &Piece) -> Html {
    match pieza {
        Piece::Button(style) => html! {
            <span
                class="btn-badge"
                style={format!("background: {}; color: {}", style.bg, style.fg)}
            >
                { style.code }
            </span>
        },
        Piece::Directions(flechas) => html! {
            <span class="dir-run">{ flechas }</span>
        },
        // Los corchetes se dejan a la vista: es la convencion para
        // "mantener pulsado", y quitarlos cambiaria el significado.
        Piece::Hold(dentro) => html! {
            <span class="hold-group" title="Mantener pulsado">
                <span class="hold-bracket">{ "[" }</span>
                { for dentro.iter().map(pintar_pieza) }
                <span class="hold-bracket">{ "]" }</span>
            </span>
        },
        // Corchetes al reves, tal cual se escriben: es lo que distingue
        // soltar de mantener de un vistazo.
        Piece::Release(dentro) => html! {
            <span class="release-group" title="Soltar">
                <span class="release-bracket">{ "]" }</span>
                { for dentro.iter().map(pintar_pieza) }
                <span class="release-bracket">{ "[" }</span>
            </span>
        },
        Piece::Label(texto) => html! {
            <span class="piece-label">{ texto }</span>
        },
        Piece::Text(texto) => html! {
            <span class="badge-plain">{ texto }</span>
        },
    }
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
                <span class="notation-group">
                    // El conector se pinta tal cual venia escrito: ">" y "<"
                    // no son lo mismo en la notacion del usuario.
                    if let Some(link) = group.link {
                        <span class="link-mark">
                            { if link == '<' { "‹" } else { "›" } }
                        </span>
                    }
                    { for group.alternatives.iter().enumerate().map(|(ai, alt)| html! {
                        <span class="notation-alt">
                            { for alt.atoms.iter().enumerate().map(|(si, atom)| html! {
                                <span class="notation-atom">
                                    { for atom.pieces.iter().map(pintar_pieza) }
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
