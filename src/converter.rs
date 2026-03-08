use crate::structure::{self, Heading, Section, Word};

pub fn parse(s: &str) -> structure::Text {
    Heading(
        None,
        vec![Heading(
            None,
            vec![
                (Heading(
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
                )),
            ],
        )],
    )
}
