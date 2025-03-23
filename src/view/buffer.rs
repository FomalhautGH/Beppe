use std::fs;

#[derive(Default)]
pub struct Buffer {
    pub lines: Vec<String>,
}

impl Buffer {
    pub fn load(path: &str) -> Result<Self, std::io::Error> {
        let lines = fs::read_to_string(path)?
            .lines()
            .map(String::from)
            .collect();
        Ok(Self { lines })
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }
}
