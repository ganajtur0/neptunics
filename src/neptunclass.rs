use chrono::{DateTime, Utc};
use icalendar::DatePerhapsTime;
use icalendar::DatePerhapsTime::DateTime as IcalDateTime;
use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};

#[derive(Clone)]
pub struct NeptunClass {
    name: String,
    code: String,
    teachers: Vec<String>,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    location: String,
}

impl Ord for NeptunClass {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.start.cmp(&other.start)
    }
}

impl PartialOrd for NeptunClass {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for NeptunClass {
    fn eq(&self, other: &Self) -> bool {
        self.code == other.code
    }
}

impl Eq for NeptunClass {}

impl Hash for NeptunClass {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.code.to_string().hash(state);
    }
}

impl NeptunClass {
    pub fn new(
        summary: String,
        perhaps_start: DatePerhapsTime,
        perhaps_end: DatePerhapsTime,
        location: String,
    ) -> Self {
        let name_and_the_rest: Vec<&str> = summary.split(" ( - ").collect::<Vec<&str>>();
        let name = name_and_the_rest[0];
        let code_and_the_rest: Vec<&str> =
            name_and_the_rest[1].split(") - ").collect::<Vec<&str>>();
        let code = code_and_the_rest[0];
        let teachers: Vec<String> = code_and_the_rest[1]
            .split(" - ")
            .next()
            .expect("Failed to parse NeptunClass")
            .split(";")
            .map(|s| s.to_owned())
            .collect::<Vec<String>>();
        let start: DateTime<Utc> = match perhaps_start {
            IcalDateTime(idt) => match idt.try_into_utc() {
                Some(dt) => dt,
                _ => DateTime::<Utc>::MIN_UTC,
            },
            _ => DateTime::<Utc>::MIN_UTC,
        };
        let end: DateTime<Utc> = match perhaps_end {
            IcalDateTime(idt) => match idt.try_into_utc() {
                Some(dt) => dt,
                _ => DateTime::<Utc>::MIN_UTC,
            },
            _ => DateTime::<Utc>::MIN_UTC,
        };
        NeptunClass {
            name: name.to_string(),
            code: code.to_string(),
            teachers,
            start,
            end,
            location,
        }
    }

    pub fn string_array(&self) -> [String; 5] {
        [
            self.name.to_owned(),
            self.code.to_owned(),
            format!(
                "{} - {}",
                self.start.time().format("%H:%M"),
                self.end.time().format("%H:%M")
            ),
            self.location.to_owned(),
            self.teachers.join(";"),
        ]
    }
}

impl fmt::Display for NeptunClass {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let string_array: [String; 5] = self.string_array();
        let mut disp_str = (0..self.name.len()).map(|_| "=").collect::<String>();
        disp_str.push('\n');
        disp_str.push_str(string_array.join("\n").as_str());
        disp_str.push('\n');
        disp_str.push_str(&(0..self.name.len()).map(|_| "=").collect::<String>());
        write!(f, "{}", disp_str)
    }
}
