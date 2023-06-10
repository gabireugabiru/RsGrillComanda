use std::collections::HashMap;

use yew::Reducible;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct StockState {
    pub stock: HashMap<String, u64>,
}
#[derive(Clone)]
pub enum StockActions {
    Add(String, i64),
    // Sub(String, u64),
    SetValue(String, u64),
    Create(String, u64),
    Set(HashMap<String, u64>),
}

impl Reducible for StockState {
    type Action = StockActions;
    fn reduce(self: std::rc::Rc<Self>, action: Self::Action) -> std::rc::Rc<Self> {
        let mut new_state = (*self).clone();
        match action {
            Self::Action::Add(name, value) => {
                if let Some(a) = new_state.stock.get_mut(&name) {
                    let real_value = (*a as i64 + value).max(0);
                    *a = real_value as u64;
                }
            }
            Self::Action::Create(name, value) => {
                if new_state.stock.get(&name).is_none() {
                    new_state.stock.insert(name, value);
                }
            }
            Self::Action::Set(a) => {
                new_state.stock = a;
            }
            Self::Action::SetValue(name, value) => {
                if let Some(a) = new_state.stock.get_mut(&name) {
                    *a = value;
                }
            }
        };
        new_state.into()
    }
}
