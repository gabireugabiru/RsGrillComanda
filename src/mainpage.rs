use shared::payment::Payment;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::{console, HtmlInputElement, HtmlSelectElement, Element};
use yew::{html::onchange::Event, prelude::*};

use crate::components::input_auto::InputAuto;
use crate::infra::{document, get, alert, window, log, scroll_into_view};
use crate::reducer::ApplicationState;
use crate::{
    header::Header,
    invoke::{invokem, Invoke, SaveArgs},
    reducer::Actions,
};
use shared::comand::{BackendInput, Comand, Input};
#[derive(Properties, PartialEq)]
pub struct Props {
    pub state: UseReducerHandle<ApplicationState>,
}

#[function_component(MainPage)]
pub fn mainpage(Props { state }: &Props) -> Html {
    if let Some(selected) = state.selected_comand.clone() {
        let (inputs, payment) = match state.comands.get(&selected).cloned() {
            Some(a) => a,
            None => {
                return html! {
                    <main>
                        <Header state={state.clone()} />
                        <div class="main">
                        </div>
                    </main>
                }
            }
        };


        let change_comand = {
            let state = state.clone();
            Callback::from(move |ev: yew::html::onchange::Event| {
                let select: HtmlSelectElement = ev
                    .target_dyn_into()
                    .expect("this was supposed to be a input");
                let new_name = select.value();
                state
                    .dispatch(Actions::ChangeSelectedComand(new_name))
            })
        };

        let onfinalize = {
            let state = state.clone();
            let selected = selected.clone();
            Callback::from(move |_: yew::html::onclick::Event| {
                if payment == Payment::NotSelected {
                    alert!(window() => "Selecione um metodo de pagamento");
                    return
                }
                let state = state.clone();
                let selected = selected.clone();
          
                spawn_local(async move {
                    let values =
                        state.comands.get(&selected).cloned();

                    if let Some(mut values) = values {
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

                        let win = window();
                        if values.is_empty() {
                            alert!(win => "Comanda vazia");
                            return;
                        }

                        if !state.confirm_finalize {
                            state.dispatch(Actions::ConfirmFinalize);
                            return;
                        }

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
                                alert!(win => "Failed to save the comand");
                                console::error_1(
                                    &format!("{:?}", err).into(),
                                );
                            }
                        };
                
                    }
                });
            })
        };
        let payment_change = {
            let state = state.clone();
            let selected = selected.clone();
            Callback::from(move |_: yew::html::onchange::Event| {
                let docs = document();
                let select: HtmlSelectElement = get!(docs => "payment_method").unwrap();
                
                let new_payment: Payment = select.value().into();
                state.dispatch(Actions::SetPayment(selected.clone(), new_payment))
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
                        if i == state.comands.get(&selected).unwrap().0.len() - 1 {
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
                                
                                let Some(prod) = state.products.get(&name.value().trim().to_string()) else {
                                    return
                                };
                                if let Some(gr) = &prod.1 {
                                    let Some(valid_list) = state.groups.get(gr) else {
                                        return
                                    };
                                    
                                    let selected_group = group.value().trim().to_string();
                                    
                                    if valid_list.contains(&selected_group) {
                                        input.focus().unwrap();
                                        scroll_into_view(input);
                                    } else {
                                        let filtered: Vec<_> = valid_list.iter().filter(|a| a.contains(&selected_group)).collect();
                                        if filtered.len() == 1{
                                            let new_val = filtered.first().unwrap();
                                            
                                            let inputs = state.comands.get(&selected);
                                            if let Some(t) = inputs {
                                                let mut ins = t.0[i].clone();
                                                ins.group = Some(new_val.to_string()); 
                                                state.dispatch(Actions::UpdateInput(selected.clone(), ins, i))
                                            }

                                        }
                                    }
                                }
                            } else if active == input.clone().into() {
                                let Some(new_line_input): 
                                Option<HtmlInputElement> = get! {
                                    docs => "name{}", i + 1
                                } else {
                                    return
                                };
                                new_line_input.focus().unwrap();
                                scroll_into_view(new_line_input);
                            } else {
                                let prod = match state.products.get(&name.value().trim().to_string()) {
                                    Some(a) => Some(a),
                                    None => {
                                        let vec: Vec<_> = state.products.iter().map(|(k, _)| k).collect();
                                        let filtered = if name.value().to_string().is_empty() {
                                            Vec::new()
                                        } else {
                                            vec
                                                .iter()
                                                .filter(|a| a.contains(&name.value().trim().to_string()))
                                                .cloned()
                                                .collect()
                                        };
                                        if filtered.len() == 1 {
                                            let a = filtered.first().unwrap();
                                            let inputs = state.comands.get(&selected);
                                            if let Some(t) = inputs {
                                                let mut ins = t.0[i].clone();
                                                ins.name = a.to_string(); 
                                                state.dispatch(Actions::UpdateInput(selected.clone(), ins, i))
                                            }
                                        } 
                                        return 
                                    }
                                };
                                if let Some(prod) = prod {
                                    if prod.1.is_some() {
                                        group.focus().unwrap();
                                        scroll_into_view(group);
                                    } else {
                                        input.focus().unwrap();
                                        scroll_into_view(input);
                                    }
                                }
                            }
                        };
                    })
                };

                let onchange = {
                    let state = state.clone();
                    let selected = selected.clone();
                    let info_quantity = info.quantity;
                    let info_group = info.group.clone();

                    Callback::from(move |ev: Event| {
                        let target: HtmlInputElement =
                            ev.target().unwrap().unchecked_into();
                        let value = target.value();

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
                    Callback::from(move |ev: yew::html::onchange::Event| {
                        let value = ev.target_dyn_into::<HtmlInputElement>().unwrap().value();
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
                        log!("teste");
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
                        log!("teste");
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
                            <button type="submit" style="display: none;"> {"Adicionar"} </button>
                            <div class="total">
                            {"R$"}{format!("{price:.2}")}
                            </div>
                        </form>
                    </li>
                }
            })
            .collect::<Html>();

        let select = state.comands.iter().map(|(k, _)| {
            let k = k.to_string();
            html! {
                <option key={k.clone()} value={k.clone()} selected={k == selected}>{k}</option>
            }
        }).collect::<Html>();
        let select_payment: Html = Payment::iter().map(|a| {
            html! {
                <option key={a.to_string()} value={a.to_string()} selected={payment == a}>{match a {
                        Payment::Credit => "Crédito",
                        Payment::Debit => "Débito",
                        Payment::Money => "Dinheiro",
                        Payment::Pix => "Pix",
                        _ => "Selectione o metodo de pagamento",
                    }}</option>
            }
        }).collect();
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
                <div class="main">
                    <div class="top_info">
                        <select value={selected} onchange={change_comand}>
                            {select}
                        </select>
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
             
                            <select id="payment_method" onchange={payment_change} value={payment.to_string()}>
                                {select_payment}
                            </select>

                            
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
                </div>
            </main>
        }
    } else {
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
                <div class="main">
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
