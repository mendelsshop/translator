use std::{collections::HashMap, fmt};

type Heading<T> = (Option<String>, T);

#[derive(Debug, Clone)]
pub struct Text(Heading<Vec<Chapter>>);

impl fmt::Display for Text {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "")
    }
}

#[derive(Debug, Clone)]
enum Punctuation {}
#[derive(Debug, Clone)]
struct Word {
    word: String,
    prounouciation: Option<String>,
    translation: Option<String>,
    punctuation: Option<Punctuation>,
}
#[derive(Debug, Clone)]
enum Section {
    Word(Word),
    Parenthesis(char, Vec<Section>),
    Sentence {
        words: Vec<Word>,
        description: String,
    },
    Points(HashMap<usize, Section>),
}

#[derive(Debug, Clone)]
pub struct Paragraph(Heading<Vec<Section>>);
#[derive(Debug, Clone)]
pub struct Chapter(Heading<Vec<Paragraph>>);

fn parse(s: &str) -> Text {
    Text((
        None,
        vec![Chapter((
            None,
            vec![
                (Paragraph((
                    None,
                    s.split(' ')
                        .map(|t| {
                            Section::Word(Word {
                                word: t.to_string(),
                                prounouciation: None,
                                translation: None,
                                punctuation: None,
                            })
                        })
                        .collect(),
                ))),
            ],
        ))],
    ))
}
