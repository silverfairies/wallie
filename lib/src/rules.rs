use chrono::{Datelike, Local, Month, NaiveDate, NaiveTime, Weekday};
use serde::{Deserialize, Serialize};

use crate::dataset::Tag;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Rule {
    name: String,
    weight: u32,
    pub level: u8,
    pub transparent: bool,
    tags: Vec<Tag>,
    condition: Condition,
}

impl Rule {
    pub fn is_true(&self) -> bool {
        self.condition.is_true()
    }

    pub fn load(&self) -> (Vec<Tag>, u32, u8) {
        (self.tags.clone(), self.weight, self.level)
    }

    pub fn new(name: String, weight: u32, level: u8, transparent: bool, tags: Vec<Tag>) -> Self {
        Self {
            name,
            weight,
            level,
            transparent,
            tags,
            condition: Condition::Always,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Condition {
    Never,
    Always,
    Time(Time),
    Script(ConditionScript),
}

impl Condition {
    fn is_true(&self) -> bool {
        match self {
            Condition::Never => false,
            Condition::Always => true,
            Condition::Time(time) => time.is_true(),
            Condition::Script(_script) => {
                eprintln!("Not implemented!");
                false
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Time {
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

impl Time {
    fn is_true(&self) -> bool {
        let now = Local::now();
        match self {
            Time::Daytime => {
                eprintln!("Not implemented!");
                false
            }
            Time::Daily { timerange } => timerange.is_true(),
            Time::Weekday { days, timerange } if days.contains(&now.weekday()) => {
                timerange.is_true()
            }
            Time::Month { months, timerange }
                if months.contains(&Month::try_from(now.month() as u8).unwrap()) =>
            {
                timerange.is_true()
            }
            Time::Date { date, timerange }
                if date.month().eq(&now.month()) && date.day().eq(&now.day()) =>
            {
                timerange.is_true()
            }
            _ => false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct TimeRange {
    start: NaiveTime,
    end: NaiveTime,
}

impl TimeRange {
    fn is_true(&self) -> bool {
        let now = Local::now().naive_local().time();
        self.start.le(&now) && self.end.ge(&now)
    }
}

#[allow(unused)]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
enum Daytime {
    Night,
    Day,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
struct ConditionScript {
    script: String,
    script_answer: ScriptAnswer,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
enum ScriptAnswer {
    ExitCode(u8),
    StdOut(String),
}
