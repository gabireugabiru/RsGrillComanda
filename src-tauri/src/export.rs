use std::{
    collections::HashMap,
    error::Error,
    ops::{Deref, DerefMut},
};

use shared::{errors::InError, payment::Payment};
use xlsxwriter::{Format, Workbook, Worksheet};

#[derive(Default, Debug)]
pub struct Value {
    pub name: String,
    pub unit_price: f64,
    pub total: f64,
    pub quantity: f64,
}
#[derive(Default, Debug)]
pub struct Comand {
    pub name: String,
    pub values: Vec<Value>,
    pub total: f64,
    pub payment: Payment,
}
#[derive(Clone, Copy, Debug)]
pub struct Position {
    pub col: u8,
    pub row: u32,
}
impl From<(u16, u32)> for Position {
    fn from(value: (u16, u32)) -> Self {
        Self {
            col: value.0 as u8,
            row: value.1,
        }
    }
}

impl ToString for Position {
    fn to_string(&self) -> String {
        format!("{}{}", (self.col + 65) as char, self.row + 1)
    }
}
#[derive(Default)]
pub struct Positions {
    vec: Vec<Position>,
}
impl Deref for Positions {
    type Target = Vec<Position>;
    fn deref(&self) -> &Self::Target {
        &self.vec
    }
}
impl DerefMut for Positions {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.vec
    }
}
impl ToString for Positions {
    fn to_string(&self) -> String {
        let mut buffer = String::new();
        for i in &self.vec {
            buffer.push_str(&format!("{},", i.to_string()))
        }

        buffer.pop();
        buffer
    }
}

pub fn calc_payment(
    position: Position,
    sheet: &mut Worksheet,
    title: &'static str,
    comands: &Vec<Comand>,
    total_pos: &Positions,
    ty: Payment,
    format: Option<&Format>,
    format2: Option<&Format>,
) -> Result<(), Box<dyn Error>> {
    sheet.write_string(position.row, position.col as u16, title, format)?;
    sheet.merge_range(
        position.row,
        position.col as u16 + 1,
        position.row,
        position.col as u16 + 2,
        "",
        format,
    )?;

    let value =
        comands.iter().fold(
            0.0,
            |acc, i| {
                if i.payment == ty {
                    acc + i.total
                } else {
                    acc
                }
            },
        );
    let mut form = String::from("=");

    for total_pos in &total_pos.vec {
        let method_pos: Position =
            ((total_pos.col - 1) as u16, total_pos.row).into();
        form.push_str(&format!(
            "SUMIF({},\"{}\",{})+",
            method_pos.to_string(),
            ty.to_string(),
            total_pos.to_string()
        ));
    }
    form.pop();

    sheet.write_formula_num(
        position.row,
        (position.col + 1) as u16,
        &form,
        format2,
        value,
    )?;
    Ok(())
}

pub fn get_comands(contents: &str) -> Result<Vec<Comand>, Box<dyn Error>> {
    let mut comands = Vec::<Comand>::new();

    {
        let mut comand = Comand::default();
        for line in contents.lines() {
            let line = line.trim();
            if line.is_empty() {
                comands.push(comand);
                comand = Comand::default();
                continue;
            }
            if line == "Pedido,Unidades,Unitario,Total" {
                continue;
            }
            let splited: Vec<&str> = line.split(',').collect();
            if splited.len() == 4 {
                comand.values.push(Value {
                    name: splited[0].to_owned(),
                    quantity: splited[1].parse()?,
                    unit_price: splited[2].replace("R$", "").parse()?,
                    total: splited[3].replace("R$", "").parse()?,
                });
                continue;
            }
            if splited.len() == 3 {
                comand.payment = splited[1].to_string().into();
                comand.total = splited[2].replace("R$", "").parse()?;
                continue;
            } else {
                comand.name = line.to_owned();
            }
        }
    }
    Ok(comands)
}

pub fn ok<A>(option: Option<A>) -> Result<A, InError> {
    option.ok_or_else(|| InError {
        a: "None in option".to_string(),
    })
}

pub fn multi_export(
    total: f64,
    payed: HashMap<Payment, f64>,
    items: HashMap<String, (f64, f64)>,
    path: &str,
) -> Result<(), Box<dyn Error>> {
    let book = Workbook::new(path)?;
    let mut sheet = book.add_worksheet(None)?;
    let default = book
        .add_format()
        .set_align(xlsxwriter::FormatAlignment::VerticalCenter)
        .set_align(xlsxwriter::FormatAlignment::Left)
        .set_border(xlsxwriter::FormatBorder::Thin);
    let highlighted = book
        .add_format()
        .set_num_format("R$0.00")
        .set_align(xlsxwriter::FormatAlignment::VerticalCenter)
        .set_align(xlsxwriter::FormatAlignment::Left)
        .set_border(xlsxwriter::FormatBorder::Thin)
        .set_bold()
        .set_bg_color(xlsxwriter::FormatColor::Custom(0xd7d7d9));
    const COL: u16 = 1;
    const ROW: u32 = 1;
    sheet.set_default_row(30.0, false);
    sheet.set_column_pixels(COL, COL + 5 as u16, 180, None)?;

    sheet.write_string(ROW, COL, "Bruto", Some(&default))?;
    sheet.write_string(ROW + 1, COL, &Payment::Debit.to_string(), Some(&default))?;
    sheet.write_string(
        ROW + 2,
        COL,
        &Payment::Credit.to_string(),
        Some(&default),
    )?;
    sheet.write_string(ROW + 3, COL, &Payment::Money.to_string(), Some(&default))?;
    sheet.write_string(ROW + 4, COL, &Payment::Pix.to_string(), Some(&default))?;

    sheet.write_number(ROW, COL + 1, total, Some(&highlighted))?;

    sheet.write_number(
        ROW + 1,
        COL + 1,
        *ok(payed.get(&Payment::Debit))?,
        Some(&highlighted),
    )?;
    sheet.write_number(
        ROW + 2,
        COL + 1,
        *ok(payed.get(&Payment::Credit))?,
        Some(&highlighted),
    )?;
    sheet.write_number(
        ROW + 3,
        COL + 1,
        *ok(payed.get(&Payment::Money))?,
        Some(&highlighted),
    )?;
    sheet.write_number(
        ROW + 4,
        COL + 1,
        *ok(payed.get(&Payment::Pix))?,
        Some(&highlighted),
    )?;

    let start_row = 6;
    sheet.write_string(ROW + start_row, COL, "Produto", Some(&default))?;
    sheet.write_string(ROW + start_row, COL + 1, "Vendidos", Some(&default))?;
    sheet.write_string(ROW + start_row, COL + 2, "Vendidos(R$)", Some(&default))?;

    for (i, (name, (value, quantity))) in items.iter().enumerate() {
        sheet.write_string(
            ROW + start_row + 1 + i as u32,
            COL as u16,
            name,
            Some(&default),
        )?;
        sheet.write_number(
            ROW + start_row + 1 + i as u32,
            COL + 1,
            *quantity,
            Some(&default),
        )?;
        sheet.write_number(
            ROW + start_row + 1 + i as u32,
            COL + 2,
            *value,
            Some(&highlighted),
        )?;
    }

    Ok(())
}

pub fn export(comands: &Vec<Comand>, path: &str) -> Result<(), Box<dyn Error>> {
    let book = Workbook::new(path)?;
    let mut sheet = book.add_worksheet(None)?;
    let format = book
        .add_format()
        .set_num_format("R$0.00")
        .set_align(xlsxwriter::FormatAlignment::VerticalCenter)
        .set_align(xlsxwriter::FormatAlignment::Left)
        .set_border(xlsxwriter::FormatBorder::Thin);
    let format1 = book
        .add_format()
        .set_align(xlsxwriter::FormatAlignment::VerticalCenter)
        .set_align(xlsxwriter::FormatAlignment::Left)
        .set_border(xlsxwriter::FormatBorder::Thin);
    let format2 = book
        .add_format()
        .set_num_format("R$0.00")
        .set_align(xlsxwriter::FormatAlignment::VerticalCenter)
        .set_align(xlsxwriter::FormatAlignment::Left)
        .set_border(xlsxwriter::FormatBorder::Thin)
        .set_bold()
        .set_bg_color(xlsxwriter::FormatColor::Custom(0xd7d7d9));
    let format3 = book
        .add_format()
        .set_align(xlsxwriter::FormatAlignment::VerticalCenter)
        .set_align(xlsxwriter::FormatAlignment::Left)
        .set_border(xlsxwriter::FormatBorder::Thin)
        .set_font_size(15.);
    const COL: u16 = 1;
    const ROW: u32 = 1;
    let mut row = ROW;
    let mut total_pos = Positions::default();
    let mut quantities: HashMap<String, (Positions, u32)> = HashMap::new();

    let mut products: HashMap<&str, f64> = HashMap::new();

    sheet.set_default_row(30.0, false);
    for i in comands {
        // COMAND AND HEADER
        sheet.set_column_pixels(COL, COL, 150, None)?;
        sheet.set_column_pixels(COL + 1, COL + 3, 80, None)?;

        sheet.write_string(row, COL, "Comanda", Some(&format))?;
        sheet.merge_range(row, COL + 1, row, COL + 3, &i.name, Some(&format3))?;
        row += 1;
        sheet.write_string(row, COL, "Pedido", Some(&format))?;
        sheet.write_string(row, COL + 1, "Unidades", Some(&format))?;
        sheet.write_string(row, COL + 2, "Unitario", Some(&format))?;
        sheet.write_string(row, COL + 3, "Total", Some(&format))?;

        row += 1;

        // VALUES
        let mut form_pos = Positions::default();
        for value in &i.values {
            products.insert(&value.name, value.unit_price);
            sheet.write_string(row, COL, &value.name, Some(&format))?;
            sheet.write_number(row, COL + 1, value.quantity, Some(&format1))?;
            sheet.write_number(row, COL + 2, value.unit_price, Some(&format))?;

            sheet.write_formula_num(
                row,
                COL + 3,
                &format!(
                    "={}*{}",
                    Position::from((COL + 1, row)).to_string(),
                    Position::from((COL + 2, row)).to_string()
                ),
                Some(&format),
                value.total,
            )?;

            form_pos.push((COL + 3, row).into());
            match quantities.get_mut(&value.name) {
                Some(a) => {
                    a.0.push((COL + 1, row).into());
                    a.1 += value.quantity as u32;
                }
                None => {
                    let mut pos = Positions::default();
                    pos.push((COL + 1, row).into());
                    quantities.insert(
                        value.name.to_string(),
                        (pos, value.quantity as u32),
                    );
                }
            }

            row += 1;
        }
        // TOTAL
        sheet.write_string(row, COL, "Total: ", Some(&format))?;
        sheet.write_string(row, COL + 1, &i.payment.to_string(), Some(&format))?;
        sheet.merge_range(row, COL + 2, row, COL + 3, "", Some(&format))?;
        sheet.write_formula_num(
            row,
            COL + 2,
            &format!("=SUM({})", form_pos.to_string()),
            Some(&format2),
            i.total,
        )?;
        total_pos.push((COL + 2, row).into());
        row += 2;
    }

    let bruto = comands.iter().fold(0.0, |acc, i| acc + i.total);
    sheet.write_string(ROW, COL + 5, "Bruto", Some(&format))?;
    sheet.merge_range(ROW, COL + 6, ROW, COL + 7, "", Some(&format2))?;
    sheet.write_formula_num(
        ROW,
        COL + 6,
        &format!("=SUM({})", total_pos.to_string()),
        Some(&format2),
        bruto,
    )?;
    {
        calc_payment(
            (COL + 5, ROW + 1).into(),
            &mut sheet,
            "Pix",
            &comands,
            &total_pos,
            Payment::Pix,
            Some(&format),
            Some(&format2),
        )?;

        calc_payment(
            (COL + 5, ROW + 2).into(),
            &mut sheet,
            "Debito",
            &comands,
            &total_pos,
            Payment::Debit,
            Some(&format),
            Some(&format2),
        )?;
        calc_payment(
            (COL + 5, ROW + 3).into(),
            &mut sheet,
            "Credito",
            &comands,
            &total_pos,
            Payment::Credit,
            Some(&format),
            Some(&format2),
        )?;
        calc_payment(
            (COL + 5, ROW + 4).into(),
            &mut sheet,
            "Dinheiro",
            &comands,
            &total_pos,
            Payment::Money,
            Some(&format),
            Some(&format2),
        )?;
    }
    let mut vec: Vec<(&String, &(Positions, u32))> = quantities.iter().collect();
    vec.sort_by(|a, b| a.0.cmp(b.0));

    sheet.set_column_pixels(COL + 9, COL + 9, 150, None)?;

    for (i, value) in vec.iter().enumerate() {
        sheet.write_string(ROW + i as u32, COL + 9, value.0, Some(&format))?;

        sheet.write_formula_num(
            ROW + i as u32,
            COL + 10,
            &format!("=SUM({})", value.1 .0.to_string()),
            Some(&format1),
            value.1 .1 as f64,
        )?;
        let Some(unit) = products.get(value.0.as_str()) else {
            continue
        };
        sheet.write_number(
            ROW + i as u32,
            COL + 11,
            value.1 .1 as f64 * unit,
            Some(&format2),
        )?;
    }

    book.close()?;
    Ok(())
}
