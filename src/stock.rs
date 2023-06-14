use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::{HtmlInputElement, SubmitEvent, MouseEvent, Event, KeyboardEvent};
use yew::{function_component, html, use_state, Callback, Html, Properties, UseReducerHandle};

use crate::{
    header::Header,
    infra::{document, get, alert, window},
    reducer::ApplicationState,
    stock_state::{StockActions, StockState}, invoke::{invokem, Invoke}, components::select::Select,
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
    let filter = use_state(String::default);
    let mut vec: Vec<(String, u64)> = stock_state
        .stock
        .iter()
        .map(|(k, v)| (k.clone(), *v))
        .collect();
    vec.sort_by(|(k, _), (k1, _)| k.cmp(k1));
    if !(*filter).is_empty() {
        vec = vec.iter().filter(|a| a.0.contains(&(*filter))).cloned().collect();
    }
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

    let generate_products = {
        let state = props.state.clone();
        let stock_state = stock_state.clone();
        Callback::from(move |_: MouseEvent| {
            let vec = state.products.iter().flat_map(|(k, v)| if v.1.is_some() {
                None
            } else {
                Some(k.clone())
            }).collect();
            stock_state.dispatch(StockActions::CreateVec(vec));
        })
    };

    let generate_group = {
        let state = props.state.clone();
        let stock_state = stock_state.clone();
        Callback::from(move |group: String| {
            let Some(vec) = state.groups.get(&group) else {
                return
            };
            stock_state.dispatch(StockActions::CreateVec(vec.iter().map(|a| format!("({a})")).collect()))
        })
    };
    let keydown = {
        let filter =filter.clone();
        Callback::from(move |ev: KeyboardEvent| {
            let Some(target) = ev.target() else {
                return
            };
            let Ok(input) = target.dyn_into::<HtmlInputElement>() else {
                return
            };

            let value = input.value();
            filter.set(value);
        })
    };

    let list = vec
        .iter()
        .map(|a| {
            let add = {
                let selected = selected.clone();
                let current = a.0.clone();
                Callback::from(move |_: yew::MouseEvent| {
                    match &(*selected) {
                        Selected::Add(_) => selected.set(Selected::None),
                        _ => selected.set(Selected::Add(current.to_string()))
                    };
                })
            };
            let change = {
                let selected = selected.clone();
                let current = a.0.clone();
                Callback::from(move |_: yew::MouseEvent| {
                    match &(*selected) {
                        Selected::Change(_) => selected.set(Selected::None),
                        _ => selected.set(Selected::Change(current.to_string())) 
                    }
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

            let remove = {
                let stock_state = stock_state.clone();
                let current = a.0.clone();
                Callback::from(move |_: MouseEvent| {
                    let stock_state = stock_state.clone();
                    let current = current.clone();

                    spawn_local(async move {
                        let Ok(is_sure) = invokem!(Invoke::Confirm("Confirmação".to_string(), format!("Tem certeza que deseja remover `{current}`")), bool) else {
                            return
                        };
                        if is_sure {
                            stock_state.dispatch(StockActions::Remove(current.clone()));
                        }
                    });
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
                                <form class="extend" {onsubmit} >
                                    <input class="default extend" id={id} type="number" />
                                    <button class="default" type="submit">
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
                                <form class="extend" {onsubmit} >
                                    <input class="default extend" min={0} id={id} type="number" />
                                    <button class="default" type="submit">
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
            let is_open = sel == a.0;
            let is_change_open = (*selected) == Selected::Change(a.0.clone());
            let is_add_open = (*selected) == Selected::Add(a.0.clone());

            html! {
                <>
                    <div class={format!("container {}", if !is_open  {
                        ""
                    } else {
                        "less"
                    })}>
                        <div>
                            <span class="bold">
                                {a.1}{"x "}
                            </span>
                            <span>
                               {" "}{&a.0}
                            </span>
                        </div>
                        <div>
                            <button class={format!("default {}", if is_change_open {
                                "selected"
                            }else { "" } )} onclick={change}>
                                {"Alterar"}
                            </button>
                            <button class={format!("default {}",if is_add_open {
                                "selected"
                            } else {""} )} onclick={add}>
                                {"Entrada"}
                            </button>
                            <button class="danger" onclick={remove}>
                                {"X"}
                            </button>
                        </div>
                    </div>
                    {form_html}
                </>
            }
        })
        .collect::<Html>();
    let groups: Vec<String> = props.state.groups.keys().cloned().collect();
    html! {
        <main>
        <Header state={props.state.clone()} />
            <div class="main stock">
                <h2 class="title">
                    {"Estoque"}
                </h2>
                <div class="container">
                    <input type="text" placeholder="Pesquisar" class="default fill" onkeyup={keydown}/>
                </div>
                <div class="container">
                    <form {onsubmit}>
                        <input {onchange} class="default" type="text" />
                        <button class="default" type="submit">
                            {"Criar"}
                        </button>
                    </form>
                    <div>
                        <button onclick={generate_products}>
                            {"Gerar Produtos"}
                        </button>
                        <Select selected={"Gerar Grupo"} values={groups}  callback={generate_group}/>
                    </div>
                </div>
                {list}
            </div>
        </main>
    }
}
