use std::collections::BTreeMap;

use rangemap::RangeMap;

use crate::structure::{self, Line, Word};

pub fn parse(s: &str) -> structure::Text {
    structure::Text {
        text: s
            .split('\n')
            .map(|text| {
                let char_indices = text.char_indices();
                let mut words = RangeMap::new();
                let mut indicies_to_words = vec![];
                let state = char_indices
                    .clone()
                    .enumerate()
                    .filter(|(_, (_, char))| char.is_alphabetic())
                    .fold(
                        None::<(usize, usize, Vec<char>)>,
                        |state, (i, (_byte_i, char))| {
                            if let Some((start, stop, mut chars)) = state {
                                if stop + 1 == i {
                                    chars.push(char);
                                    Some((start, i, chars))
                                } else {
                                    indicies_to_words.push(start..(stop + 1));
                                    words.insert(
                                        start..(stop + 1),
                                        Word {
                                            word: chars.iter().collect(),
                                            prounouciation: None,
                                            translation: None,
                                        },
                                    );
                                    Some((i, i, vec![char]))
                                }
                            } else {
                                Some((i, i, vec![char]))
                            }
                        },
                    );

                if let Some((start, stop, chars)) = state {
                    indicies_to_words.push(start..(stop + 1));
                    words.insert(
                        start..(stop + 1),
                        Word {
                            word: chars.iter().collect(),
                            prounouciation: None,
                            translation: None,
                        },
                    );
                }
                Line {
                    indicies_to_words,
                    text: text.to_string(),
                    words,
                    commentary: BTreeMap::new(),
                    len: char_indices.count(),
                }
            })
            .collect(),
    }
}
