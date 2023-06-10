use wasm_bindgen_futures::spawn_local;
use web_sys::MouseEvent;
use yew::{
    function_component, html, use_effect_with_deps, use_state, Callback, Html,
    Properties, UseReducerHandle, classes,
};

use crate::{
    header::Header,
    infra::{alert, window},
    invoke::{invokem, Invoke, MultiExportArgs, ReExportArgs},
    reducer::ApplicationState, sort_dates::FileDate,
};
#[derive(Properties, PartialEq)]
pub struct Props {
    pub state: UseReducerHandle<ApplicationState>,
}

#[function_component(Backups)]
pub fn backups(Props { state }: &Props) -> Html {
    let selected = use_state(Vec::<String>::new);
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
    let mut unknwon = Vec::new();
    let mut sorted_backups: Vec<(String, FileDate)> = backups.iter().flat_map(|a| {
        let mut a = a.clone();
        while a.chars().last() != Some('.') {
            a.pop();
        }
        a.pop();
        let result = FileDate::try_from(a.as_str());
        if result.is_err() {
            unknwon.push(a);
            Err(())
        } else {
            Ok((a, result.unwrap()))
        }
    }).collect();
    sorted_backups.sort_by(|a, b| a.1.cmp(&b.1));
    let list: Html = sorted_backups.clone()
        .iter()
        .enumerate()
        .map(|(i, (a, current))| {
            let onclick = {
                let backup = a.clone();
                Callback::from(move |_: MouseEvent| {
                    let backup = backup.clone();
                     spawn_local(async move {
                        if let Err(err) = invokem!(
                            Invoke::ReExport(ReExportArgs { backup: format!("{backup}.csv") }),
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
            let checked = selected.contains(&format!("{a}.csv"));
            let onselect ={
                let backup = a.clone();
                let selected = selected.clone();
                Callback::from(move |_: MouseEvent| {
                    if checked {
                        let new_selected: Vec<String> = selected.iter().filter(|a| a != &&format!("{backup}.csv")).cloned().collect();
                        selected.set(new_selected);
                    } else {
                        let mut new_selected: Vec<String> = (*selected).clone();
                        new_selected.push(format!("{backup}.csv"));
                        selected.set(new_selected);
                    }
                })
            };
            let is_first = i == 0;
            let is_change = if !is_first {
                sorted_backups[i - 1].1.month != current.month ||  sorted_backups[i - 1].1.year != current.year } else {
                    true
                };
            let select_month = {
                let sorted_backups = sorted_backups.clone();
                let selected = selected.clone();
                let current = current.clone();
                Callback::from(move |_: MouseEvent| {
                    let all: Vec<String> = sorted_backups.iter().filter(|a| {
                        a.1.month == current.month && a.1.year == current.year
                    }).map(|a| format!("{}.csv", a.0)).collect();
        
                    let mut new_selected: Vec<String> = (*selected).clone();
                    let are_all_int = all.iter().fold(true, |a, b| {
                        selected.contains(b) && a
                    });
                    if are_all_int {
                        new_selected = new_selected.iter().filter(|a| !all.contains(a)).cloned().collect();
                    } else {
                        for i in all {
                            new_selected.push(i);
                        }
                    }
                  
                    selected.set(new_selected);
                })
            };
            html! {
                <>
                    if is_change {
                        <div class="list_item"> 
                            <span>
                                {format!("{} de {}", current.get_month_string(), current.get_year_string())}
                            </span>
                            <div>
                                <button onclick={select_month} class="checkbox">
                                {"X"}
                                </button>
                            </div>
                        </div>
                    }
                    <div class="list_item lesser">
                        <span>
                            {a}
                        </span>
                        <div>
                            <button onclick={onselect} class={classes!(
                                if checked {
                                    "checked"
                                } else {
                                    ""
                                }, "checkbox"
                            )}>
                            if checked {
                                <svg viewBox="0 0 50 50" width="50px" height="50px"><path stroke="white" fill="white" d="M 41.9375 8.625 C 41.273438 8.648438 40.664063 9 40.3125 9.5625 L 21.5 38.34375 L 9.3125 27.8125 C 8.789063 27.269531 8.003906 27.066406 7.28125 27.292969 C 6.5625 27.515625 6.027344 28.125 5.902344 28.867188 C 5.777344 29.613281 6.078125 30.363281 6.6875 30.8125 L 20.625 42.875 C 21.0625 43.246094 21.640625 43.410156 22.207031 43.328125 C 22.777344 43.242188 23.28125 42.917969 23.59375 42.4375 L 43.6875 11.75 C 44.117188 11.121094 44.152344 10.308594 43.78125 9.644531 C 43.410156 8.984375 42.695313 8.589844 41.9375 8.625 Z"/></svg>
                            }
                            </button>
                            <button class="export" {onclick}>
                                {"Re-Exportar"}
                            </button>
                        </div>
                        
                    </div>
                </>
            }
        })
        .collect();
    let export_selected = {
        let selected = selected.clone();
        Callback::from(move |_: MouseEvent| {
            if selected.is_empty() {
                // CHANGE THE X I DONT KNOW WHAT WORD TO PUT IN YET
                alert!(window() => "Selecione pelo menos um X");
                return;
            }
            let selected = selected.clone();
            spawn_local(async move {
                if invokem!(
                    Invoke::MultiExport(MultiExportArgs {
                        backups: (*selected).clone()
                    }),
                    ()
                )
                .is_err()
                {
                    alert!(window() => "Falha ao exportar relátorio");
                } else {
                    selected.set(Vec::new());
                };
            });
        })
    };
    html! {
        <main>
            <Header state={state.clone()} />
            <div class="main backups">
                <h2 class="title">
                    {"Backups"}
                </h2>
                <div class="export_selected">
                    <button onclick={export_selected}>
                        {"Exportar selecionados "}{selected.len()}
                    </button>
                </div>
                {list}
            </div>
        </main>
    }
}
