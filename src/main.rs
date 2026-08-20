//! Punto de entrada. La interfaz solo existe compilada a WebAssembly; en
//! nativo el binario solo explica como construirla.

#[cfg(target_arch = "wasm32")]
mod app;
#[cfg(target_arch = "wasm32")]
mod components;
#[cfg(target_arch = "wasm32")]
mod platform;

#[cfg(target_arch = "wasm32")]
fn main() {
    yew::Renderer::<app::App>::new().render();
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    eprintln!(
        "Tokon Tech Log es una app web.\n\
         \n\
         Para desarrollar:  trunk serve\n\
         Para publicar:     trunk build --release\n\
         Para los tests:    cargo test\n"
    );
}
