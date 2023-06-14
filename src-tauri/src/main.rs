#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod export;
mod macros;
use chrono::{Local, Datelike};
use macros::parsed_output;
use shared::comand::Comand;
use shared::errors::InError;
use shared::payment::Payment;
use sysinfo::{SystemExt, Process};
use std::fs::File;
use std::io::Write;
use std::path::{PathBuf, Path};
use std::process::exit;
use std::{
    collections::HashMap,
    fs:: OpenOptions,
    io::Read,
};
use tauri::{WindowEvent, Window};

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

fn default_open<T>(path: T) -> Result<File, std::io::Error> where T: AsRef<Path> {
    OpenOptions::new()
        .write(true)
        .read(true)
        .create(true)
        .open(path)
}
fn truncate<T>(path: T) -> Result<File, std::io::Error> where T: AsRef<Path> {
    OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(path)
}
fn append<T>(path: T) -> Result<File, std::io::Error> where T: AsRef<Path> {
    OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(path)
}


#[tauri::command]
async fn confirm(message: String, title: String, window: tauri::Window) -> String{
    let a = tauri::async_runtime::spawn(async move {
        tauri::api::dialog::blocking::confirm(Some(&window), title, message)
    }).await;

    parsed_output!(bool, {
        // let a = tauri::api::dialog::confirm()
        Ok(match a {
            Ok(a) => a,
            Err(_) => false
        })
    })
}

#[tauri::command]
fn update_stock(stock: Vec<(String, u64)>) -> String {
    parsed_output!((),{
        let mut buffer = String::new();

        for i in stock {
            buffer.push_str(&format!("{},{}\n", i.0,i.1));
        }

        let mut file = truncate(default_join("stock.csv")?)?;
        write!(file,"{}", buffer)?;
        Ok(())
    })
}

#[tauri::command]
fn read_stock() -> String {
    parsed_output!(HashMap<String, u64>, {
        let mut hash = HashMap::new();
        let mut file = default_open(default_join("stock.csv")?)?;

        let mut buffer = String::new();
        file.read_to_string(&mut buffer)?;
        let lines = buffer.lines();

        for line in lines {
            let values: Vec<&str> = line.split(',').collect();
            if values.len() == 2 {
                let name = values[0];
                let quantity = values[1].parse::<u64>()?;
                hash.insert(name.to_string(), quantity);
            }
        }

        Ok(hash)
    })
}

#[tauri::command]
fn file() -> String {
    parsed_output!(HashMap<String, (f32, Option<String>)>,
    {
        let mut file = default_open(default_join("products.csv")?)?;

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
        let mut file = truncate(default_join("products.csv")?)?;

        write!(file, "{}", buffer)?;

        Ok(())
    })
}

#[tauri::command]
fn save_comand(comand: Comand) -> String {
    parsed_output!((), {
        let mut file = append(default_join("saved.csv")?)?;
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
        let mut file = default_open(default_join("saved.csv")?)?;
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
fn export() -> String {
    let date = Local::now();
    let year = date.year().to_string();
    let mut chars = year.chars().collect::<Vec<char>>();
    let two_last_digits = chars.split_off(2);


    tauri::api::dialog::FileDialogBuilder::new()
        .add_filter("xlsx", &["xlsx"])
        .set_file_name(&format!("{}-{}-{}.xlsx", date.day(), date.month(), two_last_digits.iter().collect::<String>()))
        .save_file(|f| {
            if let Some(path) = f {
                let mut other_path = path.clone();
                other_path.set_extension("csv");
                let Some(a) = other_path.file_name() else {
                    return
                };

                let real_file_name = a.to_string_lossy();

                let Some(str) = path.to_str() else {
                    return
                };

                let Ok(mut file) = default_open(match default_join("saved.csv") {
                        Ok(a) => a,
                        Err(_) => return
                    }) else {
                        return
                    };

                let mut string = String::new();
                
                file.read_to_string(&mut string).unwrap();
                let Ok(comands) = export::get_comands(&string) else {
                    return
                };
                if let Err(err) = export::export(&comands ,str) {
                    println!("{:?}", err);
                    return;
                };

                let Ok(backup_path) = default_join("backups") else {
                    return
                };

                let backup_file = backup_path.join(format!("{}", real_file_name));

                let Ok(mut file) = truncate(backup_file) else {
                    return
                };

                write!(file,"{}", string).unwrap();

                if truncate(
                    match default_join("saved.csv") {
                        Ok(a) => a,
                        Err(_) => return
                    }
                ).is_err()
                {
                    return;
                };
            }
        });
    parsed_output!((), {
        Ok(())
    })
}


#[tauri::command]
fn get_groups() -> String {
    parsed_output!(HashMap<String, Vec<String>>, {
        let mut file = default_open(default_join("groups.csv")?)?;

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

        let mut file = truncate(default_join("groups.csv")?)?;
        write!(file, "{}", buffer)?;
        Ok(())
    })
}

#[tauri::command]
fn read_backups() -> String {
    parsed_output!(Vec<String>, {
        let read_dir = default_join("backups")?.read_dir()?;

        let filtered = read_dir.filter(|a| {
            let Ok(a) = a else {
                return false
            };
            let path = a.path();
            let Some(ext) = path.extension() else {
                return false
            };

            if ext != "csv" {
                return false
            }

            true
        })
        .flatten()
        .map(|a| a.file_name().to_string_lossy().to_string())
        .collect::<Vec<String>>();


        Ok(filtered)
    })
}

#[tauri::command]
fn re_export(backup: String) -> String{
    let mut path = PathBuf::from(&backup);
    path.set_extension("xlsx");
    let path = match path.to_str() {
        Some(a) => a,
        None => "sheet.xlsx"
    };
    tauri::api::dialog::FileDialogBuilder::new()
        .add_filter("xlsx", &["xlsx"])
        .set_file_name(path)
        .save_file(|file| {
            let Some(file) = file else {
                return
            };
            let Some(file_str) = file.to_str() else {
                return
            };

            let Ok(backups_path) = default_join("backups") else {
                return
            };

            let Ok(mut backup_file) = default_open(backups_path.join(backup)) else {
                return
            };

            let mut string = String::new();

            if backup_file.read_to_string(&mut string).is_err() {
                return
            };

            let Ok(comands) = export::get_comands(&string) else {
                return
            };

            if export::export(&comands, file_str).is_err() {
                return
            };

        });
    parsed_output!((),{
        Ok(())
    })
}

#[tauri::command]
fn multi_export(backups: Vec<String>) -> String {


    let mut total = 0.0;
    let mut selled_items = HashMap::<String, (f64, f64)>::new();
    let mut payed = HashMap::<Payment, f64>::new();
    for i in Payment::iter() {
        payed.insert(i, 0.0);
    }
    parsed_output!((), {
        let backups_path = default_join("backups")?;
        for backup in backups {
            let mut backup_f = default_open(backups_path.join(&backup))?;
            let mut string = String::new();
            backup_f.read_to_string(&mut string)?;
            let Ok(comands) = export::get_comands(&string) else {
                return Err(InError {
                    a: format!("Failed to convert {}", backup)
                })
            };

            for comand in comands {
                total += comand.total;
                // WILL NEVER UNWRAP SINCE THE HASH IS INITIALIZED WITH ALL VALUES OF PAYMENT
                let payed = payed.get_mut(&comand.payment).unwrap();
                *payed += comand.total; 
                for value in comand.values {
                    match selled_items.get_mut(&value.name) {
                        Some(a) => {
                            a.0 += value.total;
                            a.1 += value.quantity;

                        }
                        None => {
                            selled_items.insert(value.name.clone(), (value.total, value.quantity));
                        }
                    }
                   
                }
            }
        }
        tauri::api::dialog::FileDialogBuilder::new()
        .add_filter("xlsx", &["xlsx"])
        .set_file_name("relatorio.xlsx")
        .save_file(move |file| {
            let Some(file) = file else {
                return
            };
            if export::multi_export(total, payed, selled_items, match file.to_str() {
                Some(a) => a,
                None => return
            }).is_err() {
                return
            }
        });
        
        Ok(())
    })
}
fn main() {
    let system = sysinfo::System::new_all();
    let is_first_instance_win = system.processes_by_name("RsGrill").collect::<Vec<&Process>>().len() < 2;
    let is_first_instance = system.processes_by_exact_name("rs-grill").collect::<Vec<&Process>>().len() < 2;
    
    if !is_first_instance || !is_first_instance_win {
        tauri::api::dialog::blocking::message(None::<&Window>, "Duplicata", "Esse computador já possui uma instancia desse programa rodando");
        exit(1);
    }

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            file,
            save_comand,
            export,
            is_cache_empty,
            update_products,
            get_groups,
            set_groups,
            read_backups,
            re_export,
            multi_export,
            update_stock,
            read_stock,
            confirm
        ])
        .on_window_event(|ev| match ev.event() {
            WindowEvent::CloseRequested { api, .. } => {
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
