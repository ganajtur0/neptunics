use std::ops;
use std::fmt;
use std::cmp::Ordering;
use chrono::prelude::*;
use chrono::Duration;

#[derive(Debug, Default, Copy, Clone)]
pub struct ICSTime {
    pub year:  u32,
    pub month: u32,
    pub day:   u32,
    pub hour:  u32,
    pub min:   u32,
    pub dow:   u32
}

#[derive(Copy, Clone)]
pub struct TimeStamp {
    pub h: u32,
    pub m: u32,
}

fn dow_to_str(n: u32) -> &'static str {
    match n {
        0 => "Hétfo",
        1 => "Kedd",
        2 => "Szerda",
        3 => "Csütörtök",
        4 => "Péntek",
        5 => "Szombat",
        6 => "Vasárnap",
        _ => "",
    }
}

pub fn day_to_dowstr(n: (u32, u32, u32)) -> &'static str{
    dow_to_str(day_of_week(n))
}

pub fn chrono_as_date_tuple(d: Date<Local>) -> (u32, u32, u32) {
    (d.year().try_into().unwrap(), d.month().try_into().unwrap(), d.day())
} 

pub fn today_as_date_tuple() -> (u32, u32, u32) {
    let today: Date<Local> = Local::now().date();
    chrono_as_date_tuple(today)
}

pub fn next_day(d: (u32, u32, u32)) -> (u32, u32, u32) {
    chrono_as_date_tuple(Local.ymd(d.0 as i32, d.1, d.2) + Duration::days(1))
}

pub fn prev_day(d: (u32, u32, u32)) -> (u32, u32, u32) {
    chrono_as_date_tuple(Local.ymd(d.0 as i32, d.1, d.2) - Duration::days(1))
}

pub fn current_timestamp() -> TimeStamp {
    let today: DateTime<Local> = Local::now();
    TimeStamp {h: today.hour(), m: today.minute()}
}

impl ops::Add<u32> for TimeStamp {
    type Output = TimeStamp;
    fn add(self, minute: u32) -> TimeStamp {
        let min = self.m + minute;
        if min >= 60 {
            return TimeStamp {
                      h: self.h + 1,
                      m: min-60,
                   }
        }
        TimeStamp {
            h: self.h,
            m: min
        }
    }
}

impl ops::AddAssign<u32> for TimeStamp {
    fn add_assign(&mut self, min: u32) {
        *self = *self+min;
    }
}

impl fmt::Display for TimeStamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:0>2}:{:0>2}", self.h, self.m)
    }
}

impl ICSTime {

    pub fn new(time_string: String) -> Self {

        let y = *(&time_string[..4].parse::<u32>().unwrap());
        let m = *(&time_string[4..6].parse::<u32>().unwrap());
        let d = *(&time_string[6..8].parse::<u32>().unwrap());
        let hr = *(&time_string[9..11].parse::<u32>().unwrap());
        let mi = *(&time_string[11..13].parse::<u32>().unwrap());

        let day_of_week = day_of_week((y,m,d));

        ICSTime {
            year: y,
            month: m,
            day: d,
            hour: hr+2,
            min: mi,
            dow: day_of_week,
        }
    }

    pub fn is_on_day(&self, date_tuple: (u32, u32, u32) ) -> bool {
        if (self.year, self.month, self.day) == date_tuple {
            return true;
        }
        false
    }
}

fn day_of_week(d: (u32, u32, u32)) -> u32 {
   let t: [u32; 12] = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
   let mut y = d.0;
   if d.1<3 {
        y-=1; 
   }
   let i: u32 = d.1-1;
   let i_usize: usize = i as usize;
   (y + y/4 - y/100 + y/400 + t[i_usize] + d.2 + 6) % 7
}

impl Ord for ICSTime {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.year, self.month, self.day, self.hour, self.min).cmp(&(other.year, other.month, other.day, other.hour, other.min))
    }
}

impl PartialOrd for ICSTime {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for ICSTime {
    fn eq(&self, other: &Self) -> bool{
        (self.year, self.month, self.day, self.hour, self.min) == (other.year, other.month, other.day, other.hour, other.min)
    }
}

impl PartialOrd<TimeStamp> for ICSTime {
    fn partial_cmp(&self, other: &TimeStamp) -> Option<Ordering> {
        Some((self.hour, self.min).cmp(&(other.h, other.m)))
    }
    fn le(&self, other: &TimeStamp) -> bool {
        self.hour + self.min*60 <= other.h+other.m*60
    }
    fn ge(&self, other: &TimeStamp) -> bool {
        self.hour+self.min*60 >= other.h+other.m*60
    }
}
impl PartialEq<TimeStamp> for ICSTime {
    fn eq(&self, other: &TimeStamp) -> bool {
        (self.hour, self.min) == (other.h, other.m)
    }
}

impl Eq for ICSTime {}

impl fmt::Display for ICSTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}, {}.{:0>2}.{:0>2}. {:0>2}:{:0>2}",
               dow_to_str(self.dow),
               self.year,
               self.month,
               self.day,
               self.hour,
               self.min
               )
    }
}
