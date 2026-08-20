//! Formulario de alta y edicion de tech.

use crate::components::notation_view::NotationView;
use tokon_tech_log::model::{
    character_by_id, clasificar_enlace, normalize_url, Category, Entry, EntryDraft, LinkKind,
    CHARACTERS,
};
use web_sys::{HtmlInputElement, HtmlSelectElement, HtmlTextAreaElement};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct EntryFormProps {
    /// `Some` cuando se esta editando una entrada existente.
    pub initial: Option<Entry>,
    pub character_id: String,
    pub on_cancel: Callback<()>,
    pub on_save: Callback<EntryDraft>,
}

#[function_component(EntryForm)]
pub fn entry_form(props: &EntryFormProps) -> Html {
    let title = use_state(|| props.initial.as_ref().map(|e| e.title.clone()).unwrap_or_default());
    let notation =
        use_state(|| props.initial.as_ref().map(|e| e.notation.clone()).unwrap_or_default());
    let category =
        use_state(|| props.initial.as_ref().map(|e| e.category).unwrap_or(Category::Bnb));
    let rival = use_state(|| {
        props
            .initial
            .as_ref()
            .and_then(|e| e.rival_id.clone())
            .unwrap_or_default()
    });
    let note = use_state(|| props.initial.as_ref().map(|e| e.note.clone()).unwrap_or_default());
    let x_link = use_state(|| props.initial.as_ref().map(|e| e.x_link.clone()).unwrap_or_default());

    let on_title = {
        let title = title.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            title.set(input.value());
        })
    };
    let on_notation = {
        let notation = notation.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            notation.set(input.value());
        })
    };
    let on_note = {
        let note = note.clone();
        Callback::from(move |e: InputEvent| {
            let area: HtmlTextAreaElement = e.target_unchecked_into();
            note.set(area.value());
        })
    };
    let on_x_link = {
        let x_link = x_link.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            x_link.set(input.value());
        })
    };
    let on_rival = {
        let rival = rival.clone();
        Callback::from(move |e: Event| {
            let select: HtmlSelectElement = e.target_unchecked_into();
            rival.set(select.value());
        })
    };

    let on_cancel = {
        let cb = props.on_cancel.clone();
        Callback::from(move |_: MouseEvent| cb.emit(()))
    };

    let on_save = {
        let (title, notation, category, rival, note, x_link) = (
            title.clone(),
            notation.clone(),
            category.clone(),
            rival.clone(),
            note.clone(),
            x_link.clone(),
        );
        let cb = props.on_save.clone();
        Callback::from(move |_: MouseEvent| {
            if title.trim().is_empty() {
                return;
            }
            cb.emit(EntryDraft {
                title: title.trim().to_string(),
                notation: notation.trim().to_string(),
                category: *category,
                rival_id: if *category == Category::Matchup && !rival.is_empty() {
                    Some((*rival).clone())
                } else {
                    None
                },
                note: note.trim().to_string(),
                x_link: normalize_url(&x_link),
            });
        })
    };

    let guardar_desactivado = title.trim().is_empty();
    let enlace = clasificar_enlace(&x_link);
    let notation_preview = (*notation).clone();
    let rivales: Vec<_> = CHARACTERS
        .iter()
        .filter(|c| c.id != props.character_id)
        .collect();

    html! {
        <div class="entry-form">
            <div class="form-row">
                <label class="form-label" for="f-title">{ "Título" }</label>
                <input
                    id="f-title"
                    class="form-input"
                    value={(*title).clone()}
                    oninput={on_title}
                    placeholder="Ej: Combo tras anti-air"
                />
            </div>

            <div class="form-row">
                <label class="form-label" for="f-notation">{ "Notación" }</label>
                <input
                    id="f-notation"
                    class="form-input form-input-mono"
                    value={(*notation).clone()}
                    oninput={on_notation}
                    placeholder="Ej: L, L, M, H, 2H  ·  236M  ·  M+H"
                />
                <div class="form-hint">
                    { "Se colorean L, M, H, U, A, QS, QA. Los números 1-9 se muestran como direcciones (numpad)." }
                </div>
                if !notation_preview.trim().is_empty() {
                    <div class="form-preview">
                        <NotationView value={notation_preview} />
                    </div>
                }
            </div>

            <div class="form-row form-row-split">
                <div>
                    <div class="form-label">{ "Categoría" }</div>
                    <div class="segmented">
                        { for Category::ALL.iter().map(|c| {
                            let c = *c;
                            let category = category.clone();
                            let activo = *category == c;
                            html! {
                                <button
                                    type="button"
                                    class={classes!("segmented-btn", activo.then_some("active"))}
                                    onclick={Callback::from(move |_: MouseEvent| category.set(c))}
                                >
                                    { c.label() }
                                </button>
                            }
                        }) }
                    </div>
                </div>
                if *category == Category::Matchup {
                    <div>
                        <label class="form-label" for="f-rival">{ "Rival" }</label>
                        <select id="f-rival" class="form-input" onchange={on_rival}>
                            <option value="" selected={rival.is_empty()}>{ "Selecciona…" }</option>
                            { for rivales.iter().map(|c| html! {
                                <option value={c.id} selected={*rival == c.id}>{ c.name }</option>
                            }) }
                        </select>
                    </div>
                }
            </div>

            <div class="form-row">
                <label class="form-label" for="f-note">{ "Nota" }</label>
                <textarea
                    id="f-note"
                    class="form-input form-textarea"
                    rows="3"
                    value={(*note).clone()}
                    oninput={on_note}
                    placeholder="Detalles, timing, condiciones…"
                />
            </div>

            <div class="form-row">
                <label class="form-label" for="f-xlink">{ "Enlace" }</label>
                <input
                    id="f-xlink"
                    class={classes!("form-input", enlace.es_problema().then_some("form-input-error"))}
                    value={(*x_link).clone()}
                    oninput={on_x_link}
                    placeholder="https://x.com/…  ·  https://youtu.be/…"
                />
                if enlace != LinkKind::Vacio {
                    <div class={classes!(
                        "link-hint",
                        if enlace.es_problema() { "link-hint-error" } else { "link-hint-ok" }
                    )}>
                        { enlace.mensaje() }
                    </div>
                } else {
                    <div class="form-hint">
                        { "Cualquier enlace vale: X, YouTube, Dustloop… Si te dejas el https://, se añade solo." }
                    </div>
                }
            </div>

            <div class="form-actions">
                <button type="button" class="btn btn-ghost" onclick={on_cancel}>{ "Cancelar" }</button>
                <button
                    type="button"
                    class="btn btn-primary"
                    disabled={guardar_desactivado}
                    onclick={on_save}
                >
                    { "Guardar" }
                </button>
            </div>
        </div>
    }
}

/// Nombre del rival para mostrar en la tarjeta, si lo hay.
pub fn rival_label(rival_id: &Option<String>) -> Option<String> {
    rival_id
        .as_deref()
        .and_then(character_by_id)
        .map(|c| format!("vs {}", c.name))
}
