#![allow(dead_code)]

use crate::NeptunClass;
use ratatui::style::Style;

pub struct TimeTableState {
    pub(crate) offset: usize,
    pub(crate) selected: Option<usize>,
}

pub struct TimeTable<'a> {
    classes: &'a Vec<NeptunClass>,
    style: Style,
    highlight_style: Style,
}
