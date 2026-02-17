use std::collections::HashMap;

struct Heading<T>(Option<String>, T);
struct Text(Heading<Vec<Chapter>>);

enum Punctuation {}
struct Word {
    word: String,
    prounouciation: Option<String>,
    translation: Option<String>,
    punctuation: Punctuation,
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

fn parse() -> Text {
    todo!()
}
