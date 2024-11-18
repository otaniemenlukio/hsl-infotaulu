use chrono::{Datelike, Timelike, Weekday};

pub struct DateInfo {
    pub time: String,
    pub date: String,
    pub week: String,
}

impl DateInfo {
    fn weekday_to_string(value: Weekday) -> String {
        match value {
            Weekday::Mon => String::from("Maanantai"),
            Weekday::Tue => String::from("Tiistai"),
            Weekday::Wed => String::from("Keskiviikko"),
            Weekday::Thu => String::from("Torstai"),
            Weekday::Fri => String::from("Perjantai"),
            Weekday::Sat => String::from("Lauantai"),
            Weekday::Sun => String::from("Sunnuntai"),
        }
    }

    fn month_to_string(value: u32) -> String {
        match value {
            1 => String::from("tammikuuta"),
            2 => String::from("helmikuuta"),
            3 => String::from("maaliskuuta"),
            4 => String::from("huhtikuuta"),
            5 => String::from("toukokuuta"),
            6 => String::from("kesäkuuta"),
            7 => String::from("heinäkuuta"),
            8 => String::from("elokuuta"),
            9 => String::from("syyskuuta"),
            10 => String::from("lokakuuta"),
            11 => String::from("marraskuuta"),
            12 => String::from("joulukuuta"),
            _ => String::from("?"),
        }
    }

    pub fn calculate() -> Self {
        let now = chrono::offset::Local::now();
        let time = format!("{:02}:{:02}", now.hour(), now.minute());
        let date = format!(
            "{} {}. {}",
            Self::weekday_to_string(now.weekday()),
            now.day(),
            Self::month_to_string(now.month())
        );
        let week = format!("Viikko {}", now.iso_week().week());

        Self { time, date, week }
    }
}
