use std::env::var;
use super::*;

pub struct Conf {
    pub ics_path: Option<String>,
}

pub fn parse_config() -> std::io::Result<Conf> {

    let config_home = var("XDG_CONFIG_HOME")
        .or_else(|_| var("HOME").map(|home| format!("{}/.config", home)));

    let mut reader = my_reader::BufReader::open(format!("{}/orarend.conf", config_home.unwrap()))?;
    let mut buffer = String::new();

    let mut conf = Conf {
        ics_path: None,
    };

    while let Some(line) = reader.read_line(&mut buffer) {
        let mut line_iter = line?.split("=");
        let key = line_iter.next().unwrap().trim();
        let val = line_iter.next().unwrap().trim();
        
        match key {
            "path" => {
                conf.ics_path = Some(val.to_string());
                return Ok(conf);
            },
            _ => (),
        }
    }

    Ok(conf)
    
}

mod my_reader {
    use std::{
        fs::File,
        io::{self, prelude::*},
    };

    pub struct BufReader {
        reader: io::BufReader<File>,
    }

    impl BufReader {
        pub fn open(path: impl AsRef<std::path::Path>) -> io::Result<Self> {
            let file = File::open(path)?;
            let reader = io::BufReader::new(file);

            Ok(Self { reader })
        }

        pub fn read_line<'buf>(
            &mut self,
            buffer: &'buf mut String,
        ) -> Option<io::Result<&'buf mut String>> {
            buffer.clear();

            self.reader
                .read_line(buffer)
                .map(|u| if u == 0 { None } else { Some(buffer) })
                .transpose()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_config_parser() {
        let c = parse_config();
        println!("{}", c.unwrap().ics_path);
    }
}
