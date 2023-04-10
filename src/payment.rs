use serde::{Deserialize, Serialize};

#[derive(
    Clone, Copy, Eq, PartialEq, Debug, Serialize, Deserialize,
)]
pub enum Payment {
    Pix,
    Debit,
    Credit,
    Money,
    NotSelected,
}

impl Payment {
    pub fn iter() -> impl Iterator<Item = Payment> {
        [
            Self::NotSelected,
            Self::Debit,
            Self::Credit,
            Self::Money,
            Self::Pix,
        ]
        .iter()
        .copied()
    }
}

impl From<String> for Payment {
    fn from(value: String) -> Self {
        match value.as_str() {
            "Débito" => Self::Debit,
            "Crédito" => Self::Credit,
            "Dinheiro" => Self::Money,
            "Pix" => Self::Pix,
            _ => Self::NotSelected,
        }
    }
}
impl ToString for Payment {
    fn to_string(&self) -> String {
        match self {
            Self::NotSelected => "Metodo de pagamento",
            Self::Debit => "Débito",
            Self::Credit => "Crédito",
            Self::Pix => "Pix",
            Self::Money => "Dinheiro",
        }
        .to_string()
    }
}
impl Default for Payment {
    fn default() -> Self {
        Self::NotSelected
    }
}
