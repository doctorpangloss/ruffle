//! The FteTextLine DisplayObject, backing flash.text.engine TextLine.

#![allow(dead_code)]

use crate::avm2::StageObject as Avm2StageObject;
use crate::context::UpdateContext;
use crate::display_object::interactive::InteractiveObjectBase;
use crate::display_object::{BoundsMode, DisplayObjectBase};
use crate::font::FontLike;
use crate::html::LayoutLine;
use crate::prelude::*;
use crate::string::{WStr, WString};
use crate::tag_utils::SwfMovie;
use core::fmt;
use gc_arena::barrier::unlock;
use gc_arena::lock::{Lock, RefLock};
use gc_arena::{Collect, Gc, Mutation};
use ruffle_common::utils::HasPrefixField;
use std::cell::Ref;
use std::sync::Arc;
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

fn combine_typo_metrics(
    per_font: impl Iterator<Item = (f32, f32)>,
    fallback: (f32, f32),
) -> (f32, f32) {
    let mut out: Option<(f32, f32)> = None;
    for (a, d) in per_font {
        out = Some(match out {
            Some((oa, od)) => (oa.max(a), od.max(d)),
            None => (a, d),
        });
    }
    out.unwrap_or(fallback)
}

fn typo_metrics(line: &LayoutLine<'_>, text: &WStr) -> (f32, f32) {
    let per_font = line.boxes_iter().filter_map(|lbox| {
        let (_, _, font_set, params, _) = lbox.as_renderable_text(text)?;
        let font = font_set.main_font();
        Some((
            font.typo_ascent(params.height()).to_pixels() as f32,
            font.typo_descent(params.height()).to_pixels() as f32,
        ))
    });
    combine_typo_metrics(
        per_font,
        (
            line.ascent().to_pixels() as f32,
            line.descent().to_pixels() as f32,
        ),
    )
}

fn trailing_trimmed_width(atoms: &[Atom], chars: &[u16], start: usize) -> f32 {
    for (i, atom) in atoms.iter().enumerate().rev() {
        let blank = match chars.get(start + i) {
            Some(&c) => matches!(c, 0x20 | 0x09 | 0x0a | 0x0d | 0x2028 | 0x2029),
            None => true,
        };
        if !blank {
            return atom.x + atom.width;
        }
    }
    0.0
}

#[derive(Collect)]
#[collect(no_drop)]
pub struct FteLine<'gc> {
    html_line: LayoutLine<'gc>,
    #[collect(require_static)]
    text: WString,
    #[collect(require_static)]
    atoms: Vec<Atom>,
    #[collect(require_static)]
    ascent: f32,
    #[collect(require_static)]
    descent: f32,
}

impl<'gc> FteLine<'gc> {
    pub fn new(html_line: LayoutLine<'gc>, text: WString, text_block_begin: usize) -> Self {
        let atoms = build_atoms(&html_line, &text, text_block_begin);
        let (ascent, descent) = typo_metrics(&html_line, &text);
        Self {
            html_line,
            text,
            atoms,
            ascent,
            descent,
        }
    }

    pub fn ascent(&self) -> f32 {
        self.ascent
    }

    pub fn descent(&self) -> f32 {
        self.descent
    }

    pub fn width(&self) -> f32 {
        self.html_line.bounds().width().to_pixels() as f32
    }

    pub fn text_width(&self) -> f32 {
        let start = self.html_line.text_range().start;
        let chars: Vec<u16> = self.text.iter().collect();
        trailing_trimmed_width(&self.atoms, &chars, start)
    }

    pub fn raw_text_length(&self) -> usize {
        self.html_line.text_range().len()
    }

    pub fn atoms(&self) -> &[Atom] {
        &self.atoms
    }
}

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct FteTextLine<'gc>(Gc<'gc, FteTextLineData<'gc>>);

impl fmt::Debug for FteTextLine<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FteTextLine")
            .field("ptr", &Gc::as_ptr(self.0))
            .finish()
    }
}

#[derive(Collect, HasPrefixField)]
#[collect(no_drop)]
#[repr(C, align(8))]
pub struct FteTextLineData<'gc> {
    base: InteractiveObjectBase<'gc>,
    avm2_object: Lock<Option<Avm2StageObject<'gc>>>,
    line: RefLock<FteLine<'gc>>,
    #[collect(require_static)]
    movie: Arc<SwfMovie>,
}

impl<'gc> FteTextLine<'gc> {
    pub fn new(context: &mut UpdateContext<'gc>, movie: Arc<SwfMovie>, line: FteLine<'gc>) -> Self {
        FteTextLine(Gc::new(
            context.gc(),
            FteTextLineData {
                base: Default::default(),
                avm2_object: Lock::new(None),
                line: RefLock::new(line),
                movie,
            },
        ))
    }

    pub fn line(self) -> Ref<'gc, FteLine<'gc>> {
        Gc::as_ref(self.0).line.borrow()
    }

    pub fn set_line(self, context: &mut UpdateContext<'gc>, line: FteLine<'gc>) {
        let mc = context.gc();
        unlock!(Gc::write(mc, self.0), FteTextLineData, line).replace(line);
    }
}

fn baseline_origin_bounds(width: f32, ascent: f32, descent: f32) -> Rectangle<Twips> {
    Rectangle {
        x_min: Twips::ZERO,
        x_max: Twips::from_pixels(width as f64),
        y_min: Twips::from_pixels(-(ascent as f64)),
        y_max: Twips::from_pixels(descent as f64),
    }
}

impl<'gc> TDisplayObject<'gc> for FteTextLine<'gc> {
    fn base(self) -> Gc<'gc, DisplayObjectBase<'gc>> {
        let interactive: Gc<'gc, InteractiveObjectBase<'gc>> = HasPrefixField::as_prefix_gc(self.0);
        HasPrefixField::as_prefix_gc(interactive)
    }

    fn instantiate(self, gc_context: &Mutation<'gc>) -> DisplayObject<'gc> {
        let borrowed = self.0.line.borrow();
        let cloned = FteTextLineData {
            base: Default::default(),
            avm2_object: Lock::new(None),
            line: RefLock::new(FteLine {
                html_line: borrowed.html_line.clone(),
                text: borrowed.text.clone(),
                atoms: borrowed.atoms.clone(),
                ascent: borrowed.ascent,
                descent: borrowed.descent,
            }),
            movie: self.0.movie.clone(),
        };
        drop(borrowed);
        Self(Gc::new(gc_context, cloned)).into()
    }

    fn id(self) -> CharacterId {
        0
    }

    fn movie(self) -> Arc<SwfMovie> {
        self.0.movie.clone()
    }

    fn replace_with(self, _context: &mut UpdateContext<'gc>, _id: CharacterId) {}

    fn self_bounds(self, _mode: BoundsMode) -> Rectangle<Twips> {
        let line = self.0.line.borrow();
        baseline_origin_bounds(line.width(), line.ascent(), line.descent())
    }

    fn hit_test_shape(
        self,
        _context: &mut UpdateContext<'gc>,
        point: Point<Twips>,
        options: HitTestOptions,
    ) -> bool {
        if options.contains(HitTestOptions::SKIP_INVISIBLE) && !self.visible() {
            return false;
        }
        self.world_bounds(BoundsMode::Engine).contains(point)
    }

    fn object1(self) -> Option<crate::avm1::Object<'gc>> {
        None
    }

    fn object2(self) -> Option<Avm2StageObject<'gc>> {
        self.0.avm2_object.get()
    }

    fn set_object2(self, context: &mut UpdateContext<'gc>, to: Avm2StageObject<'gc>) {
        let mc = context.gc();
        unlock!(Gc::write(mc, self.0), FteTextLineData, avm2_object).set(Some(to));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn self_bounds_place_the_origin_on_the_baseline() {
        let bounds = baseline_origin_bounds(120.0, 16.0, 4.0);
        assert_eq!(bounds.x_min, Twips::ZERO);
        assert_eq!(bounds.x_max, Twips::from_pixels(120.0));
        assert_eq!(bounds.y_min, Twips::from_pixels(-16.0));
        assert_eq!(bounds.y_max, Twips::from_pixels(4.0));
    }

    #[test]
    fn fte_text_line_handle_is_a_single_gc_pointer() {
        fn assert_copy<T: Copy>() {}
        assert_copy::<FteTextLine<'_>>();
        assert_eq!(
            std::mem::size_of::<FteTextLine<'_>>(),
            std::mem::size_of::<usize>(),
        );
    }

    #[test]
    fn typo_metrics_take_the_max_across_fonts_and_fall_back_when_empty() {
        let mixed = combine_typo_metrics([(10.0, 3.0), (14.0, 2.0)].into_iter(), (0.0, 0.0));
        assert_eq!(mixed, (14.0, 3.0));
        let empty = combine_typo_metrics(std::iter::empty(), (8.0, 2.5));
        assert_eq!(empty, (8.0, 2.5));
    }

    #[test]
    fn text_width_drops_trailing_whitespace() {
        let atom = |x: f32, width: f32| Atom {
            char_start: 0,
            char_end: 1,
            x,
            width,
            bidi_level: 0,
            word_boundary_on_left: false,
        };
        let atoms = [atom(0.0, 10.0), atom(10.0, 12.0), atom(22.0, 6.0)];
        let chars = [b'a' as u16, b'b' as u16, b' ' as u16];
        assert_eq!(trailing_trimmed_width(&atoms, &chars, 0), 22.0);

        let blanks = [atom(0.0, 6.0), atom(6.0, 6.0)];
        assert_eq!(trailing_trimmed_width(&blanks, &[0x20, 0x20], 0), 0.0);
    }

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
