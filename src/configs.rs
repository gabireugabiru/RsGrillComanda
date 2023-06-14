use web_sys::{HtmlInputElement, SubmitEvent, MouseEvent};
use yew::{
    function_component, html, use_state, Html, Properties,
    UseReducerHandle, Callback, platform::spawn_local, classes,
};

use crate::{header::Header, reducer::{ApplicationState, Actions}, invoke::{invokem, Invoke, UpdateArgs, SetGroupsArgs}, infra::{document, get, window, alert}};
#[derive(Properties, PartialEq)]
pub struct Props {
    pub state: UseReducerHandle<ApplicationState>,
}
#[function_component(Configs)]
pub fn configs(Props { state }: &Props) -> Html {
    let edit = use_state(String::default);
    let edit_group = use_state(String::default);
    
    let mut sorted: Vec<_> = state.products.iter().collect();
    sorted.sort_by(|a, b| a.0.cmp(b.0));



    let list: Html = sorted
        .iter()
        .cloned()
        .map(|(name, value)| {
            let onclick = {
                let edit = edit.clone();
                let name = name.clone();
                Callback::from(move |_: yew::html::onclick::Event| {
                    edit.set(name.clone())
                })
            };

            let onfinalize = {
                let state = state.clone();
                let name = name.clone();
                let edit = edit.clone();
                Callback::from(move |ev: yew::html::onsubmit::Event| {

                    ev.prevent_default();

                    let doc = document();

                    let name_input: HtmlInputElement = get! {
                        doc => "name-{}", &name 
                    }.unwrap();
                    let value_input: HtmlInputElement = get! {
                        doc => "price-{}", &name 
                    }.unwrap();

                    let group_input: HtmlInputElement = get!(
                        doc => "group-{}", &name
                    ).unwrap();

                    let new_name = name_input.value();
                    let Ok(new_price) = value_input.value().parse::<f32>() else {
                        return
                    };
                    let new_group = match group_input.value().trim() {
                        "" => None,
                        a => Some(a.to_string())
                    };
                    let mut products = state.products.clone();
                    products.remove(&name);
                    products.insert(new_name.trim().to_string(), (new_price, new_group));
                    
                    let edit = edit.clone();
                    let state = state.clone();
                    spawn_local(async move {
                        if let Err(err) = invokem!(Invoke::UpdateProducts(UpdateArgs {
                            products: products.iter().map(|(k, v)| {
                                (k.clone(), v.clone())
                            }).collect()
                        }), ()) {
                            alert!(window() => "Failed to update products {:?}", err);
                            return
                        };
                        edit.set("".to_string());
                        state.dispatch(Actions::SetProducts(products));
                    })
                   
                })
            };
            let onremove = {
                let name = name.clone();
                let state = state.clone();
                Callback::from(move |_ : yew::html::ondblclick::Event| {
                    let name = name.clone();
                    let mut hash = state.products.clone();
                    let state = state.clone();
                    spawn_local(async move {
                        let Ok(is_sure) = invokem!(Invoke::Confirm("Confirmação".to_string(), format!("Tem certeza que deseja remover `{name}`")), bool) else {
                            return
                        };
                        if !is_sure {
                            return
                        }
                        
                        hash.remove(&name);
                        if let Err(err) = invokem!(Invoke::UpdateProducts(UpdateArgs {
                            products: hash.iter().map(|(k, v)| {
                                (k.clone(), v.clone())
                            }).collect()
                        }), ()) {
                            alert!(window() => "Failed to update products {:?}", err);
                            return
                        };
                        state.dispatch(Actions::SetProducts(hash));
                    });


                })
            };
            html! {
                <li class="products" key={name.clone()}>
               
                   
                    if (*edit).clone() == name.clone() {
                        <form onsubmit={onfinalize}>
                            <input class="default" type="text" id={format!("name-{}", name.clone())} value={name.clone()} />
                            <input class="default" type="number" step="any" id={format!("price-{}", name.clone())} value={value.0.to_string()} />
                            <input class="default" type="text" class="group" id={format!("group-{}", name.clone())} value={if let Some(group) = value.1.clone() {
                                group
                            } else {
                                "".to_string()
                            }}  />
                            <button type="button" class="danger" onclick={onremove}>{"X"}</button>
                            <button type="submit"> {"confirmar"} </button>
                        </form> 
                    } else {
                        <div>
                            <input class="default" type="text"  disabled={true} value={name.clone()}/>
                            <input class="default" type="number"  disabled={true} step="any" value={value.0.to_string()} />
                            <input class="default" type="text" class="group" disabled={true} value={if let Some(group) = value.1.clone() {
                                group
                            } else {
                                "".to_string()
                            }}  />
                            <button type="button" {onclick}>
                                {"Editar"}
                            </button>
                        </div>
                    }
                </li>
            }
        })
        .collect();


    let groups: Html = state.groups.iter()
    .map(|(name, values)| {
        let actual = &*edit_group == name;
        let view = {
            let edit_group = edit_group.clone();
            let name = name.clone();
            Callback::from(move |_: yew::html::onclick::Event| {

                edit_group.set(if actual {
                    "".to_string()
                } else {
                    name.clone()
                });
            })
        };
        let remove = {
            let state = state.clone();
            let name = name.clone();
            Callback::from(move |_: yew::html::ondblclick::Event| {
                let mut new_groups = state.groups.clone();
                let name = name.clone();
                let state = state.clone();
                spawn_local(async move {
                    let Ok(is_sure) = invokem!(Invoke::Confirm("Confirmação".to_string(), format!("Tem certeza que deseja remover {name}")), bool) else {
                        return
                    };
                    if !is_sure {
                        return
                    }
                    
                    new_groups.remove(&name);
                    let Ok(_) = invokem!(Invoke::SetGroups(SetGroupsArgs { 
                        groups: new_groups.iter().map(|(k,v)| (k.clone(), v.clone())).collect()
                    }), ()) else {
                        return
                    };
                    state.dispatch(Actions::SetGroups(new_groups));
                });
            })
        };

        html! {
            <li key={name.clone()} class="groups">
            <div class="group">
                <div class="group_name">
                    <input class="default" type="text"  disabled={true} value={name.to_string()}  />
                    <button class="danger" onclick={remove}>
                        {"X"}
                    </button>
                    <button onclick={view} class={classes!(if actual {
                        "selected"
                    } else {
                        ""
                    })}>
                        if !actual {{"Ver"}} else {{"Fechar"}}
                    </button>
                </div>
                
                if *edit_group == name.to_string() {
                    <div class={"group_values"}>
                        {values.iter().enumerate().map(|(i, a)| {
                        let onsubmit = {
                            let state = state.clone();
                            let name = name.clone();
                            Callback::from(move |ev: SubmitEvent| {
                                ev.prevent_default();
                                let docs = document();
                                let name_input: HtmlInputElement = get!(docs => "group_{}_{}", name, i).unwrap();
                                let mut groups = state.groups.clone();
                                let a = groups.get_mut(&name).unwrap();
                                let real_value = name_input.value().trim().to_string(); 
                                if a.contains(&real_value) {
                                    return
                                }
                                a[i] = real_value;
                                let state = state.clone();
                                let name = name.clone();
                                spawn_local(async move {
                                    let Ok(_) = invokem!(Invoke::SetGroups(SetGroupsArgs {
                                        groups:groups.iter().map(|(k,v)| {
                                            (k.clone(), v.clone())
                                        }).collect()
                                    }), ()) else {
                                        return
                                    };
                                    state.dispatch(Actions::SetGroups(groups));
                                    let Some(next): Option<HtmlInputElement> = get!(docs => "group_{}_{}", name, i + 1) else {
                                        return
                                    };

                                    next.focus().unwrap();
                                });
                            })
                        };
                        html! {
                            
                            <form key={i} {onsubmit}>
                                <input class="default" type="text" id={format!("group_{name}_{i}")} value={a.to_string()} />
                                <button type="button" class="danger" onclick={{
                                    let state = state.clone();
                                    let name = name.clone();

                                    Callback::from(move |ev: MouseEvent| {
                                        ev.prevent_default();
                                        let docs = document();
                                        let name_input: HtmlInputElement = get!(docs => "group_{}_{}", name, i).unwrap();
                                        let mut groups = state.groups.clone();
                                        let state = state.clone();
                                        let name = name.clone();
                                        spawn_local(async move {

                                            let Ok(is_sure) = invokem!(Invoke::Confirm(
                                                "Confirmação".to_string(), 
                                                format!("Tem certeza que deseja remover `{}`", name_input.value()
                                            )), bool) else {
                                                return
                                            };
                                            if !is_sure {
                                                return
                                            }

                                            let a = groups.get_mut(&name).unwrap();
                                            *a = a.iter().filter(|a| a.as_str() != name_input.value().trim()).cloned().collect();
                                         


                                            let Ok(_) = invokem!(Invoke::SetGroups(SetGroupsArgs {
                                                groups:groups.iter().map(|(k,v)| {
                                                    (k.clone(), v.clone())
                                                }).collect()
                                            }), ()) else {
                                                return
                                            };
                                            state.dispatch(Actions::SetGroups(groups));
                                            let Some(next): Option<HtmlInputElement> = get!(docs => "group_{}_{}", name, i + 1) else {
                                                return
                                            };
        
                                            next.focus().unwrap();
                                        });
                                    })
                                }}>
                                    {"X"}
                                </button>
                                <button type="submit">
                                {"Salvar"}
                                </button>
                            </form>
                        }}).collect::<Html>()}
                        <form onsubmit={{
                            let state = state.clone();
                            let name = name.clone();
                            Callback::from(move |ev: SubmitEvent| {
                                ev.prevent_default();
                                let docs = document();
                                let name_input: HtmlInputElement = get!(docs => "new_value_{}", name).unwrap();
                                let mut groups = state.groups.clone();
                                let a = groups.get_mut(&name).unwrap();
                                let real_value = name_input.value().trim().to_string();
                                if a.contains(&real_value) {
                                    return
                                }
                                a.push(real_value);
                                let state = state.clone();
                                spawn_local(async move {
                                    let Ok(_) = invokem!(Invoke::SetGroups(SetGroupsArgs {
                                        groups:groups.iter().map(|(k,v)| {
                                            (k.clone(), v.clone())
                                        }).collect()
                                    }), ()) else {
                                        return
                                    };
                                    state.dispatch(Actions::SetGroups(groups));
                                    name_input.set_value("");
                                });
                            })
                        }}>
                            <input class="default" type="text" id={format!("new_value_{name}")} />
                            <button type="submit">
                                {"Adicionar"}
                            </button> 
                        </form>
                    </div>
                }
               
            </div>
            </li>
        }
    }).collect();

    let onfinalize = {
        let state = state.clone();
        Callback::from(move |ev : yew::html::onsubmit::Event| {
            ev.prevent_default();
            let doc = document();
            let name_input: HtmlInputElement = get!(doc => "new_product_name")
                .unwrap();
            let price_input: HtmlInputElement = get!(doc => "new_product_price")
                .unwrap();
            let group_input: HtmlInputElement = get!(doc => "new_product_group")
            .unwrap();

            let Some(active) = doc.active_element() else {
                return
            };

            if active == name_input.clone().into() {
                price_input.focus().unwrap();
                return 
            }

            let new_name = name_input.value();
            let Ok(new_price) = price_input.value().parse::<f32>() else {
                return
            };
            let new_group = match group_input.value().trim() {
                "" => None,
                a => Some(a.to_string())
            };
            let mut products = state.products.clone();
            products.insert(new_name.trim().to_string(), (new_price, new_group));
            
            let edit = edit.clone();
            let state = state.clone();
            spawn_local(async move {
                if let Err(err) = invokem!(Invoke::UpdateProducts(UpdateArgs {
                    products: products.iter().map(|(k, v)| {
                        (k.clone(), v.clone())
                    }).collect()
                }), ()) {
                    alert!(window() => "Failed to update products {:?}", err);
                    return
                };
                name_input.set_value("");
                price_input.set_value("");

                edit.set("".to_string());
                state.dispatch(Actions::SetProducts(products));
            })
        })
    };
    let new_group = {
        let state = state.clone();
        Callback::from(move |ev: SubmitEvent| {
            ev.prevent_default();
            let docs = document();
            let name: HtmlInputElement = get!(docs => "new_group_name").unwrap();
            let mut new_groups = state.groups.clone();
            new_groups.insert(name.value().trim().to_string(), Vec::new());

            let state = state.clone();
            spawn_local(async move {
                let Ok(_) = invokem!(Invoke::SetGroups(SetGroupsArgs { 
                    groups: new_groups.iter().map(|(k,v)| (k.clone(), v.clone())).collect()
                }), ()) else {
                    return
                };
                state.dispatch(Actions::SetGroups(new_groups));
                name.set_value("");
            });
        })
    };
    html! {
        <main>
            <Header state={state.clone()} />
            <div class="main configs">
                <ul>
                    <li class="products"> {"Novo Grupo"} </li>
                    <li class="groups">
                        <form class="new_group" onsubmit={new_group}>
                            <input class="default" type="text" id="new_group_name"/>
                            <button type="submit">
                                {"Adicionar"}
                            </button>
                        </form> 
                    </li>
                    <li class="products"> {"Lista Grupos"} </li>
                    {groups}
                    <li class="products">{"Novo Produto"}</li>
                    <li class="products new" >
                        <form onsubmit={onfinalize}>
                            <input class="default" type="text" id="new_product_name" />
                            <input class="default" type="number" step="any" id="new_product_price" />
                            <input class="default" type="text" class="group" id="new_product_group" />
                            <button type="submit">
                            {"Adicionar"}
                            </button>
                        </form>
                    </li>
                    <li class="products">{"Lista de Produtos"}</li>
                    {list}
                </ul>
            </div>
        </main>
    }
}
