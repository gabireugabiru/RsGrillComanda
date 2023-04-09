use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::to_value;
use shared::{comand::Comand, errors::InError};
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "tauri"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[derive(Serialize, Deserialize)]
pub struct GreetArgs<'a> {
    pub name: &'a str,
}
#[derive(Serialize, Deserialize)]
pub struct SaveArgs {
    pub comand: Comand,
}
#[derive(Serialize, Deserialize)]
pub struct UpdateArgs {
    pub products: Vec<(String, (f32, Option<String>))>,
}
#[derive(Serialize, Deserialize)]
pub struct SetGroupsArgs {
    pub groups: Vec<(String, Vec<String>)>,
}
pub enum Invoke {
    File,
    GetGroups,
    SetGroups(SetGroupsArgs),
    SaveComand(SaveArgs),
    Export,
    IsCacheEmpty,
    UpdateProducts(UpdateArgs),
}

impl Invoke {
    pub async fn string_invoke(
        a: &'static str,
        args: JsValue,
    ) -> Result<String, InError> {
        invoke(a, args).await.as_string().ok_or_else(|| InError {
            a: "failed".to_string(),
        })
    }

    pub async fn call<'a, T: Deserialize<'a>>(
        &self,
        buffer: &'a mut String,
    ) -> Result<T, InError> {
        match self {
            Self::File => {
                *buffer =
                    Self::string_invoke("file", JsValue::default())
                        .await?;
            }
            Self::SaveComand(saveargs) => {
                let args = to_value(saveargs).unwrap();
                *buffer =
                    Self::string_invoke("save_comand", args).await?;
            }
            Self::Export => {
                *buffer =
                    Self::string_invoke("export", JsValue::default())
                        .await?;
            }
            Self::IsCacheEmpty => {
                *buffer = Self::string_invoke(
                    "is_cache_empty",
                    JsValue::default(),
                )
                .await?;
            }
            Self::UpdateProducts(updateargs) => {
                let args = to_value(updateargs).unwrap();
                *buffer =
                    Self::string_invoke("update_products", args)
                        .await?;
            }
            Self::GetGroups => {
                *buffer = Self::string_invoke(
                    "get_groups",
                    JsValue::default(),
                )
                .await?;
            }
            Self::SetGroups(args) => {
                let args = to_value(args).unwrap();
                *buffer =
                    Self::string_invoke("set_groups", args).await?;
            }
        };
        serde_json::from_str::<Result<T, InError>>(buffer)?
    }
}

macro_rules! invokem {
    ($expr:expr, $t:ty) => {{
        let mut buffer = String::new();
        $expr.call::<$t>(&mut buffer).await
    }};
}
pub(crate) use invokem;
