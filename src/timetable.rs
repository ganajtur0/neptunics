#![allow(dead_code)]
#![allow(unused_imports)]

use crate::NeptunClass;
use chrono::Datelike;
use ratatui::prelude::{Buffer, Frame, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::{
    canvas::{Canvas, Rectangle},
    StatefulWidget, StatefulWidgetRef, Widget, WidgetRef,
};

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
            .x_bounds([-45.0, 45.0])
            .y_bounds([-100.0, 100.0])
            .paint(|ctx| {
                ctx.draw(&Rectangle {
                    x: 10.0,
                    y: 10.0,
                    width: 10.0,
                    height: 20.0,
                    color: Color::Cyan,
                })
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
