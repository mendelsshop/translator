use std::collections::HashMap;

use rangemap::RangeMap;

// Word(Word),
// Parenthesis(char, Vec<Section>),
// Sentence(Sentence),
// Points(HashMap<usize, Section>),
#[derive(Debug, Clone)]
pub struct Text {
    pub text: Vec<String>,
    pub words: RangeMap<(usize, usize), Word>,
    pub sentence_translation: HashMap<(usize, usize), String>,
    pub description: HashMap<(usize, usize), String>,
}

#[derive(Debug, Clone)]
pub struct Word {
    pub(crate) word: String,
    pub(crate) prounouciation: Option<String>,
    pub(crate) translation: Option<String>,
}
