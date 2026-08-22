use chrono::{Month, NaiveDate, NaiveTime, Weekday};
use serde::{Deserialize, Serialize};

use crate::dataset::Tag;

#[derive(Serialize, Deserialize)]
pub struct Rule {
    name: String,
    weight: u8,
    level: u8,
    transparent: bool,
    tags: Vec<Tag>,
    condition: Condition,
}

#[derive(Serialize, Deserialize)]
enum Condition {
    Never,
    Always,
    Time { time: Time },
    Script(ConditionScript),
}

#[derive(Serialize, Deserialize)]
enum Time {
    #[allow(unused)]
    Daytime,
    Daily {
        timerange: TimeRange,
    },
    Weekday {
        days: Vec<Weekday>,
        timerange: TimeRange,
    },
    Month {
        months: Vec<Month>,
        timerange: TimeRange,
    },
    Date {
        date: NaiveDate,
        timerange: TimeRange,
    },
}

#[derive(Serialize, Deserialize)]
struct TimeRange {
    start: NaiveTime,
    end: NaiveTime,
}

#[allow(unused)]
#[derive(Serialize, Deserialize)]
enum Daytime {
    Night,
    Day,
}

#[derive(Serialize, Deserialize)]
struct ConditionScript {
    script: String,
    script_answer: ScriptAnswer,
}

#[derive(Serialize, Deserialize)]
enum ScriptAnswer {
    ExitCode(u8),
    StdOut(String),
}
