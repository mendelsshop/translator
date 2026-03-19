use core::fmt;
use std::collections::HashMap;

use rangemap::RangeMap;

// Word(Word),
// Parenthesis(char, Vec<Section>),
// Sentence(Sentence),
// Points(HashMap<usize, Section>),
#[derive(Debug, Clone)]
pub struct Text {
    pub text: String,
    pub words: RangeMap<usize, Word>,
    pub sentence_translation: HashMap<usize, String>,
    pub description: HashMap<usize, String>,
}

#[derive(Debug, Clone)]
pub(crate) struct Word {
    pub(crate) word: String,
    pub(crate) prounouciation: Option<String>,
    pub(crate) translation: Option<String>,
}

impl fmt::Display for Text {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = self.text.clone();
        self.description.iter().for_each(|(i, str)| {
            s.insert_str(*i, str);
        });
        write!(f, "{s}")
    }
}
