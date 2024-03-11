use std::cmp::max;
use std::fmt;
use std::iter::repeat;

extern crate unicode_width;
use unicode_width::UnicodeWidthStr;

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum Alignment {
    Left,
    Right,
}

#[derive(PartialEq, Debug, Clone)]
pub struct Cell {
    pub contents: String,
    pub width: usize,
    pub alignment: Alignment,
}

impl From<String> for Cell {
    fn from(string: String) -> Self {
        Self {
            width: UnicodeWidthStr::width(&*string),
            contents: string,
            alignment: Alignment::Left,
        }
    }
}