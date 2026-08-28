use std::collections::BTreeMap;

use rangemap::{RangeInclusiveMap, RangeMap};

use crate::structure::{self, Line, Word};

pub fn parse(s: &str) -> structure::Text {
    structure::Text {
        text: s
            .split('\n')
            .map(|text| {
                let char_indices = text.chars();
                let (mut words, state) = char_indices
                    .clone()
                    .enumerate()
                    .filter(|(_, char)| {
                        char.is_alphabetic()
                            || *char == '“'
                            || *char == '"'
                            || *char == '\''
                            || *char == '’'
                    })
                    .fold(
                        (
                            RangeInclusiveMap::new(),
                            None::<(usize, usize, Option<usize>, Vec<char>)>,
                        ),
                        |(mut map, state), (i, char)| {
                            log::info!("{i} {char} ");
                            if let Some((start, stop, last_stop, mut chars)) = state {
                                log::info!("{stop}");
                                if stop + 1 == i {
                                    chars.push(char);
                                    (map, Some((start, i, Some(stop), chars)))
                                } else {
                                    let range = if chars
                                        .pop_if(|char| *char == '“' || *char == '"')
                                        .is_some()
                                    {
                                        start..=(last_stop.unwrap())
                                    } else {
                                        start..=(stop)
                                    };
                                    log::info!("chars: {chars:?} {range:?}");
                                    map.insert(
                                        range,
                                        Word {
                                            word: chars.iter().collect(),
                                            prounouciation: None,
                                            translation: None,
                                        },
                                    );
                                    (
                                        map,
                                        if char == '“' || char == '"' {
                                            None
                                        } else {
                                            Some((i, i, None, vec![char]))
                                        },
                                    )
                                }
                            } else {
                                (
                                    map,
                                    if char == '“' || char == '"' {
                                        None
                                    } else {
                                        Some((i, i, None, vec![char]))
                                    },
                                )
                            }
                        },
                    );

                if let Some((start, stop, last_stop, mut chars)) = state {
                    let range = if chars.pop_if(|char| *char == '“' || *char == '"').is_some() {
                        start..=(last_stop.unwrap())
                    } else {
                        start..=(stop)
                    };
                    log::info!("chars: {chars:?} {range:?}");
                    words.insert(
                        range,
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
