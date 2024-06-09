use delta_encoding::SimpleDirectDeltaEncoding;
use yew::prelude::*;
use wasm_bindgen::{JsCast, JsValue};

#[function_component]
fn App() -> Html {
    let active_tab = use_state(|| String::from("result"));
    let input_ref = use_node_ref();
    let current_input = use_state(String::new);
    let encoding_data_bytes = use_state(Vec::new);
    let current_diffs = use_state(Vec::new);
    let current_patch = use_state(Vec::new);

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
                //web_sys::console::log_1(&JsValue::from_str(&content));
                current_input.set(content);
            }
        })
    };

    let mut enc = SimpleDirectDeltaEncoding::new((*encoding_data_bytes).clone());
    //web_sys::console::log_1(&JsValue::from_str(&(format!("Data len: {}", enc.data.len()))));
    let patch = enc.patch((*current_input).as_bytes());
    if enc.data != (*encoding_data_bytes) {
        encoding_data_bytes.set(enc.data.clone());
        let diffs = if SimpleDirectDeltaEncoding::validate_patch_differences(&patch).is_ok() {
            SimpleDirectDeltaEncoding::get_differences(&patch) 
        } else { Vec::new() };
        current_diffs.set(diffs);
        current_patch.set(patch.clone());
    }

    html! {
        <div>
            <div class="top-bar">
                <div class="padded">
                    {"Simple direct delta encoding"}
                </div>
            </div>
            <div class="content">
                <div class="section">
                    <p>{"Previous data"}</p>
                    <div class="padded full">
                        { format!("{:?}", String::from_utf8(enc.data.clone()).expect("Failed to convert data to string")) }
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
                <div class="section">
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
                            html! { <div>{
                                current_diffs.iter().map(|diff| {
                                    html! { <div>
                                        { format!("{:?}", diff) }
                                    </div> }
                                }).collect::<Html>()
                            }</div> }
                        } else {
                            html! { <div>
                                { format!("{:?}", (*current_patch).clone()) }
                            </div> }
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
