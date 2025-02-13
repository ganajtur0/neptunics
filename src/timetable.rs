#![allow(dead_code)]
#![allow(unused_imports)]

use crate::NeptunClass;
use chrono::{Datelike, NaiveTime, TimeDelta, Timelike};
use ratatui::layout::Alignment;
use ratatui::prelude::{Buffer, Frame, Rect};
use ratatui::style::{Color, Style};
use ratatui::symbols;
use ratatui::text::Line;
use ratatui::widgets::{
    canvas::{Canvas, Painter, Rectangle},
    StatefulWidget, StatefulWidgetRef, Widget, WidgetRef,
};
use unicode_segmentation::UnicodeSegmentation;

const HOUR_SEVEN_AS_QUARTERS: u8 = 28;
const HOUR_TWENTY_AS_QUARTERS: u8 = 76;

pub enum TimeTableNavigation {
    UP,
    DOWN,
    LEFT,
    RIGHT,
}

pub struct TimeTableState {
    pub(crate) selected_day: usize,
    pub(crate) selected_class: Option<usize>,
    pub(crate) index: Option<usize>,
    distribution: [usize; 5],
}

impl Default for TimeTableState {
    fn default() -> Self {
        Self {
            selected_day: 0,
            selected_class: None,
            distribution: [0; 5],
            index: None,
        }
    }
}

impl TimeTableState {
    pub fn navigate(&mut self, nav: TimeTableNavigation) {
        let selected_class: usize;
        match self.selected_class {
            Some(n) => selected_class = n,
            _ => {
                self.selected_class = Some(0);
                self.index = Some(0);
                return;
            }
        }
        match nav {
            TimeTableNavigation::UP => {
                if selected_class == 0 {
                    self.selected_class = Some(
                        self.distribution[self.selected_day]
                            .checked_sub(1)
                            .unwrap_or(0),
                    );
                } else {
                    self.selected_class = Some(selected_class - 1);
                }
            }
            TimeTableNavigation::DOWN => {
                if selected_class
                    == self.distribution[self.selected_day]
                        .checked_sub(1)
                        .unwrap_or(0)
                {
                    self.selected_class = Some(0);
                } else {
                    self.selected_class = Some(selected_class + 1);
                }
            }
            TimeTableNavigation::LEFT => {
                if self.selected_day == 0 {
                    self.selected_day = 4;
                } else {
                    self.selected_day -= 1;
                }
                self.selected_class = Some(0);
            }
            TimeTableNavigation::RIGHT => {
                if self.selected_day == 4 {
                    self.selected_day = 0;
                } else {
                    self.selected_day += 1;
                }
                self.selected_class = Some(0);
            }
        }
        self.index = Some(
            self.distribution[..self.selected_day]
                .into_iter()
                .sum::<usize>()
                + self.selected_class.unwrap_or(0),
        );
    }

    pub fn set_distribution(&mut self, tt: &TimeTable) {
        for i in 0..5 {
            self.distribution[i] = tt.classes[i].len();
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

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let canvas = Canvas::default()
            .marker(symbols::Marker::HalfBlock)
            .x_bounds([0.0, 70.0])
            .y_bounds([0.0, 52.0])
            .paint(|ctx| {
                let mut x_coord = 5.0;
                for (i, day) in self.classes.clone().into_iter().enumerate() {
                    for (j, class) in day.into_iter().enumerate() {
                        let y_coord = f64::from(TimeTable::quarters_from_twenty(&class));
                        let height = f64::from(TimeTable::height_in_quarters(&class));
                        let color: Color;
                        if let Some(n) = state.selected_class {
                            if state.selected_day == i && n == j {
                                color = Color::White;
                            } else {
                                color = Color::Cyan;
                            }
                        } else {
                            color = Color::Cyan;
                        }

                        ctx.draw(&Rectangle {
                            x: x_coord,
                            y: y_coord,
                            height,
                            width: 7.0,
                            color,
                        });
                        let mut text = class.name.graphemes(true).take(10).collect::<String>();
                        text.push_str("...");
                        ctx.print(x_coord + 1.0, y_coord + (height / 2.0), text);
                    }
                    x_coord = x_coord + 10.0;
                }
                let mut time_iter = NaiveTime::from_hms_opt(20, 0, 0).unwrap();
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
