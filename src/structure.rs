use core::fmt;
use std::collections::HashMap;

// TODO: maybe make sense to do a flat repr and not have to do this fancy cursor enum
#[derive(Debug, Clone)]
pub struct Heading<T>(pub Option<String>, pub Vec<T>);

pub type Text = Heading<Chapter>;
pub type Chapter = Heading<Paragraph>;
pub type Paragraph = Heading<Section>;
pub type Sentence = Heading<Word>;

pub type Cursor = HeadingCursor<ChapterCursor>;
pub type ChapterCursor = HeadingCursor<ParagraphCursor>;
pub type ParagraphCursor = HeadingCursor<SectionCursor>;
pub type SentenceCursor = HeadingCursor<WordCursor>;

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
    Sentence(Sentence),
    Points(HashMap<usize, Section>),
}
impl Get for Section {
    fn get_mut(&mut self, chapter_cursor: SectionCursor) -> Option<Edit<'_>> {
        match (self, chapter_cursor) {
            (Section::Word(word), SectionCursor::Word(word_cursor)) => word.get_mut(word_cursor),
            (
                Section::Parenthesis(parenthesis, sections),
                SectionCursor::Parenthesis(parenthesis_cursor),
            ) => match parenthesis_cursor {
                ParenthesisCursor::Char => Some(Edit {
                    position: 0,
                    kind: EditKind::Char(parenthesis),
                }),
                ParenthesisCursor::Section(cursor, section_cursor) => sections
                    .get_mut(cursor)
                    .and_then(|section| section.get_mut(*section_cursor)),
            },
            (Section::Sentence(sentence), SectionCursor::Sentence(sentence_cursor)) => {
                sentence.get_mut(sentence_cursor)
            }
            (Section::Points(hash_map), SectionCursor::Points(point_cursor)) => hash_map
                .get_mut(&point_cursor.0)
                .and_then(|s| s.get_mut(*point_cursor.1)),
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
pub enum WordCursor {
    Word(usize),
    Prounouciation(usize),
    Translation(usize),
    Punctuation(usize),
}

impl Get for Word {
    type Cursor = WordCursor;

    fn get_mut(&mut self, cursor: Self::Cursor) -> Option<Edit<'_>> {
        match cursor {
            WordCursor::Word(position) => Some(Edit {
                position,
                kind: EditKind::String(&mut self.word),
            }),
            WordCursor::Prounouciation(position) => Some(Edit {
                position,
                kind: EditKind::Option(&mut self.prounouciation),
            }),
            WordCursor::Translation(position) => Some(Edit {
                position,
                kind: EditKind::Option(&mut self.translation),
            }),
            WordCursor::Punctuation(position) => Some(Edit {
                position,
                kind: EditKind::Punctuation(&mut self.punctuation),
            }),
        }
    }
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
    Punctuation(&'a mut Option<Punctuation>),
}

pub trait Get {
    type Cursor;
    fn get_mut(&mut self, cursor: Self::Cursor) -> Option<Edit<'_>>;
}
pub trait Next {
    type Cursor;
    // TODO: maybe all get the Edit at the new position
    fn next(&mut self, cursor: Self::Cursor) -> Option<Self::Cursor>;
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
impl<T: Next> Next for Heading<T>
where
    T::Cursor: Default,
{
    type Cursor = HeadingCursor<T::Cursor>;
    fn next(&mut self, cursor: Self::Cursor) -> Option<Self::Cursor> {
        match cursor {
            HeadingCursor::Heading(position)
                if self.0.is_some_and(|header| header.len() < position) =>
            {
                Some(HeadingCursor::Heading(position + 1))
            }

            HeadingCursor::Heading(position) if self.1.is_empty() => None,
            HeadingCursor::Heading(position) => Some(HeadingCursor::Inner(0, T::Cursor::default())),
            HeadingCursor::Inner(positon, cursor) => self
                .1
                // TODO: maybe add some sort of Cursor trait
                // and then don't limit `Heading` contents to being a vec (and then we could have
                // HeadedSection)
                .get_mut(positon)
                .and_then(|chapter| chapter.next(cursor)),
        }
    }
}
