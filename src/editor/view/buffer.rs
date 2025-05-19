use super::{Location, line::Line};
use std::{
    fs::{self, File},
    io::{Error, Write},
};

#[derive(Default)]
pub struct Buffer {
    pub lines: Vec<Line>,
    file_name: Option<String>,
}

impl Buffer {
    pub fn load(file_path: &str) -> Result<Self, Error> {
        let lines = fs::read_to_string(file_path)?
            .lines()
            .map(Line::from)
            .collect();

        Ok(Self {
            lines,
            file_name: Some(file_path.to_string()),
        })
    }

    pub fn save(&self) -> Result<(), Error> {
        if let Some(file_name) = &self.file_name {
            let mut file = File::create(file_name)?;
            for line in &self.lines {
                writeln!(&mut file, "{line}")?;
            }
        }

        Ok(())
    }

    pub fn insert_char(&mut self, character: char, at: Location) {
        // If I'm in a valid line i need to insert the character inside otherwise i push another
        // line to the document
        if at.line_index == self.height() {
            self.lines.push(Line::from(&character.to_string()));
        } else {
            let line = self.lines.get_mut(at.line_index).unwrap();
            line.insert_char_at(at.grapheme_index, character);
        }
    }

    pub fn delete(&mut self, at: Location) {
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
        if let Some(line) = self.lines.get_mut(at.line_index) {
            let rem = line.split_off(at.grapheme_index);
            self.lines.insert(at.line_index.saturating_add(1), rem);
        } else {
            self.lines.push(Line::default());
        }
        // let at = self.text_location.grapheme_index;
        // let line = self.text_location.line_index;
        //
        // if let Some(current_line) = self.current_line_mut(0) {
        //     let rem = current_line.split_off(at);
        //     self.handle_movement(Direction::Down);
        //     self.buffer.lines.insert(line.saturating_add(1), rem);
        //     self.handle_movement(Direction::Home);
        // } else {
        //     self.buffer.lines.push(Line::default());
        //     self.handle_movement(Direction::Down);
        // }
    }

    pub fn height(&self) -> usize {
        self.lines.len()
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }
}
