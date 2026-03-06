use core::fmt;
use std::collections::HashMap;

// TODO: maybe make sense to do a flat repr and not have to do this fancy cursor enum
pub(crate) type Heading<T> = (Option<String>, T);

#[derive(Debug, Clone)]
pub struct Text(pub Heading<Vec<Chapter>>);

impl fmt::Display for Text {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "")
    }
}

#[derive(Debug, Clone)]
pub(crate) enum Punctuation {}

#[derive(Debug, Clone)]
pub(crate) struct Word {
    pub(crate) word: String,
    pub(crate) prounouciation: Option<String>,
    pub(crate) translation: Option<String>,
    pub(crate) punctuation: Option<Punctuation>,
}

#[derive(Debug, Clone)]
pub(crate) enum Section {
    Word(Word),
    Parenthesis(char, Vec<Section>),
    Sentence {
        words: Vec<Word>,
        description: String,
    },
    Points(HashMap<usize, Section>),
}

#[derive(Debug, Clone)]
pub struct Paragraph(pub Heading<Vec<Section>>);

#[derive(Debug, Clone)]
pub struct Chapter(pub Heading<Vec<Paragraph>>);

#[derive(Debug, Clone)]
pub enum Cursor {
    // Editing character n of heading of translation
    Heading(usize),
    // Editing chapter n of translation
    Chapter(usize, ChapterCursor),
}

impl Default for Cursor {
    fn default() -> Self {
        Self::Heading(0)
    }
}

#[derive(Debug, Clone)]
pub enum ChapterCursor {
    // Editing character n of heading of chapter
    Heading(usize),
    // Editing paragraph n of chapter
    Paragraph(usize, ParagraphCursor),
}

#[derive(Debug, Clone)]
pub enum ParagraphCursor {
    // Editing character n of heading of paragraph
    Heading(usize),
    // Editing section n of paragraph
    Paragraph(usize, HeadedSectionCursor),
}
#[derive(Debug, Clone)]
pub enum HeadedSectionCursor {
    // Editing character n of section header
    Heading(usize),
    // Editing section
    Paragraph(SectionCursor),
}

#[derive(Debug, Clone)]
pub struct PointCursor(pub usize, pub Box<SectionCursor>);

#[derive(Debug, Clone)]
pub enum SectionCursor {
    Word(WordCursor),
    Points(PointCursor),
    Sentence(SentenceCursor),
    Parenthesis(ParenthesisCursor),
}
#[derive(Debug, Clone)]
pub enum ParenthesisCursor {
    Char,
    Section(usize, Box<SectionCursor>),
}
#[derive(Debug, Clone)]
pub enum SentenceCursor {
    Description(usize),
    Word(usize, WordCursor),
}
#[derive(Debug, Clone)]
pub enum WordCursor {
    Word(usize),
    Prounouciation(usize),
    Translation(usize),
    Punctuation(usize),
}
