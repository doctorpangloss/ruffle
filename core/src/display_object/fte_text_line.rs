//! The FteTextLine DisplayObject, backing flash.text.engine TextLine.

#![allow(dead_code)]

use crate::string::WStr;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Clone, Copy, Debug)]
pub struct Atom {
    pub char_start: usize,
    pub char_end: usize,
    pub x: f32,
    pub width: f32,
    pub bidi_level: u8,
    pub word_boundary_on_left: bool,
}

fn word_boundary_offsets(text: &WStr) -> Vec<usize> {
    let utf8 = text.to_utf8_lossy();
    let mut prefix = vec![0usize; utf8.len() + 1];
    let mut utf16 = 0usize;
    let mut prev = 0usize;
    for (b, c) in utf8.char_indices() {
        for slot in prefix.iter_mut().take(b + 1).skip(prev) {
            *slot = utf16;
        }
        utf16 += c.len_utf16();
        prev = b + c.len_utf8();
    }
    for slot in prefix.iter_mut().skip(prev) {
        *slot = utf16;
    }

    let mut bounds: Vec<usize> = utf8
        .split_word_bound_indices()
        .map(|(b, _)| prefix[b])
        .collect();
    bounds.dedup();
    bounds
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::string::WString;

    #[test]
    fn word_boundary_offsets_finds_uax29_boundaries() {
        let text = WString::from_utf8("hi there");
        assert_eq!(word_boundary_offsets(&text), vec![0, 2, 3]);
    }

    #[test]
    fn word_boundary_offsets_counts_astral_chars_as_two_units() {
        let text = WString::from_utf8("\u{1F600} ok");
        assert_eq!(word_boundary_offsets(&text), vec![0, 2, 3]);
    }
}
