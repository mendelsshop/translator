use core::fmt;
use std::collections::HashMap;

// TODO: maybe make sense to do a flat repr and not have to do this fancy cursor enum
#[derive(Debug, Clone)]
pub struct Heading<T>(pub Option<String>, pub Vec<T>);

pub type Text = Heading<Chapter>;
pub type Chapter = Heading<Paragraph>;
pub type Paragraph = Heading<Section>;

pub type Cursor = HeadingCursor<ChapterCursor>;
pub type ChapterCursor = HeadingCursor<ParagraphCursor>;
pub type ParagraphCursor = HeadingCursor<SectionCursor>;

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
impl Get for Section {
    fn get_mut(&mut self, chapter_cursor: SectionCursor) -> Option<Edit<'_>> {
        match (self, chapter_cursor) {
            (Section::Word(_word), SectionCursor::Word(_word_cursor)) => todo!(),
            (
                Section::Parenthesis(_, _sections),
                SectionCursor::Parenthesis(_parenthesis_cursor),
            ) => {
                todo!()
            }
            (
                Section::Sentence {
                    words: _,
                    description: _,
                },
                SectionCursor::Sentence(_sentence_cursor),
            ) => todo!(),
            (Section::Points(_hash_map), SectionCursor::Points(_point_cursor)) => todo!(),
            _ => None,
        }
    }

    type Cursor = SectionCursor;
}

#[derive(Debug, Clone)]
pub enum HeadingCursor<InnerCursor> {
    // Editing character n of heading of translation
    Heading(usize),
    // Editing part n of translation
    Inner(usize, InnerCursor),
}

impl<T> Default for HeadingCursor<T> {
    fn default() -> Self {
        Self::Heading(0)
    }
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

#[derive(Debug)]
pub struct Edit<'a> {
    position: usize,
    kind: EditKind<'a>,
}
#[derive(Debug)]
enum EditKind<'a> {
    String(&'a mut String),
    Char(&'a mut char),
    Option(&'a mut Option<String>),
}

pub trait Get {
    type Cursor;
    fn get_mut(&mut self, cursor: Self::Cursor) -> Option<Edit<'_>>;
}
impl<T: Get> Get for Heading<T> {
    type Cursor = HeadingCursor<T::Cursor>;
    fn get_mut(&mut self, cursor: Self::Cursor) -> Option<Edit<'_>> {
        match cursor {
            HeadingCursor::Heading(position) => Some(Edit {
                position,
                kind: EditKind::Option(&mut self.0),
            }),
            HeadingCursor::Inner(positon, cursor) => self
                .1
                // TODO: maybe add some sort of Cursor trait
                // and then don't limit `Heading` contents to being a vec (and then we could have
                // HeadedSection)
                .get_mut(positon)
                .and_then(|chapter| chapter.get_mut(cursor)),
        }
    }
}
