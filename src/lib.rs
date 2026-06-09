pub mod app;
pub mod post_functions;

#[cfg(feature = "ssr")]
pub mod admin_posts;

#[cfg(feature = "ssr")]
pub mod auth;

#[cfg(feature = "ssr")]
pub mod config;

#[cfg(feature = "ssr")]
pub mod db;

#[cfg(feature = "ssr")]
pub mod repositories;

#[cfg(feature = "ssr")]
pub mod storage;

#[cfg(feature = "ssr")]
pub mod uploads;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::App;

    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}
