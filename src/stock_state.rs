use std::collections::HashMap;

use yew::Reducible;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct StockState {
    pub stock: HashMap<String, u64>,
}
impl StockState {
    fn get_mut<'a>(stock: &'a mut HashMap<String, u64>, name: &str) -> Option<&'a mut u64> {
        let splited: Vec<&str> = name.split('(').collect();
        if splited.len() != 2 {
            return stock.get_mut(name);
        }
        let first = splited[0].trim();
        let mut last = splited[1].trim().to_string();
        if last.ends_with(')') {
            last.pop();
        }

        let specific = stock.get(name);
        let global = stock.get(&format!("({last})"));
        if specific.is_some() {
            stock.get_mut(name)
        } else if global.is_some() {
            stock.get_mut(&format!("({last})"))
        } else if first.is_empty() {
            let mut key = String::default();

            for k in stock.keys() {
                let splited: Vec<&str> = k.split('(').collect();
                if splited.len() == 2 {
                    let mut a = splited[1].trim().to_string();
                    if a.ends_with(')') {
                        a.pop();
                    }
                    if a == last {
                        key = k.clone();
                    }
                }
            }
            if key.is_empty() {
                return None;
            }
            stock.get_mut(&key)
        } else {
            stock.get_mut(name)
        }
    }
    fn get<'a>(stock: &'a HashMap<String, u64>, name: &str) -> Option<&'a u64> {
        let splited: Vec<&str> = name.split('(').collect();
        if splited.len() != 2 {
            return stock.get(name);
        }
        let first = splited[0].trim();
        let mut last = splited[1].trim().to_string();
        if last.ends_with(')') {
            last.pop();
        }

        let specific = stock.get(name);
        let global = stock.get(&format!("({last})"));

        if specific.is_some() {
            specific
        } else if global.is_some() {
            global
        } else if first.is_empty() {
            let mut key = String::default();

            for k in stock.keys() {
                let splited: Vec<&str> = k.split('(').collect();
                if splited.len() == 2 {
                    let mut a = splited[1].trim().to_string();
                    if a.ends_with(')') {
                        a.pop();
                    }
                    if a == last {
                        key = k.clone();
                    }
                }
            }
            if key.is_empty() {
                return None;
            }
            stock.get(&key)
        } else {
            stock.get(name)
        }
    }
}

#[derive(Clone)]
pub enum StockActions {
    Add(String, i64),
    SubVec(Vec<(String, i64)>),
    SetValue(String, u64),
    Create(String, u64),
    CreateVec(Vec<String>),
    Set(HashMap<String, u64>),
    Remove(String),
}

impl Reducible for StockState {
    type Action = StockActions;
    fn reduce(self: std::rc::Rc<Self>, action: Self::Action) -> std::rc::Rc<Self> {
        let mut new_state = (*self).clone();
        match action {
            Self::Action::Add(name, value) => {
                if let Some(a) = Self::get_mut(&mut new_state.stock, &name) {
                    let real_value = (*a as i64 + value).max(0);
                    *a = real_value as u64;
                }
            }
            Self::Action::Create(name, value) => {
                if Self::get(&new_state.stock, &name).is_none() {
                    new_state.stock.insert(name.trim().to_string(), value);
                }
            }
            Self::Action::Set(a) => {
                new_state.stock = a;
            }
            Self::Action::SetValue(name, value) => {
                if let Some(a) = Self::get_mut(&mut new_state.stock, &name) {
                    *a = value;
                }
            }
            Self::Action::SubVec(values) => {
                for i in values {
                    if let Some(a) = Self::get_mut(&mut new_state.stock, &i.0) {
                        let real_value = (*a as i64 - i.1).max(0);
                        *a = real_value as u64;
                    }
                }
            }
            Self::Action::Remove(name) => {
                new_state.stock.remove(&name);
            }
            Self::Action::CreateVec(vec) => {
                for i in vec {
                    if Self::get(&new_state.stock, &i).is_none() {
                        new_state.stock.insert(i, 0);
                    }
                }
            }
        };
        new_state.into()
    }
}
