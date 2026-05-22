//! The FteTextLine DisplayObject, backing flash.text.engine TextLine.

#![allow(dead_code)]

use crate::font::FontLike;
use crate::html::LayoutLine;
use crate::string::WStr;
use swf::Twips;
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

fn split_atom_edges(x_min: Twips, x_max: Twips, kern_in: i32, kern_out: i32) -> (f32, f32) {
    let x = x_min - Twips::new(kern_in / 2);
    let x_max = x_max - Twips::new(kern_out / 2);
    (x.to_pixels() as f32, (x_max - x).to_pixels() as f32)
}

fn build_atoms(line: &LayoutLine<'_>, text: &WStr, text_block_begin: usize) -> Vec<Atom> {
    let range = line.text_range();
    let line_width = line.bounds().width().to_pixels() as f32;
    let word_bounds = word_boundary_offsets(text);

    let mut kern_after = vec![0i32; range.len()];
    for lbox in line.boxes_iter() {
        let Some((box_text, _tf, font, params, _color)) = lbox.as_renderable_text(text) else {
            continue;
        };
        if !params.kerning || !font.has_kerning_info() {
            continue;
        }
        let units: Vec<u16> = box_text.iter().collect();
        for k in 0..units.len().saturating_sub(1) {
            let left = char::from_u32(units[k] as u32).unwrap_or(char::REPLACEMENT_CHARACTER);
            let right = char::from_u32(units[k + 1] as u32).unwrap_or(char::REPLACEMENT_CHARACTER);
            let twips = font.pair_kerning(params, left, right).get();
            let Some(i) = (lbox.start() + k).checked_sub(range.start) else {
                continue;
            };
            if let Some(slot) = kern_after.get_mut(i) {
                *slot = twips;
            }
        }
    }

    let mut atoms = Vec::with_capacity(range.len());
    for (i, pos) in range.clone().enumerate() {
        let (x, width) = match line.char_bounds(pos) {
            Some(rect) => {
                let kern_in = if i > 0 { kern_after[i - 1] } else { 0 };
                split_atom_edges(rect.x_min, rect.x_max, kern_in, kern_after[i])
            }
            None => (line_width, 0.0),
        };
        atoms.push(Atom {
            char_start: text_block_begin + pos,
            char_end: text_block_begin + pos + 1,
            x,
            width,
            bidi_level: 0,
            word_boundary_on_left: pos == range.start
                || word_bounds.binary_search(&pos).is_ok(),
        });
    }
    atoms
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::string::WString;

    #[test]
    fn kern_split_makes_adjacent_atoms_tile_at_the_midpoint() {
        let (lx, lw) = split_atom_edges(Twips::new(0), Twips::new(240), 0, 40);
        let (rx, rw) = split_atom_edges(Twips::new(240), Twips::new(440), 40, 0);
        assert_eq!(lx + lw, rx);
        assert_eq!(lw, 11.0);
        assert_eq!(rw, 11.0);
    }

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
