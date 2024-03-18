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

    pub fn add(&mut self, cell: Cell) {
        if cell.width > self.widest_cell_length {
            self.widest_cell_length = cell.width;
        }
        self.width_sum += cell.width;
        self.cell_count += 1;
        self.cells.push(cell)
    }

    pub fn fit_into_width(&self, maximum_width: Width) -> Option<Display<'_>> {
        self.width_dimensions(maximum_width)
            .map(|dims| Display {
                grid:       self,
                dimensions: dims,
            })
    }

    fn column_widths(&self, num_lines: usize, num_columns: usize) -> Dimensions {
        let mut widths: Vec<Width> = repeat(0).take(num_columns).collect();
        for (index, cell) in self.cells.iter().enumerate() {
            let index = match self.options.direction {
                Direction::LeftToRight  => index % num_columns,
                Direction::TopToBottom  => index / num_lines,
            };
            widths[index] = max(widths[index], cell.width);
        }

        Dimensions { num_lines, widths }
    }

    fn theoretical_max_num_lines(&self, maximum_width: usize) -> usize {
        // TODO: Make code readable / efficient.
        let mut theoretical_min_num_cols = 0;
        let mut col_total_width_so_far = 0;

        let mut cells = self.cells.clone();
        cells.sort_unstable_by(|a, b| b.width.cmp(&a.width)); // Sort in reverse order

        for cell in &cells {
            if cell.width + col_total_width_so_far <= maximum_width {
                theoretical_min_num_cols += 1;
                col_total_width_so_far += cell.width;
            } else {
                let mut theoretical_max_num_lines = self.cell_count / theoretical_min_num_cols;
                if self.cell_count % theoretical_min_num_cols != 0 {
                    theoretical_max_num_lines += 1;
                }
                return theoretical_max_num_lines;
            }
            col_total_width_so_far += self.options.filling.width()
        }
        1
    }

    fn width_dimensions(&self, maximum_width: Width) -> Option<Dimensions> {
        if self.widest_cell_length > maximum_width {
            // Largest cell is wider than maximum width; it is impossible to fit.
            return None;
        }

        if self.cell_count == 0 {
            return Some(Dimensions { num_lines: 0, widths: Vec::new() });
        }

        if self.cell_count == 1 {
            let the_cell = &self.cells[0];
            return Some(Dimensions { num_lines: 1, widths: vec![ the_cell.width ] });
        }

        let theoretical_max_num_lines = self.theoretical_max_num_lines(maximum_width);
        if theoretical_max_num_lines == 1 {
            return Some(Dimensions {
                num_lines: 1,
                widths: self.cells.clone().into_iter().map(|cell| cell.width).collect()
            });
        }
        let mut smallest_dimensions_yet = None;
        for num_lines in (1 .. theoretical_max_num_lines).rev() {

            // The number of columns is the number of cells divided by the number
            // of lines, *rounded up*.
            let mut num_columns = self.cell_count / num_lines;
            if self.cell_count % num_lines != 0 {
                num_columns += 1;
            }
            let total_separator_width = (num_columns - 1) * self.options.filling.width();
            if maximum_width < total_separator_width {
                continue;
            }

            // Remove the separator width from the available space.
            let adjusted_width = maximum_width - total_separator_width;

            let potential_dimensions = self.column_widths(num_lines, num_columns);
            if potential_dimensions.widths.iter().sum::<Width>() < adjusted_width {
                smallest_dimensions_yet = Some(potential_dimensions);
            } else {
                return smallest_dimensions_yet;
            }
        }

        None
    }

}

pub struct Display<'grid> {
    grid: &'grid Grid,
    dimensions: Dimensions,
}