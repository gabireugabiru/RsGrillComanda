use wasm_bindgen::JsCast;
use web_sys::{HtmlInputElement, SubmitEvent};
use yew::{function_component, html, use_state, Callback, Html, Properties, UseReducerHandle};

use crate::{
    header::Header,
    infra::{document, get, alert, window},
    reducer::ApplicationState,
    stock_state::{StockActions, StockState},
};
#[derive(PartialEq, Properties)]
pub struct Props {
    pub state: UseReducerHandle<ApplicationState>,
    pub stock_state: UseReducerHandle<StockState>,
}
#[derive(PartialEq, Eq, Clone)]
pub enum Selected {
    Add(String),
    Change(String),
    None,
}
impl Selected {
    pub fn value(&self) -> Option<&String> {
        match self {
            Self::Add(a) => Some(a),
            Self::Change(a) => Some(a),
            Self::None => None
        }
    }
}
impl Default for Selected {
    fn default() -> Self {
        Self::None
    }
}

#[function_component(Stock)]
pub fn stock(props: &Props) -> Html {
    let create_name = use_state(String::new);
    let stock_state = &props.stock_state;
    let selected = use_state(Selected::default);

    let mut vec: Vec<(String, u64)> = stock_state
        .stock
        .iter()
        .map(|(k, v)| (k.clone(), *v))
        .collect();
    vec.sort_by(|(k, _), (k1, _)| k.cmp(k1));

    let onsubmit = {
        let stock_state = stock_state.clone();
        let create_name = create_name.clone();
        Callback::from(move |ev: yew::events::SubmitEvent| {
            ev.prevent_default();
            stock_state.dispatch(StockActions::Create((*create_name).clone(), 0));
        })
    };

    let onchange = {
        let create_name = create_name.clone();
        Callback::from(move |ev: yew::html::onchange::Event| {
            let target: HtmlInputElement = ev.target().unwrap().unchecked_into();
            let new_name = target.value();
            create_name.set(new_name);
        })
    };

    let list = vec
        .iter()
        .map(|a| {
            let add = {
                let selected = selected.clone();
                let current = a.0.clone();
                Callback::from(move |_: yew::MouseEvent| {
                    selected.set(Selected::Add(current.to_string()));
                })
            };
            let change = {
                let selected = selected.clone();
                let current = a.0.clone();
                Callback::from(move |_: yew::MouseEvent| {
                    selected.set(Selected::Change(current.to_string()));
                })
            };

            let s = selected.clone();
            let st = stock_state.clone();
            let create_submit = move |is_set: bool, id: String| {
                let selected = s.clone();
                let stock_state = st.clone();
                Callback::from(move |ev: SubmitEvent| {
                    ev.prevent_default();
                    let sel = match (*selected).clone() {
                        Selected::Add(a) => a,
                        Selected::Change(a) => a,
                        Selected::None => return
                    };
                    let input: HtmlInputElement =
                        get!(document() => "{}", id).unwrap();
                    let Ok(quantity) = input.value().parse::<i64>() else {
                        alert!(window() => "Numero de entrada inválido");
                        return 
                    };
                    stock_state.dispatch(if is_set {
                        StockActions::SetValue(sel, quantity.max(0) as u64)
                    } else {
                        StockActions::Add(sel, quantity)
                    });
                    selected.set(Selected::None);
                })
            };

            let form_html = match (*selected).clone() {
                Selected::Add(value) => {
                    if a.0 != value {
                        html! {}
                    } else {
                        let id = format!("add_{}", &value);
                        let onsubmit = create_submit(false, id.clone());
                        html! {
                            <div class="container">
                                <form {onsubmit} >
                                    <input class="default" id={id} type="number" />
                                    <button type="submit">
                                        {"Dar Entrada"}
                                    </button>
                                </form>
                            </div>
                        }
                    }
                }
                Selected::Change(value) => {
                    if a.0 != value {
                        html! {}
                    } else {
                        let id = format!("change_{}", value);

                        let onsubmit = create_submit(true, id.clone());

                        html! {
                            <div class="container">
                                <form {onsubmit} >
                                    <input class="default" min={0} id={id} type="number" />
                                    <button type="submit">
                                        {"Alterar"}
                                    </button>
                                </form>
                            </div>
                        }
                    }
                }
                _ => {
                    html! {}
                }
            };
            let sel = selected.value().cloned().unwrap_or_default();
            html! {
                <>
                    <div class={format!("container {}", if sel != a.0 {
                        ""
                    } else {
                        "less"
                    })}>
                        <div>
                            <span>
                                {&a.0}
                            </span>
                            <span>
                                {","} {a.1}
                            </span>
                        </div>
                        <div>
                            <button onclick={change}>
                                {"Alterar"}
                            </button>
                            <button onclick={add}>
                                {"Entrada"}
                            </button>
                        </div>
                    </div>
                    {form_html}
                </>
            }
        })
        .collect::<Html>();
    html! {
        <main>
        <Header state={props.state.clone()} />
            <div class="main stock">
                <h2 class="title">
                    {"teste"}
                </h2>
                <div class="container">
                    <form {onsubmit}>
                        <input {onchange} class="default" type="text" />
                        <button type="submit">
                            {"Criar"}
                        </button>
                    </form>
                </div>
                {list}
            </div>
        </main>
    }
}
