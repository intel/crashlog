// Copyright (C) 2025 Intel Corporation
// SPDX-License-Identifier: MIT

use super::Pager;
use super::input::Command;

impl Pager {
    fn search(&mut self, indexes: impl IntoIterator<Item = usize>) {
        for idx in indexes {
            if let Some(line) = self.lines.get(idx) {
                if line.contains(&self.search_pattern) {
                    self.window.line = idx;
                    return;
                }
            }
        }
    }

    fn move_window(&mut self, delta: i32) {
        self.window.line =
            (self.window.line as i32 + delta).clamp(0, self.lines.len() as i32 - 1) as usize;
    }

    pub(super) fn run(&mut self) -> uefi::Result {
        loop {
            self.render()?;

            match self.next_command()? {
                Command::Top => self.window.line = 0,
                Command::Bottom => {
                    self.window.line = self.lines.len().saturating_sub(self.window.rows - 1)
                }
                Command::Down => self.move_window(1),
                Command::Up => self.move_window(-1),
                Command::PageUp => self.move_window(-(self.window.rows as i32 - 2)),
                Command::PageDown => self.move_window(self.window.rows as i32 - 2),
                Command::Next => self.search(self.window.line + 1..self.lines.len()),
                Command::Previous => self.search((0..self.window.line).rev()),
                Command::Quit => {
                    self.clear_status_bar()?;
                    break;
                }
                _ => continue,
            }
        }

        Ok(())
    }
}
