use std::collections::HashMap;

use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use crate::{
    backups::Backups,
    configs::Configs,
    infra::{alert, window},
    invoke::{invokem, Invoke, UpdateStockArgs},
    mainpage::MainPage,
    reducer::{Actions, ApplicationState, Pages},
    stock::Stock,
    stock_state::{StockActions, StockState},
};

#[function_component(App)]
pub fn app() -> Html {
    let stock_state = use_reducer(StockState::default);
    let isloading = use_state(|| true);
    let state = use_reducer(ApplicationState::default);
    use_effect_with_deps(
        |(value, isloading)| {
            let value = value.clone();
            if !isloading {
                spawn_local(async move {
                    let stock = value.iter().map(|(k, v)| (k.clone(), *v)).collect();
                    if let Err(err) =
                        invokem!(Invoke::UpdateStock(UpdateStockArgs { stock }), ())
                    {
                        alert!( window() => "Falha ao atualizar estoque `{}`", err);
                    };
                })
            }
        },
        (stock_state.stock.clone(), *isloading),
    );
    {
        let statea = state.clone();
        let isloading = isloading.clone();
        let stock_state = stock_state.clone();
        use_effect_with_deps(
            |_| {
                spawn_local(async move {
                    let win = window();
                    let res = match invokem!(Invoke::File, HashMap<String, (f32, Option<String>)>)
                    {
                        Ok(a) => a,
                        Err(err) => {
                            alert!(win => "Falha ao ler produtos `{}`", err);
                            return;
                        }
                    };
                    statea.dispatch(Actions::SetProducts(res));
                    let groups = match invokem!(Invoke::GetGroups, HashMap<String, Vec<String>>)
                    {
                        Ok(a) => a,
                        Err(err) => {
                            alert!(win => "Falha ao ler grupos `{}`", err);
                            return;
                        }
                    };
                    statea.dispatch(Actions::SetGroups(groups));

                    let hash = match invokem!(Invoke::ReadStock, HashMap<String, u64>) {
                        Ok(a) => a,
                        Err(err) => {
                            alert!( win => "Falha ao ler estoque `{}`", err);
                            return;
                        }
                    };
                    stock_state.dispatch(StockActions::Set(hash));

                    isloading.set(false);
                })
            },
            (),
        );
    }
    match state.pages {
        Pages::Main => {
            html! {<MainPage state={state} stock_state={stock_state.dispatcher()} />}
        }
        Pages::Configs => {
            html! { <Configs state={state} />}
        }
        Pages::Backups => {
            html! { <Backups state={state} />}
        }
        Pages::Stock => {
            html! {
                html! { <Stock state={state} stock_state={stock_state}/>}
            }
        }
    }
}
