use std::collections::BTreeMap;

use rangemap::RangeMap;

use crate::structure::{self, Line};

pub fn parse(s: &str) -> structure::Text {
    structure::Text {
        text: s
            .to_string()
            .split('\n')
            .map(|text| Line {
                text: text.to_string(),
                words: RangeMap::new(),
                commentary: BTreeMap::new(),
            })
            .collect(),
    }
}
