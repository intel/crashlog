// Copyright (C) 2025 Intel Corporation
// SPDX-License-Identifier: MIT

use super::keys;
use super::{Pager, PagerMode};
use core::fmt::Write;
use uefi::proto::console::text::Color;

const SEARCH_PATTERN_SIZE: usize = 32;

impl Pager {
    fn render_line(&mut self, idx: usize) -> uefi::Result {
        if let Some(line) = self.lines.get(idx) {
            let line = &line[0..self.window.columns.min(line.len())];

            if self.search_pattern.is_empty() {
                let _ = writeln!(self.output, "{line}");
                return Ok(());
            }

            for s in line.split_inclusive(&self.search_pattern) {
                self.output.set_color(Color::LightGray, Color::Black)?;
                let _ = write!(self.output, "{}", s.trim_end_matches(&self.search_pattern));

                if s.ends_with(&self.search_pattern) {
                    self.output.set_color(Color::Yellow, Color::Black)?;
                    let _ = write!(self.output, "{}", &self.search_pattern);
                }
            }

            self.output.set_color(Color::LightGray, Color::Black)?;
            let _ = writeln!(self.output);
        } else {
            let _ = writeln!(self.output, "~");
        }

        Ok(())
    }

    fn render_page(&mut self) -> uefi::Result {
        for idx in self.window.line..self.window.line + self.window.rows - 1 {
            self.render_line(idx)?
        }
        Ok(())
    }

    fn render_key_descr(
        &mut self,
        primary_key: char,
        secondary_key: Option<char>,
        descr: &str,
    ) -> uefi::Result {
        self.output.set_color(Color::Yellow, Color::Black)?;
        let _ = if let Some(secondary_key) = secondary_key {
            write!(self.output, "{primary_key}/{secondary_key} ")
        } else {
            write!(self.output, "{primary_key} ")
        };
        self.output.set_color(Color::White, Color::Black)?;
        let _ = write!(self.output, "{descr} ");
        self.output.set_color(Color::LightGray, Color::Black)?;
        Ok(())
    }

    fn render_search_pattern(&mut self) -> uefi::Result {
        self.output.set_color(Color::Yellow, Color::Black)?;
        let _ = write!(self.output, "Search: ");
        self.output.set_color(Color::LightGray, Color::Black)?;
        let _ = if self.search_pattern.len() < SEARCH_PATTERN_SIZE {
            write!(self.output, "{}", self.search_pattern)
        } else {
            write!(
                self.output,
                "...{}",
                &self.search_pattern[self.search_pattern.len() - SEARCH_PATTERN_SIZE + 3..]
            )
        };
        Ok(())
    }

    fn render_line_status(&mut self) -> uefi::Result {
        let end = self.lines.len() - self.window.rows + 1;
        let percent = (100 * self.window.line.min(end)) / end;

        self.output
            .set_cursor_position(self.window.columns - 12, self.window.rows - 1)?;
        let _ = write!(self.output, "{:6} {:3}%", self.window.line + 1, percent);
        Ok(())
    }

    fn render_status_bar(&mut self) -> uefi::Result {
        match self.mode {
            PagerMode::View => {
                self.render_key_descr(keys::QUIT, None, "Quit")?;
                self.render_key_descr(keys::DOWN, Some(keys::UP), "Down/Up")?;
                self.render_key_descr(keys::TOP, Some(keys::BOTTOM), "Top/Bottom")?;
                self.render_key_descr(keys::SEARCH, None, "Search")?;
                if !self.search_pattern.is_empty() {
                    self.render_key_descr(keys::NEXT, Some(keys::PREV), "Next/Previous")?;
                }
                self.render_line_status()?
            }
            PagerMode::Search => self.render_search_pattern()?,
        }

        Ok(())
    }

    pub(super) fn clear_status_bar(&mut self) -> uefi::Result {
        self.output.set_cursor_position(0, self.window.rows - 1)?;
        for _ in 0..self.window.columns - 1 {
            let _ = write!(self.output, " ");
        }
        self.output.set_cursor_position(0, self.window.rows - 1)?;
        Ok(())
    }

    pub(super) fn render(&mut self) -> uefi::Result {
        self.output.clear()?;
        self.render_page()?;
        self.render_status_bar()?;
        Ok(())
    }
}
