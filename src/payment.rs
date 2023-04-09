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
        [Self::NotSelected, Self::Debit, Self::Credit, Self::Money, Self::Pix].iter().copied()
    }
}

impl From<String> for Payment {
    fn from(value: String) -> Self {
        match value.as_str() {
            "debito" => Self::Debit,
            "credito" => Self::Credit,
            "dinheiro" => Self::Money,
            "pix" => Self::Pix,
            _ => Self::NotSelected,
        }
    }
}
impl ToString for Payment {
    fn to_string(&self) -> String {
        match self {
            Self::NotSelected => "",
            Self::Debit => "debito",
            Self::Credit => "credito",
            Self::Pix => "pix",
            Self::Money => "dinheiro",
        }
        .to_string()
    }
}
impl Default for Payment {
    fn default() -> Self {
        Self::NotSelected
    }
}
