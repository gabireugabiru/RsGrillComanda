use std::cmp::Ordering;

#[derive(PartialEq, Eq, Debug, PartialOrd, Copy, Clone)]
pub struct FileDate {
    pub day: u8,
    pub month: u8,
    pub year: u8,
}
#[derive(Clone, Copy, Debug)]
pub enum FileDateError {
    FailedToParseMonth,
    FailedToParseDay,
    FailedToParseYear,
}
const MONTHS: [&str; 12] = [
    "Janeiro",
    "Fevereiro",
    "Março",
    "Abril",
    "Maio",
    "Junho",
    "Julho",
    "Agosto",
    "Setembro",
    "Outubro",
    "Novembro",
    "Dezembro",
];
impl FileDate {
    fn convert_option<T: Copy>(op: Option<&str>, err: T) -> Result<u8, T> {
        let a = op.ok_or(err)?;
        a.parse().or_else(move |_| Err(err))
    }
    pub fn get_month_string(&self) -> &str {
        let real = self.month.min(11).max(1) as usize;
        MONTHS[real - 1]
    }
    pub fn get_year_string(&self) -> String {
        if self.year > 100 {
            self.year.to_string()
        } else {
            format!("20{}", self.year)
        }
    }
}

impl TryFrom<&str> for FileDate {
    type Error = FileDateError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut splited = value.split('-');

        let day =
            Self::convert_option(splited.next(), FileDateError::FailedToParseDay)?;
        let month =
            Self::convert_option(splited.next(), FileDateError::FailedToParseMonth)?;
        let year =
            Self::convert_option(splited.next(), FileDateError::FailedToParseYear)?;

        Ok(FileDate { day, month, year })
    }
}

impl std::cmp::Ord for FileDate {
    fn cmp(&self, other: &Self) -> Ordering {
        let year = self.year.cmp(&other.year);
        let month = self.month.cmp(&other.month);
        let day = self.day.cmp(&other.day);
        if year.is_ne() {
            return year.reverse();
        }
        if month.is_ne() {
            return month.reverse();
        }
        day.reverse()
    }
}
