#![feature(async_await)]
#![recursion_limit = "512"]

use console_error_panic_hook;
use futures::{
    channel::mpsc::{self, Receiver, Sender},
    executor,
    future::{FutureObj, LocalFutureObj},
    io,
    lock::Mutex,
    sink::SinkExt,
    stream::StreamExt,
    task::{LocalSpawn, LocalSpawnExt},
};
use log::{info, Level};
use std::rc::Rc;
use std::time::Duration;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::futures_0_3::spawn_local;
use wasm_timer::Delay;
use web_logger;
use yew::{self, html::Scope, App};
mod app;
mod connector;

#[wasm_bindgen]
pub fn run() {
    console_error_panic_hook::set_once();
    web_logger::init();

    let mut scope = make_scope();
}

fn make_scope() -> yew::html::Scope<app::Model> {
    let app: App<app::Model> = App::new();
    app.mount_to_body()
}
