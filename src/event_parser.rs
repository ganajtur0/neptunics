use std::{fmt, u32};
use std::fs::File;
use std::io::{self, BufRead};
use std::cmp::Ordering;
use std::vec::Vec;
use crate::icstime::ICSTime;
// use chrono::prelude::*;

#[derive(Debug)]
pub struct Vevent {
    uid: u32,
    pub start: ICSTime,
    pub end: ICSTime,
    pub location: String,
    pub summary: String,
}

impl Ord for Vevent {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.start, self.end).cmp(&(other.start, other.end))
    }
}

impl PartialOrd for Vevent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering>{
        Some(self.cmp(other))
    }
}

impl PartialEq for Vevent {
    fn eq(&self, other: &Self) -> bool {
        self.uid == other.uid
    }
}

impl Eq for Vevent {}

pub fn get_events_by_date(events: &Vec<Vevent>, date_tuple: (u32, u32, u32) ) -> Vec<&Vevent> {
    
    let mut todays_events: Vec<&Vevent> = events
                                        .iter()
                                        .filter( |ev| ev.start.is_on_day(date_tuple))
                                        .collect::<Vec<&Vevent>>();
    todays_events.sort();
    todays_events

}

pub fn parse_events(path: &std::path::PathBuf) -> Vec<Vevent>{

    let file = match File::open(path){
        Err(why) => panic!("Could not open {}: {}", path.display(), why),
        Ok(file) => file,
    };
    let mut lines = io::BufReader::new(file).lines();
   
    let mut events: Vec<Vevent> = Vec::new();

    while let Some(Ok(row)) = lines.next() {
        let mut data = row.split(":");
        let key = data.next().unwrap();
        let value = data.next().unwrap();

        if key == "BEGIN" && value == "VEVENT" {

            let mut uid: u32 = 0;
            let mut start: ICSTime = ICSTime::default();
            let mut end: ICSTime = ICSTime::default();
            let mut location: String = "".to_string();
            let mut summary: String = "".to_string();

            while let Some(Ok(line)) = lines.next() {

                let mut data = line.split(":");
                let key = data.next().unwrap();
                let value = data.next().unwrap();

                match key {
                    "DTSTART" => start = ICSTime::new(value.to_string()),
                    "DTEND" => end = ICSTime::new(value.to_string()),
                    "LOCATION" => location = value.to_string(),
                    "SUMMARY"=> summary = (&value.to_string()[..value.len()-10]).to_string(),
                    "UID"=> uid = u32::from_str_radix(&value[6..11], 16).unwrap(),
                    "END" => { events.push(Vevent{
                                            uid: uid,
                                            start: start,
                                            end: end,
                                            location: location.clone(),
                                            summary: summary.clone(),
                                           });
                                break;
                             }
                    _ => (),
                }
            } 
        }
    }
    events
}

impl fmt::Display for Vevent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}\n{}\n{}\n{}\n{}",
                   self.uid,
                   self.summary,
                   self.location,
                   self.start,
                   self.end
              )
    }
}
