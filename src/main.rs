mod icstime;
mod event_parser;

use ncurses::*;
use icstime::{day_to_dowstr, today_as_date_tuple};
use std::char;

fn main() {
    let events = event_parser::parse_events("/home/sanyi/Documents/orarend.ics");

    ncurses_init();

    attron(A_BOLD());
    addstr("Press Q to quit\n\n");
    attroff(A_BOLD());


    let mut d = today_as_date_tuple();
    let mut events_today = event_parser::get_events_by_date(&events, (d.0, d.1, d.2));

    loop {
        attron(A_BOLD());
        addstr(day_to_dowstr(d));
        addch('\n' as chtype);
        addch('\n' as chtype);
        attroff(A_BOLD());

        for event in &events_today{
            addstr(event.summary.as_str());
            addch('\n' as chtype);
            addch('\n' as chtype);
            addch('\n' as chtype);
        }
        let ch = getch();
        let ch = char::from_u32(ch as u32).expect("Invalid character!");
        match ch {
            'l' => {
                        d = next_day(d);
                        events_today = event_parser::get_events_by_date(&events, d);
                        ncurses_clean();
                    },
            'h' => {
                        d = prev_day(d);
                        events_today = event_parser::get_events_by_date(&events, d);
                        ncurses_clean();
                    },
            'q' => break,
            _   => (),
        };
        refresh();
    }

    ncurses_die();

}

fn next_day(d: (u32, u32, u32)) -> (u32, u32, u32){
    if d.1 == 4 {
        return d
    }
    (d.0, d.1, d.2+1)
}

fn prev_day(d: (u32, u32, u32)) -> (u32, u32, u32){
    if d.1 == 0 {
        return d
    }
    (d.0, d.1, d.2-1)
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
