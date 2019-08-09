use super::connector;

use std::time::Duration;
use wasm_bindgen_futures::futures_0_3::spawn_local;
use wasm_timer::Delay;
use yew::{html, Component, ComponentLink, Html, Renderable, ShouldRender};

pub struct Model {
    tasks: Vec<Task>,
}

pub struct Task {
    pub status: i32,
    pub fav: bool,
    pub info: Option<connector::Payload>,
}

pub enum TaskMsg {
    Fetchit(connector::Payload),
    ToggleFav,
}

impl Component for Task {
    type Message = TaskMsg;
    type Properties = ();

    fn create(_: Self::Properties, mut link: ComponentLink<Self>) -> Self {
        let inst = Task {
            status: 0,
            fav: false,
            info: None,
        };

        spawn_local(async move {
            loop {
                Delay::new(Duration::from_secs(2)).await.unwrap();
                let info = connector::fetchit().await.unwrap();
                if let Some(info) = info {
                    link.send_self(TaskMsg::Fetchit(info));
                }
            }
        });
        inst
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Self::Message::Fetchit(info) => {
                self.info = Some(info);
                true
            }
            Self::Message::ToggleFav => {
                self.fav = !self.fav;
                true
            }
        }
    }

    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        true
    }
}

impl Renderable<Task> for Task {
    fn view(&self) -> Html<Self> {
        html! {
            <div class="box">
                <p class="heading">
                { self.info.as_ref().map(|val| val.clone().quote.author).unwrap_or("fetching --->".to_string())}
                </p>
                <div class="container">
                    { self.info.as_ref().map(|val| val.quote.body.clone() ).unwrap_or("".to_string())}
                </div>
            </div>
        }
    }
}

pub enum Msg {
    DoIt,
}

impl Component for Model {
    // Some details omitted. Explore the examples to see more.

    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, _: ComponentLink<Self>) -> Self {
        Model { tasks: Vec::new() }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::DoIt => {
                let task = Task {
                    fav: false,
                    info: None,
                    status: 0,
                };
                self.tasks.push(task);
                true
            }
        }
    }

    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        true
    }
}

impl Renderable<Model> for Model {
    fn view(&self) -> Html<Self> {
        html! {
            <div class="card">
                <div class="card-content">
                    <div class="box">
                        <div class="container has-text-centered">
                            <div class="button" onclick=|_| Msg::DoIt>
                            { "fetch quotes of the day" }
                            </div>
                        </div>
                        { for self.tasks.iter().map(|tsk| html! {
                            <Task />
                        })}
                    </div>
                </div>
            </div>
        }
    }
}
