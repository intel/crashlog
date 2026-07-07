// Copyright (C) 2026 Intel Corporation
// SPDX-License-Identifier: MIT

#[derive(Default)]
pub(crate) enum Alignment {
    #[default]
    Left,
    Right,
    Center,
}

#[derive(Default)]
pub(crate) struct Column {
    pub title: String,
    pub width: usize,
    pub alignment: Alignment,
}

impl Column {
    fn from_title(title: &str) -> Self {
        Self {
            title: title.to_string(),
            width: title.len(),
            ..Self::default()
        }
    }

    pub fn expand(&mut self, width: usize) {
        if width > self.width {
            self.width = width;
        }
    }
}

#[derive(Default)]
pub struct Row {
    pub cells: Vec<String>,
}

impl<const N: usize> From<[&str; N]> for Row {
    fn from(cells: [&str; N]) -> Self {
        Self {
            cells: cells.into_iter().map(String::from).collect(),
        }
    }
}

impl<const N: usize> From<[String; N]> for Row {
    fn from(cells: [String; N]) -> Self {
        Self {
            cells: cells.into_iter().collect(),
        }
    }
}

#[derive(Default)]
pub(crate) struct Table {
    pub columns: Vec<Column>,
    rows: Vec<Row>,
}

impl<const N: usize> From<[&str; N]> for Table {
    fn from(columns: [&str; N]) -> Self {
        Self {
            columns: columns.into_iter().map(Column::from_title).collect(),
            ..Self::default()
        }
    }
}

impl Table {
    pub fn append_row(&mut self, row: Row) {
        for (i, cell) in row.cells.iter().enumerate() {
            let width = cell.len();
            if let Some(column) = self.columns.get_mut(i) {
                column.expand(width);
            }
        }

        self.rows.push(row);
    }

    pub fn render(&self) {
        for column in self.columns.iter() {
            print!("{:width$}", column.title, width = column.width + 2);
        }
        println!();

        for column in self.columns.iter() {
            print!("{:->width$}", " ", width = column.width + 2);
        }
        println!();

        for row in self.rows.iter() {
            for (i, cell) in row.cells.iter().enumerate() {
                let Some(column) = self.columns.get(i) else {
                    break;
                };

                let width = column.width + 2;
                match column.alignment {
                    Alignment::Left => print!("{:width$}", cell),
                    Alignment::Right => print!("{:>width$} ", cell, width = width - 1),
                    Alignment::Center => print!("{:^width$}", cell),
                }
            }
            println!();
        }
    }

    pub fn render_markdown(&self) {
        print!("|");
        for column in self.columns.iter() {
            print!(" {:width$}|", column.title, width = column.width + 2);
        }
        println!();

        print!("|");
        for column in self.columns.iter() {
            print!(" {:->width$}|", " ", width = column.width + 2);
        }
        println!();

        for row in self.rows.iter() {
            print!("|");
            for (i, cell) in row.cells.iter().enumerate() {
                let Some(column) = self.columns.get(i) else {
                    break;
                };

                let width = column.width + 2;
                print!(" ");
                match column.alignment {
                    Alignment::Left => print!("{:width$}", cell),
                    Alignment::Right => print!("{:>width$} ", cell, width = width - 1),
                    Alignment::Center => print!("{:^width$}", cell),
                }
                print!("|");
            }
            println!();
        }
    }
}
