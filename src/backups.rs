use wasm_bindgen_futures::spawn_local;
use web_sys::MouseEvent;
use yew::{
    function_component, html, use_effect_with_deps, use_state, Callback, Html,
    Properties, UseReducerHandle,
};

use crate::{
    header::Header,
    infra::{alert, window},
    invoke::{invokem, Invoke, ReExportArgs},
    reducer::ApplicationState,
};
#[derive(Properties, PartialEq)]
pub struct Props {
    pub state: UseReducerHandle<ApplicationState>,
}

#[function_component(Backups)]
pub fn backups(Props { state }: &Props) -> Html {
    let backups = use_state(Vec::<String>::new);
    {
        let backups = backups.clone();
        use_effect_with_deps(
            move |_| {
                let backups = backups.clone();
                spawn_local(async move {
                    let Ok(backs) = invokem!(Invoke::ListBakcups, Vec<String>) else {
                        let window = window();
                        alert!(window => "Não foi possivel encontrar backups");
                        return
                    };
                    backups.set(backs);
                })
            },
            (),
        );
    }

    let list: Html = backups
        .iter()
        .map(|a| {
            let onclick = {
                let backup = a.clone();
                Callback::from(move |_: MouseEvent| {
                    let backup = backup.clone();
                     spawn_local(async move {
                        if let Err(err) = invokem!(
                            Invoke::ReExport(ReExportArgs { backup: backup.clone() }),
                            ()
                        )
                        {
                            let window = window();
                            alert!(window => "Não foi possivel exportar '{}', {:?}", backup, err);
                            return;
                        }
                    });
                })
            };
            html! {
                <div class="list_item">
                    <span>
                        {a}
                    </span>
                    <button {onclick}>
                        {"Re-Exportar"}
                    </button>
                </div>
            }
        })
        .collect();
    html! {
        <main>
            <Header state={state.clone()} />
            <div class="main backups">
                <h2>
                    {"Backups"}
                </h2>
                {list}
            </div>
        </main>


    }
}
