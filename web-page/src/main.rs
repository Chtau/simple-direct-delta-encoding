use std::{borrow::BorrowMut, ops::Deref};

use delta_encoding::SimpleDirectDeltaEncoding;
use yew::prelude::*;
use wasm_bindgen::{JsCast, JsValue};

#[function_component]
fn App() -> Html {
    let active_tab = use_state(|| String::from("result"));
    let input_ref = use_node_ref();
    let current_input = use_state(String::new);
    //let encoding = use_state(|| SimpleDirectDeltaEncoding::new(Vec::new()));

    let active_tab_signal = active_tab.clone();
    let switch_tab = Callback::from({move |tab: String| {
        active_tab_signal.set(tab);
    }});

    let on_generate_click = {
        let input_ref = input_ref.clone();
        let current_input = current_input.clone();
        Callback::from(move |_| {
            if let Some(input) = input_ref.get().expect("Input element should exist").dyn_ref::<web_sys::HtmlElement>() {
                let content = input.inner_text();
                web_sys::console::log_1(&JsValue::from_str(&content));
                current_input.set(content);
            }
        })
    };

    html! {
        <div>
            <div class="top-bar">
                <div class="padded">
                    {"Simple direct delta encoding"}
                </div>
            </div>
            <div class="content">
                <div class="section">
                    <div class="padded full">
                        {"Previous data"}
                    </div>
                </div>
                <div class="section">
                    <button onclick={on_generate_click} style="position: absolute; right: 10px; top: 10px;">{"Generate Patch"}</button>
                    <div class="padded full">
                        <div ref={input_ref} class="input-content" contenteditable={"true"}>
                            {"Paste new data"}
                        </div>
                    </div>
                </div>
                <div class="section" style="flex-direction: column;">
                    <div class="tabs">
                        <button onclick={{
                            let switch_tab = switch_tab.clone();
                            move|_|switch_tab.emit(String::from("result"))
                        }} class={if *active_tab == "result" {"active"} else {""}}>
                            {"Result"}
                        </button>
                        <button onclick={move|_|switch_tab.clone().emit(String::from("raw"))} class={if *active_tab == "raw" {"active"} else {""}}>
                            {"Raw Patch"}
                        </button>
                    </div>
                    <div class="padded full">
                        {if *active_tab == "result" {
                            html! { <div>{"Result content goes here"}</div> }
                        } else {
                            html! { <div>{"Raw Patch content goes here"}</div> }
                        }}
                    </div>
                </div>
            </div>

        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
