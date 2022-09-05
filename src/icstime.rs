use std::fmt;
use std::cmp::Ordering;

#[derive(Debug, Default, Copy, Clone)]
pub struct ICSTime {
    pub year:  u32,
    pub month: u32,
    pub day:   u32,
    pub hour:  u32,
    pub min:   u32,
    pub dow:   u32
}

pub fn int_to_weekday(n: u32) -> &'static str {
    match n {
        0 => "Hétfő",
        1 => "Kedd",
        2 => "Szerda",
        3 => "Csütörtök",
        4 => "Péntek",
        5 => "Szombat",
        6 => "Vasárnap",
        _ => "",
    }
}

impl ICSTime {

    pub fn new(time_string: String) -> Self {

        let y = *(&time_string[..4].parse::<u32>().unwrap());
        let m = *(&time_string[4..6].parse::<u32>().unwrap());
        let d = *(&time_string[6..8].parse::<u32>().unwrap());
        let hr = *(&time_string[9..11].parse::<u32>().unwrap());
        let mi = *(&time_string[11..13].parse::<u32>().unwrap());

        let day_of_week = ICSTime::day_of_week(y,m,d);

        ICSTime {
            year: y,
            month: m,
            day: d,
            hour: hr,
            min: mi,
            dow: day_of_week,
        }
    }

    pub fn is_on_day(&self, y: u32, m: u32, d: u32) -> bool {
        if (self.year, self.month, self.day) == (y,m,d) {
            return true;
        }
        false
    }

    fn day_of_week(year: u32, m: u32, d: u32) -> u32 {
       let t: [u32; 12] = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
       let mut y = year;
       if m<3 {
            y-=1; 
       }
       let i: u32 = m-1;
       let i_usize: usize = i as usize;
       (y + y/4 - y/100 + y/400 + t[i_usize] + d + 6) % 7
    }
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

impl Eq for ICSTime {}

impl fmt::Display for ICSTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}, {}.{:0>2}.{:0>2}. {:0>2}:{:0>2}",
               int_to_weekday(self.dow),
               self.year,
               self.month,
               self.day,
               self.hour,
               self.min
               )
    }
}
