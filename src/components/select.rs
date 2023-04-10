// pub fn

use web_sys::MouseEvent;
use yew::{
    classes, function_component, html, use_state, AttrValue,
    Callback, Html, Properties,
};
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
        <div class={classes!(class.to_string(), "select_component")}>
            <button class="top" onclick={{
                let is_open = is_open.clone();
                Callback::from(move |_: MouseEvent| {
                    is_open.set(!*is_open);
                })
            }}>
                {selected}
            </button>

            if *is_open {
                <div>
                    {list}
                </div>
            }
        </div>
    }
}
