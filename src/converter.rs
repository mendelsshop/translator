use std::collections::HashMap;

use rangemap::RangeMap;

use crate::structure::{self};

pub fn parse(s: &str) -> structure::Text {
    structure::Text {
        text: s.to_string().split('\n').map(ToString::to_string).collect(),
        words: RangeMap::new(),
        sentence_translation: HashMap::new(),
        description: HashMap::new(),
    }
}
