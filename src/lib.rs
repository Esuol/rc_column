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