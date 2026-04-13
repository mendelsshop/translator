use std::collections::BTreeMap;

use rangemap::RangeMap;

use crate::structure::{self, Line, Word};

pub fn parse(s: &str) -> structure::Text {
    structure::Text {
        text: s
            .split('\n')
            .map(|text| {
                let char_indices = text.char_indices();
                let (mut words, state) = char_indices
                    .clone()
                    .enumerate()
                    .filter(|(_, (_, char))| char.is_alphabetic())
                    .fold(
                        (RangeMap::new(), None::<(usize, usize, Vec<char>)>),
                        |(mut map, state), (i, (_byte_i, char))| {
                            if let Some((start, stop, mut chars)) = state {
                                if stop + 1 == i {
                                    chars.push(char);
                                    (map, Some((start, i, chars)))
                                } else {
                                    map.insert(
                                        start..(stop + 1),
                                        Word {
                                            word: chars.iter().collect(),
                                            prounouciation: None,
                                            translation: None,
                                        },
                                    );
                                    (map, Some((i, i, vec![char])))
                                }
                            } else {
                                (map, Some((i, i, vec![char])))
                            }
                        },
                    );

                if let Some((start, stop, chars)) = state {
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
                    text: text.to_string(),
                    words,
                    commentary: BTreeMap::new(),
                    len: char_indices.count(),
                }
            })
            .collect(),
    }
}
