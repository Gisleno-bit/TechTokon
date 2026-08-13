//! Componente raiz: estado de la app, barra lateral y acciones de la cabecera.

use crate::components::entry_card::EntryCard;
use crate::components::entry_form::EntryForm;
use crate::components::icons;
use crate::components::team_editor::TeamEditor;
use crate::platform;
use tokon_tech_log::csv;
use tokon_tech_log::model::{
    character_by_id, hashtag_for, x_search_url, Category, Character, Entry, EntryDraft, TeamConfig,
    TechData, CHARACTERS,
};
use tokon_tech_log::notation::{button_style, LEGEND};
use web_sys::HtmlInputElement;
use yew::prelude::*;

/// Filtro de categoria de la barra de chips. `None` = todo.
type Filtro = Option<Category>;

#[derive(Properties, PartialEq)]
struct CharRowProps {
    character: &'static Character,
    active: bool,
    count: usize,
    in_team: bool,
    on_click: Callback<()>,
}

#[function_component(CharRow)]
fn char_row(props: &CharRowProps) -> Html {
    let onclick = {
        let cb = props.on_click.clone();
        Callback::from(move |_: MouseEvent| cb.emit(()))
    };
    html! {
        <button
            class={classes!(
                "char-item",
                props.active.then_some("active"),
                props.in_team.then_some("char-item-team"),
            )}
            {onclick}
        >
            <span class="char-mono">{ props.character.mono }</span>
            <span class="char-name">{ props.character.name }</span>
            if props.count > 0 {
                <span class="char-count">{ props.count }</span>
            }
        </button>
    }
}

#[function_component(App)]
pub fn app() -> Html {
    let data = use_state(platform::load_data);
    let team = use_state(platform::load_team);
    let selected = use_state(|| {
        // Arranca en el primer miembro del equipo, o en el primero del roster.
        platform::load_team()
            .ids
            .first()
            .cloned()
            .unwrap_or_else(|| CHARACTERS[0].id.to_string())
    });
    let save_error = use_state(|| false);
    let filtro = use_state(|| None as Filtro);
    let busqueda = use_state(String::new);
    let form_open = use_state(|| false);
    let editing_id = use_state(|| None as Option<String>);
    let team_editor_open = use_state(|| false);
    let import_result = use_state(|| None as Option<csv::ImportResult>);
    let file_ref = use_node_ref();

    // Guarda y avisa en la cabecera si el navegador rechaza el guardado.
    let persist = {
        let data = data.clone();
        let save_error = save_error.clone();
        move |nuevo: TechData| {
            save_error.set(!platform::save_data(&nuevo));
            data.set(nuevo);
        }
    };

    let selected_char = character_by_id(&selected).unwrap_or(&CHARACTERS[0]);
    let entradas: Vec<Entry> = data.get(&*selected).cloned().unwrap_or_default();

    let visibles: Vec<Entry> = {
        let q = busqueda.trim().to_lowercase();
        let mut v: Vec<Entry> = entradas
            .iter()
            .filter(|e| filtro.map_or(true, |f| e.category == f))
            .filter(|e| {
                q.is_empty()
                    || e.title.to_lowercase().contains(&q)
                    || e.notation.to_lowercase().contains(&q)
                    || e.note.to_lowercase().contains(&q)
            })
            .cloned()
            .collect();
        // Lo mas reciente arriba.
        v.sort_by(|a, b| b.created_at.total_cmp(&a.created_at));
        v
    };

    let seleccionar = {
        let (selected, form_open, editing_id, filtro, busqueda, team_editor_open) = (
            selected.clone(),
            form_open.clone(),
            editing_id.clone(),
            filtro.clone(),
            busqueda.clone(),
            team_editor_open.clone(),
        );
        Callback::from(move |id: String| {
            selected.set(id);
            form_open.set(false);
            editing_id.set(None);
            team_editor_open.set(false);
            filtro.set(None);
            busqueda.set(String::new());
        })
    };

    let on_add = {
        let (selected, persist, data, form_open) =
            (selected.clone(), persist.clone(), data.clone(), form_open.clone());
        Callback::from(move |draft: EntryDraft| {
            let mut nuevo = (*data).clone();
            nuevo.entry((*selected).clone()).or_default().push(Entry {
                id: platform::new_id(),
                title: draft.title,
                notation: draft.notation,
                category: draft.category,
                rival_id: draft.rival_id,
                note: draft.note,
                x_link: draft.x_link,
                created_at: platform::now_ms(),
            });
            persist(nuevo);
            form_open.set(false);
        })
    };

    let on_update = {
        let (selected, persist, data, editing_id) =
            (selected.clone(), persist.clone(), data.clone(), editing_id.clone());
        Callback::from(move |(id, draft): (String, EntryDraft)| {
            let mut nuevo = (*data).clone();
            if let Some(lista) = nuevo.get_mut(&*selected) {
                if let Some(e) = lista.iter_mut().find(|e| e.id == id) {
                    e.title = draft.title;
                    e.notation = draft.notation;
                    e.category = draft.category;
                    e.rival_id = draft.rival_id;
                    e.note = draft.note;
                    e.x_link = draft.x_link;
                }
            }
            persist(nuevo);
            editing_id.set(None);
        })
    };

    let on_delete = {
        let (selected, persist, data) = (selected.clone(), persist.clone(), data.clone());
        Callback::from(move |id: String| {
            if !platform::confirm("¿Eliminar esta tech?") {
                return;
            }
            let mut nuevo = (*data).clone();
            if let Some(lista) = nuevo.get_mut(&*selected) {
                lista.retain(|e| e.id != id);
            }
            persist(nuevo);
        })
    };

    let on_clear_all = {
        let persist = persist.clone();
        Callback::from(move |_: MouseEvent| {
            if platform::confirm(
                "Esto borra toda la tech guardada de los 20 personajes en este navegador. ¿Seguro?",
            ) {
                persist(TechData::new());
            }
        })
    };

    let on_export = {
        let data = data.clone();
        Callback::from(move |_: MouseEvent| {
            let contenido = csv::to_csv(&csv::export_rows(&data));
            platform::download_text("tokon-tech-log.csv", &contenido);
        })
    };

    let on_template = Callback::from(move |_: MouseEvent| {
        let contenido = csv::to_csv(&csv::template_rows());
        platform::download_text("plantilla-tech.csv", &contenido);
    });

    let on_pick_file = {
        let file_ref = file_ref.clone();
        Callback::from(move |_: MouseEvent| {
            if let Some(input) = file_ref.cast::<HtmlInputElement>() {
                input.click();
            }
        })
    };

    let on_file_change = {
        let (data, persist, import_result) =
            (data.clone(), persist.clone(), import_result.clone());
        Callback::from(move |e: Event| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let Some(file) = input.files().and_then(|f| f.get(0)) else {
                return;
            };
            // Permite volver a elegir el mismo archivo despues de corregirlo.
            input.set_value("");

            let (data, persist, import_result) =
                (data.clone(), persist.clone(), import_result.clone());
            wasm_bindgen_futures::spawn_local(async move {
                match gloo_file::futures::read_as_text(&gloo_file::File::from(file)).await {
                    Ok(texto) => {
                        let mut nuevo = (*data).clone();
                        let mut next_id = platform::new_id;
                        let resultado = csv::import_into(
                            &texto,
                            &mut nuevo,
                            platform::now_ms(),
                            &mut next_id,
                        );
                        persist(nuevo);
                        import_result.set(Some(resultado));
                    }
                    Err(_) => import_result.set(Some(csv::ImportResult {
                        added: 0,
                        errors: vec!["No he podido leer el archivo.".to_string()],
                    })),
                }
            });
        })
    };

    let on_save_team = {
        let (team, team_editor_open) = (team.clone(), team_editor_open.clone());
        Callback::from(move |nuevo: TeamConfig| {
            let limpio = nuevo.sanitized();
            platform::save_team(&limpio);
            team.set(limpio);
            team_editor_open.set(false);
        })
    };

    let miembros: Vec<&'static Character> = team
        .ids
        .iter()
        .filter_map(|id| character_by_id(id))
        .collect();
    let resto: Vec<&'static Character> = CHARACTERS
        .iter()
        .filter(|c| !team.contains(c.id))
        .collect();

    let tag_char = hashtag_for(selected_char.name);
    let tag_general = "#MARVELTōkon".to_string();

    html! {
        <div class="app">
            <header class="header">
                <div>
                    <div class="header-title">{ "TOKON // TECH LOG" }</div>
                    <div class="header-sub">
                        { &team.name }
                        if *save_error { { " · cambios sin guardar" } }
                    </div>
                </div>
                <div class="header-actions">
                    <input
                        ref={file_ref}
                        type="file"
                        accept=".csv,text/csv"
                        style="display: none"
                        onchange={on_file_change}
                    />
                    <button class="btn btn-ghost btn-sm" onclick={on_template} title="Plantilla CSV para tu hoja de cálculo">
                        { icons::download(14) }{ " Plantilla" }
                    </button>
                    <button class="btn btn-ghost btn-sm" onclick={on_pick_file} title="Importar CSV">
                        { icons::upload(14) }{ " Importar" }
                    </button>
                    <button class="btn btn-ghost btn-sm" onclick={on_export} title="Exportar a CSV">
                        { icons::download(14) }{ " Exportar" }
                    </button>
                    <button class="btn btn-ghost btn-sm btn-danger-text" onclick={on_clear_all}>
                        { "Vaciar todo" }
                    </button>
                </div>
            </header>

            if let Some(resultado) = &*import_result {
                <div class={classes!(
                    "import-banner",
                    (!resultado.errors.is_empty()).then_some("has-errors")
                )}>
                    <div class="import-banner-text">
                        if resultado.added > 0 {
                            { format!(
                                "{} tech importada{}. ",
                                resultado.added,
                                if resultado.added == 1 { "" } else { "s" }
                            ) }
                        }
                        if !resultado.errors.is_empty() {
                            { format!(
                                "{} fila{} con problemas: {}",
                                resultado.errors.len(),
                                if resultado.errors.len() == 1 { "" } else { "s" },
                                resultado.errors.iter().take(3).cloned().collect::<Vec<_>>().join(" ")
                            ) }
                        }
                    </div>
                    <button class="icon-btn" onclick={{
                        let import_result = import_result.clone();
                        Callback::from(move |_: MouseEvent| import_result.set(None))
                    }}>
                        { icons::close(13) }
                    </button>
                </div>
            }

            <div class="body">
                <nav class="sidebar">
                    <div class="sidebar-section-label sidebar-section-label-row">
                        <span>{ "Equipo" }</span>
                        <button
                            class="icon-btn"
                            title="Editar equipo"
                            onclick={{
                                let (team_editor_open, form_open, editing_id) =
                                    (team_editor_open.clone(), form_open.clone(), editing_id.clone());
                                Callback::from(move |_: MouseEvent| {
                                    team_editor_open.set(true);
                                    form_open.set(false);
                                    editing_id.set(None);
                                })
                            }}
                        >
                            { icons::pencil(12) }
                        </button>
                    </div>

                    if miembros.is_empty() {
                        <div class="team-empty-hint">{ "Sin equipo elegido — edítalo arriba." }</div>
                    }
                    { for miembros.iter().map(|c| html! {
                        <CharRow
                            key={c.id}
                            character={*c}
                            active={*selected == c.id}
                            count={data.get(c.id).map_or(0, |v| v.len())}
                            in_team={true}
                            on_click={seleccionar.reform({ let id = c.id.to_string(); move |_| id.clone() })}
                        />
                    }) }

                    <div class="sidebar-section-label sidebar-section-label-2">{ "Resto del roster" }</div>
                    { for resto.iter().map(|c| html! {
                        <CharRow
                            key={c.id}
                            character={*c}
                            active={*selected == c.id}
                            count={data.get(c.id).map_or(0, |v| v.len())}
                            in_team={false}
                            on_click={seleccionar.reform({ let id = c.id.to_string(); move |_| id.clone() })}
                        />
                    }) }
                </nav>

                <main class="main">
                    if *team_editor_open {
                        <TeamEditor
                            team={(*team).clone()}
                            on_cancel={{
                                let team_editor_open = team_editor_open.clone();
                                Callback::from(move |_| team_editor_open.set(false))
                            }}
                            on_save={on_save_team}
                        />
                    } else {
                        <div class="main-header">
                            <div class="main-header-left">
                                <div class="char-mono char-mono-lg">{ selected_char.mono }</div>
                                <div>
                                    <div class="main-title">{ selected_char.name }</div>
                                    <div class="hashtag-row">
                                        <a
                                            class="hashtag-chip"
                                            href={x_search_url(&tag_char)}
                                            target="_blank"
                                            rel="noopener noreferrer"
                                            title="Buscar este hashtag en X"
                                        >
                                            { icons::search(11) }{ tag_char.clone() }
                                        </a>
                                        <a
                                            class="hashtag-chip"
                                            href={x_search_url(&tag_general)}
                                            target="_blank"
                                            rel="noopener noreferrer"
                                            title="Buscar este hashtag en X"
                                        >
                                            { icons::search(11) }{ tag_general.clone() }
                                        </a>
                                    </div>
                                </div>
                            </div>
                            <button
                                class="btn btn-primary btn-sm"
                                onclick={{
                                    let (form_open, editing_id) = (form_open.clone(), editing_id.clone());
                                    Callback::from(move |_: MouseEvent| {
                                        form_open.set(!*form_open);
                                        editing_id.set(None);
                                    })
                                }}
                            >
                                { icons::plus(14) }{ " Nueva tech" }
                            </button>
                        </div>

                        <div class="legend">
                            { for LEGEND.iter().map(|(code, label)| {
                                let color = button_style(code).map(|b| b.bg).unwrap_or("#888");
                                html! {
                                    <span class="legend-item">
                                        <span class="legend-dot" style={format!("background: {color}")} />
                                        { *code }
                                        <span class="legend-label">{ *label }</span>
                                    </span>
                                }
                            }) }
                        </div>

                        <div class="filter-row">
                            <div class="chip-row">
                                { for std::iter::once((None as Filtro, "Todo"))
                                    .chain(Category::ALL.iter().map(|c| (Some(*c), c.label())))
                                    .map(|(valor, etiqueta)| {
                                        let filtro = filtro.clone();
                                        let activo = *filtro == valor;
                                        html! {
                                            <button
                                                class={classes!("chip", activo.then_some("active"))}
                                                onclick={Callback::from(move |_: MouseEvent| filtro.set(valor))}
                                            >
                                                { etiqueta }
                                            </button>
                                        }
                                    })
                                }
                            </div>
                            <div class="search-box">
                                { icons::search(14) }
                                <input
                                    value={(*busqueda).clone()}
                                    placeholder="Buscar…"
                                    oninput={{
                                        let busqueda = busqueda.clone();
                                        Callback::from(move |e: InputEvent| {
                                            let input: HtmlInputElement = e.target_unchecked_into();
                                            busqueda.set(input.value());
                                        })
                                    }}
                                />
                            </div>
                        </div>

                        if *form_open {
                            <EntryForm
                                initial={None}
                                character_id={(*selected).clone()}
                                on_cancel={{
                                    let form_open = form_open.clone();
                                    Callback::from(move |_| form_open.set(false))
                                }}
                                on_save={on_add}
                            />
                        }

                        <div class="entry-list">
                            if visibles.is_empty() && !*form_open {
                                <div class="empty-state">
                                    if entradas.is_empty() {
                                        { format!("Aún no hay tech para {}. Añade la primera.", selected_char.name) }
                                    } else {
                                        { "No hay resultados para este filtro." }
                                    }
                                </div>
                            }
                            { for visibles.iter().map(|entry| {
                                let id = entry.id.clone();
                                // `key` va aparte: html! construye los closures
                                // antes de evaluarla, y para entonces `id` ya
                                // se ha movido dentro de ellos.
                                let key = entry.id.clone();
                                if editing_id.as_deref() == Some(id.as_str()) {
                                    html! {
                                        <EntryForm
                                            key={key}
                                            initial={Some(entry.clone())}
                                            character_id={(*selected).clone()}
                                            on_cancel={{
                                                let editing_id = editing_id.clone();
                                                Callback::from(move |_| editing_id.set(None))
                                            }}
                                            on_save={on_update.reform(move |draft| (id.clone(), draft))}
                                        />
                                    }
                                } else {
                                    html! {
                                        <EntryCard
                                            key={key}
                                            entry={entry.clone()}
                                            on_edit={{
                                                let (editing_id, form_open, id) =
                                                    (editing_id.clone(), form_open.clone(), id.clone());
                                                Callback::from(move |_| {
                                                    editing_id.set(Some(id.clone()));
                                                    form_open.set(false);
                                                })
                                            }}
                                            on_delete={on_delete.reform(move |_| id.clone())}
                                        />
                                    }
                                }
                            }) }
                        </div>
                    }
                </main>
            </div>
        </div>
    }
}
