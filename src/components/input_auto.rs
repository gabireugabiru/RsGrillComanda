use web_sys::{Element, HtmlInputElement};
use yew::{
    classes, function_component, html, use_state, AttrValue, Callback, Classes,
    Html, Properties, TargetCast,
};

use crate::infra::{document, get, log, scroll_into_view};

#[derive(PartialEq, Properties)]
pub struct Props {
    #[prop_or_default]
    pub callback: Callback<String>,
    #[prop_or_default]
    pub value: AttrValue,
    #[prop_or_default]
    pub complete: Vec<String>,
    #[prop_or_default]
    pub id: AttrValue,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub ty: AttrValue,
    // #[prop_or_default]
    pub onselect: Callback<yew::html::onclick::Event>,
}

#[function_component(InputAuto)]
pub fn input_auto(
    Props {
        callback,
        complete,
        value,
        class,
        id,
        ty,
        onselect,
    }: &Props,
) -> Html {
    let is_open = use_state(|| false);

    let selected = use_state(|| 0usize);
    let mut filterd_autocomplete: Vec<_> = if value.to_string().is_empty() {
        Vec::new()
    } else {
        complete
            .iter()
            .filter(|a| a.contains(&value.trim().to_string()))
            .cloned()
            .collect()
    };

    filterd_autocomplete.sort_by(|_, a| a.cmp(&value.to_string()));
    let real_selected = if !filterd_autocomplete.is_empty() {
        selected.min(filterd_autocomplete.len() - 1)
    } else {
        0
    };
    let height = std::cmp::min(25 + filterd_autocomplete.len() * 70, 300);
    let mut autocomplete = !filterd_autocomplete.is_empty();

    let keychange = {
        let filterd_autocomplete = filterd_autocomplete.clone();
        let callback = callback.clone();
        let is_open = is_open.clone();
        let selected = selected.clone();
        let id = id.clone();
        Callback::from(move |ev: yew::html::onkeyup::Event| {
            let element = ev.target_dyn_into::<HtmlInputElement>().unwrap();
            let mut value = element.value();
            let key = ev.key();

            // AUTOCOMPLETE WHEN ENTER
            if key == "Enter" && !filterd_autocomplete.is_empty() {
                value = filterd_autocomplete[real_selected].clone();
            }

            let docs = document();
            // CHANGE SELECTED AUTOCOMPLETE
            if key == "ArrowDown" {
                let new_selected =
                    (real_selected + 1).min(filterd_autocomplete.len() - 1);
                selected.set(new_selected);
                let a: Element =
                    get!(docs => "sugestions_{id}_{new_selected}").unwrap();
                scroll_into_view(a);
            }
            if key == "ArrowUp" {
                let new_selected = real_selected.max(1) - 1;
                selected.set(new_selected);
                ev.prevent_default();
                log!("{}", new_selected);
                let Some(a): Option<Element> =
                    get!(docs => "sugestions_{id}_{new_selected}")
                else {
                    return
                };
                scroll_into_view(a);
            }

            // PASS TO PARENT
            callback.emit(value.trim().to_string());
            if !*is_open {
                is_open.set(true);
            }
        })
    };

    if complete.contains(&value.trim().to_string()) {
        autocomplete = false
    }

    autocomplete = autocomplete && *is_open;
    let complete: Html = filterd_autocomplete
        .iter()
        .enumerate()
        .map(|(i, str)| {
            html! {
                <div id={format!("sugestions_{id}_{i}")} onclick={onselect} class={classes!(if real_selected == i {
                    "selected"
                } else { "" }, if i == 0 {
                    "first"
                } else {
                    ""
                })}>
                    {str}
                </div>
            }
        })
        .collect();
    let focus = {
        let is_open = is_open.clone();
        Callback::from(move |_: yew::html::onfocus::Event| {
            is_open.set(true);
        })
    };
    let onclose = {
        let is_open = is_open.clone();
        Callback::from(move |_: yew::html::onclick::Event| {
            is_open.set(false);
        })
    };
    let onfocus_out = {
        let is_open = is_open.clone();
        Callback::from(move |_: yew::html::onfocusout::Event| {
            let is_open = is_open.clone();
            if *is_open {
                gloo_timers::callback::Timeout::new(200, move || is_open.set(false))
                    .forget();
            }
        })
    };
    html! {
        <>
            if autocomplete {
                <div class="sugestions"  style={format!("height: {}px;bottom: -{}px;", height, height as i32 - 7)}>
                    <header onclick={onclose}>
                    <div >
                        {"X"}
                    </div>
                    </header>
                    {complete}
                </div>
            }
            <input onfocusout={onfocus_out} onfocus={focus} onkeyup={keychange} autocomplete="off" onkeydown={Callback::from(|ev: yew::html::onkeydown::Event| {
                let key = ev.key();
                if key == "ArrowDown" {
                    ev.prevent_default();
                }
                if key == "ArrowUp" {
                    ev.prevent_default();
                }
            })} type={ty} id={id} class={class.clone()} value={value} />
        </>
    }
}
