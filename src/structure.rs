use std::collections::BTreeMap;

use rangemap::RangeMap;

// Word(Word),
// Parenthesis(char, Vec<Section>),
// Sentence(Sentence),
// Points(HashMap<usize, Section>),
#[derive(Debug, Clone)]
pub struct Text {
    pub text: Vec<Line>,
}

#[derive(Debug, Clone)]
pub struct Line {
    pub text: String,
    pub words: RangeMap<usize, Word>,
    pub commentary: BTreeMap<usize, Commentary>,
}

#[derive(Debug, Clone)]
pub enum Commentary {
    SentenceTranslation(String),
    DescriptionParagraph(String),
}

#[derive(Debug, Clone)]
pub struct Word {
    pub(crate) word: String,
    pub(crate) prounouciation: Option<String>,
    pub(crate) translation: Option<String>,
}
