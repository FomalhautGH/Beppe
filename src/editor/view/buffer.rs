use crate::editor::{line::Line, view::file_info::FileInfo};

use super::Location;
use std::{
    fs::{self, File},
    io::{Error, ErrorKind, Write},
};

#[derive(Default)]
pub struct Buffer {
    pub lines: Vec<Line>,
    pub file_info: FileInfo,
    dirty: bool,
}

impl Buffer {
    pub fn load(file_path: &str) -> Result<Self, std::io::Error> {
        let lines: Vec<Line> = fs::read_to_string(file_path)?
            .lines()
            .map(Line::from)
            .collect();

        Ok(Self {
            lines,
            file_info: FileInfo::from(file_path),
            dirty: false,
        })
    }

    pub fn save(&mut self) -> Result<(), Error> {
        if let Some(file_path) = &self.file_info.path {
            let mut file = File::create(file_path)?;

            for line in &self.lines {
                writeln!(&mut file, "{line}")?;
            }

            self.dirty = false;
            Ok(())
        } else {
            Err(Error::new(ErrorKind::NotFound, "File name wasn't provided"))
        }
    }

    pub fn save_as(&mut self, file_name: &str) -> Result<(), Error> {
        self.file_info = FileInfo::from(file_name);
        self.save()
    }

    pub fn insert_char(&mut self, character: char, at: Location) {
        // If I'm in a valid line i need to insert the character inside otherwise i push another
        // line to the document
        self.dirty = true;
        if at.line_index == self.height() {
            self.lines.push(Line::from(&character.to_string()));
        } else {
            let line = self.lines.get_mut(at.line_index).unwrap();
            line.insert_char_at(at.grapheme_index, character);
        }
    }

    pub fn delete(&mut self, at: Location) {
        self.dirty = true;
        if let Some(line) = self.lines.get_mut(at.line_index) {
            if at.grapheme_index < line.grapheme_count() {
                line.remove_at(at.grapheme_index);
            } else if at.line_index.saturating_add(1) < self.height() {
                let next_line = self.lines.remove(at.line_index.saturating_add(1));
                self.lines[at.line_index].append(&next_line);
            }
        }
    }

    pub fn insert_newline(&mut self, at: Location) {
        self.dirty = true;
        if let Some(line) = self.lines.get_mut(at.line_index) {
            let rem = line.split_off(at.grapheme_index);
            self.lines.insert(at.line_index.saturating_add(1), rem);
        } else {
            self.lines.push(Line::default());
        }
    }

    pub fn search_forward(&self, needle: &str, start_location: Location) -> Option<Location> {
        let mut is_first = true;

        for (i, line) in self
            .lines
            .iter()
            .enumerate()
            .cycle()
            .skip(start_location.line_index)
            .take(self.lines.len().saturating_add(1))
        {
            let start = if is_first {
                is_first = false;
                start_location.grapheme_index
            } else {
                0
            };

            if let Some(index) = line.search_forward(needle, start) {
                return Some(Location {
                    grapheme_index: index,
                    line_index: i,
                });
            }
        }

        None
    }

    pub fn search_backwards(&self, needle: &str, start_location: Location) -> Option<Location> {
        let mut is_first = true;

        for (i, line) in self
            .lines
            .iter()
            .enumerate()
            .rev()
            .cycle()
            .skip(
                self.lines
                    .len()
                    .saturating_sub(start_location.line_index)
                    .saturating_sub(1),
            )
            .take(self.lines.len().saturating_add(1))
        {
            let end = if is_first {
                is_first = false;
                start_location.grapheme_index
            } else {
                line.grapheme_count()
            };

            if let Some(index) = line.search_backwards(needle, end) {
                return Some(Location {
                    grapheme_index: index,
                    line_index: i,
                });
            }
        }

        None
    }

    pub fn height(&self) -> usize {
        self.lines.len()
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }
}
