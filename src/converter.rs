use crate::structure::{self, Heading, Section, Word};

pub fn parse(s: &str) -> structure::Text {
    Heading(
        None,
        vec![Heading(
            None,
            vec![
                (Heading(
                    None,
                    // TODO: split white space removes white space but we eventually want to
                    // differentiate betweeen paragraphs
                    s.split_whitespace()
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
