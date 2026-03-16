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

impl Punctuation {
    fn len(&self) -> usize {
        1
    }
}
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
#[derive(Debug, Clone, Copy)]
pub enum NextElement {
    // Jump around in editing word
    ToProunouciation,
    ToTranslation,
    ToPunctuation,
    ToWord,
    // Jump around edition parenthesis
    ToParenthesis,
    ToParenthesisContents,
    // Paragraph,
    // Chapter,
    // Sentence,
    // Point,
    // Word,
    None,
}
pub trait Next: CursorAble {
    // TODO: maybe all get the Edit at the new position
    fn next(&mut self, cursor: Self::Cursor, next: NextElement) -> Option<Self::Cursor>;
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

impl<T: CursorAble> CursorAble for Heading<T> {
    type Cursor = HeadingCursor<T::Cursor>;
}

impl<T: DefaultCursor> DefaultCursor for Heading<T> {
    fn cursor(&self) -> Self::Cursor {
        self.0
            .as_ref()
            // TODO: assumes > 1 content maybe make this optional
            .map_or(HeadingCursor::Inner(0, self.1[0].cursor()), |_| {
                HeadingCursor::Heading(0)
            })
    }
}

impl<IC, T: Next + DefaultCursor<Cursor = IC>> Next for Heading<T> {
    fn next(&mut self, cursor: Self::Cursor, next: NextElement) -> Option<Self::Cursor> {
        match cursor {
            HeadingCursor::Heading(position)
                if self
                    .0
                    .as_ref()
                    .is_some_and(|header| header.len() < position) =>
            {
                Some(HeadingCursor::Heading(position + 1))
            }

            // TODO: add parameter to say that to create if needed
            HeadingCursor::Heading(position) if self.1.is_empty() => None,
            HeadingCursor::Heading(_) => Some(HeadingCursor::Inner(0, self.1[0].cursor())),
            HeadingCursor::Inner(positon, cursor) => self
                .1
                .get_mut(positon)
                .and_then(|chapter| chapter.next(cursor, next))
                .map(|cursor| HeadingCursor::Inner(positon, cursor))
                .or_else(|| {
                    if positon + 1 < self.1.len() {
                        Some(HeadingCursor::Inner(
                            positon + 1,
                            self.1[positon + 1].cursor(),
                        ))
                    } else {
                        None
                    }
                }),
        }
    }
}
impl CursorAble for Word {
    type Cursor = WordCursor;
}
impl Next for Word {
    fn next(&mut self, cursor: Self::Cursor, _next: NextElement) -> Option<WordCursor> {
        match cursor {
            WordCursor::Word(_position) if _position < self.word.len() => {
                Some(WordCursor::Word(_position + 1))
            }
            WordCursor::Prounouciation(_position)
                if self
                    .prounouciation
                    .as_ref()
                    .is_some_and(|prounouciation| _position < prounouciation.len()) =>
            {
                Some(WordCursor::Prounouciation(_position + 1))
            }

            WordCursor::Translation(_position)
                if self
                    .translation
                    .as_ref()
                    .is_some_and(|translation| _position < translation.len()) =>
            {
                Some(WordCursor::Translation(_position + 1))
            }
            WordCursor::Punctuation(_position)
                if self
                    .punctuation
                    .as_ref()
                    .is_some_and(|punctuation| _position < punctuation.len()) =>
            {
                Some(WordCursor::Punctuation(_position + 1))
            }
            WordCursor::Translation(_position)
            | WordCursor::Word(_position)
            | WordCursor::Prounouciation(_position)
            | WordCursor::Punctuation(_position) => match _next {
                NextElement::ToProunouciation => {
                    self.prounouciation = Some(String::new());
                    Some(WordCursor::Prounouciation(0))
                }
                NextElement::ToTranslation => {
                    self.translation = Some(String::new());
                    Some(WordCursor::Translation(0))
                }
                NextElement::ToPunctuation => {
                    self.punctuation = Some(todo!());
                    Some(WordCursor::Punctuation(0))
                }
                NextElement::ToWord => Some(WordCursor::Word(0)),
                NextElement::ToParenthesis => None,
                NextElement::ToParenthesisContents => None,
                NextElement::None => None,
            },
        }
    }
}

impl CursorAble for Section {
    type Cursor = SectionCursor;
}
impl Next for Section {
    fn next(&mut self, chapter_cursor: SectionCursor, next: NextElement) -> Option<SectionCursor> {
        match (self, chapter_cursor) {
            (Section::Word(_word), SectionCursor::Word(_word_cursor)) => {
                _word.next(_word_cursor, next).map(SectionCursor::Word)
            }
            (
                Section::Parenthesis(_parenthesis, _sections),
                SectionCursor::Parenthesis(parenthesis_cursor),
            ) => match parenthesis_cursor {
                ParenthesisCursor::Char => match next {
                    NextElement::ToParenthesis => {
                        Some(SectionCursor::Parenthesis(ParenthesisCursor::Char))
                    }
                    NextElement::ToParenthesisContents => _sections.first()
                        .map(|section| section.cursor())
                        .map(|cursor| {
                            SectionCursor::Parenthesis(ParenthesisCursor::Section(
                                0,
                                Box::new(cursor),
                            ))
                        }),

                    NextElement::ToProunouciation
                    | NextElement::ToTranslation
                    | NextElement::ToPunctuation
                    | NextElement::ToWord
                    | NextElement::None => None,
                },
                ParenthesisCursor::Section(_cursor, _section_cursor) => _sections
                    .get_mut(_cursor)
                    .and_then(|section| section.next(*_section_cursor, next))
                    .or_else(|| _sections.get(_cursor + 1).map(|section| section.cursor()))
                    .or(match next {
                        NextElement::ToParenthesis => {
                            Some(SectionCursor::Parenthesis(ParenthesisCursor::Char))
                        }

                        NextElement::ToParenthesisContents
                        | NextElement::ToProunouciation
                        | NextElement::ToTranslation
                        | NextElement::ToPunctuation
                        | NextElement::ToWord
                        | NextElement::None => None,
                    }),
            },
            (Section::Sentence(_sentence), SectionCursor::Sentence(_sentence_cursor)) => _sentence
                .next(_sentence_cursor, next)
                .map(SectionCursor::Sentence),
            (Section::Points(_hash_map), SectionCursor::Points(_point_cursor)) => _hash_map
                .get_mut(&_point_cursor.0)
                .and_then(|section| section.next(*_point_cursor.1, next))
                .or_else(|| {
                    _hash_map
                        .get(&_point_cursor.0)
                        .map(|section| section.cursor())
                }),
            _ => None,
        }
    }
}

pub trait CursorAble {
    type Cursor;
}
pub trait DefaultCursor: CursorAble {
    fn cursor(&self) -> Self::Cursor;
}

impl DefaultCursor for Section {
    fn cursor(&self) -> Self::Cursor {
        match self {
            Section::Word(_) => SectionCursor::Word(WordCursor::Word(0)),
            Section::Parenthesis(_, _) => SectionCursor::Parenthesis(ParenthesisCursor::Char),
            Section::Sentence(heading) => SectionCursor::Sentence(heading.cursor()),
            // TODO: assumes > 1 points
            Section::Points(hash_map) => {
                SectionCursor::Points(PointCursor(0, Box::new(hash_map[&0].cursor())))
            }
        }
    }
}
impl DefaultCursor for Word {
    fn cursor(&self) -> Self::Cursor {
        WordCursor::Word(0)
    }
}
