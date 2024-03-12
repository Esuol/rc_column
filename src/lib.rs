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

impl<'a> From<&'a str> for Cell {
    fn from(string: &'a str) -> Self {
        Self {
            width: UnicodeWidthStr::width(&*string),
            contents: string.into(),
            alignment: Alignment::Left,
        }
    }
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Direction {
    LeftToRight,
    TopToBottom,
}

pub type Width = usize;

#[derive(PartialEq, Debug)]
pub enum Filling {
    Spaces(Width),
    Text(String),
}

impl Filling {
    fn width(&self) -> Width {
        match *self {
            Filling::Spaces(width) => width,
            // ref text 是一个模式，它匹配 Text 分支，并将其内部的字符串引用绑定到变量 text。
            Filling::Text(ref text) => UnicodeWidthStr::width(&text[..]),
        }
    }
}

#[derive(PartialEq, Debug)]
struct Dimensions {
    num_lines: Width,

    widths: Vec<Width>
}

impl Dimensions {
    fn total_width(&self, separator_width: Width) -> Width {
        if self.widths.is_empty() {
           0
        }
        else {
            let values = self.widths.iter().sum::<Width>();
            let separators = separator_width * (self.widths.len() - 1);

            values + separators
        }
    }
}

#[derive(PartialEq, Debug)]
pub struct GridOptions {
    filling: Filling,
    direction: Direction,
}

#[derive(PartialEq, Debug)]
pub struct Grid {
    options: GridOptions,
    cells: Vec<Cell>,
    widest_cell_length: Width,
    width_sum: Width,
    cell_count: usize,
}

impl Grid {
    pub fn new(options: GridOptions) -> Self {
        let cells = Vec::new();
        Self {
            options,
            cells,
            widest_cell_length: 0,
            width_sum: 0,
            cell_count: 0,
        }
    }

    pub fn reserve(&mut self, additional: usize) {
        self.cells.reserve(additional);
    }
}