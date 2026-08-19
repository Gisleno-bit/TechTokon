// Sin esto, Windows abre una ventana negra de consola detras de la app.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("no se ha podido abrir la ventana de Tokon Tech Log");
}
