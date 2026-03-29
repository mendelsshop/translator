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
