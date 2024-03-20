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
#[derive(PartialEq, Debug)]
pub struct Display<'grid> {
    grid: &'grid Grid,
    dimensions: Dimensions,
}

impl Display<'_> {
    pub fn width(&self) -> Width {
        self.dimensions.total_width(self.grid.options.filling.width())
    }

    pub fn row_count(&self) -> usize {
        self.dimensions.num_lines
    }

    pub fn is_complete(&self) -> bool {
        self.dimensions.widths.iter().all(|&x| x > 0)
    }
}


impl fmt::Display for Display<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        for y in 0 .. self.dimensions.num_lines {
            for x in 0 .. self.dimensions.widths.len() {
                let num = match self.grid.options.direction {
                    Direction::LeftToRight  => y * self.dimensions.widths.len() + x,
                    Direction::TopToBottom  => y + self.dimensions.num_lines * x,
                };

                // Abandon a line mid-way through if that’s where the cells end
                if num >= self.grid.cells.len() {
                    continue;
                }

                let cell = &self.grid.cells[num];
                if x == self.dimensions.widths.len() - 1 {
                    match cell.alignment {
                        Alignment::Left => {
                            // The final column doesn’t need to have trailing spaces,
                            // as long as it’s left-aligned.
                            write!(f, "{}", cell.contents)?;
                        },
                        Alignment::Right => {
                            let extra_spaces: usize = self.dimensions.widths[x] - cell.width;
                            write!(f, "{}", pad_string(&cell.contents, extra_spaces, Alignment::Right))?;
                        }
                    }
                }
                else {
                    assert!(self.dimensions.widths[x] >= cell.width);
                    match (&self.grid.options.filling, cell.alignment) {
                        (Filling::Spaces(n), Alignment::Left) => {
                            let extra_spaces = self.dimensions.widths[x] - cell.width + n;
                            write!(f, "{}", pad_string(&cell.contents, extra_spaces, cell.alignment))?;
                        },
                        (Filling::Spaces(n), Alignment::Right) => {
                            let s = spaces(*n);
                            let extra_spaces = self.dimensions.widths[x] - cell.width;
                            write!(f, "{}{}", pad_string(&cell.contents, extra_spaces, cell.alignment), s)?;
                        },
                        (Filling::Text(ref t), _) => {
                            let extra_spaces = self.dimensions.widths[x] - cell.width;
                            write!(f, "{}{}", pad_string(&cell.contents, extra_spaces, cell.alignment), t)?;
                        },
                    }
                }
            }

            writeln!(f)?;
        }

        Ok(())
    }
}

/// Pad a string with the given number of spaces.
fn spaces(length: usize) -> String {
    repeat(" ").take(length).collect()
}


fn pad_string(string: &str, padding: usize, alignment: Alignment) -> String {
    if alignment == Alignment::Left {
        format!("{}{}", string, spaces(padding))
    }
    else {
        format!("{}{}", spaces(padding), string)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn no_items() {
        let grid = Grid::new(GridOptions {
            direction:  Direction::TopToBottom,
            filling:    Filling::Spaces(2),
        });

        let display = grid.fit_into_width(40).unwrap();

        assert_eq!(display.dimensions.num_lines, 0);
        assert!(display.dimensions.widths.is_empty());

        assert_eq!(display.width(), 0);
    }


    #[test]
    fn one_item() {
        let mut grid = Grid::new(GridOptions {
            direction:  Direction::TopToBottom,
            filling:    Filling::Spaces(2),
        });

        grid.add(Cell::from("1"));

        let display = grid.fit_into_width(40).unwrap();

        assert_eq!(display.dimensions.num_lines, 1);
        assert_eq!(display.dimensions.widths, vec![ 1 ]);

        assert_eq!(display.width(), 1);
    }

    #[test]
    fn one_item_exact_width() {
        let mut grid = Grid::new(GridOptions {
            direction:  Direction::TopToBottom,
            filling:    Filling::Spaces(2),
        });

        grid.add(Cell::from("1234567890"));

        let display = grid.fit_into_width(10).unwrap();

        assert_eq!(display.dimensions.num_lines, 1);
        assert_eq!(display.dimensions.widths, vec![ 10 ]);

        assert_eq!(display.width(), 10);
    }

    #[test]
    fn one_item_just_over() {
        let mut grid = Grid::new(GridOptions {
            direction:  Direction::TopToBottom,
            filling:    Filling::Spaces(2),
        });

        grid.add(Cell::from("1234567890!"));

        assert_eq!(grid.fit_into_width(10), None);
    }

}



