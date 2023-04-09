#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod export;
mod macros;
use macros::parsed_output;
use shared::comand::Comand;
use shared::errors::InError;
use std::io::Write;
use std::path::{PathBuf, Path};
use std::{
    collections::HashMap,
    fs::{self, OpenOptions},
    io::Read,
};
use tauri::WindowEvent;

const RS_GRILL_FOLDER: &str = ".RSGRILL";

fn default_join(file: &'static str) -> Result<PathBuf, InError>{
    let Some(mut home) = home::home_dir() else {
        return Err(InError { a: String::from("Failed to find home") })
    };
    home = home.join(RS_GRILL_FOLDER);
    if !Path::new(&home).exists() {
        std::fs::create_dir_all(&home)?;
    };
    home = home.join(file);
    Ok(home)
}

#[tauri::command]
fn file() -> String {
    parsed_output!(HashMap<String, (f32, Option<String>)>,
    {
        let mut file =
        match fs::File::options().read(true).open(default_join("products.csv")?) {
            Ok(a) => a,
            Err(err) => {
                if err.kind() == std::io::ErrorKind::NotFound {
                    let file = OpenOptions::new()
                        .write(true)
                        .read(true)
                        .create(true)
                        .open(default_join("products.csv")?)?;
                    file
                } else {
                    return Err(err.into());
                }
            }
        };

        let mut buffer = String::new();
        file.read_to_string(&mut buffer)?;
        let lines = buffer.lines();

        let mut hash = HashMap::new();

        for i in lines {
            let mut splited = i.split(',');
            if let Some(name) = splited.next() {
                if let Some(price) = splited.next() {
                    if let Some(group) = splited.next() {
                        hash.insert(name.to_string(), (price.trim().parse::<f32>()?, Some(group.to_string())));
                    } else {
                        hash.insert(name.to_string(), (price.trim().parse::<f32>()?, None));
                    }
                }
            }
        }
        Ok(hash)
    })
}

#[tauri::command]
fn update_products(products: Vec<(String, (f32, Option<String>))>) -> String {
    parsed_output!((), {
        let mut buffer = String::new();
        for (k, v) in products {
            if let Some(a) = v.1 {
                buffer.push_str(&format!(
                    "{},{:.2},{}\n",
                    k.trim().to_lowercase(),
                    v.0,
                    a
                ));
            } else {
                buffer.push_str(&format!(
                    "{},{:.2}\n",
                    k.trim().to_lowercase(),
                    v.0
                ));
            }
            
        }
        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(default_join("products.csv")?)?;

        write!(file, "{}", buffer)?;

        Ok(())
    })
}

#[tauri::command]
fn save_comand(comand: Comand) -> String {
    parsed_output!((), {
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open(default_join("saved.csv")?)?;
        writeln!(
            file,
            "{}\nPedido,Unidades,Unitario,Total",
            comand.comand_name
        )?;
        for i in &comand.values {
            writeln!(
                file,
                "{},{},R${:.2},R${:.2}",
                i.name,
                i.quantity,
                i.unit_price,
                i.unit_price * i.quantity as f32
            )?;
        }
        let total = comand.values.iter().fold(0.0, |acc, i| {
            acc + (i.unit_price * i.quantity as f32)
        });

        writeln!(
            file,
            "total,{},R${total:.2}\n",
            comand.payment_method.to_string()
        )?;

        Ok(())
    })
}

#[tauri::command]
fn is_cache_empty() -> String {
    parsed_output!(bool, {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(default_join("saved.csv")?)?;
        let mut string = String::new();
        file.read_to_string(&mut string)?;
        for i in string.lines() {
            if !i.is_empty() {
                return Ok(false);
            }
        }
        Ok(true)
    })
}

#[tauri::command]
fn export() {
    tauri::api::dialog::FileDialogBuilder::new()
        .add_filter("xlsx", &["xlsx"])
        .set_file_name("sheet.xlsx")
        .save_file(|f| {
            if let Some(path) = f {
                let Some(str) = path.to_str() else {
                    return
                };
                if let Err(err) = export::export(str) {
                    println!("{:?}", err);
                    return;
                };
                if OpenOptions::new()
                    .write(true)
                    .truncate(true)
                    .open(match default_join("saved.csv") {
                        Ok(a) => a,
                        Err(_) => return
                    })
                    .is_err()
                {
                    return;
                };
            }
        });
}


#[tauri::command]
fn get_groups() -> String {
    parsed_output!(HashMap<String, Vec<String>>, {
        let mut file = OpenOptions::new()
            .write(true)
            .read(true)
            .create(true)
            .open(default_join("groups.csv")?)?;

        let mut hash = HashMap::new();
        let mut buffer = String::new(); 
        file.read_to_string(&mut buffer)?;
        for i in buffer.lines() {
            let mut split = i.split(',');
            if let Some(name) = split.next() {
                if let Some(values) = split.next() {
                    let values_inside: Vec<String> = values.split(';')
                    .map(|a| a.to_string()).collect();
                    hash.insert(name.to_string(), values_inside);
                }
            } 
        }
        Ok(hash)
    })

}
#[tauri::command]
fn set_groups(groups: Vec<(String, Vec<String>)>) -> String {
    parsed_output!((), {
        let mut buffer = String::new();

        for (name, values) in groups {
            let mut line_buffer = String::new();

            for i in values {
                line_buffer.push_str(&format!("{};", i));
            }
            line_buffer.pop();

            buffer.push_str(&format!("{},{}\n", name, line_buffer));
        }

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(default_join("groups.csv")?)?;
        write!(file, "{}", buffer)?;
        Ok(())
    })
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            file,
            save_comand,
            export,
            is_cache_empty,
            update_products,
            get_groups,
            set_groups
        ])
        .on_window_event(|ev| match ev.event() {
            WindowEvent::CloseRequested { api, .. } => {
                // if 
                let window = ev.window();
                let api = api.clone();
                api.prevent_close();
                let win = ev.window().clone();
                tauri::api::dialog::ask(
                    Some(window), 
                    "Deseja sair?", 
                    "Qualquer comanda não finalizada será APAGADA, tem certeza que desaja sair?", 
                    move |a| {
                    if a {
                        win.close().unwrap();  
                    }
                });
            }


            _ => {}
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
