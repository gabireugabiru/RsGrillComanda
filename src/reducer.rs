use std::collections::HashMap;

use shared::{comand::Input, payment::Payment};
use yew::Reducible;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Pages {
    Main,
    Configs,
    Backups,
    Stock,
}
impl Default for Pages {
    fn default() -> Self {
        Self::Main
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ApplicationState {
    pub products: HashMap<String, (f32, Option<String>)>,
    pub comands: HashMap<String, (Vec<Input>, Payment)>,
    pub selected_comand: Option<String>,
    pub confirm_finalize: bool,
    pub groups: HashMap<String, Vec<String>>,
    pub pages: Pages,
}
pub enum Actions {
    SetProducts(HashMap<String, (f32, Option<String>)>),
    PushInput(String, Input),
    UpdateInput(String, Input, usize),
    PushComand(String),
    ChangeSelectedComand(String),
    RemoveComand(String),
    ConfirmFinalize,
    SetPage(Pages),
    SetPayment(String, Payment),
    SetGroups(HashMap<String, Vec<String>>),
}

impl Reducible for ApplicationState {
    type Action = Actions;
    fn reduce(self: std::rc::Rc<Self>, action: Self::Action) -> std::rc::Rc<Self> {
        let mut new_state = (*self).clone();
        new_state.confirm_finalize = false;
        match action {
            Actions::SetProducts(products) => {
                new_state.products = products;
            }
            Actions::PushInput(comand, input) => {
                match new_state.comands.get_mut(&comand) {
                    Some(inputs) => inputs.0.push(input),
                    None => {
                        new_state
                            .comands
                            .insert(comand, (vec![input], Payment::default()));
                    }
                }
            }
            Actions::UpdateInput(comand, input, pos) => {
                if let Some(inputs) = new_state.comands.get_mut(&comand) {
                    inputs.0[pos] = input
                }
            }
            Actions::PushComand(mut name) => {
                name = name.trim().to_string();
                if !name.is_empty() {
                    if new_state.comands.get(&name).is_none() {
                        new_state.comands.insert(
                            name.clone(),
                            (vec![Input::default()], Payment::default()),
                        );
                        new_state.selected_comand = Some(name);
                    }
                }
            }
            Actions::ChangeSelectedComand(name) => {
                new_state.selected_comand = Some(name);
            }
            Actions::RemoveComand(name) => {
                new_state.comands.remove(&name);
                let a = new_state.comands.iter().next();
                if let Some((new, _)) = a {
                    new_state.selected_comand = Some(new.clone());
                } else {
                    new_state.selected_comand = None;
                }
            }
            Actions::ConfirmFinalize => {
                new_state.confirm_finalize = true;
            }
            Actions::SetPage(page) => {
                new_state.pages = page;
            }
            Actions::SetPayment(comand, payment) => {
                if let Some(a) = new_state.comands.get_mut(&comand) {
                    a.1 = payment;
                }
            }
            Actions::SetGroups(groups) => {
                new_state.groups = groups;
            }
        };
        new_state.into()
    }
}
