//! Logica de Tokon Tech Log, sin nada del navegador.
//!
//! Todo lo de aqui compila y se testea en cualquier maquina con `cargo test`,
//! sin necesitar el target de WebAssembly. La interfaz (Yew) vive aparte, en
//! el binario, y solo se compila para wasm32.

pub mod csv;
pub mod model;
pub mod notation;
