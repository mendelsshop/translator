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
impl Line {
    pub fn position_or_text_len(&self, postion: usize) -> usize {
        (self.text.len().saturating_sub(1)).min(postion)
    }

    pub fn get_commentary_mut(&mut self, postion: usize) -> Option<&mut Commentary> {
        self.commentary.get_mut(&self.position_or_text_len(postion))
    }
    pub fn get_commentary(&self, postion: usize) -> Option<&Commentary> {
        self.commentary.get(&self.position_or_text_len(postion))
    }

    pub fn get_commentary_unchecked(&self, postion: usize) -> &Commentary {
        &self.commentary[&self.position_or_text_len(postion)]
    }
}

#[derive(Debug, Clone)]
pub struct Commentary {
    pub sentence_translation: Option<String>,
    pub description_paragraph: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Word {
    pub(crate) word: String,
    pub(crate) prounouciation: Option<String>,
    pub(crate) translation: Option<String>,
}
