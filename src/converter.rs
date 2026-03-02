use std::collections::HashMap;

type Heading<T> = (Option<String>, T);
struct Text(Heading<Vec<Chapter>>);

enum Punctuation {}
struct Word {
    word: String,
    prounouciation: Option<String>,
    translation: Option<String>,
    punctuation: Option<Punctuation>,
}
enum Section {
    Word(Word),
    Parenthesis(char, Vec<Section>),
    Sentence {
        words: Vec<Word>,
        description: String,
    },
    Points(HashMap<usize, Section>),
}

pub struct Paragraph(Heading<Vec<Section>>);
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
