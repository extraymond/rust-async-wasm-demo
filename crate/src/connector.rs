use futures::{
    executor,
    future::{self, TryFutureExt},
    io, Future,
};
use js_sys::Promise;
use log::info;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::futures_0_3::spawn_local;
use wasm_bindgen_futures::futures_0_3::{future_to_promise, JsFuture};
use wasm_timer::Delay;
use web_sys::{
    ErrorEvent, MessageEvent, Notification, Request, RequestInit, RequestMode, Response, WebSocket,
};

/// A struct to hold some data from the github Branch API.
///
/// Note how we don't have to define every member -- serde will ignore extra
/// data when deserializing
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Payload {
    pub qotd_date: String,
    pub quote: Quote,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Quote {
    pub id: i32,
    pub dialogue: bool,
    pub private: bool,
    pub tags: Vec<String>,
    pub favorites_count: i32,
    pub upvotes_count: i32,
    pub downvotes_count: i32,
    pub author: String,
    pub author_permalink: String,
    pub body: String,
}

pub async fn fetchit() -> Result<Option<Payload>, JsValue> {
    let mut opts = RequestInit::new();
    opts.method("GET");
    opts.mode(RequestMode::Cors);

    let request = Request::new_with_str_and_init("https://favqs.com/api/qotd", &opts).unwrap();

    let window = web_sys::window().unwrap();
    let request_promise = window.fetch_with_request(&request);
    let future = JsFuture::from(request_promise);

    let resp = future.await?;
    let resp: Response = resp.dyn_into().expect("response not working...");
    if let Ok(json) = resp.json() {
        if let Ok(json) = JsFuture::from(json).await {
            if let Ok(rv) = json.into_serde::<Payload>() {
                Ok(Some(rv))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
}
