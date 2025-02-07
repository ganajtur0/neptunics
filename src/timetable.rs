#![allow(dead_code)]
#![allow(unused_imports)]

use crate::NeptunClass;
use chrono::{Datelike, NaiveTime, TimeDelta, Timelike};
use ratatui::prelude::{Buffer, Frame, Rect};
use ratatui::style::{Color, Style};
use ratatui::symbols;
use ratatui::widgets::{
    canvas::{Canvas, Line, Rectangle},
    StatefulWidget, StatefulWidgetRef, Widget, WidgetRef,
};
use unicode_segmentation::UnicodeSegmentation;

const HOUR_SEVEN_AS_QUARTERS: u8 = 28;
const HOUR_TWENTY_AS_QUARTERS: u8 = 80;

pub struct TimeTableState {
    pub(crate) offset: usize,
    pub(crate) selected: Option<usize>,
}

impl Default for TimeTableState {
    fn default() -> Self {
        Self {
            offset: 0,
            selected: None,
        }
    }
}

pub struct TimeTable<'a> {
    classes: [Vec<&'a NeptunClass>; 5],
    style: Style,
    highlight_style: Style,
}

impl<'a> Default for TimeTable<'a> {
    fn default() -> Self {
        Self {
            classes: [const { Vec::new() }; 5],
            style: Style::new(),
            highlight_style: Style::new(),
        }
    }
}

impl<'a> TimeTable<'a> {
    pub fn from_classes(classes: Vec<&'a NeptunClass>) -> Self {
        let mut classes_array = [const { Vec::new() }; 5];

        for class in classes {
            classes_array[class.start.weekday().num_days_from_monday() as usize].push(class);
        }

        for class_vec in &mut classes_array {
            class_vec.sort()
        }

        Self {
            classes: classes_array,
            style: Style::new(),
            highlight_style: Style::new(),
        }
    }

    pub fn style<S: Into<Style>>(mut self, style: S) -> Self {
        self.style = style.into();
        self
    }

    pub fn highlight_style<S: Into<Style>>(mut self, style: S) -> Self {
        self.highlight_style = style.into();
        self
    }

    fn quarters_from_seven(class: &NeptunClass) -> u8 {
        let quarters_from_midnight = (class.start.hour() * 60 + class.start.minute()) / 15;
        quarters_from_midnight as u8 - HOUR_SEVEN_AS_QUARTERS
    }

    fn quarters_from_twenty(class: &NeptunClass) -> u8 {
        let quarters_from_midnight = (class.end.hour() * 60 + class.end.minute()) / 15;
        HOUR_TWENTY_AS_QUARTERS - quarters_from_midnight as u8
    }

    fn height_in_quarters(class: &NeptunClass) -> u8 {
        let start_as_quarters = (class.start.hour() * 60 + class.start.minute()) / 15;
        let end_as_quarters = (class.end.hour() * 60 + class.end.minute()) / 15;
        (end_as_quarters - start_as_quarters) as u8
    }
}

impl Widget for TimeTable<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        WidgetRef::render_ref(&self, area, buf);
    }
}

impl WidgetRef for TimeTable<'_> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let mut state = TimeTableState::default();
        StatefulWidget::render(self, area, buf, &mut state);
    }
}

impl StatefulWidget for TimeTable<'_> {
    type State = TimeTableState;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        StatefulWidgetRef::render_ref(&self, area, buf, state);
    }
}

impl StatefulWidget for &TimeTable<'_> {
    type State = TimeTableState;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        StatefulWidgetRef::render_ref(self, area, buf, state);
    }
}

impl StatefulWidgetRef for TimeTable<'_> {
    type State = TimeTableState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, _state: &mut Self::State) {
        let canvas = Canvas::default()
            .marker(symbols::Marker::HalfBlock)
            .x_bounds([0.0, 70.0])
            .y_bounds([0.0, 52.0])
            .paint(|ctx| {
                let mut x_coord = 5.0;
                for day in &self.classes {
                    for class in day {
                        let y_coord = f64::from(TimeTable::quarters_from_twenty(&class));
                        let height = f64::from(TimeTable::height_in_quarters(&class));
                        ctx.draw(&Rectangle {
                            x: x_coord,
                            y: y_coord,
                            height,
                            width: 7.0,
                            color: Color::Cyan,
                        });
                        ctx.print(
                            x_coord + 1.0,
                            y_coord + height - 2.0,
                            class.name.graphemes(true).take(10).collect::<String>(),
                        );
                    }
                    x_coord = x_coord + 10.0;
                }
                let mut time_iter = NaiveTime::from_hms_opt(21, 0, 0).unwrap();
                let quarter_delta = TimeDelta::minutes(15);
                for i in 0..=52 {
                    if time_iter.minute() == 0 {
                        ctx.print(0.0, i as f64, time_iter.format("%H:%M").to_string());
                    }
                    time_iter = time_iter - quarter_delta;
                }
            });
        canvas.render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::prelude::{Buffer, Rect};
    use ratatui::widgets::Widget;

    #[test]
    fn butterdog() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 15, 3));
        let timetable = TimeTable::default();
        Widget::render(timetable, Rect::new(0, 0, 15, 3), &mut buf);
        assert_eq!(true, true);
    }
}
