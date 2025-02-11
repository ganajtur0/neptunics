mod neptunclass;
mod timetable;

use chrono::{Datelike, NaiveDate, NaiveTime, TimeDelta, Weekday};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::style::palette::tailwind;
use ratatui::{
    layout::{Constraint, Layout, Margin, Rect},
    prelude::Direction,
    style::{Color, Modifier, Style, Stylize},
    text::Text,
    widgets::{
        Block, BorderType, Cell, HighlightSpacing, Paragraph, Row, Scrollbar, ScrollbarOrientation,
        ScrollbarState, Table, TableState,
    },
    DefaultTerminal, Frame,
};
use std::io::Result;
use timetable::TimeTable;

use icalendar::{Calendar, CalendarComponent, Component, DatePerhapsTime, EventLike};
use std::fs::read_to_string;

use unicode_segmentation::UnicodeSegmentation;

use ratatui_explorer::FileExplorer;

use neptunclass::NeptunClass;

const FILENAME: &'static str = "NeptunCalendarExport.ics";
// const FILENAME: &'static str = "Karpatia_Ahol_Zug_az_a_4_folyo.mp3";
const ITEM_HEIGHT: usize = 4;
const MAIN_INFO_TEXT: &str =
    "(Esc) kilépés | (↑) lépés felfelé | (↓) lépés lefelé | (←) előző nap | (→) következő nap";
const FILE_NOT_FOUND_INFO_TEXT: &str = "(Esc) kilépés | (Enter) Új fájl kiválasztása";
const FILE_SELECT_INFO_TEXT: [&str; 2] = [
    "(Esc) kilépés | (↑) lépés felfelé | (↓) lépés lefelé ",
    "(Enter) könyvtár: belépés | (Enter) fájl: kiválasztás",
];
const LONGEST_ITEMS_LENS: (u16, u16, u16, u16, u16) = (25, 20, 13, 17, 25);

enum CurrentScreen {
    FileSelect,
    FileNotFound,
    DailyView,
    TimeTableView,
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
    file_explorer: FileExplorer,
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
        let today: NaiveDate = chrono::offset::Local::now().date_naive();
        // let today: NaiveDate = NaiveDate::from_ymd_opt(2024, 11, 20).unwrap();
        let file_explorer_theme = ratatui_explorer::Theme::default().add_default_title();
        if success {
            Self {
                state: TableState::default().with_selected(0),
                classes: classes.clone(),
                longest_items_lens: LONGEST_ITEMS_LENS,
                scroll_state: ScrollbarState::new(0),
                selected_classes: 0,
                colors: TableColors::new(),
                selected_date: today,
                current_screen: CurrentScreen::TimeTableView,
                file_explorer: FileExplorer::with_theme(file_explorer_theme).unwrap(),
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
                current_screen: CurrentScreen::FileSelect,
                file_explorer: FileExplorer::with_theme(file_explorer_theme).unwrap(),
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

    fn get_classes_by_week(
        classes: &'a Vec<NeptunClass>,
        selected_date: &NaiveDate,
    ) -> Vec<&'a NeptunClass> {
        let week_of_year = selected_date.iso_week().week();
        let mon = NaiveDate::from_isoywd_opt(selected_date.year(), week_of_year, Weekday::Mon)
            .unwrap_or(NaiveDate::MIN);
        let sun = NaiveDate::from_isoywd_opt(selected_date.year(), week_of_year, Weekday::Sun)
            .unwrap_or(NaiveDate::MAX);
        let mut weekly_classes = classes
            .iter()
            .filter(|&x| x.start.date_naive() >= mon && x.start.date_naive() <= sun)
            .collect::<Vec<&NeptunClass>>();
        weekly_classes.sort_unstable();
        weekly_classes
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

    fn try_to_parse_calendar(&mut self) {
        let path = self
            .file_explorer
            .current()
            .path()
            .as_path()
            .display()
            .to_string();
        let cal_opt = parse_calendar(path.as_str());
        match cal_opt {
            Some(cal) => {
                self.classes = get_classes(cal);
                self.current_screen = CurrentScreen::TimeTableView;
            }
            _ => self.current_screen = CurrentScreen::FileNotFound,
        }
    }

    fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        loop {
            terminal.draw(|frame| self.draw(frame))?;

            let event = event::read()?;
            if let Event::Key(key) = event {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Tab => match self.current_screen {
                            CurrentScreen::DailyView => {
                                self.current_screen = CurrentScreen::TimeTableView
                            }
                            CurrentScreen::TimeTableView => {
                                self.current_screen = CurrentScreen::DailyView
                            }
                            _ => {}
                        },
                        KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                        _ => {}
                    }
                    match self.current_screen {
                        // let shift_pressed: bool = key.modifiers.contains(KeyModifiers::SHIFT);
                        CurrentScreen::DailyView => match key.code {
                            // KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                            KeyCode::Char('j') | KeyCode::Down => self.next_row(),
                            KeyCode::Char('k') | KeyCode::Up => self.prev_row(),
                            KeyCode::Char('h') | KeyCode::Left => self.prev_day(),
                            KeyCode::Char('l') | KeyCode::Right => self.next_day(),
                            _ => {}
                        },
                        CurrentScreen::TimeTableView => match key.code {
                            // KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                            _ => {}
                        },
                        CurrentScreen::FileSelect => match key.code {
                            // KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                            KeyCode::Enter => {
                                if self.file_explorer.current().is_dir() {
                                    self.file_explorer.handle(&event)?;
                                } else {
                                    self.try_to_parse_calendar();
                                }
                            }
                            _ => self.file_explorer.handle(&event)?,
                        },
                        CurrentScreen::FileNotFound => match key.code {
                            KeyCode::Enter => self.current_screen = CurrentScreen::FileSelect,
                            // KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                            _ => {}
                        },
                    }
                }
            }
        }
    }

    fn draw(&mut self, frame: &mut Frame) {
        match self.current_screen {
            CurrentScreen::FileNotFound => {
                let vertical = &Layout::vertical([Constraint::Min(5), Constraint::Length(4)]);
                let rects = vertical.split(frame.area());
                self.render_file_not_found(frame, rects[0]);
                self.render_footer(frame, rects[1]);
            }
            CurrentScreen::DailyView => {
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
            CurrentScreen::TimeTableView => {
                self.render_timetable(frame, frame.area());
            }
            CurrentScreen::FileSelect => {
                let widget = self.file_explorer.widget();
                let vertical = &Layout::vertical([Constraint::Min(5), Constraint::Length(4)]);
                let rects = vertical.split(frame.area());
                frame.render_widget(&widget, rects[0]);
                self.render_footer(frame, rects[1]);
            }
        }
    }

    fn render_file_not_found(&self, frame: &mut Frame, area: Rect) {
        let sub_area = centered_rect(50, 15, area);
        let error_box = Paragraph::new(Text::from_iter([
            "A megadott fájl nem található, vagy nem megfelelő formátumú.",
            "Szeretnél megadni egy új elérési utat?",
        ]))
        .style(Style::new().fg(Color::Red))
        .centered()
        .block(
            Block::bordered()
                .title("HIBA")
                .border_type(BorderType::Double)
                .border_style(Style::new().fg(Color::Red))
                .style(Style::new().bg(self.colors.buffer_bg)),
        );
        frame.render_widget(error_box, sub_area);
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

    fn render_timetable(&mut self, frame: &mut Frame, area: Rect) {
        let selected_classes = App::get_classes_by_week(&self.classes, &self.selected_date);
        self.selected_classes = selected_classes.len();

        let tt = TimeTable::from_classes(selected_classes);
        frame.render_widget(&tt, area);
    }

    fn render_table(&mut self, frame: &mut Frame, area: Rect) {
        let header_style = Style::default()
            .fg(self.colors.header_fg)
            .bg(self.colors.header_bg);
        let selected_row_style = Style::default()
            .add_modifier(Modifier::REVERSED)
            .fg(self.colors.selected_row_style_fg);
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
        let text = match self.current_screen {
            CurrentScreen::DailyView => Text::from(MAIN_INFO_TEXT),
            CurrentScreen::TimeTableView => Text::from(""),
            CurrentScreen::FileNotFound => Text::from(FILE_NOT_FOUND_INFO_TEXT),
            CurrentScreen::FileSelect => Text::from_iter(FILE_SELECT_INFO_TEXT),
        };
        let info_footer = Paragraph::new(text)
            .style(
                Style::new()
                    .fg(self.colors.row_fg)
                    .bg(self.colors.buffer_bg),
            )
            .centered()
            .block(
                Block::bordered()
                    .border_type(BorderType::Double)
                    .border_style(Style::new().fg(match self.current_screen {
                        CurrentScreen::DailyView => self.colors.footer_border_color,
                        CurrentScreen::TimeTableView => Color::Magenta,
                        CurrentScreen::FileSelect => Color::White,
                        CurrentScreen::FileNotFound => Color::Red,
                    })),
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

fn parse_calendar(filename: &str) -> Option<Calendar> {
    let file_contents_result = read_to_string(filename);
    let file_contents = match file_contents_result {
        Ok(string) => string,
        Err(_err) => {
            return None;
        }
    };
    let calendar_result = file_contents.parse();
    match calendar_result {
        Ok(cal) => Some(cal),
        Err(_err) => {
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
