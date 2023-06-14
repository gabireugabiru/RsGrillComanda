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
#[derive(Deserialize, Serialize)]
pub struct ReExportArgs {
    pub backup: String,
}

#[derive(Deserialize, Serialize)]
pub struct MultiExportArgs {
    pub backups: Vec<String>,
}

#[derive(Deserialize, Serialize)]
pub struct UpdateStockArgs {
    pub stock: Vec<(String, u64)>,
}
#[derive(Serialize, Deserialize)]
pub struct ConfirmArgs {
    pub message: String,
    pub title: String,
}
pub enum Invoke {
    File,
    GetGroups,
    SetGroups(SetGroupsArgs),
    SaveComand(SaveArgs),
    Export,
    IsCacheEmpty,
    UpdateProducts(UpdateArgs),
    ListBakcups,
    ReExport(ReExportArgs),
    MultiExport(MultiExportArgs),
    UpdateStock(UpdateStockArgs),
    ReadStock,
    Confirm(String, String),
}

impl Invoke {
    pub async fn string_invoke<T>(a: &'static str, args: T) -> Result<String, InError>
    where
        T: Serialize,
    {
        invoke(
            a,
            match to_value(&args) {
                Ok(a) => a,
                Err(_) => {
                    return Err(InError {
                        a: "Failed to parse".to_string(),
                    })
                }
            },
        )
        .await
        .as_string()
        .ok_or_else(|| InError {
            a: "Failed".to_string(),
        })
    }

    pub async fn call<'a, T: Deserialize<'a>>(
        &self,
        buffer: &'a mut String,
    ) -> Result<T, InError> {
        match self {
            Self::File => {
                *buffer = Self::string_invoke("file", ()).await?;
            }
            Self::SaveComand(saveargs) => {
                *buffer = Self::string_invoke("save_comand", saveargs).await?;
            }
            Self::Export => {
                *buffer = Self::string_invoke("export", ()).await?;
            }
            Self::IsCacheEmpty => {
                *buffer = Self::string_invoke("is_cache_empty", ()).await?;
            }
            Self::UpdateProducts(args) => {
                *buffer = Self::string_invoke("update_products", args).await?;
            }
            Self::GetGroups => {
                *buffer = Self::string_invoke("get_groups", ()).await?;
            }
            Self::SetGroups(args) => {
                *buffer = Self::string_invoke("set_groups", args).await?;
            }
            Self::ListBakcups => {
                *buffer = Self::string_invoke("read_backups", ()).await?;
            }
            Self::ReExport(args) => {
                *buffer = Self::string_invoke("re_export", args).await?;
            }
            Self::MultiExport(args) => {
                *buffer = Self::string_invoke("multi_export", args).await?;
            }
            Self::UpdateStock(args) => {
                *buffer = Self::string_invoke("update_stock", args).await?;
            }
            Self::ReadStock => {
                *buffer = Self::string_invoke("read_stock", ()).await?;
            }
            Self::Confirm(title, message) => {
                // let val = json!({});
                *buffer = Self::string_invoke(
                    "confirm",
                    ConfirmArgs {
                        message: message.clone(),
                        title: title.clone(),
                    },
                )
                .await?;
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
