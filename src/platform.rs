//! Lo poco que la app necesita del navegador: guardar, descargar y fechas.

use gloo_storage::{LocalStorage, Storage};
use tokon_tech_log::model::{format_day_month, TeamConfig, TechData};
use wasm_bindgen::JsCast;
use web_sys::HtmlAnchorElement;

const DATA_KEY: &str = "tokon-tech-log-v1";
const TEAM_KEY: &str = "tokon-team-v1";

pub fn load_data() -> TechData {
    LocalStorage::get(DATA_KEY).unwrap_or_default()
}

/// Devuelve false si el navegador rechaza el guardado (modo privado, disco
/// lleno). La app lo avisa en la cabecera en vez de fallar en silencio.
pub fn save_data(data: &TechData) -> bool {
    LocalStorage::set(DATA_KEY, data).is_ok()
}

pub fn load_team() -> TeamConfig {
    LocalStorage::get::<TeamConfig>(TEAM_KEY)
        .map(|t| t.sanitized())
        .unwrap_or_default()
}

pub fn save_team(team: &TeamConfig) -> bool {
    LocalStorage::set(TEAM_KEY, team).is_ok()
}

/// Id corto y suficientemente unico para distinguir entradas locales.
pub fn new_id() -> String {
    let random = (js_sys::Math::random() * 1.0e9) as u64;
    let now = js_sys::Date::now() as u64;
    format!("{random:x}{now:x}")
}

pub fn now_ms() -> f64 {
    js_sys::Date::now()
}

/// "09 ago" a partir de un timestamp en milisegundos.
pub fn format_date(ts: f64) -> String {
    let date = js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(ts));
    format_day_month(date.get_date(), date.get_month() as usize)
}

pub fn confirm(message: &str) -> bool {
    web_sys::window()
        .and_then(|w| w.confirm_with_message(message).ok())
        .unwrap_or(false)
}

/// True si la app corre dentro de la ventana de escritorio (Tauri) en vez de
/// en un navegador. Tauri inyecta estos objetos en `window`.
///
/// Importa porque el webview de Tauri ignora las descargas por blob: el enlace
/// se pulsa y no pasa nada, sin error. Ahi hay que entregar el CSV de otra
/// forma.
pub fn es_escritorio() -> bool {
    let Some(window) = web_sys::window() else {
        return false;
    };
    ["__TAURI_INTERNALS__", "__TAURI__"].iter().any(|clave| {
        js_sys::Reflect::get(&window, &wasm_bindgen::JsValue::from_str(clave))
            .map(|v| !v.is_undefined() && !v.is_null())
            .unwrap_or(false)
    })
}

/// Abre un enlace en el navegador del sistema.
///
/// En navegador no hace falta (el `<a>` ya funciona). En la app de escritorio
/// se lo pide a Rust, porque el webview de Tauri ignora los enlaces normales.
///
/// Devuelve false si no ha podido: quien llama debe tener un plan B en vez de
/// dejar al usuario con un clic que no hace nada.
pub fn abrir_enlace(url: &str) -> bool {
    let Some(window) = web_sys::window() else {
        return false;
    };

    // window.__TAURI__.core.invoke("abrir_en_navegador", { url })
    let buscar = |objeto: &wasm_bindgen::JsValue, clave: &str| {
        js_sys::Reflect::get(objeto, &wasm_bindgen::JsValue::from_str(clave))
            .ok()
            .filter(|v| !v.is_undefined() && !v.is_null())
    };

    let Some(tauri) = buscar(&window, "__TAURI__") else {
        return false;
    };
    let Some(core) = buscar(&tauri, "core") else {
        return false;
    };
    let Some(invoke) = buscar(&core, "invoke") else {
        return false;
    };
    let Ok(funcion) = invoke.dyn_into::<js_sys::Function>() else {
        return false;
    };

    let args = js_sys::Object::new();
    if js_sys::Reflect::set(
        &args,
        &wasm_bindgen::JsValue::from_str("url"),
        &wasm_bindgen::JsValue::from_str(url),
    )
    .is_err()
    {
        return false;
    }

    funcion
        .call2(
            &core,
            &wasm_bindgen::JsValue::from_str("abrir_en_navegador"),
            &args,
        )
        .is_ok()
}

/// Lanza la descarga de un archivo de texto generado en memoria.
pub fn download_text(filename: &str, content: &str) {
    let Some(window) = web_sys::window() else { return };
    let Some(document) = window.document() else { return };

    let parts = js_sys::Array::new();
    parts.push(&wasm_bindgen::JsValue::from_str(content));

    // Sin BlobPropertyBag: la extension del nombre ya define el tipo para la
    // descarga, y asi no dependemos de una API que cambia entre versiones.
    let Ok(blob) = web_sys::Blob::new_with_str_sequence(&parts) else { return };
    let Ok(url) = web_sys::Url::create_object_url_with_blob(&blob) else { return };

    let Ok(element) = document.create_element("a") else { return };
    let anchor: HtmlAnchorElement = element.unchecked_into();
    anchor.set_href(&url);
    anchor.set_download(filename);

    if let Some(body) = document.body() {
        let _ = body.append_child(&anchor);
        anchor.click();
        let _ = body.remove_child(&anchor);
    }
    let _ = web_sys::Url::revoke_object_url(&url);
}
