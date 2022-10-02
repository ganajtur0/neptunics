// TODO:
// - fix issue with comparing TimeStamp to ICSTime
// - add config file

mod icstime;
mod event_parser;

use ncurses::*;
use icstime::{
    day_to_dowstr,
    today_as_date_tuple,
    TimeStamp,
    current_timestamp,
    next_day,
    prev_day
};
use std::char;
use clap::Parser;

#[derive(Parser)]
struct Cli {
    path: std::path::PathBuf,
}

fn main() {

    let args = Cli::parse();

    let events = event_parser::parse_events(&args.path);

    ncurses_init();

    attron(A_BOLD());
    addstr("Press Q to quit\n\n");
    attroff(A_BOLD());

    let mut d = today_as_date_tuple();
    let mut events_today = event_parser::get_events_by_date(&events, d);

    let timestamps: [TimeStamp; 6] = [
        TimeStamp{h:7,m:45},
        TimeStamp{h:9,m:30},
        TimeStamp{h:11,m:15},
        TimeStamp{h:13,m:15},
        TimeStamp{h:15,m:0},
        TimeStamp{h:16,m:45}
    ];

    loop {
        let cts = current_timestamp();
        attron(A_BOLD());
        addstr(day_to_dowstr(d));
        attroff(A_BOLD());
        ncurses_ch('\n', 2);


        let mut e_index = 0;

        for ts in timestamps {

            attron(A_UNDERLINE());
            addstr(ts.to_string().as_str());
            attroff(A_UNDERLINE());
            ncurses_ch(' ',2);

            if e_index < events_today.len(){
                let event = &events_today[e_index];

                if event.start == ts {
                    if event.start <= cts && event.end >= cts {
                        attron(A_STANDOUT());
                    } 
                    addstr(event.summary.as_str());
                    ncurses_ch('\n', 1);
                    addstr(event.location.as_str());
                    e_index+=1;
                    if event.start <= cts && event.end >= cts {
                        attroff(A_STANDOUT());
                    } 
                }
            
            }
            ncurses_ch('\n', 2);
        }

        let ch = getch();
        let ch = char::from_u32(ch as u32).expect("Invalid character!");

        match ch {
            'l' => {
                d = next_day(d);
                events_today = event_parser::get_events_by_date(&events, d);
            },
            'h' => {
                d = prev_day(d);
                events_today = event_parser::get_events_by_date(&events, d);
            },
            ' ' => {
                d = today_as_date_tuple();
                events_today = event_parser::get_events_by_date(&events, d);
            },
            'q' => break,
            _   => (),
        };

        ncurses_clean();
        refresh();
    }

    ncurses_die();

}

fn ncurses_ch(c: char, repeat: u8) {
    for _i in 0..repeat {
        addch(c as chtype);
    }
}

fn ncurses_clean() {
    mv(2,0);
    wclrtobot(stdscr());
}

fn ncurses_init() {
    initscr();
    raw();
    keypad(stdscr(), true);
    noecho();
}

fn ncurses_die() {
    endwin();
}
