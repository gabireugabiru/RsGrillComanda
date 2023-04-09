use std::collections::HashMap;

use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use crate::{
    configs::Configs,
    invoke::{invokem, Invoke},
    mainpage::MainPage,
    reducer::{Actions, ApplicationState, Pages},
};

#[function_component(App)]
pub fn app() -> Html {
    let state = use_reducer(ApplicationState::default);
    {
        let statea = state.clone();
        use_effect_with_deps(
            |_| {
                spawn_local(async move {
                    let res =
                        invokem!(Invoke::File, HashMap<String, (f32, Option<String>)>)
                            .unwrap();
                    statea.dispatch(Actions::SetProducts(res));
                    let groups = invokem!(Invoke::GetGroups, HashMap<String, Vec<String>>).unwrap();
                    statea.dispatch(Actions::SetGroups(groups));
                })
            },
            (),
        );
    }
    match state.pages {
        Pages::Main => {
            html! {<MainPage state={state} />}
        }
        Pages::Configs => {
            html! { <Configs state={state} />}
        }
    }
}
