#![feature(async_await)]
#![recursion_limit = "512"]

use console_error_panic_hook;
use wasm_bindgen::prelude::*;
use web_logger;
use yew::{self, App};
mod app;
mod connector;

#[wasm_bindgen]
pub fn run() {
    console_error_panic_hook::set_once();
    web_logger::init();

    let scope = make_scope();
}

fn make_scope() -> yew::html::Scope<app::Model> {
    let app: App<app::Model> = App::new();
    app.mount_to_body()
}
