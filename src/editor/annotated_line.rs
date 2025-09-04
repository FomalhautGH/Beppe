use crate::editor::{
    annotated_line_iterator::{AnnotatedLineIterator, AnnotatedLinePart},
    line::ByteIndex,
};
use std::ops::Range;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AnnotationType {
    None,
    Number,
    Keyword,
    Type,
    Match,
    Char,
    String,
    Lifetime,
    SelectedMatch,
}

#[derive(Debug)]
pub struct Annotation {
    pub range: Range<ByteIndex>,
    pub ty: AnnotationType,
}

#[derive(Default)]
pub struct AnnotatedLine {
    line: String,
    annotations: Vec<Annotation>,
}

impl AnnotatedLine {
    pub fn from(str: &str) -> Self {
        Self {
            line: str.to_owned(),
            annotations: Vec::new(),
        }
    }

    pub fn push_annotation(&mut self, range: Range<ByteIndex>, ty: AnnotationType) {
        if !range.is_empty() {
            self.annotations.push(Annotation { range, ty });
        }
    }

    pub fn push_annotations(&mut self, annotations: &[Annotation]) {
        for a in annotations {
            self.push_annotation(a.range.clone(), a.ty);
        }
    }

    pub fn append_str(&mut self, str: &str) {
        self.line.push_str(str);
    }

    pub fn replace(&mut self, range: Range<ByteIndex>, replacement: &str) {
        if range.is_empty() {
            return;
        }

        let prev_len = self.line.len();
        self.line.replace_range(range.clone(), replacement);
        let len = self.line.len();

        let diff = len.abs_diff(prev_len);
        if diff == 0 {
            return;
        }

        let widened = len > prev_len;
        for ann in &mut self.annotations {
            let ann_start = &mut ann.range.start;
            let ann_end = &mut ann.range.end;
            if *ann_end <= range.start {
                continue;
            }

            *ann_start = if *ann_start >= range.end {
                if widened {
                    ann_start.saturating_add(diff)
                } else {
                    ann_start.saturating_sub(diff)
                }
            } else {
                *ann_start
            };

            *ann_end = if widened {
                ann_end.saturating_add(diff)
            } else {
                ann_end.saturating_sub(diff)
            }
        }

        self.annotations.retain(|ann| !ann.range.is_empty());
    }

    pub fn get_line(&self) -> &str {
        &self.line
    }

    pub fn get_annotations(&self) -> &[Annotation] {
        &self.annotations
    }
}

impl<'a> IntoIterator for &'a AnnotatedLine {
    type Item = AnnotatedLinePart<'a>;
    type IntoIter = AnnotatedLineIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        AnnotatedLineIterator {
            annotated_line: self,
            index: 0,
        }
    }
}
