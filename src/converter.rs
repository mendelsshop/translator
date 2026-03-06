use crate::structure::{self, Chapter, Paragraph, Section, Word};

pub fn parse(s: &str) -> structure::Text {
    structure::Text((
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
