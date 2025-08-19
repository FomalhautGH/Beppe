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

    pub fn find_from(
        &self,
        needle: &str,
        mut start_location: Location,
        mut times: usize,
    ) -> Option<Location> {
        let mut iter = self
            .lines
            .iter()
            .skip(start_location.line_index)
            .enumerate()
            .peekable();

        while let Some(&(mut y, line)) = iter.peek() {
            let line_start = if y == 0 {
                start_location.grapheme_index
            } else {
                0
            };

            if let Some(x) = line.find(needle, line_start) {
                if times.checked_sub(1).is_none() {
                    y = y.saturating_add(start_location.line_index);
                    return Some(Location {
                        grapheme_index: x,
                        line_index: y,
                    });
                }

                start_location.grapheme_index =
                    line.next_index(start_location.grapheme_index).unwrap();
                times = times.saturating_sub(1);
            } else {
                iter.next();
            }
        }

        let head = self.lines.get(..start_location.line_index)?;
        let mut iter = head.iter().enumerate().peekable();

        while let Some(&(y, line)) = iter.peek() {
            if let Some(x) = line.find(needle, 0) {
                if times.checked_sub(1).is_none() {
                    return Some(Location {
                        grapheme_index: x,
                        line_index: y,
                    });
                }

                start_location.grapheme_index =
                    line.next_index(start_location.grapheme_index).unwrap();
                times = times.saturating_sub(1);
            } else {
                iter.next();
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
