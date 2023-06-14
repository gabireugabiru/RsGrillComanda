// pub fn
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::MouseEvent;
use yew::{
    classes, function_component, html, use_state, AttrValue,
    Callback, Html, Properties, use_effect_with_deps,
};

use crate::infra::{window, log};
#[derive(Properties, PartialEq)]
pub struct Props {
    #[prop_or_default]
    pub class: AttrValue,
    pub values: Vec<String>,
    pub selected: String,
    pub callback: Callback<String>,
}

#[function_component(Select)]
pub fn select(
    Props {
        class,
        selected,
        values,
        callback
    }: &Props,
) -> Html {
    let is_open = use_state(|| false);


    use_effect_with_deps(|is_open| {
        let mut callback_listener = None;
        
        if **is_open {
            let is_open = is_open.clone();
            let callback = Callback::from(move |_: MouseEvent| {
                log!("Teste");
                is_open.set(false);
            });

            let listener =
                Closure::<dyn Fn(MouseEvent)>::wrap(
                    Box::new(move |e: MouseEvent| callback.emit(e))
                );

            window()
                .add_event_listener_with_callback(
                    "click",
                    listener.as_ref().unchecked_ref()
                )
                .unwrap();

            callback_listener = Some(listener);
        }

        move || {
            if let Some(callback) = callback_listener {
                drop(&callback);
                window().remove_event_listener_with_callback("click", callback.as_ref().unchecked_ref()).unwrap();
            }
        }
    }, is_open.clone());

    let list: Html = values
        .iter()
        .map(|a| {
            let onclick = {
                let a = a.clone();
                let is_open = is_open.clone();
                let callback = callback.clone();
                Callback::from(move |_: MouseEvent| {
                    callback.emit(a.to_string());    
                    is_open.set(false);
                })
            };
            html! {
                <div key={a.to_string()} {onclick} class={classes!("element", if a == selected {
                    "selected"
                } else {
                    ""
                })}>
                    {a}
                </div>
            }
        })
        .collect();

    html! {
        <div onclick={Callback::from(|ev: MouseEvent| {
            ev.stop_propagation();
        })} class={classes!(class.to_string(), "select_component")}>
            <button class="top" onclick={{
                let is_open = is_open.clone();
                Callback::from(move |_: MouseEvent| {
                    is_open.set(!*is_open);
                })
            }}>
                {selected}
            </button>

            if *is_open {
                <div >
                    {list}
                </div>
            }
        </div>
    }
}
