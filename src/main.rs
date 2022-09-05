mod icstime;
mod event_parser;

fn main() {
    let events_today = event_parser::get_events_today(event_parser::parse_events("src/res/orarend.ics"));

    for event in events_today {
        println!("\n{}", event);        
    }
}


