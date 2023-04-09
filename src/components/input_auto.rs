use web_sys::{Event, HtmlInputElement};
use yew::{
    function_component, html, use_state, AttrValue, Callback,
    Classes, Html, Properties, TargetCast,
};

#[derive(PartialEq, Properties)]
pub struct Props {
    #[prop_or_default]
    pub callback: Callback<yew::html::onchange::Event>,
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
    let is_open = use_state(|| true);

    let filterd_autocomplete: Vec<_> = if value.to_string().is_empty()
    {
        Vec::new()
    } else {
        complete
            .iter()
            .filter(|a| a.contains(&value.trim().to_string()))
            .cloned()
            .collect()
    };
    let height =
        std::cmp::min(25 + filterd_autocomplete.len() * 60, 300);
    let mut autocomplete = !filterd_autocomplete.is_empty();

    let keychange = {
        let complete = complete.clone();
        let filterd_autocomplete = filterd_autocomplete.clone();

        Callback::from(move |ev: yew::html::onkeyup::Event| {
            let element =
                ev.target_dyn_into::<HtmlInputElement>().unwrap();
            let value = element.value();
            let new_filtered: Vec<_> = complete
                .iter()
                .filter(|b| b.contains(&value))
                .cloned()
                .collect();
            if new_filtered != filterd_autocomplete {
                let Ok(event) = Event::new("change") else {
                    return
                };
                element.dispatch_event(&event).unwrap();
            }
        })
    };

    if filterd_autocomplete.len() == 1 {
        let first = filterd_autocomplete.first().unwrap();
        if first.trim() == value.trim() {
            autocomplete = false;
        }
    }

    autocomplete = autocomplete && *is_open;

    let complete: Html = filterd_autocomplete
        .iter()
        .map(|str| {
            html! {
                <div onclick={onselect}>
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
    html! {
        <>
            if autocomplete {
                <div class="sugestions"  style={format!("height: {}px;bottom: -{}px;", height, height as i32 - 7)}>
                    <header>
                    <span>
                    </span>
                    <span onclick={onclose}>
                        {"X"}
                    </span>
                    </header>
                    {complete}
                </div>
            }
            <input onchange={callback} onfocus={focus} onkeyup={keychange} type={ty} id={id} class={class.clone()} value={value} />
        </>
    }
}
