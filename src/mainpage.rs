use shared::payment::Payment;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::{ HtmlInputElement, Element};
use yew::{html::onchange::Event, prelude::*};

use crate::components::input_auto::InputAuto;
use crate::components::select::Select;
use crate::infra::{document, get, alert, window, scroll_into_view, log};
use crate::reducer::ApplicationState;
use crate::stock_state::{StockState, StockActions};
use crate::{
    header::Header,
    invoke::{invokem, Invoke, SaveArgs},
    reducer::Actions,
};
use shared::comand::{BackendInput, Comand, Input};
#[derive(Properties, PartialEq)]
pub struct Props {
    pub state: UseReducerHandle<ApplicationState>,
    pub stock_state: UseReducerDispatcher<StockState>
}

#[function_component]
pub fn MainPage(Props { state, stock_state }: &Props) -> Html {
    let exchange = use_state(|| 0.0);
    if let Some(selected) = state.selected_comand.clone() {
        let (inputs, payment) = match state.comands.get(&selected).cloned() {
            Some(a) => a,
            None => {
                return html! {
                    <main>
                        <Header state={state.clone()} />
                        <div class="main comandas">
                        </div>
                    </main>
                }
            }
        };


        let change_comand = {
            let state = state.clone();
            Callback::from(move |new_name: String| {
                state
                    .dispatch(Actions::ChangeSelectedComand(new_name))
            })
        };

        // FINSH COMAND
        let onfinalize = {
            let state = state.clone();
            let selected = selected.clone();
            let stock_state = stock_state.clone();
            
            Callback::from(move |_: yew::html::onclick::Event| {
                // FORCE A PAYMENT SELECTED
                if payment == Payment::NotSelected {
                    alert!(window() => "Selecione um metodo de pagamento");
                    return
                }
                let state = state.clone();
                let selected = selected.clone();
                let stock_state = stock_state.clone();
                spawn_local(async move {
                    let win = window();

                    // GET SELECTED COMAND
                    let Some(mut values) =
                        state.comands.get(&selected).cloned() else {
                            return
                        };
                    log!("{:?}", values);

                    let mut error = String::new();

                    let validated = values.0.iter().fold(true, |prev, input| {
                        let Some((_, group_name)) = state.products.get(input.name.trim()) else {
                            return prev
                        };

                        if group_name.is_none() {
                            return prev
                        }

                        let Some(valid_groups) = state.groups.get(group_name.as_ref().unwrap()) else {
                            return prev
                        };

                        if let Some(a) = &&input.group {
                            if valid_groups.contains(a) {
                                
                                return prev
                            }
                        }
                        error = format!("Escolha um valor no grupo de '{}'", input.name); 
                        false
                    });

                    if !validated {
                        alert!(win => "{}", error);
                    }

                    //CONVERT THE COMAND FOR BACKEND VALUES
                    let values: Vec<BackendInput> = values.0
                        .iter_mut()
                        .flat_map(|i| {

                            let a = state.products.get(&i.name)?;
                            Some(BackendInput {
                                name:  if let Some(group) = &i.group {
                                    if a.1.is_some() {
                                        format!("{}({})", i.name, group.trim())
                                    } else {
                                        i.name.clone()
                                    }
                                } else {
                                    i.name.clone()
                                },
                                quantity: i.quantity,
                                unit_price: a.0,
                            })
                        })
                        .collect();

                    let stock_values: Vec<(String, i64)> = values.iter().map(|a| (a.name.clone(), a.quantity as i64)).collect();
                    
                    if values.is_empty() {
                        alert!(win => "Comanda vazia");
                        return;
                    }
                 

                    if !state.confirm_finalize {
                        state.dispatch(Actions::ConfirmFinalize);
                        return;
                    }

                    // CALL THE BACKEND TO SAVE THE COMAND
                    let comand = Comand {
                        comand_name: selected.clone(),
                        values,
                        payment_method: payment
                    };

                    match invokem!(
                        Invoke::SaveComand(SaveArgs { comand }),
                        ()
                    ) {
                        Ok(_) => {
                            state.dispatch(Actions::RemoveComand(
                                selected,
                            ))
                        }
                        Err(err) => {
                            alert!(win => "Falha ao salvar comanda '{}'", err);
                        }
                    };
                    stock_state.dispatch(StockActions::SubVec(stock_values));
                });
            })
        };


        // HANDLE PAYMENT CHANGE
        let payment_change = {
            let state = state.clone();
            let selected = selected.clone();
            Callback::from(move |new_payment: String| {
                state.dispatch(Actions::SetPayment(selected.clone(), new_payment.into()))
            }) 
        };

        let a = inputs
            .iter()
            .enumerate()
            .map(|(i, info)| {

                let onsubmit = {
                    let state = state.clone();
                    let selected = selected.clone();
                    Callback::from(move |ev: SubmitEvent| {
                        ev.prevent_default();

                        // IF IS LAST INPUT PUSH A NEW INPUT
                        if i + 1 == state.comands.get(&selected).unwrap().0.len() {
                            state.dispatch(Actions::PushInput(selected.clone(), Input::default()));
                        }
                        let docs = document();
                        let input: HtmlInputElement = get!(docs => "quantity{}", i)
                            .unwrap();
                        let group: HtmlInputElement = get!(docs => "group{}", i).unwrap();
                        let name: HtmlInputElement = get!(docs => "name{}", i).unwrap();
                        
                        if let Some(active) =
                            docs.active_element()
                        {
                            if active == group.clone().into() {
                                // GET THE PRODUCT BASED ON TYPED NAME
                                let Some(prod) = state.products.get(&name.value().trim().to_string()) else {
                                    return
                                };
                                if let Some(gp) = &prod.1 {
                                    // CHECK IF TYPED VALUE IS INSIDE ITS GROUP 
                                    let selected_group = group.value().trim().to_string();
                                    let Some(valid_list) = state.groups.get(gp) else {
                                        return
                                    };
                                    if valid_list.contains(&selected_group) {
                                        input.focus().unwrap();
                                        scroll_into_view(input);
                                    }
                                }
                            } else if active == input.clone().into() {
                                // FOCUS ON NEW LINE
                                let Some(new_line_input): Option<HtmlInputElement> = 
                                get! (docs => "name{}", i + 1) else {
                                    return
                                };
                                new_line_input.focus().unwrap();
                                scroll_into_view(new_line_input);
                            } else if active == name.clone().into() {
                                let Some(prod) = state.products.get(&name.value().trim().to_string()) else {
                                    return
                                };
                                // FOCUS ON GROUP IF IT HAS A GROUP
                                if prod.1.is_some() {
                                    group.focus().unwrap();
                                    scroll_into_view(group);
                                } else {
                                    input.focus().unwrap();
                                    scroll_into_view(input);
                                }
                            }
                        }
                    })
                };

                let onchange = {
                    let state = state.clone();
                    let selected = selected.clone();
                    let info_quantity = info.quantity;
                    let info_group = info.group.clone();

                    Callback::from(move |value: String| {
                        let new_input = Input {
                            name: value.trim().to_string(),
                            quantity: info_quantity,
                            group: info_group.clone()
                        };
                        state.dispatch(Actions::UpdateInput(selected.clone(), new_input, i));
                    })
                };
                let change_quantity = {
                    let state = state.clone();
                    let selected = selected.clone();
                    let info_name = info.name.clone();
                    let info_group = info.group.clone();
                    Callback::from(move |ev: Event| {
                        let target: HtmlInputElement =
                            ev.target().unwrap().unchecked_into();
                        let value = target.value();

                        let new_input = Input {
                            name: info_name.clone(),
                            quantity: match value.parse() {
                                Ok(a) => a,
                                Err(_) => 1,
                            },
                            group: info_group.clone()
                        };
                        state.dispatch(Actions::UpdateInput(selected.clone(), new_input, i));
                    })
                };

                let change_group = {
                    let info_name = info.name.clone();
                    let info_quantity = info.quantity;
                    let state = state.clone();
                    let selected = selected.clone();
                    Callback::from(move |value: String| {
                        let new_input = Input {
                            name: info_name.clone(),
                            quantity: info_quantity,
                            group: if value.trim().is_empty() {
                                None
                            } else {
                                Some(value)
                            }
                        };
                        state.dispatch(Actions::UpdateInput(selected.clone(), new_input, i));
                    })
                };

                let autocomplete_selected = {
                    let state = state.clone();
                    let selected = selected.clone();
                    Callback::from(move |a: yew::html::onclick::Event| {
                        let inputs = state.comands.get(&selected).unwrap();                        
                        let mut ins = inputs.0[i].clone();
                        ins.name = a.target_dyn_into::<Element>().unwrap().inner_html().trim().to_string();
                        state.dispatch(Actions::UpdateInput(selected.clone(), ins, i));
                    })
                };
                let group_autocomplete = {
                    let state = state.clone();
                    let selected = selected.clone();
                    Callback::from(move |a: yew::html::onclick::Event| {
                        let inputs = state.comands.get(&selected).unwrap();                        
                        let mut ins = inputs.0[i].clone();
                        ins.group = Some(a.target_dyn_into::<Element>().unwrap().inner_html().trim().to_string());
                        state.dispatch(Actions::UpdateInput(selected.clone(), ins, i));
                    })
                };

                let mut hidden = true;
                
                let mut group_name = None::<String>;
                
                let price = match state.products.get(&info.name.trim().to_ascii_lowercase()) {
                    Some(a) => {
                        hidden = a.1.is_none();
                        group_name = a.1.clone();
                        a.0 * info.quantity as f32},
                    None => 0.0,
                };
                
                html! {
                    <li class="comand_item" key={i} {onsubmit}>
                        <form>
                            <div class="name">
                                <InputAuto 
                                onselect={autocomplete_selected} 
                                callback={onchange} 
                                id={format!("name{i}")} 
                                ty="text" 
                                value={info.name.trim().to_string()} 
                                class={classes!("name", if price == 0.0 && !info.name.is_empty() {
                                        "invalid"
                                    } else { "" }, if hidden {""} else {"include"})} 
                                complete={state.products.iter().map(|(k,_)| {
                                    k.clone()
                                }).collect::<Vec<String>>()} />
                                <InputAuto 
                                onselect={group_autocomplete} 
                                callback={change_group} 
                                id={format!("group{i}")} 
                                value={if let Some(group) = info.group.clone() {
                                    group
                                } else {
                                    "".to_string()
                                }} 
                                class={classes!(if hidden {
                                    "hidden"
                                } else {
                                    ""
                                }, "group")}
                                complete={if let Some(group_name) = group_name {
                                    match state.groups.get(&group_name){
                                        Some(a) => a.clone(),
                                        None => Vec::new()
                                    }
                                } else {
                                    Vec::new()
                                }}/>
                            </div>
                            
                            <input class="quantity" id={format!("quantity{i}")} value={info.quantity.to_string()} onchange={change_quantity} type="number" />
                            <button type="submit"> {"Novo"} </button>
                            <div class="total">
                            {"R$"}{format!("{price:.2}")}
                            </div>
                        </form>
                    </li>
                }
            })
            .collect::<Html>();

        let total = (*inputs).iter().fold(0.0, |a, b| {
            let price = match state.products.get(&b.name) {
                Some(a) => a.0 * b.quantity as f32,
                None => 0.0,
            };
            a + price as f64
        });

        let new_comand = {
            let onsubmit = {
                let state = state.clone();
                Callback::from(
                    move |ev: yew::html::onsubmit::Event| {
                        ev.prevent_default();
                        let docs = document();
                        let a: HtmlInputElement =get!(docs => "append_comand")
                            .unwrap();

                        let new_name = a.value();

                        if state
                            .comands
                            .get(&(new_name.clone()))
                            .is_none()
                        {
                            state.dispatch(Actions::PushComand(
                                new_name,
                            ));
                            a.set_value("");
                        } else {
                            a.set_value("");
                        }
                    },
                )
            };
            html! {
                <form class="inline_new_comand" {onsubmit}>
                    <input type="text" id="append_comand"/>
                    <button type="submit" > {"Nova comanda"} </button>
                </form>
            }
        };

        html! {
            <main>
                <Header state={state.clone()} />
                <div class="main comandas">
                    <div class="top_info">

                        <Select class="comand_option" callback ={change_comand} values={state.comands.iter().map(|(k, _)| {
                            k.clone()
                        }).collect::<Vec<String>>()} selected={selected}  />
                        {new_comand}
                    </div>
                    <div class="top_info">
                        <div class="name">
                            {"Nome"}
                        </div>
                        <div class="quant">
                            {"Qnts"}
                        </div>
                        <div class="total">
                            {"Total"}
                        </div>
                    </div>
                    <ul class="comand_items">
                        {a}
                    </ul>
                    <div class="total_comand">
                        <span>{"Total:"}</span>
                        <div>
                        {"R$"} {format!("{total:.2}")}
                        </div>
                    </div>
                    <div class="finalize">
                        <div>
                            <Select 
                                class="payment"
                                callback={payment_change} 
                                values={Payment::iter().map(|a| a.to_string()).collect::<Vec<String>>()} 
                                selected={payment.to_string()} />
                        </div>
                        <div>
                            <button onclick={onfinalize}>
                                if state.confirm_finalize {
                                    {"Confirmar"}
                                } else {
                                    {"Finalizar"}
                                }
                            </button>
                        </div>
                    </div>
                    if payment == Payment::Money {
                        <div class="exchange">
                            <form onsubmit={{
                                let exchange = exchange.clone();
                                Callback::from(move |ev: SubmitEvent| {
                                    ev.prevent_default();
                                    let docs = document();
                                    let input: HtmlInputElement = get!(docs => "payed_amount").unwrap();
                                    let Ok(payed) = input.value().parse::<f64>() else {
                                        return
                                    };
                                    exchange.set(payed); 
                                })
                            }}>
                                <span>
                                    {"Cliente pagou: "}
                                </span>
                                <input type="number" id="payed_amount" step="any" value={exchange.to_string()} />
                                <button type="submit">
                                    {"Calcular"}
                                </button>
                            </form>
                            <div>
                                if (total - *exchange) < 0.0 {
                                    {format!("Troco R${:.2}", (total - *exchange).abs())}
                                } else if (total - *exchange) > 0.0 {
                                    {format!("Cliente devendo R${:.2}", (total - *exchange).abs())}
                                } else {

                                }
                            </div>
                        </div>
                    }
                </div>
            </main>
        }
    } else {

        // CREATE A NEW COMAND
        let onfinalize = {
            let state = state.clone();
            Callback::from(move |ev: yew::html::onsubmit::Event| {
                ev.prevent_default();
                let docs =
                    document();
                let input: HtmlInputElement =
                    get!(docs => "new_comand").unwrap();
                let name = input.value();
                state.dispatch(Actions::PushComand(name));
            })
        };
        return html! {
            <main >
                <Header state={state.clone()} />
                <div class="main comandas">
                    <div class="wrapper">
                        <h2>
                            {"Nova comanda"}
                        </h2>
                        <form onsubmit={onfinalize} class="new_comand">
                            <input type="text" id="new_comand" />
                            <button type="submit" >
                                {"Criar"}
                            </button>
                        </form>
                    </div>
                </div>
            </main>
        };
    }
}
