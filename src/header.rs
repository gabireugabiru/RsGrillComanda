use web_sys::console;
use yew::{
    classes, function_component, html, platform::spawn_local, use_state, Callback,
    Html, Properties, UseReducerHandle,
};

use crate::{
    infra::window,
    invoke::{invokem, Invoke},
    reducer::{Actions, ApplicationState, Pages},
};
#[derive(Properties, PartialEq)]
pub struct Props {
    pub state: UseReducerHandle<ApplicationState>,
}

#[function_component(Header)]
pub fn header(Props { state }: &Props) -> Html {
    let is_confirm = use_state(|| false);
    let onclick = {
        let is_confirm = is_confirm.clone();
        Callback::from(move |_: yew::html::onclick::Event| {
            let is_confirm = is_confirm.clone();
            if *is_confirm {
                spawn_local(async {
                    let window = window();
                    let Ok(is_cache_empty) =
                        invokem!(Invoke::IsCacheEmpty, bool) else {
                            window
                            .alert_with_message("Falha ao buscar comandas").unwrap();
                            return
                        };
                    if !is_cache_empty {
                        let Ok(_) = invokem!(Invoke::Export, ()) else {
                            console::error_1(&"Failed to export".into());
                            return
                        };
                    } else {
                        window
                            .alert_with_message(
                                "Não existe nenhuma comanda salva no momento",
                            )
                            .unwrap();
                    }
                });
                is_confirm.set(false);
            } else {
                is_confirm.set(true);
                let is_confirm = is_confirm.clone();
                let timeout = gloo_timers::callback::Timeout::new(2000, move || {
                    is_confirm.set(false)
                });
                timeout.forget();
            }
        })
    };
    let configs = {
        let state = state.clone();
        Callback::from(move |_: yew::html::onclick::Event| match state.pages {
            Pages::Configs => {}
            _ => state.dispatch(Actions::SetPage(Pages::Configs)),
        })
    };
    let main = {
        let state = state.clone();
        Callback::from(move |_: yew::html::onclick::Event| match state.pages {
            Pages::Main => {}
            _ => state.dispatch(Actions::SetPage(Pages::Main)),
        })
    };
    let backups = {
        let state = state.clone();
        Callback::from(move |_: yew::html::onclick::Event| match state.pages {
            Pages::Backups => {}
            _ => state.dispatch(Actions::SetPage(Pages::Backups)),
        })
    };
    let stock = {
        let state = state.clone();
        Callback::from(move |_: yew::html::onclick::Event| match state.pages {
            Pages::Stock => {}
            _ => state.dispatch(Actions::SetPage(Pages::Stock)),
        })
    };
    html! {
        <header class="main">
            <img src="/public/espetaria_light.png" />
            <div>
                <button onclick={stock} class={classes!(if state.pages == Pages::Stock {
                    "selected"
                } else {""})}>
                    {"Estoque"}
                </button>
                <button onclick={backups} class={classes!(if state.pages == Pages::Backups {
                    "selected"
                } else {""})}>
                    {"Backups"}
                </button>
                <button onclick={main} class={classes!(if state.pages == Pages::Main {
                    "selected"
                } else {""})}>
                    {"Comandas"}
                </button>
                <button onclick={configs} class={classes!(if state.pages == Pages::Configs {
                    "selected"
                } else {""})}>
                    {"Configurações"}
                </button>

                <button {onclick} >

                    if !(*is_confirm) {
                        {"Exportar"}
                    } else {
                        {"Confirmar"}
                    }
                </button>
            </div>
        </header>
    }
}
