> **Remember since we're using async-await, this is only possible under nightly rust!!!**
> Based on project template based [rustwasm-parcel template](https://github.com/rustwasm/rust-parcel-template).
> Frontend using [Yew framework](https://github.com/yewstack/yew)
> Rust/Js interop via [wasm_bindgen](https://github.com/rustwasm/wasm-bindgen)

# A rust based async/await web-ui/webextension demo

## Outline

1. project setup
2. frontend rust using yew
3. async rust in wasm
4. Js Promise and Rust Future
5. Distribute it to the web
6. Distribute it as web-extension

---

### 1. project setup

![Screenshot_20190809_144943](/assets/Screenshot_20190809_144943.png)


**components diagram**


Let's break down the components I used in this project:

1. Yew - an elm-inspired rust frontend to creat webapp:
This elm-inspired framework will separate code into a model->update->view cycle. Interaction will affect with model data via Message enum and decide whether a rerender is needed.
2. futures 0.3 - so we can write async-awiat more easily.
3. wasm-bindgen - importing/exporting functions/data structures between rust/js
4. wasm-bindgen-futures - convertion between rust future and js promise
5. web-sys - use web-api

### 2. frontend rust using yew

Yew is a rust frontend framework that is inspired by elm, a framework focused on model/view/update framework.

Things I like about yew:

1. html! macro to write view properly using old good old html.
2. Update model data via message enum
3. parent/child communication: parent->child(props slot), and child->parent(message over component-link)
4. Active community!!

**minimal wasm ui in yew**

Let's understand yew by walkthrough this minimal program.

```rust
use wasm-bindgen::prelude::*;
use yew::{self, html, Component, ComponentLink, Html, Renderable, ShouldRender};

struct Model {
  count: i32,
}

enum Msg {
  Bumped
}

impl Component for Model {
  type Message = Msg;
  // parent component don't need this.
  type Properties = ();

  fn create(prop: Self::Props, link: ComponentLink<Self>) -> Self {
    Model {
      count: 0
    }
  }

  fn update(&mut self, msg: Self::Message) -> ShouldRender {
    match msg {
      Msg::Bumped => {
        self.count +=1;
        // return true if rerender is needed.
        true
      }
    }
  }
}

// Model can be viewed differently under differnt component
impl Renderable<Model> for Model {
  fn view(&self) -> Html<Self> {
    html! {
      <button onclick=|event|Msg::Bumped>{ format!("clicked :{} times", self.count) }</button>
    }
  }
}

#[wasm-bindgen]
fn start_app() -> Result<(), JsValue> {
  yew::start_app::<Model>();
  Ok(())
}

```

### 3. async/await rust in wasm

**async await in rust**

Since async/await in rust is getting stablized quite a bit, let's start with a minimal code block that represents the idea of async programming in rust wasm.

```rust
#![feature(async_await)]
use futures::{self, io, executor};
use wasm_bindgen_futures::futures_0_3::spawn_local;

// a wasm compatible async timer
use wasm_timer::Delay;
use std::time::Duration;


async fn start_first() -> Result<(), io::Error> {
  // regain control after the following future is resolved.
  Delay::new(Duration::from_secs(1)).await?;
  Ok(())
}

async fn start_next() -> Result<(), io::Error> {
  Delay::new(Duration::from_secs(1)).await?;
  Ok(())
}

#[wasm_bindgen]
fn main() {
  // typically rust program need to start a executor to poll futures,
  // but in wasm, the loop is in js so the following may not be needed.

  // let mut loop = executor::LocalPool::new();
  // let mut spawner = loop.spawner();
  // spawner.spawn_local(start_first()).unwrap();
  // spawner.spawn_local(start_next()).unwrap();
  // loop.run();


  // submit futures to the js microtask queue.
  spawn_local(async {
    // sleeping for 1+1 = 2 seconds
    start_first().await;
    start_second().await;

    // sleeping concurrently, so sleep for 1 seconds only.
    let combined_futures = futures::futures::join(start_first(), start_next());
    combined_futures.await;

  });
}

```

There can be various way to utilize async/await in a frontend framework, particularly with a elm-inspired framework. I found it helpful when there are expected events that is scheduled to mutate model data need to be defined. Since mostly data mutation in yew is done via message passing in the update function, we can schedule async event emitter that tigger those messages. This way we avoid figuring out the proper lifetime about data mutation in as async context, they only notify that mutation is needed.

In conjunction with the following code, we can leverage async in the yew minimal example mentioned earlier:

```rust
// a clonable link that can be used to send message to self
struct Modle {
  link: ComponentLink<Self>,
  count: i32
}

// since we can't lock the resource on the model,
// it's easier to clone it instead of fighting with the lifetime under async funtion
async fn count_to_ten(mut from :i32, link: ComponentLink<Model>) {
  while from < 10 {
    Delay::new(Duration::from_secs(2)).await.unwrap();
    link.send_self(Msg::Bumped);
  }
}

// let say we want to start counting to ten from 5
// change the update function in the impl Component for Self
fn update(&mut self, msg: Self::Message) -> ShouldRender {
  match msg => {
    Msg::Bumped => {
      // only trigger it when we reach 5
      if self.count == 5 {
        let cloned_link = 5;
        let start = self.count.clone();
        spawn_local(async move{
          count_to_ten(start, link).await;
        });
      }
      self.count +=1;
      true
    }
  }
}

```

This way, we'll start getting updates from the asyn function after the trigger is reached in the message handler in the update function.

**notes about async-await in wasm**

From what I've gathered, since wasm thread implementation in the web (not sure about wasi) is not complete right now, when we try to execute rust future in wasm, here's what's happened:

1. rust futures will be converted to js promise via wasm-bindgen-futures
2. promises gets queued into the microtask queue in the unblockable js event loop
3. return control to wasm when the promise is fulfilled

So there is a rather huge difference between typical concurrency programming between native rust and python asyncio (got me started learning async) and wasm program.

When we are programming natively(python/rust), the event loop is blockable therefore we can do loop level access about how futures will be executed.

That enables us to use async lock/event/condvar to postpone operation to make loops not starving the main loop, or thus making it more like a embeded loop where it only cares about the resources that granted it to continue.

In wasm however, although we can use queue(futures::channel::mpsc) to control how futures relate to each other, we can not block it, therefore it is not be possible to use ```rust Rc<futures::lock::Mutex<Data>``` to guard resources that might have concurrent access by mutiple async funtion. Theoretically, it might be able to use lock from the js side and make sure no two futures are accessing the resource at the same time.


### 4. Js Promise interop Rust Future


**make fetch-api awaitable**

While we can already use fetch api in the web-sys crate, would it be more awesome if we can manage futures all in one side, so we can chain futures together in a more unified interface?

With wasm-bindgen-futures, it's now possible to do so:

```rust
use wasm_bindgen_futures::futures_0_3::{future_to_promise, JsFuture, spawn_local};

// this can be awaited in a async context in rust
let future = JsFuture::from(js_promise);
...
```

We will then be able create an async funtion the wrap a JsPromise into a rust awaitable future.

Let's create a future from the JsPromise created by the web-sys fetch-api.

```rust
async fn fetch_data() -> Result<Option<Payload>, JsValue> {
  // creat a request
  let mut opts = RequestInit::new();
  opts.method("GET");
  opts.mode(RequestMode::Cors);
  let request = Request::new_with_str_and_init("https://favqs.com/api/qotd", &opts).unwrap();

  // generate the promise and convert it to future
  let window = web_sys::window().unwrap();
  let request_promise = window.fetch_with_request(&request);
  let future = JsFuture::from(request_promise);

  // continue if fetch result is gathered
  let resp = future.await?;
  let resp: Response = resp.dyn_into().expect("response not working...");
  let mut rv = None;
  if let Ok(json) = resp.json() {

      // continue if parsing response to json complete
      if let Ok(json) = JsFuture::from(json).await {
          if let Ok(rv) = json.into_serde::<Payload>() {
              rv = Ok(Some(rv));
          }
      }
  }
  rv
}
```


Based on the knowledge we now have about how yew components work, we can now build a component with conditional rendering where the content is fetched via an async function.

```rust

// a yew component to render the parsed result
struct Pannel {
  payload: Option<Payload>
}

// conditional rendering with yew
impl Renderable<Pannel> for Pannel {
  fn view(&self) -> Html<Self> {
    if let Some(payload) = self.payload {
      html! {
        <div>{ payload.content }</div>
      }
    } else {
      <div>{ "sorry... nothing to be seen here..." }</div>
    }
  }
}
```

**mutation data/view in by an asyn context in yew**

I think how one want to do this might change depends on their use cases.

Considering the possible combinations of components(parent/children) and interactions(notify/async_context/mutation), you will have a rather flexible degree of freedom to decide how an async context mutate the data and therefore altering the view.

It can be as simple as:
1. Component issue an async context
2. Receive the context body
3. Mutate data via it's component link.

Or a bit more complex as:
1. The parent issue a async conext.
2. Receive the response body
3. Issue a mutation via childrens' component link.
4. Directly mutating the children's data via props change.

The two different ways to mutate data that involve parent/children communicationcan be showcased in the example below:

**mutation via message passing in children's link**

```rust

// you will need a context to trigger that async context, maybe in a parent component
html! {
  <button onclick=|_| Msg::GoFetch>{  "fetchit" }</button>
}

// tigger the event from the parent component, mutate data at children component.
match msg {
  Msg::GoFetch => {

    // use the link to notify the child to accept a mutation.
    let mut child_link = self.children.link.clone();

    // remember that spawn_local have a requirement of static lifetime,
    // so it's easier to pour external data inside instead of the opposite
    spawn_local(async move {
      let payload = fetch_data().await.unwrap();
      child_link.send_self(ChldMsg::GotPayload(payload));
    });

    // no rerender needed in the parent component.
    false
  }
}

```

**mutation via props change**

```rust

// dictate child data via props
impl Renderable<Parent> for Parent {
  fn view(&self) -> Html<Self> {
    html! {
      <div>
      <Children payload=self.payload />
      </div>
    }
  }
}

// child will mutate data in the change/create function
impl Component for Child {

  // remember to define props struct
  type Properties = ChildProperties;
  type Message = ChildMsg;

  // mount data upon creating
  fn create(props: Self::Properties, link: ComponentLin<Self>) -> Self {
    Self {
      payload: props:payload
    }
  }

  // fn update() {}...

  // mutate data when props changed in parent
  fn change(&mut self, props: Self::Properties) ShouldRender {
    self.payload = props.payload;
    true
  }
}

```

### Distribute it to the web

This is rather simple thanks to wasm-bindgen and the rustwasm team for providing parcel/webpack templates.

For rustaceans that are not familiar with webdev and frontend bundlers, the minimal process to build rust to wasm is like this:

![Screenshot_20190809_144959](/assets/Screenshot_20190809_144959.png)

This will make your rust program a wasm binary. However, for calling rust functions in js more friendly, wasm-bindgen will help us tag our rust functions in the wasm module along with stuffs like type conversion in the generated wasm between rust/wasm.

```rust
#[wasm_bindgen]
fn func_name() {}
```

If we tagged our rust function with wasm_bindgen attribute, after we import our wasm, it will be mounted under module.func_name()

```js
module.func_name()
```

**bundler to the rescue**

Back to the bundlers.

While we're talking about fronend development, prototyping quickly is rather important, since it's less cool to trigger the build process, write colde that import the wasm blob, and eventually execute functions in that wasm module everytime we made some changes to the rust code, we might leave it to the bundlers.

So the build process now looks like this:

![Screenshot_20190809_145011](/assets/Screenshot_20190809_145011.png)

Eventually, rust code is still governed by Cargo.toml, and will build with the right rlease falg when your bundler says so, but not in dev-mode (often enable hot-reload).

Since we will stil need js as a entry point in the web, with the help of the bundler, we can make to be as simple as:

```js
import { start } from "../Cargo.toml"
start()
```

Isn't it just pure awesome that we can focus on writing rust?

If you prepare a minimal html fild the import the js file, you can start a dev-server and see live-action of your rust code.

```bash
parcel index.html
```

And trigger the production build where rust is build with --release

```bash
parcel build index.html
```

This will create a production build in dist folder by default.

**managing frontend/dependencis using npm**

npm was designed to manage frontend dependencis, similar to the concept how Cargo.toml work, we can add depedency and let it resolve and update depedencies automatically.

```bash
npm --save bulma
```

This will save the depedency to package.json a Cargo.toml equivalent for frontend-dev.

```js
import "bulma"
```

Since bulma is under a known namespace now, it can be used in our rust program too.

```rust
html! {
  <div class="card"></div>
}
```

package.json can save script to save keystrokes:

If you navigate through the package.json provided by rustwasm-team, you can see that script has two keys defined by a script.

So it's now fewer clicks with autocomplet:

```bash
npm run start
npm run build
```

Remember that bundler/npm are all optional to work with the web.

I recommend to use those tool to make it easier to manager your project and simply focus on wrinting rust code.

### Distribute it as web-extension

Traditionally, web-extensions are written using html/js/css, which is awesome enough. What if we can write rust based web-extensions?

Although I've never written a web-extension project, I found that it's not a unreachable goal. In stead, the tooling around really creat wonders.

There's a [web-ext](https://www.npmjs.com/package/web-ext) node module that provide a command-lined interface to focus on creating web-extensions for browser.

For creating web-extension file, we will need a manifest.json. I can't go into detail since I'm new to this too.


```json
{
  "manifest_version": 1,
  "name": "rustwasm-addon",
  "version": "0.0.1",
  "description": "rustwasm-addon",
  "permissions": [
    "activeTab"
  ],
  "sidebar_action": {
    "default_title": "rustwasm-addon",
    "default_panel": "index.html"
  }
}

```

What this does is to regierter the sidebar to show our index.html (which includes the wasm file we created from rust) when user open the extension in the sidebar.

**web-ext in action**

After we've installed web-ext:

```bash
npm -g install web-ext
```

We can test the webextension by:

```bash
web-ext run
```

Or build it as bundle by:

```bash
web-ext build
```

**notes about web-ext and bundler**

It seems that when package.json is discoverable under the same folder with manifest.json, the web-ext will try to build the project using the defined bundler. Unfortunately I can't seem to get it work with parcel.

Therefore, I've tried to build the dist bundle first, move it to another folder, and just let web-ext build it as a static resource.

---

# Demo

This demo will let you spawn quotes fetched from public api, and auto refresh then every few seconds by an async function.

![Peek 2019-08-06 04-14](/assets/Peek%202019-08-06%2004-14.gif)
