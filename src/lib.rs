pub mod app;
pub mod error_template;
#[cfg(feature = "ssr")]
pub mod fileserv;

use std::collections::HashMap;

#[derive(Default, serde::Serialize, serde::Deserialize, Debug)]
pub struct Commands(pub HashMap<String, String>);
pub struct PasswordHashString(pub String);

#[derive(Default, Clone)]
pub struct RateLimiting {
    pub last_request_time: Option<std::time::Instant>,
}

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::*;
    console_error_panic_hook::set_once();
    leptos::mount_to_body(App);
}
