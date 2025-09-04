use std::ops::{Not, Range};
use unicode_segmentation::UnicodeSegmentation;

use crate::editor::{
    annotated_line::{Annotation, AnnotationType},
    file_type::FileType,
    line::{ByteIndex, GraphemeIndex, Line},
    view::Location,
};

// fn identifier(str: &str) -> Self {
//     match str {
//         "i8" | "i16" | "i32" | "i64" | "i128" | "isize" | "u8" | "u16" | "u32" | "u64"
//         | "u128" | "usize" | "f32" | "f64" | "char" | "bool" | "String" | "Vec" | "Option"
//         | "Result" => TokenType::Type,
//
//         "async" | "await" | "dyn" | "as" | "break" | "const" | "continue" | "crate"
//         | "else" | "enum" | "extern" | "false" | "fn" | "for" | "if" | "impl" | "in"
//         | "let" | "loop" | "match" | "mod" | "move" | "mut" | "pub" | "ref" | "return"
//         | "self" | "Self" | "static" | "struct" | "super" | "trait" | "true" | "type"
//         | "unsafe" | "use" | "where" | "while" => TokenType::Keyword,
//
//         _ => TokenType::Bogus,
//     }
// }

#[derive(Default)]
pub struct Highlighter<'a> {
    file_type: FileType,
    query: Option<&'a str>,
    selected_match: Option<Location>,
    highlighting: Vec<Vec<Annotation>>,
}

impl<'a> Highlighter<'a> {
    pub fn new(
        len: usize,
        query: Option<&'a str>,
        selected_match: Option<Location>,
        file_type: FileType,
    ) -> Self {
        let mut highlighting = Vec::with_capacity(len);

        for _ in 0..len {
            highlighting.push(Vec::new());
        }

        Self {
            file_type,
            query,
            selected_match,
            highlighting,
        }
    }

    pub fn highlight(&mut self, row: usize, line: &Line) {
        self.matches(row, line);
        if self.file_type == FileType::Rust {
            self.rust_highlighting(row, line);
        }
    }

    fn rust_highlighting(&mut self, row: usize, line: &Line) {
        let string = line.get_string();
        let iter = string.split_word_bound_indices().peekable();

        let mut ignore = 0;
        for (i, word) in iter {
            if i < ignore {
                continue;
            }

            let ann = match word {
                "'" => Self::char_or_lifetime(&string[i..]),
                _ => match Self::first_char_of(word) {
                    ch if ch.is_ascii_digit() => Self::number(word),
                    _ => None,
                },
            };

            if let Some(ann) = ann {
                let start = ann.range.start.saturating_add(i);
                let end = ann.range.end.saturating_add(i);
                ignore = end;
                self.push_annotation(row, start..end, ann.ty);
            }
        }
    }

    fn char(line: &str) -> Option<Annotation> {
        let mut escaped = false;
        for (i, ch) in line.char_indices().skip(1) {
            match ch {
                '\\' => escaped = escaped.not(),
                '\'' if !escaped => {
                    return Some(Annotation {
                        range: 0..i.saturating_add(1),
                        ty: AnnotationType::Char,
                    });
                }
                _ => escaped = false,
            }
        }

        Some(Annotation {
            range: 0..line.len(),
            ty: AnnotationType::Char,
        })
    }

    fn lifetime(line: &str) -> Option<Annotation> {
        for (i, ch) in line.char_indices().skip(1) {
            match ch {
                '\'' => return None,
                ch if !ch.is_ascii_alphanumeric() && ch != '_' && i == 1 => return None,
                ch if !ch.is_ascii_alphanumeric() && ch != '_' => {
                    return Some(Annotation {
                        range: 0..i,
                        ty: AnnotationType::Lifetime,
                    });
                }
                _ => {}
            }
        }

        Some(Annotation {
            range: 0..line.len(),
            ty: AnnotationType::Lifetime,
        })
    }

    fn char_or_lifetime(line: &str) -> Option<Annotation> {
        let ach = Self::char(line);
        let lch = Self::lifetime(line);
        if lch.is_some() { lch } else { ach }
    }

    fn number(num: &str) -> Option<Annotation> {
        let mut base = 10;
        let mut dot = false;
        let mut one_more = false;

        let iter = num.chars().enumerate();
        for (i, ch) in iter {
            match ch {
                '_' => {}
                '.' if dot => return None,
                '.' => dot = true,
                'e' => {
                    dot = true;
                    one_more = true;
                }
                'b' | 'B' => {
                    if i != 1 {
                        return None;
                    }

                    base = 2;
                    one_more = true;
                }
                'o' | 'O' => {
                    if i != 1 {
                        return None;
                    }

                    base = 8;
                    one_more = true;
                }
                'x' | 'X' => {
                    if i != 1 {
                        return None;
                    }

                    base = 16;
                    one_more = true;
                }
                ch if !ch.is_digit(base) => return None,
                _ => one_more = false,
            }
        }

        (!one_more).then_some(Annotation {
            range: 0..num.len(),
            ty: AnnotationType::Number,
        })
    }

    fn first_char_of(word: &str) -> char {
        word.chars().next().unwrap_or_else(|| unreachable!())
    }

    fn matches(&mut self, row: usize, line: &Line) {
        if let Some(needle) = self.query {
            let end = line.get_string().len();
            let matches = line.find_all(needle, 0..end);

            for mat in matches {
                let from: ByteIndex = mat.0;
                let from_gr: GraphemeIndex = mat.1;

                let len: ByteIndex = needle.len();
                let to: ByteIndex = from.saturating_add(len);

                // TODO: there might be graphemes in the search term
                let to_gr: GraphemeIndex = from_gr.saturating_add(len);

                if let Some(on) = self.selected_match
                    && on.line_index == row
                    && on.grapheme_index >= from_gr
                    && on.grapheme_index < to_gr
                {
                    self.push_annotation(row, from..to, AnnotationType::SelectedMatch);
                } else {
                    self.push_annotation(row, from..to, AnnotationType::Match);
                }
            }
        }
    }

    pub fn get_annotations(&self, row: usize) -> &[Annotation] {
        &self.highlighting[row]
    }

    fn push_annotation(&mut self, row: usize, range: Range<ByteIndex>, ty: AnnotationType) {
        self.highlighting[row].push(Annotation { range, ty });
    }
}
