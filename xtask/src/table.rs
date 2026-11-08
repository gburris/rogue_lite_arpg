//! Plain-text tables with a header rule and auto-sized columns. A column whose cells are all
//! numbers is right-aligned.

use std::fmt::{self, Display, Formatter};

const COLUMN_GAP: &str = "  ";

/// Text shown for a missing value.
pub(super) const MISSING: &str = "-";

/// A table of text cells, printed with `{table}`.
pub(super) struct Table {
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
}

impl Table {
    pub(super) fn new<Header: Into<String>>(headers: impl IntoIterator<Item = Header>) -> Self {
        Self {
            headers: headers.into_iter().map(Into::into).collect(),
            rows: Vec::new(),
        }
    }

    /// Appends a row, which must have one cell per header.
    pub(super) fn add_row(&mut self, cells: Vec<String>) {
        assert_eq!(
            cells.len(),
            self.headers.len(),
            "a row needs one cell per column: {cells:?}"
        );
        self.rows.push(cells);
    }

    fn column_layout(&self, column: usize) -> ColumnLayout {
        let header_width = self.headers[column].chars().count();
        let widest_cell = self.cells_in(column).map(|cell| cell.chars().count()).max();

        ColumnLayout {
            width: header_width.max(widest_cell.unwrap_or(0)),
            is_numeric: self.cells_in(column).all(is_numeric),
        }
    }

    fn cells_in(&self, column: usize) -> impl Iterator<Item = &str> {
        self.rows.iter().map(move |row| row[column].as_str())
    }
}

impl Display for Table {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        let layouts = (0..self.headers.len())
            .map(|column| self.column_layout(column))
            .collect::<Vec<_>>();
        let rules = layouts
            .iter()
            .map(|layout| "-".repeat(layout.width))
            .collect::<Vec<_>>();

        write_line(formatter, &layouts, &self.headers)?;
        write_line(formatter, &layouts, &rules)?;
        for row in &self.rows {
            write_line(formatter, &layouts, row)?;
        }
        Ok(())
    }
}

/// How wide a column is and which side its cells line up on.
#[derive(Clone, Copy, Debug)]
struct ColumnLayout {
    width: usize,
    is_numeric: bool,
}

impl ColumnLayout {
    fn pad(self, cell: &str) -> String {
        let width = self.width;
        if self.is_numeric {
            format!("{cell:>width$}")
        } else {
            format!("{cell:<width$}")
        }
    }
}

fn write_line(
    formatter: &mut Formatter,
    layouts: &[ColumnLayout],
    cells: &[String],
) -> fmt::Result {
    let padded_cells = layouts
        .iter()
        .zip(cells)
        .map(|(layout, cell)| layout.pad(cell))
        .collect::<Vec<_>>();

    writeln!(formatter, "{}", padded_cells.join(COLUMN_GAP).trim_end())
}

/// A value as a cell, or [`MISSING`].
pub(super) fn optional_cell(value: Option<f64>) -> String {
    value.map_or_else(|| MISSING.to_owned(), |value| value.to_string())
}

/// A whole number with a comma between each group of three digits, such as `5,476`.
pub(super) fn with_thousands_separators(number: u64) -> String {
    let digits = number.to_string();
    let mut grouped = String::new();

    for (index, digit) in digits.chars().enumerate() {
        let digits_left = digits.len() - index;
        if index > 0 && digits_left % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(digit);
    }
    grouped
}

fn is_numeric(cell: &str) -> bool {
    cell == MISSING || cell.replace(',', "").parse::<f64>().is_ok()
}
