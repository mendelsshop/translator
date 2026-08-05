use std::collections::BTreeMap;

use rangemap::RangeMap;
use serde::{Deserialize, Serialize};

// Word(Word),
// Parenthesis(char, Vec<Section>),
// Sentence(Sentence),
// Points(HashMap<usize, Section>),
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Text {
    pub text: Vec<Line>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Line {
    pub len: usize,
    pub text: String,
    pub words: RangeMap<usize, Word>,
    pub commentary: BTreeMap<usize, Commentary>,
}
impl Line {
    pub fn position_or_text_len(&self, position: usize) -> usize {
        (self.text.len().saturating_sub(1)).min(position)
    }

    pub fn get_commentary_mut(&mut self, position: usize) -> Option<&mut Commentary> {
        self.commentary
            .get_mut(&self.position_or_text_len(position))
    }
    pub fn get_commentary(&self, position: usize) -> Option<&Commentary> {
        self.commentary.get(&self.position_or_text_len(position))
    }

    pub fn get_commentary_unchecked(&self, position: usize) -> &Commentary {
        &self.commentary[&self.position_or_text_len(position)]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commentary {
    pub sentence_translation: Option<String>,
    pub description_paragraph: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Word {
    pub(crate) word: String,
    pub(crate) prounouciation: Option<String>,
    pub(crate) translation: Option<String>,
}
pub trait CharLength {
    fn char_len(&self) -> usize;
}

impl CharLength for &str {
    fn char_len(&self) -> usize {
        self.chars().count()
    }
}

impl CharLength for &Line {
    fn char_len(&self) -> usize {
        self.len
    }
}
impl CharLength for Line {
    fn char_len(&self) -> usize {
        self.len
    }
}
