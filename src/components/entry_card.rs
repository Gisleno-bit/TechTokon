//! Tarjeta de una tech ya guardada.

use crate::components::entry_form::rival_label;
use crate::components::icons;
use crate::components::notation_view::NotationView;
use crate::platform::format_date;
use tokon_tech_log::model::{clasificar_enlace, Entry, LinkKind};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct EntryCardProps {
    pub entry: Entry,
    pub on_edit: Callback<()>,
    pub on_delete: Callback<()>,
    /// Se llama si el enlace no ha podido abrirse, para enseñarlo y que el
    /// usuario lo copie en vez de quedarse con un clic que no hace nada.
    pub on_link_fallback: Callback<String>,
}

#[function_component(EntryCard)]
pub fn entry_card(props: &EntryCardProps) -> Html {
    let entry = &props.entry;

    let on_edit = {
        let cb = props.on_edit.clone();
        Callback::from(move |_: MouseEvent| cb.emit(()))
    };
    let on_delete = {
        let cb = props.on_delete.clone();
        Callback::from(move |_: MouseEvent| cb.emit(()))
    };

    // En navegador no se toca nada: el <a> abre la pestaña como siempre.
    // En escritorio hay que pedirselo a Rust.
    let on_link = {
        let url = entry.x_link.clone();
        let fallback = props.on_link_fallback.clone();
        Callback::from(move |e: MouseEvent| {
            if !crate::platform::es_escritorio() {
                return;
            }
            e.prevent_default();
            if !crate::platform::abrir_enlace(&url) {
                fallback.emit(url.clone());
            }
        })
    };

    html! {
        <div class="entry-card">
            <div class="entry-card-top">
                <NotationView value={entry.notation.clone()} />
                <div class="entry-card-tags">
                    <span class={format!("tag tag-{}", entry.category.slug())}>
                        { entry.category.label() }
                    </span>
                    if let Some(rival) = rival_label(&entry.rival_id) {
                        <span class="tag tag-rival">{ rival }</span>
                    }
                </div>
            </div>

            <div class="entry-card-title">{ &entry.title }</div>
            if !entry.note.is_empty() {
                <div class="entry-card-note">{ &entry.note }</div>
            }

            <div class="entry-card-bottom">
                <span class="entry-card-date">{ format_date(entry.created_at) }</span>
                <div class="entry-card-actions">
                    if !entry.x_link.is_empty() {
                        <a
                            class="icon-link"
                            href={entry.x_link.clone()}
                            target="_blank"
                            rel="noopener noreferrer"
                            onclick={on_link}
                            title={match clasificar_enlace(&entry.x_link) {
                                LinkKind::X => "Abrir en X",
                                LinkKind::YouTube => "Abrir en YouTube",
                                _ => "Abrir el enlace",
                            }}
                        >
                            { icons::external(14) }
                        </a>
                    }
                    <button class="icon-btn" onclick={on_edit} title="Editar">
                        { icons::pencil(14) }
                    </button>
                    <button class="icon-btn icon-btn-danger" onclick={on_delete} title="Eliminar">
                        { icons::trash(14) }
                    </button>
                </div>
            </div>
        </div>
    }
}
