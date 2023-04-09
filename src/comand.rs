use serde::{Deserialize, Serialize};

use crate::payment::Payment;

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Input {
    pub name: String,
    pub quantity: u32,
    pub group: Option<String>
}
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct BackendInput {
    pub name: String,
    pub quantity: u32,
    pub unit_price: f32,
}

impl Default for Input {
    fn default() -> Self {
        Self {
            name: String::default(),
            quantity: 1,
            group: None
        }
    }
}
#[derive(Serialize, Deserialize, Debug)]
pub struct Comand {
    pub comand_name: String,
    pub values: Vec<BackendInput>,
    pub payment_method: Payment
}
