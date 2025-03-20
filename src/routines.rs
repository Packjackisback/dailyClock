use chrono::{Datelike, Local, Weekday};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct BaseballInSeasonTrainingSchedule {
    #[serde(rename = "Weeks")]
    pub weeks: Vec<Week>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Week {
    #[serde(rename = "Week")]
    pub week: String,
    #[serde(rename = "Schedule")]
    pub schedule: Vec<DaySchedule>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DaySchedule {
    #[serde(rename = "Day")]
    pub day: String,
    #[serde(rename = "Date")]
    pub date: String,
    #[serde(rename = "Throwing")]
    pub throwing: String,
    #[serde(rename = "Lifting")]
    pub lifting: String,
    #[serde(rename = "Game")]
    pub game: String,
}

pub fn get_today_schedule(schedule_json: &str) -> Option<DaySchedule> {
    let schedule: BaseballInSeasonTrainingSchedule = match serde_json::from_str(schedule_json) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("JSON parsing error: {}", e);
            return None;
        }
    };
    let today = Local::now();
    let today_weekday = today.weekday();
    let today_day = today.day();
    let today_month = today.month();

    let weekday_str = match today_weekday {
        Weekday::Mon => "MON",
        Weekday::Tue => "TUE",
        Weekday::Wed => "WED",
        Weekday::Thu => "THU",
        Weekday::Fri => "FRI",
        Weekday::Sat => "SAT",
        Weekday::Sun => "SUN",
    };

    // let weekday_str = "MON";

    let formatted_date = format!("{:02}-{:02}", today_day, today_month);
    // let formatted_date = "17-03";

    for week in schedule.weeks {
        for day_schedule in week.schedule {
            if day_schedule.day == weekday_str && day_schedule.date == formatted_date {
                return Some(day_schedule);
            }
        }
    }

    None
}
