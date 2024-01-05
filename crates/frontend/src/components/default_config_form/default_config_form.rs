use leptos::*;
use serde_json::{json, Value};
use web_sys::MouseEvent;

use crate::{components::button::button::Button, utils::parse_string_to_json_value_vec};

use super::{types::DefaultConfigCreateReq, utils::create_default_config};

#[component]
pub fn default_config_form<NF>(
    #[prop(default = false)] edit: bool,
    #[prop(default = String::new())] config_key: String,
    #[prop(default = String::new())] config_type: String,
    #[prop(default = String::new())] config_pattern: String,
    #[prop(default = String::new())] config_value: String,
    handle_submit: NF,
) -> impl IntoView
where
    NF: Fn() + 'static + Clone,
{
    let tenant_rs = use_context::<ReadSignal<String>>().unwrap();

    let (config_key, set_config_key) = create_signal(config_key);
    let (config_type, set_config_type) = create_signal(config_type);
    let (config_pattern, set_config_pattern) = create_signal(config_pattern);
    let (config_value, set_config_value) = create_signal(config_value);

    let (show_labels, set_show_labels) = create_signal(edit);

    let (error_message, set_error_message) = create_signal("".to_string());

    let on_submit = move |ev: MouseEvent| {
        ev.prevent_default();
        let f_name = config_key.get();
        let f_type = config_type.get();
        let f_pattern = config_pattern.get();
        let f_value = config_value.get();

        let f_value = match f_type.as_str() {
            "number" => Value::Number(f_value.parse::<i32>().unwrap().into()),
            _ => Value::String(f_value),
        };

        let f_schema = match f_type.as_str() {
            "number" => {
                json!({
                    "type": f_type.to_string(),
                })
            }
            "enum" => {
                json!({
                    "type": "string",
                    "enum": parse_string_to_json_value_vec(f_pattern.as_str())
                })
            }
            "pattern" => {
                json!({
                    "type": "string",
                    "pattern": f_pattern.to_string()
                })
            }
            _ => {
                json!(f_pattern.to_string())
            }
        };

        let payload = DefaultConfigCreateReq {
            schema: f_schema,
            value: f_value,
        };

        let handle_submit_clone = handle_submit.clone();
        spawn_local({
            let handle_submit = handle_submit_clone;
            async move {
                let result = create_default_config(
                    f_name.clone(),
                    tenant_rs.get(),
                    payload.clone(),
                )
                .await;

                match result {
                    Ok(_) => {
                        handle_submit();
                    }
                    Err(e) => {
                        set_error_message.set(e);
                        // Handle error
                        // Consider logging or displaying the error
                    }
                }
            }
        });
    };
    view! {
        <form class="form-control w-full space-y-4 bg-white text-gray-700 font-mono">
            <div class="form-control">
                <label class="label font-mono">
                    <span class="label-text text-gray-700 font-mono">Key</span>
                </label>
                <input
                    disabled=edit
                    type="text"
                    placeholder="Key"
                    class="input input-bordered w-full bg-white text-gray-700 shadow-md"
                    value=config_key.get()
                    on:change=move |ev| {
                        let value = event_target_value(&ev);
                        set_config_key.set(value);
                    }
                />
            </div>

            <select
                name="schemaType[]"
                on:change=move |ev| {
                    set_show_labels.set(true);
                    match event_target_value(&ev).as_str() {
                        "number" => {
                            set_config_type.set("number".to_string());
                        }
                        "enum" => {
                            set_config_type.set("enum".to_string());
                            set_config_pattern.set(format!("{:?}", vec!["android", "web", "ios"]));
                        }
                        "pattern" => {
                            set_config_type.set("pattern".to_string());
                            set_config_pattern.set(".*".to_string());
                        }
                        _ => {
                            set_config_type.set("other".to_string());
                            set_config_pattern.set("".to_string());
                        }
                    };
                }

                class="select select-bordered"
            >
                <option disabled selected>
                    Set Schema
                </option>

                <option value="number" selected=move || { config_type.get() == "number".to_string() }>
                    "Number"
                </option>
                <option value="enum" selected=move || { config_type.get() == "enum".to_string() }>
                    "String (Enum)"
                </option>
                <option value="pattern" selected=move || { config_type.get() == "pattern".to_string() }>
                    "String (regex)"
                </option>
                <option value="other" selected=move || { config_type.get() == "other".to_string() }>
                    "Other"
                </option>
            </select>

            {move || {
                view! {
                    <Show when=move || (config_type.get() == "number")>
                        <div class="form-control">
                            <label class="label font-mono">
                                <span class="label-text text-gray-700 font-mono">Value</span>
                            </label>
                            <input
                                type="number"
                                placeholder="Value"
                                class="input input-bordered w-full bg-white text-gray-700 shadow-md"
                                value=config_value.get()
                                on:change=move |ev| {
                                    logging::log!(
                                        "{:?}", event_target_value(&ev)
                                    );
                                    set_config_value.set(event_target_value(&ev));
                                }
                            />
                        </div>
                    </Show>

                    <Show when=move || (show_labels.get() && (config_type.get() != "number"))>
                        <div class="form-control">
                            <label class="label font-mono">
                                <span class="label-text text-gray-700 font-mono">Value</span>
                            </label>
                            <input
                                type="text"
                                placeholder="Value"
                                class="input input-bordered w-full bg-white text-gray-700 shadow-md"
                                value=config_value.get()
                                on:change=move |ev| {
                                    logging::log!(
                                        "{:?}", event_target_value(&ev)
                                    );
                                    set_config_value.set(event_target_value(&ev));
                                }
                            />
                        </div>
                        <div class="form-control">
                            <label class="label font-mono">
                                <span class="label-text text-gray-700 font-mono">
                                    {config_type.get()}
                                </span>
                            </label>
                            <textarea
                                type="text"
                                class="input input-bordered w-full bg-white text-gray-700 shadow-md"
                                on:change=move |ev| {
                                    let value = event_target_value(&ev);
                                    logging::log!("{:?}", value);
                                    set_config_pattern.set(value);
                                }
                            >
                                {config_pattern.get()}
                            </textarea>

                        </div>
                    </Show>
                }
            }}

            <div class="form-control mt-6">
                <Button text="Submit".to_string() on_click=on_submit/>
            </div>

            {
                view! {
                    <div>
                        <p class="text-red-500">{move || error_message.get()}</p>
                    </div>
                }
            }

        </form>
    }
}