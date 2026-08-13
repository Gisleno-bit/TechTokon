//! Iconos SVG a mano. Son cuatro trazos y evitan meter una crate de iconos
//! entera en el binario wasm.

use yew::prelude::*;

fn svg(size: u32, children: Html) -> Html {
    html! {
        <svg
            width={size.to_string()}
            height={size.to_string()}
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            aria-hidden="true"
        >
            { children }
        </svg>
    }
}

pub fn plus(size: u32) -> Html {
    svg(size, html! { <><line x1="12" y1="5" x2="12" y2="19" /><line x1="5" y1="12" x2="19" y2="12" /></> })
}

pub fn pencil(size: u32) -> Html {
    svg(size, html! { <><path d="M12 20h9" /><path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4Z" /></> })
}

pub fn trash(size: u32) -> Html {
    svg(
        size,
        html! {
            <>
                <path d="M3 6h18" />
                <path d="M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
                <path d="M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6" />
                <line x1="10" y1="11" x2="10" y2="17" />
                <line x1="14" y1="11" x2="14" y2="17" />
            </>
        },
    )
}

pub fn external(size: u32) -> Html {
    svg(
        size,
        html! {
            <>
                <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6" />
                <polyline points="15 3 21 3 21 9" />
                <line x1="10" y1="14" x2="21" y2="3" />
            </>
        },
    )
}

pub fn download(size: u32) -> Html {
    svg(
        size,
        html! {
            <>
                <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
                <polyline points="7 10 12 15 17 10" />
                <line x1="12" y1="15" x2="12" y2="3" />
            </>
        },
    )
}

pub fn upload(size: u32) -> Html {
    svg(
        size,
        html! {
            <>
                <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
                <polyline points="17 8 12 3 7 8" />
                <line x1="12" y1="3" x2="12" y2="15" />
            </>
        },
    )
}

pub fn search(size: u32) -> Html {
    svg(size, html! { <><circle cx="11" cy="11" r="8" /><line x1="21" y1="21" x2="16.65" y2="16.65" /></> })
}

pub fn check(size: u32) -> Html {
    svg(size, html! { <polyline points="20 6 9 17 4 12" /> })
}

pub fn close(size: u32) -> Html {
    svg(size, html! { <><line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" /></> })
}
