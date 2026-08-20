//! Ventana de escritorio. Este binario solo abre la ventana y sirve la app
//! web que Tauri lleva dentro; toda la logica esta en el proyecto raiz.

// Sin esto, Windows abre una ventana negra de consola detras de la app.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

/// Abre un enlace en el navegador del sistema.
///
/// Hace falta porque el webview de Tauri no abre los enlaces normales: segun
/// la version, o los abre dentro de otra ventana de Tauri o no hace nada.
#[tauri::command]
fn abrir_en_navegador(url: String) -> Result<(), String> {
    // Solo http y https. Sin esto, un enlace guardado con "file:" o similar
    // podria lanzar cualquier cosa del sistema.
    let minus = url.to_lowercase();
    if !minus.starts_with("http://") && !minus.starts_with("https://") {
        return Err("Solo se abren enlaces http o https".into());
    }
    lanzar(&url).map_err(|e| e.to_string())
}

#[cfg(target_os = "windows")]
fn lanzar(url: &str) -> std::io::Result<()> {
    // rundll32 recibe la URL como argumento tal cual. Con "cmd /C start" un
    // "&" dentro del enlace (los de YouTube los llevan) haria que Windows
    // ejecutase lo que viniera detras.
    std::process::Command::new("rundll32")
        .args(["url.dll,FileProtocolHandler", url])
        .spawn()
        .map(|_| ())
}

#[cfg(not(target_os = "windows"))]
fn lanzar(url: &str) -> std::io::Result<()> {
    let programa = if cfg!(target_os = "macos") { "open" } else { "xdg-open" };
    std::process::Command::new(programa).arg(url).spawn().map(|_| ())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![abrir_en_navegador])
        .run(tauri::generate_context!())
        .expect("no se ha podido abrir la ventana de Tokon Tech Log");
}
