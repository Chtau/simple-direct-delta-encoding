use std::collections::BTreeMap;

use delta_encoding::{IndexedData, SimpleDirectDeltaEncoding};
use wasm_bindgen::JsCast;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Routable)]
enum Route {
    #[at("/")]
    GeneratePatch,
    #[at("/apply-patch")]
    ApplyPatch,
    #[not_found]
    #[at("/404")]
    NotFound,
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::GeneratePatch => html! { <GeneratePatch /> },
        Route::ApplyPatch => html! {
            <ApplyPatch />
        },
        Route::NotFound => html! { <h1>{ "404" }</h1> },
    }
}

#[function_component]
fn App() -> Html {
    let on_close = Callback::from(move |_| {
        // Execute JavaScript here
         
        let window = web_sys::window().expect("no global `window` exists");
        let document = window.document().expect("should have a document on window");
        let error_display = document.get_element_by_id("error-display").expect("should have #error-display on the page");
        let style = error_display.get_attribute("style").expect("could not get style");
        if style.contains("display: flex;") {
            let style = style.replace("display: flex;", "display: none;");
            error_display.set_attribute("style", &style).expect("could not set style");
        }
    });

    html! {
    <BrowserRouter>
        <div>
            <div id="error-display" style="z-index: 1000; position: fixed; bottom: 0; left: 0; width: 100%; height: 100px; background-color: rgba(0, 0, 0, 0.5); display: none; justify-content: center; align-items: center; overflow: auto;">
                <div style="position: absolute; top: 0; left: 0; width: 100%; height: 100%; text-align: center;">{ "Open the developer console for more details" }</div>
                <button onclick={on_close} style="position: absolute; top: 10px; right: 10px;">{ "Close" }</button>
                <div style="background-color: white; padding: 20px; border-radius: 5px; position: relative;">
                    <div id="error-display-text" style="color: red;">
                        
                    </div>
                </div>
            </div>
            <Topbar />
            <div class="content">
                <Switch<Route> render={switch} />
            </div>
        </div>
    </BrowserRouter>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}

#[function_component]
fn Topbar() -> Html {
    let nav = use_navigator().expect("Navigator not found");
    let location = use_location().expect("Location not found");

    let on_nav_generate_patch = {
        let nav = nav.clone();
        Callback::from(move |_| {
            nav.push(&Route::GeneratePatch);
        })
    };

    let on_nav_apply_patch = {
        let nav = nav.clone();
        Callback::from(move |_| {
            nav.push(&Route::ApplyPatch);
        })
    };

    let path = location.path();

    html! {
        <div class="top-bar">
            <div style="margin-right: 10px; margin-left: 10px;">
                {"Simple direct delta encoding"}
            </div>
            <div>
                <button disabled={path == "/"} onclick={on_nav_generate_patch}>{ "Generate Patch" }</button>
                <button disabled={path == "/apply-patch"} onclick={on_nav_apply_patch}>{ "Apply Patch" }</button>
            </div>
        </div>
    }
}

#[function_component]
fn GeneratePatch() -> Html {
    let clipboard = yew_hooks::use_clipboard();
    let active_tab = use_state(|| String::from("result"));
    let previous_data_ref = use_node_ref();
    let input_ref = use_node_ref();
    let current_input = use_state(String::new);
    let encoding_data_bytes: UseStateHandle<Vec<u8>> = use_state(Vec::new);
    let current_diffs = use_state(BTreeMap::new);
    let current_patch = use_state(Vec::new);
    let current_byte_size = use_state(|| 0);
    let samples: UseStateHandle<Vec<(&str, &str, &str, &str, usize)>> = use_state(|| {
        vec![
            ("0", "Select Sample", "", "", 0),
            ("1", "Test insert", "test", "test1", 0),
            ("2", "Test replace", "test123", "test321", 0),
            ("3", "Test remove", "test1test", "test", 0),
            (
                "4",
                "Json same size change",
                "{ \"name\": \"John\", \"age\": 30 }",
                "{ \"name\": \"Will\", \"age\": 21 }",
                1
            ),
            (
                "5",
                "Json porperty size changed",
                "{ \"name\": \"John\", \"age\": 30 }",
                "{ \"name\": \"Patrick\", \"age\": 9 }",
                1
            ),
            (
                "6",
                "Json porperty key name changed",
                "{ \"name\": \"John\", \"age\": 30 }",
                "{ \"firstname\": \"John\", \"age\": 30 }",
                1
            ),
        ]
    });
    let parser_types = use_state(|| {
        vec![
            ("0", "Plain"),
            ("1", "Json"),
        ]
    });
    let selected_parser_type = use_state(|| 0);

    let active_tab_signal = active_tab.clone();
    let switch_tab = Callback::from({
        move |tab: String| {
            active_tab_signal.set(tab);
        }
    });

    let on_generate_click = {
        let input_ref = input_ref.clone();
        let current_input = current_input.clone();
        Callback::from(move |_| {
            if let Some(input) = input_ref
                .get()
                .expect("Input element should exist")
                .dyn_ref::<web_sys::HtmlElement>()
            {
                let content = input.inner_text();
                current_input.set(content);
            }
        })
    };

    let on_copy_to_clipboard_click = {
        let clipboard = clipboard.clone();
        let current_patch = current_patch.clone();
        Callback::from(move |_| {
            let patch_str = format!("{:?}", (*current_patch).clone());
            clipboard.write_text(patch_str);
        })
    };

    let on_sample_select_change = {
        let samples = samples.clone();
        let previous_data_ref = previous_data_ref.clone();
        let input_ref = input_ref.clone();
        let selected_parser_type = selected_parser_type.clone();
        Callback::from(move |e: Event| {
            let target = e.target_dyn_into::<web_sys::HtmlSelectElement>().unwrap();
            let selected_value = target.value();
            if selected_value == "0" {
                return;
            }
            let sample = samples
                .iter()
                .find(|x| x.0 == selected_value)
                .expect("Sample not found");

            if let Some(input) = previous_data_ref
                .get()
                .expect("Input element should exist")
                .dyn_ref::<web_sys::HtmlElement>()
            {
                input.set_inner_text(sample.2);
            }

            if let Some(input) = input_ref
                .get()
                .expect("Input element should exist")
                .dyn_ref::<web_sys::HtmlElement>()
            {
                input.set_inner_text(sample.3);
            }
            selected_parser_type.set(sample.4);
        })
    };

    let on_parser_select_change = {
        let samples = samples.clone();
        let previous_data_ref = previous_data_ref.clone();
        let input_ref = input_ref.clone();
        Callback::from(move |e: Event| {
            let target = e.target_dyn_into::<web_sys::HtmlSelectElement>().unwrap();
            let selected_value = target.value();
            if selected_value == "0" {
                return;
            }
            let sample = samples
                .iter()
                .find(|x| x.0 == selected_value)
                .expect("Sample not found");

            if let Some(input) = previous_data_ref
                .get()
                .expect("Input element should exist")
                .dyn_ref::<web_sys::HtmlElement>()
            {
                input.set_inner_text(sample.2);
            }

            if let Some(input) = input_ref
                .get()
                .expect("Input element should exist")
                .dyn_ref::<web_sys::HtmlElement>()
            {
                input.set_inner_text(sample.3);
            }
        })
    };

    let prev_data_input = if let Some(input) = previous_data_ref.get() {
        if let Some(input) = input.dyn_ref::<web_sys::HtmlElement>() {
            let input = input.inner_text();
            if !input.is_empty() {
                input
            } else {
                "".to_owned()
            }
        } else {
            "".to_owned()
        }
    } else {
        "".to_owned()
    };

    let mapped_src: Vec<(String, IndexedData)> = if *selected_parser_type == 1 {
        let a = serde_json::from_str::<serde_json::Value>(&prev_data_input);
        if a.is_ok() {
            let mut properties_indexed: Vec<(String, IndexedData)> = Vec::new();
            if let Ok(serde_json::Value::Object(map)) = a {
                for (index, (key, value)) in map.iter().enumerate() {
                    properties_indexed.push((key.to_string(), IndexedData::new(index as u8, value.to_string().trim().as_bytes().to_vec())));
                }
            }
            web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(&format!("Previous: {:?}", &properties_indexed)));
            properties_indexed
        } else {
            [("".to_string(), IndexedData::new(0, vec![]))].to_vec()
        }
    } else {
        [("".to_string(), IndexedData::new(0, prev_data_input.as_bytes().to_vec()))].to_vec()
    };

    let enc_src = mapped_src.iter().map(|(_, indexed_data)| indexed_data.clone()).collect::<Vec<IndexedData>>();
    let mut enc = SimpleDirectDeltaEncoding::new(&enc_src);

    let keys = mapped_src.iter().map(|(key, i)| (key.clone(), i.index)).collect::<Vec<(String, u8)>>();
    if !keys.is_empty() {
        for (key, index) in keys {
            enc.change_index_mapping(index, key.as_bytes());
        }
        enc.apply_index_mappings();
    }

    let mapped_target: Vec<(String, IndexedData)> = if *selected_parser_type == 1 {
        let a = serde_json::from_str::<serde_json::Value>(&current_input);
        if a.is_ok() {
             let mut properties_indexed: Vec<(String, IndexedData)> = Vec::new();
            if let Ok(serde_json::Value::Object(map)) = a {
                for (index, (key, value)) in map.iter().enumerate() {
                    properties_indexed.push((key.to_string(), IndexedData::new(index as u8, value.to_string().trim().as_bytes().to_vec())));
                }
            }
            web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(&format!("Current: {:?}", &properties_indexed)));
            properties_indexed
        } else {
            [("".to_string(), IndexedData::new(0, vec![]))].to_vec()
        }
    } else {
        [("".to_string(), IndexedData::new(0, current_input.as_bytes().to_vec()))].to_vec()
    };

    // apply the target key mappings and then patch the the target source data
    let keys = mapped_target.iter().map(|(key, i)| (key.clone(), i.index)).collect::<Vec<(String, u8)>>();
    if !keys.is_empty() {
        for (key, index) in keys {
            enc.change_index_mapping(index, key.as_bytes());
        }
    }

    let enc_target = mapped_target.iter().map(|(_, indexed_data)| indexed_data.clone()).collect::<Vec<IndexedData>>();
    let patch = enc.patch(&enc_target);

    let enc_data = enc.get_state();

    if enc_data != *encoding_data_bytes {
        encoding_data_bytes.set(enc_data);

        let diffs = SimpleDirectDeltaEncoding::get_differences(&patch);
        web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(&format!("Diffs: {:?}", &diffs)));
        current_diffs.set(diffs);
        current_patch.set(patch.clone());
        current_byte_size
            .set(SimpleDirectDeltaEncoding::get_differences_bytes_with_crc(&patch).len());
    }

    html! {
         <>
            <div class="section section-3">
                <div class="padded full" style="flex-direction: column;">
                    <div style="height: 40px;">
                        <select onchange={on_sample_select_change}>
                            { for samples.iter().enumerate().map(|(index, sample)| html! {
                                <option value={sample.0} selected={index == 0}>{ &sample.1 }</option>
                            }) }
                        </select>
                        <select onchange={on_parser_select_change}>
                            { for parser_types.iter().enumerate().map(|(index, parser)| html! {
                                <option value={parser.0} selected={index == *selected_parser_type}>{ &parser.1 }</option>
                            }) }
                        </select>
                    </div>
                    <div>{"Previous data:"}</div>
                    <div ref={previous_data_ref} class="input-content full" contenteditable={"true"}>
                    </div>
                </div>
            </div>
            <div class="section section-3">
                <div class="padded full" style="flex-direction: column;">
                    <div style="height: 40px; align-self: end;">
                        <button onclick={on_generate_click} style="">{"Generate Patch"}</button>
                    </div>
                    <div>{"New data:"}</div>
                    <div ref={input_ref} class="input-content full" contenteditable={"true"}>
                    </div>
                </div>
            </div>
            <div class="section section-3">
                <div class="tabs">
                    <button onclick={{
                        let switch_tab = switch_tab.clone();
                        move|_|switch_tab.emit(String::from("result"))
                    }} class={if *active_tab == "result" {"active"} else {""}}>
                        {"Info"}
                    </button>
                    <button onclick={move|_|switch_tab.clone().emit(String::from("raw"))} class={if *active_tab == "raw" {"active"} else {""}}>
                        {"Raw Patch"}
                    </button>
                </div>
                <div class="padded full">
                    {if *active_tab == "result" {
                        html! {
                        <div style="width: 100%;">
                            <div>{format!("Total byte size (CRC + differences): {}", current_patch.len())}</div>
                            <div>{format!("Byte size difference only: {}", *current_byte_size)}</div>
                            <div>{format!("Plain byte size (raw input as UTF-8): {}", (*current_input).as_bytes().len())}</div>
                            <hr/>
                            <div>{format!("{} difference Tokens:", current_diffs.len())}</div>
                            {
                                current_diffs.iter().map(|diff| {
                                    html! { <div>
                                        { format!("{:?}", diff) }
                                    </div> }
                                }).collect::<Html>()
                            }
                        </div>
                        }
                    } else {
                        html! {
                        <div>
                            <button onclick={on_copy_to_clipboard_click} >
                                {"Copy to clipboard"}
                            </button>
                            <hr/>
                            { format!("{:?}", (*current_patch).clone()) }
                        </div> }
                    }}
                </div>
            </div>
        </>
    }
}

#[function_component]
fn ApplyPatch() -> Html {
    let samples: UseStateHandle<Vec<(&str, &str, &str, &str, usize)>> = use_state(|| {
        vec![
            ("0", "Select Sample", "", "", 0),
            (
                "1",
                "Test insert",
                "test",
                "[10, 50, 50, 53, 56, 54, 54, 50, 48, 56, 48, 118, 0, 6, 105, 58, 4, 45, 1, 49]",
                0,
            ),
            (
                "2",
                "Test replace",
                "test123",
                "[10, 49, 55, 49, 56, 53, 50, 48, 49, 54, 49, 118, 0, 8, 114, 58, 4, 45, 3, 51, 50, 49]",
                0,
            ),
            ("3", "Test remove", "test1test", "[10, 50, 56, 56, 57, 48, 48, 52, 52, 53, 50, 118, 0, 5, 100, 58, 4, 45, 5]", 0),
            (
                "4",
                "Json same size change",
                "{ \"name\": \"John\", \"age\": 30 }",
                "[10, 50, 54, 48, 48, 53, 49, 52, 53, 55, 55, 118, 0, 7, 114, 58, 0, 45, 2, 50, 49, 118, 1, 9, 114, 58, 1, 45, 4, 87, 105, 108, 108]",
                1,
            ),
            (
                "5",
                "Json porperty size changed",
                "{ \"name\": \"John\", \"age\": 30 }",
                "[10, 50, 54, 48, 48, 53, 49, 52, 53, 55, 55, 118, 0, 6, 114, 58, 0, 45, 1, 57, 5, 100, 58, 1, 45, 1, 118, 1, 10, 114, 58, 1, 45, 5, 80, 97, 116, 114, 105, 8, 105, 58, 6, 45, 3, 99, 107, 34]",
                1,
            ),
            (
                "6",
                "Json porperty key name changed",
                "{ \"name\": \"John\", \"age\": 30 }",
                "[10, 50, 54, 48, 48, 53, 49, 52, 53, 55, 55, 118, 1, 109, 21, 9, 114, 58, 0, 45, 4, 102, 105, 114, 115, 10, 105, 58, 4, 45, 5, 116, 110, 97, 109, 101]",
                1
            ),
        ]
    });
    let parser_types = use_state(|| {
        vec![
            ("0", "Plain"),
            ("1", "Json"),
        ]
    });
    let selected_parser_type = use_state(|| 0);

    let input_ref = use_node_ref();
    let source_input_ref = use_node_ref();
    let result_input_ref = use_node_ref();

    let on_sample_select_change = {
        let samples = samples.clone();
        let source_input_ref = source_input_ref.clone();
        let input_ref = input_ref.clone();
        let selected_parser_type = selected_parser_type.clone();
        Callback::from(move |e: Event| {
            let target = e.target_dyn_into::<web_sys::HtmlSelectElement>().unwrap();
            let selected_value = target.value();
            if selected_value == "0" {
                return;
            }
            let sample = samples
                .iter()
                .find(|x| x.0 == selected_value)
                .expect("Sample not found");

            if let Some(input) = source_input_ref
                .get()
                .expect("Input element should exist")
                .dyn_ref::<web_sys::HtmlElement>()
            {
                input.set_inner_text(sample.2);
            }

            if let Some(input) = input_ref
                .get()
                .expect("Input element should exist")
                .dyn_ref::<web_sys::HtmlElement>()
            {
                input.set_inner_text(sample.3);
            }
            selected_parser_type.set(sample.4);
        })
    };

    let on_parser_select_change = {
        let selected_parser_type = selected_parser_type.clone();
        Callback::from(move |e: Event| {
            let target = e.target_dyn_into::<web_sys::HtmlSelectElement>().unwrap();
            let selected_value = target.value();
            // parse as usize
            let selected_value = selected_value.parse::<usize>().unwrap();
            selected_parser_type.set(selected_value);
        })
    };

    let on_apply_click = {
        let source_input_ref = source_input_ref.clone();
        let input_ref = input_ref.clone();
        let result_input_ref = result_input_ref.clone();
        let selected_parser_type = selected_parser_type.clone();
        Callback::from(move |_| {
            if let Some(input) = input_ref
                .get()
                .expect("Input element should exist")
                .dyn_ref::<web_sys::HtmlElement>()
            {
                let source_element = source_input_ref.get().expect("Input element should exist");
                let source_element = source_element
                    .dyn_ref::<web_sys::HtmlElement>()
                    .expect("Input element should be HtmlElement");
                let source_content = source_element.inner_text();
                let patch_input = input.inner_text();

                let mut patch = Vec::new();

                // parse patch
                let patch_str = patch_input.trim();
                if patch_str.is_empty() {
                    if let Some(input) = result_input_ref
                        .get()
                        .expect("Input element should exist")
                        .dyn_ref::<web_sys::HtmlElement>()
                    {
                        input.set_inner_text("Patch is empty");
                    }
                    return;
                }
                let patch_str = patch_str.replace(['[', ']'], "");
                if !patch_str.contains(',') {
                    if let Some(input) = result_input_ref
                        .get()
                        .expect("Input element should exist")
                        .dyn_ref::<web_sys::HtmlElement>()
                    {
                        input.set_inner_text("Patch is not in correct format");
                    }
                    return;
                }
                for (index, str) in patch_str.split(',').enumerate() {
                    let s = str.trim();
                    if s.is_empty() {
                        if let Some(input) = result_input_ref
                            .get()
                            .expect("Input element should exist")
                            .dyn_ref::<web_sys::HtmlElement>()
                        {
                            input.set_inner_text(&format!(
                                "Patch has empty byte value. \nPosition: {}",
                                index
                            ));
                        }
                        return;
                    }
                    let s_r = s.parse::<u8>();
                    if let Err(e) = s_r {
                        if let Some(input) = result_input_ref
                            .get()
                            .expect("Input element should exist")
                            .dyn_ref::<web_sys::HtmlElement>()
                        {
                            input.set_inner_text(&format!("Patch has invalid byte value. \nValue: {:?}\nError: {:?} \nPosition: {}", s, e, index));
                        }
                        return;
                    }
                    patch.push(s_r.unwrap());
                }

                // apply patch
                // source should be handled based on the parser type (json, plain)
                web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(&format!("Parser type: {:?}", &selected_parser_type)));
                let mapped_src: Vec<(String, IndexedData)> = if *selected_parser_type == 1 {
                    let a = serde_json::from_str::<serde_json::Value>(&source_content);
                    if a.is_ok() {
                        let mut properties_indexed: Vec<(String, IndexedData)> = Vec::new();
                        if let Ok(serde_json::Value::Object(map)) = a {
                            for (index, (key, value)) in map.iter().enumerate() {
                                properties_indexed.push((key.to_string(), IndexedData::new(index as u8, value.to_string().trim().as_bytes().to_vec())));
                            }
                        }
                        web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(&format!("Previous: {:?}", &properties_indexed)));
                        properties_indexed
                    } else {
                        [("".to_string(), IndexedData::new(0, vec![]))].to_vec()
                    }
                } else {
                    [("".to_string(), IndexedData::new(0, source_content.as_bytes().to_vec()))].to_vec()
                };

                let enc_src = mapped_src.iter().map(|(_, indexed_data)| indexed_data.clone()).collect::<Vec<IndexedData>>();
                let mut enc = SimpleDirectDeltaEncoding::new(&enc_src);

                let keys = mapped_src.iter().map(|(key, i)| (key.clone(), i.index)).collect::<Vec<(String, u8)>>();
                if !keys.is_empty() {
                    for (key, index) in keys {
                        enc.change_index_mapping(index, key.as_bytes());
                    }
                    enc.apply_index_mappings();
                }

                // handle apply patch for different parser types
                let apply_result = enc.apply_patch(&patch);
                if let Err(err) = apply_result {
                    if let Some(input) = result_input_ref
                        .get()
                        .expect("Input element should exist")
                        .dyn_ref::<web_sys::HtmlElement>()
                    {
                        input.set_inner_text(
                            format!("Failed to apply patch.\nError: {:?}", err).as_str(),
                        );
                    }
                    return;
                }
                let patched = apply_result.expect("Failed to apply patch");

                if let Some(input) = result_input_ref
                    .get()
                    .expect("Input element should exist")
                    .dyn_ref::<web_sys::HtmlElement>()
                {
                    // for the parser type 1 we patch the properties of the json object
                    if *selected_parser_type == 1 {
                        let a = serde_json::from_str::<serde_json::Value>(&source_content);
                        if a.is_ok() {
                            let mut result = "{ ".to_string();
                            if let Ok(serde_json::Value::Object(map)) = a {
                                for (index, (key, value)) in map.iter().enumerate() {
                                    web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(&format!("Index: {:?} key: {:?} value: {:?} patched: {:?}", &index, key, value, patched)));
                                    let b = patched.iter().find(|i|i.index == index as u8);
                                    if let Some(b) = b {
                                        let name = if let Some(n_changed) = &b.map_name_changed {
                                            String::from_utf8(n_changed.to_owned()).expect("Failed to convert patched data to string")
                                        } else {
                                            key.to_string()
                                        };
                                        result.push_str(&format!("\"{}\": {}", name, String::from_utf8(SimpleDirectDeltaEncoding::fold_index_result(&[b.to_owned()])).expect("Failed to convert patched data to string")));
                                    } else {
                                        // no changes
                                        result.push_str(&format!("\"{}\": {}", key, value));
                                    }
                                    if index < map.len() - 1 {
                                        result.push_str(", ");
                                    }
                                }
                            }
                            input.set_inner_text(&(result + " }"));
                        } else {
                            input.set_inner_text("Invalid json object");
                        }
                    } else {
                        input.set_inner_text(
                            &String::from_utf8(SimpleDirectDeltaEncoding::fold_index_result(&patched))
                                .expect("Failed to convert patched data to string"),
                        );
                    }
                }
            }
        })
    };

    html! {
         <>
            <div class="section section-3">
                <div class="padded full" style="flex-direction: column;">
                    <div style="height: 40px;">
                        <select onchange={on_sample_select_change}>
                            { for samples.iter().enumerate().map(|(index, sample)| html! {
                                <option value={sample.0} selected={index == 0}>{ &sample.1 }</option>
                            }) }
                        </select>
                        <select onchange={on_parser_select_change}>
                            { for parser_types.iter().enumerate().map(|(index, parser)| html! {
                                <option value={parser.0} selected={index == *selected_parser_type}>{ &parser.1 }</option>
                            }) }
                        </select>
                    </div>
                    <div>{"Source data:"}</div>
                    <div ref={source_input_ref} class="input-content full" contenteditable={"true"}>
                    </div>
                </div>
            </div>
            <div class="section section-3">
                <div class="padded full" style="flex-direction: column;">
                    <div style="height: 40px;">
                         <button onclick={on_apply_click} style="position: absolute; right: 10px; top: 10px;">{"Apply"}</button>
                    </div>
                    <div>{"Raw Patch:"}</div>
                    <div ref={input_ref} class="input-content full" contenteditable={"true"}>
                    </div>
                </div>
            </div>
            <div class="section section-3">
                <div class="padded full" style="flex-direction: column;">
                    <div style="height: 40px;">

                    </div>
                    <div>{"Result data:"}</div>
                    <div ref={result_input_ref} class="input-content full" contenteditable={"true"}>
                    </div>
                </div>
            </div>
        </>
    }
}