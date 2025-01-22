use chrono::{DateTime, Datelike, NaiveDate, NaiveTime, TimeDelta, Utc, Weekday};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::style::palette::tailwind;
use ratatui::{
    layout::{Constraint, Layout, Margin, Rect},
    prelude::Direction,
    style::{Color, Modifier, Style, Stylize},
    text::Text,
    widgets::{
        Block, BorderType, Borders, Cell, HighlightSpacing, Paragraph, Row, Scrollbar,
        ScrollbarOrientation, ScrollbarState, Table, TableState, Wrap,
    },
    DefaultTerminal, Frame,
};
use std::io::Result;

use icalendar::{
    Calendar, CalendarComponent, Component, DatePerhapsTime,
    DatePerhapsTime::DateTime as IcalDateTime, EventLike,
};
use std::cmp::Ordering;
use std::fmt;
use std::fs::read_to_string;
use std::hash::Hash;
use std::hash::Hasher;

use unicode_segmentation::UnicodeSegmentation;

// const FILENAME: &'static str = "NeptunCalendarExport.ics";
const FILENAME: &'static str = "Karpatia_Ahol_Zug_az_a_4_folyo.mp3";
const ITEM_HEIGHT: usize = 4;
const INFO_TEXT: &str =
    "(Esc) kilépés | (↑) lépés felfelé | (↓) lépés lefelé | (←) előző nap | (→) következő nap";
const LONGEST_ITEMS_LENS: (u16, u16, u16, u16, u16) = (25, 20, 13, 17, 25);

enum CurrentScreen {
    FileSelect,
    FileNotFound,
    Main,
}

struct App {
    state: TableState,
    classes: Vec<NeptunClass>,
    selected_classes: usize,
    longest_items_lens: (u16, u16, u16, u16, u16), // name, code, duration, location, teachers
    scroll_state: ScrollbarState,
    colors: TableColors,
    selected_date: NaiveDate,
    current_screen: CurrentScreen,
}

impl<'a> App {
    fn new(calendar: Option<Calendar>) -> Self {
        let success: bool;
        let classes: Vec<NeptunClass>;
        if let Some(cal) = calendar {
            classes = get_classes(cal);
            success = true;
        } else {
            classes = Vec::new();
            success = false;
        }
        // let today: NaiveDate = chrono::offset::Local::now().date_naive();
        let today: NaiveDate = NaiveDate::from_ymd_opt(2024, 11, 20).unwrap();
        if success {
            Self {
                state: TableState::default().with_selected(0),
                classes: classes.clone(),
                longest_items_lens: LONGEST_ITEMS_LENS,
                scroll_state: ScrollbarState::new(0),
                selected_classes: 0,
                colors: TableColors::new(),
                selected_date: today,
                current_screen: CurrentScreen::Main,
            }
        } else {
            Self {
                state: TableState::default().with_selected(0),
                classes: classes.clone(),
                longest_items_lens: (25, 20, 13, 17, 25),
                scroll_state: ScrollbarState::new(0),
                selected_classes: 0,
                colors: TableColors::new(),
                selected_date: today,
                current_screen: CurrentScreen::FileNotFound,
            }
        }
    }

    fn get_classes_by_day(
        classes: &'a Vec<NeptunClass>,
        selected_date: &NaiveDate,
    ) -> Vec<&'a NeptunClass> {
        let mut daily_classes = classes
            .iter()
            .filter(|&x| x.start.date_naive() == *selected_date)
            .collect::<Vec<&NeptunClass>>();

        daily_classes.sort_unstable();
        daily_classes
    }

    fn index_of_ongoing(&self, selected_classes: &Vec<&'a NeptunClass>) -> Option<usize> {
        let time: NaiveTime = chrono::offset::Local::now().time();
        for i in 0..selected_classes.len() {
            if selected_classes[i].start.time() <= time && selected_classes[i].end.time() >= time {
                return Some(i);
            }
        }
        None
    }

    fn truncate_string(&self, str: &String, index: usize) -> String {
        let len = match index {
            0 => self.longest_items_lens.0,
            1 => self.longest_items_lens.1,
            2 => self.longest_items_lens.2,
            3 => self.longest_items_lens.3,
            4 => self.longest_items_lens.4,
            _ => 0,
        };
        if str.graphemes(true).count() > len as usize {
            let mut trunc_str: String = str.graphemes(true).take(len as usize).collect();
            trunc_str.push_str("...");
            trunc_str
        } else {
            str.to_string()
        }
    }

    fn next_day(&mut self) {
        self.selected_date += TimeDelta::days(1);
    }

    fn prev_day(&mut self) {
        self.selected_date -= TimeDelta::days(1);
    }

    pub fn next_row(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.selected_classes - 1 {
                    0
                } else {
                    i + 1
                }
            }
            _ => 0,
        };
        self.state.select(Some(i));
        self.scroll_state = self.scroll_state.position(i * ITEM_HEIGHT);
    }

    pub fn prev_row(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.selected_classes - 1
                } else {
                    i - 1
                }
            }
            _ => 0,
        };
        self.state.select(Some(i));
        self.scroll_state = self.scroll_state.position(i * ITEM_HEIGHT);
    }

    fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        loop {
            terminal.draw(|frame| self.draw(frame))?;

            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    // let shift_pressed: bool = key.modifiers.contains(KeyModifiers::SHIFT);
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                        KeyCode::Char('j') | KeyCode::Down => self.next_row(),
                        KeyCode::Char('k') | KeyCode::Up => self.prev_row(),
                        // KeyCode::Char('h') | KeyCode::Left if shift_pressed => self.prev_day(),
                        // KeyCode::Char('l') | KeyCode::Right if shift_pressed => self.next_day(),
                        KeyCode::Char('h') | KeyCode::Left => self.prev_day(),
                        KeyCode::Char('l') | KeyCode::Right => self.next_day(),
                        _ => {}
                    }
                }
            }
        }
    }

    fn draw(&mut self, frame: &mut Frame) {
        match self.current_screen {
            CurrentScreen::FileNotFound => self.render_file_not_found(frame),
            CurrentScreen::Main => {
                let vertical = &Layout::vertical([
                    Constraint::Length(4),
                    Constraint::Min(5),
                    Constraint::Length(4),
                    Constraint::Length(4),
                ]);
                let rects = vertical.split(frame.area());
                self.render_date_bar(frame, rects[0]);
                self.render_table(frame, rects[1]);
                self.render_scrollbar(frame, rects[1]);
                self.render_info_bar(frame, rects[2]);
                self.render_footer(frame, rects[3]);
            }
            CurrentScreen::FileSelect => {}
        }
    }

    fn render_file_not_found(&self, frame: &mut Frame) {
        let popup_block = Block::default()
            .title("Igen/Nem")
            .borders(Borders::NONE)
            .style(Style::default().bg(Color::DarkGray));

        let error_text = Text::from_iter([
            "A megadott fájl nem található, vagy nem megfelelő formátumú.",
            "Szeretnél megadni egy új elérési utat?",
        ])
        .style(Style::default().fg(Color::Red));

        let error_paragraph = Paragraph::new(error_text)
            .block(popup_block)
            .wrap(Wrap { trim: false });

        let area = centered_rect(60, 30, frame.area());
        frame.render_widget(error_paragraph, area);
    }

    fn render_date_bar(&mut self, frame: &mut Frame, area: Rect) {
        let info_footer = Paragraph::new(Text::from_iter([
            self.selected_date.format("%Y-%m-%d").to_string(),
            match self.selected_date.weekday() {
                Weekday::Mon => "Hétfő",
                Weekday::Tue => "Kedd",
                Weekday::Wed => "Szerda",
                Weekday::Thu => "Csütörtök",
                Weekday::Fri => "Péntek",
                Weekday::Sat => "Szombat",
                Weekday::Sun => "Vasárnap",
            }
            .to_string(),
        ]))
        .style(
            Style::new()
                .fg(self.colors.row_fg)
                .bg(self.colors.buffer_bg),
        )
        .centered()
        .block(
            Block::bordered()
                .border_type(BorderType::Double)
                .border_style(Style::new().fg(self.colors.footer_border_color)),
        );
        frame.render_widget(info_footer, area);
    }

    fn render_table(&mut self, frame: &mut Frame, area: Rect) {
        let header_style = Style::default()
            .fg(self.colors.header_fg)
            .bg(self.colors.header_bg);
        let selected_row_style = Style::default()
            .add_modifier(Modifier::REVERSED)
            .fg(self.colors.selected_row_style_fg);
        /*
        let selected_col_style = Style::default().fg(self.colors.selected_column_style_fg);
        let selected_cell_style = Style::default()
            .add_modifier(Modifier::REVERSED)
            .fg(self.colors.selected_cell_style_fg);
        */
        let selected_classes = App::get_classes_by_day(&self.classes, &self.selected_date);
        self.selected_classes = selected_classes.len();

        let header = ["Név", "Kód", "Időpont", "Terem", "Tanárok"]
            .into_iter()
            .map(Cell::from)
            .collect::<Row>()
            .style(header_style)
            .height(1);
        let ongoing_idx = self.index_of_ongoing(&selected_classes).unwrap_or(0);
        let rows = selected_classes.iter().enumerate().map(|(i, data)| {
            let color = if i == ongoing_idx {
                self.colors.ongoing_class
            } else {
                match i % 2 {
                    0 => self.colors.normal_row_color,
                    _ => self.colors.alt_row_color,
                }
            };
            let item = data.string_array();
            item.into_iter()
                .enumerate()
                .map(|(i, content)| {
                    Cell::from(Text::from(format!(
                        "\n{}\n",
                        self.truncate_string(&content, i)
                    )))
                })
                .collect::<Row>()
                .style(Style::new().fg(self.colors.row_fg).bg(color))
                .height(4)
        });
        // let bar = " █ ";
        let t = Table::new(
            rows,
            [
                Constraint::Min(self.longest_items_lens.0 + 1),
                Constraint::Length(self.longest_items_lens.1 + 1),
                Constraint::Length(self.longest_items_lens.2 + 1),
                Constraint::Length(self.longest_items_lens.3 + 1),
                Constraint::Min(self.longest_items_lens.4),
            ],
        )
        .header(header)
        .row_highlight_style(selected_row_style)
        // .column_highlight_style(selected_col_style)
        // .cell_highlight_style(selected_cell_style)
        .highlight_symbol(Text::from(vec!["".into(), "⮞".into(), "".into()]))
        .bg(self.colors.buffer_bg)
        .highlight_spacing(HighlightSpacing::Always);
        frame.render_stateful_widget(t, area, &mut self.state);
    }

    fn render_scrollbar(&mut self, frame: &mut Frame, area: Rect) {
        frame.render_stateful_widget(
            Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(None)
                .end_symbol(None),
            area.inner(Margin {
                vertical: 1,
                horizontal: 1,
            }),
            &mut self.scroll_state,
        );
    }

    fn render_info_bar(&self, frame: &mut Frame, area: Rect) {
        let selected_classes = App::get_classes_by_day(&self.classes, &self.selected_date);
        let info = match self.state.selected() {
            Some(i) => {
                let str_arr = selected_classes[i].string_array();
                [str_arr[0].clone(), str_arr[4].clone()]
            }
            _ => ["".to_owned(), "".to_owned()],
        };

        let info_bar = Paragraph::new(Text::from_iter(info))
            .style(
                Style::new()
                    .fg(self.colors.row_fg)
                    .bg(self.colors.buffer_bg),
            )
            .centered()
            .block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .border_style(Style::new().fg(self.colors.footer_border_color)),
            );
        frame.render_widget(info_bar, area);
    }

    fn render_footer(&self, frame: &mut Frame, area: Rect) {
        let info_footer = Paragraph::new(Text::from(INFO_TEXT))
            .style(
                Style::new()
                    .fg(self.colors.row_fg)
                    .bg(self.colors.buffer_bg),
            )
            .centered()
            .block(
                Block::bordered()
                    .border_type(BorderType::Double)
                    .border_style(Style::new().fg(self.colors.footer_border_color)),
            );
        frame.render_widget(info_footer, area);
    }
}

struct TableColors {
    buffer_bg: Color,
    header_bg: Color,
    header_fg: Color,
    row_fg: Color,
    selected_row_style_fg: Color,
    // selected_column_style_fg: Color,
    // selected_cell_style_fg: Color,
    normal_row_color: Color,
    alt_row_color: Color,
    footer_border_color: Color,
    ongoing_class: Color,
}

impl TableColors {
    fn new() -> Self {
        Self {
            buffer_bg: tailwind::SLATE.c950,
            header_bg: tailwind::CYAN.c900,
            header_fg: tailwind::SLATE.c200,
            row_fg: tailwind::SLATE.c200,
            selected_row_style_fg: tailwind::CYAN.c400,
            // selected_column_style_fg: tailwind::CYAN.c400,
            // selected_cell_style_fg: tailwind::CYAN.c600,
            normal_row_color: tailwind::SLATE.c950,
            alt_row_color: tailwind::SLATE.c900,
            footer_border_color: tailwind::CYAN.c400,
            ongoing_class: tailwind::SLATE.c600,
        }
    }
}
/*
#[derive(Clone)]
struct NeptunClass<'a> {
    name: &'a str,
    code: &'a str,
    teachers: Vec<&'a str>,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    location: &'a str,
}
*/
#[derive(Clone)]
struct NeptunClass {
    name: String,
    code: String,
    teachers: Vec<String>,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
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
    fn new(
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

    fn string_array(&self) -> [String; 5] {
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

fn parse_calendar(filename: &str) -> Option<Calendar> {
    let file_contents_result = read_to_string(filename);
    let file_contents = match file_contents_result {
        Ok(string) => string,
        Err(err) => {
            println!("Could not read the file ({}): {}", filename, err);
            return None;
        }
    };
    let calendar_result = file_contents.parse();
    match calendar_result {
        Ok(cal) => Some(cal),
        Err(err) => {
            println!("Could not parse the file ({}): {}", filename, err);
            return None;
        }
    }
}

fn get_classes(cal: Calendar) -> Vec<NeptunClass> {
    let mut class_vec: Vec<NeptunClass> = Vec::new();

    for component in &cal.components {
        if let CalendarComponent::Event(event) = component {
            let event_summary: &str = event.get_summary().unwrap();
            if event_summary.contains("Tanóra") {
                let start: DatePerhapsTime = event.get_start().unwrap();
                let end: DatePerhapsTime = event.get_end().unwrap();
                let location: &str = event.get_location().unwrap();
                class_vec.push(NeptunClass::new(
                    event_summary.to_string(),
                    start,
                    end,
                    location.to_string(),
                ));
            }
        }
    }

    class_vec
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn main() -> Result<()> {
    let terminal = ratatui::init();
    let app_result = App::new(parse_calendar(FILENAME)).run(terminal);
    ratatui::restore();
    app_result
}
