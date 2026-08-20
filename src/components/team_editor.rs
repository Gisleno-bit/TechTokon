//! Editor del equipo: nombre y que personajes lo forman.

use crate::components::icons;
use tokon_tech_log::model::{TeamConfig, CHARACTERS, DEFAULT_TEAM_NAME};
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct TeamEditorProps {
    pub team: TeamConfig,
    pub on_cancel: Callback<()>,
    pub on_save: Callback<TeamConfig>,
}

#[function_component(TeamEditor)]
pub fn team_editor(props: &TeamEditorProps) -> Html {
    let name = use_state(|| props.team.name.clone());
    let ids = use_state(|| props.team.ids.clone());

    let on_name = {
        let name = name.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            name.set(input.value());
        })
    };

    let on_cancel = {
        let cb = props.on_cancel.clone();
        Callback::from(move |_: MouseEvent| cb.emit(()))
    };

    let on_save = {
        let (name, ids) = (name.clone(), ids.clone());
        let cb = props.on_save.clone();
        Callback::from(move |_: MouseEvent| {
            let nombre = if name.trim().is_empty() {
                DEFAULT_TEAM_NAME.to_string()
            } else {
                name.trim().to_string()
            };
            cb.emit(TeamConfig { name: nombre, ids: (*ids).clone() });
        })
    };

    let seleccionados = ids.len();

    html! {
        <div class="team-editor">
            <div class="form-row">
                <label class="form-label" for="team-name">{ "Nombre del equipo" }</label>
                <input
                    id="team-name"
                    class="form-input"
                    value={(*name).clone()}
                    oninput={on_name}
                    placeholder="Ej: Latverian Fliers"
                />
            </div>

            <div class="form-row">
                <div class="form-label">{ "Miembros" }</div>
                <div class="form-hint">
                    { format!(
                        "Tōkon usa equipos de 4 · llevas {seleccionados} seleccionado{}",
                        if seleccionados == 1 { "" } else { "s" }
                    ) }
                </div>
                <div class="team-grid">
                    { for CHARACTERS.iter().map(|c| {
                        let activo = ids.contains(&c.id.to_string());
                        let onclick = {
                            let ids = ids.clone();
                            let id = c.id.to_string();
                            Callback::from(move |_: MouseEvent| {
                                let mut nuevos = (*ids).clone();
                                match nuevos.iter().position(|x| x == &id) {
                                    Some(pos) => { nuevos.remove(pos); }
                                    None => nuevos.push(id.clone()),
                                }
                                ids.set(nuevos);
                            })
                        };
                        html! {
                            <button
                                type="button"
                                key={c.id}
                                class={classes!("team-pick", activo.then_some("active"))}
                                {onclick}
                            >
                                <span class="char-mono">{ c.mono }</span>
                                <span class="char-name">{ c.name }</span>
                                if activo { { icons::check(13) } }
                            </button>
                        }
                    }) }
                </div>
            </div>

            <div class="form-actions">
                <button type="button" class="btn btn-ghost" onclick={on_cancel}>{ "Cancelar" }</button>
                <button type="button" class="btn btn-primary" onclick={on_save}>{ "Guardar equipo" }</button>
            </div>
        </div>
    }
}
