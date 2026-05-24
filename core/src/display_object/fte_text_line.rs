//! The FteTextLine DisplayObject, backing flash.text.engine TextLine.

use crate::avm2::StageObject as Avm2StageObject;
use crate::backend::ui::MouseCursor;
use crate::context::{RenderContext, UpdateContext};
use crate::display_object::interactive::{InteractiveObjectBase, TInteractiveObject};
use crate::display_object::{
    Avm2MousePick, BoundsMode, DisplayObjectBase, EditText, InteractiveObject,
};
use crate::events::{ClipEvent, ClipEventResult};
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
use ruffle_render::quality::StageQuality;
use ruffle_render::transform::Transform;
use std::cell::Ref;
use std::sync::Arc;

#[derive(Clone, Copy, Debug)]
pub struct Atom {
    pub char_start: usize,
    pub char_end: usize,
    pub x: f32,
    pub width: f32,
    pub word_boundary_on_left: bool,
}

#[derive(Collect)]
#[collect(no_drop)]
pub struct FteLine<'gc> {
    html_line: LayoutLine<'gc>,
    #[collect(require_static)]
    text: WString,
    text_block_begin: usize,
    #[collect(require_static)]
    atoms: Vec<Atom>,
    #[collect(require_static)]
    ascent: f32,
    #[collect(require_static)]
    descent: f32,
}

impl<'gc> FteLine<'gc> {
    pub fn new(html_line: LayoutLine<'gc>, text: WString, text_block_begin: usize) -> Self {
        let (ascent, descent) = typo_metrics(&html_line, &text);
        let atoms = html_line
            .text_range()
            .map(|pos| Atom {
                char_start: text_block_begin + pos,
                char_end: text_block_begin + pos + 1,
                x: html_line
                    .char_bounds(pos)
                    .map(|bounds| bounds.x_min.to_pixels() as f32)
                    .unwrap_or_else(|| html_line.bounds().width().to_pixels() as f32),
                width: html_line
                    .char_bounds(pos)
                    .map(|bounds| bounds.width().to_pixels() as f32)
                    .unwrap_or(0.0),
                word_boundary_on_left: pos == html_line.text_range().start
                    || text
                        .iter()
                        .nth(pos)
                        .map(|c| matches!(c, 0x20 | 0x09 | 0x0a | 0x0d))
                        .unwrap_or(false),
            })
            .collect();
        Self {
            html_line,
            text,
            text_block_begin,
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

    pub fn text_width(&self) -> f32 {
        let chars: Vec<u16> = self.text.iter().collect();
        for atom in self.atoms.iter().rev() {
            let pos = atom.char_start.saturating_sub(self.text_block_begin);
            let blank = chars
                .get(pos)
                .map(|c| matches!(c, 0x20 | 0x09 | 0x0a | 0x0d | 0x2028 | 0x2029))
                .unwrap_or(true);
            if !blank {
                return atom.x + atom.width;
            }
        }
        0.0
    }

    pub fn width(&self) -> f32 {
        self.html_line.bounds().width().to_pixels() as f32
    }

    pub fn raw_text_length(&self) -> usize {
        self.html_line.text_range().len()
    }

    pub fn atoms(&self) -> &[Atom] {
        &self.atoms
    }
}

fn typo_metrics(line: &LayoutLine<'_>, text: &WStr) -> (f32, f32) {
    let mut ascent = 0.0_f32;
    let mut descent = 0.0_f32;
    let mut found = false;

    for lbox in line.boxes_iter() {
        if let Some((_, _, font_set, params, _)) = lbox.as_renderable_text(text) {
            let font = font_set.main_font();
            ascent = ascent.max(font.typo_ascent(params.height()).to_pixels() as f32);
            descent = descent.max(font.typo_descent(params.height()).to_pixels() as f32);
            found = true;
        }
    }

    if found {
        (ascent, descent)
    } else {
        (
            line.ascent().to_pixels() as f32,
            line.descent().to_pixels() as f32,
        )
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
    fallback: RefLock<Option<EditText<'gc>>>,
    #[collect(require_static)]
    movie: Arc<SwfMovie>,
}

impl<'gc> FteTextLine<'gc> {
    pub fn new(
        context: &mut UpdateContext<'gc>,
        movie: Arc<SwfMovie>,
        line: FteLine<'gc>,
        fallback: Option<EditText<'gc>>,
    ) -> Self {
        FteTextLine(Gc::new(
            context.gc(),
            FteTextLineData {
                base: Default::default(),
                avm2_object: Lock::new(None),
                line: RefLock::new(line),
                fallback: RefLock::new(fallback),
                movie,
            },
        ))
    }

    pub fn line(self) -> Ref<'gc, FteLine<'gc>> {
        Gc::as_ref(self.0).line.borrow()
    }

    pub fn set_line(
        self,
        context: &mut UpdateContext<'gc>,
        line: FteLine<'gc>,
        fallback: Option<EditText<'gc>>,
    ) {
        let mc = context.gc();
        unlock!(Gc::write(mc, self.0), FteTextLineData, line).replace(line);
        *unlock!(Gc::write(mc, self.0), FteTextLineData, fallback).borrow_mut() = fallback;
    }
}

impl<'gc> TDisplayObject<'gc> for FteTextLine<'gc> {
    fn base(self) -> Gc<'gc, DisplayObjectBase<'gc>> {
        let interactive: Gc<'gc, InteractiveObjectBase<'gc>> = HasPrefixField::as_prefix_gc(self.0);
        HasPrefixField::as_prefix_gc(interactive)
    }

    fn instantiate(self, gc_context: &Mutation<'gc>) -> DisplayObject<'gc> {
        let borrowed = self.0.line.borrow();
        Self(Gc::new(
            gc_context,
            FteTextLineData {
                base: Default::default(),
                avm2_object: Lock::new(None),
                line: RefLock::new(FteLine {
                    html_line: borrowed.html_line.clone(),
                    text: borrowed.text.clone(),
                    text_block_begin: borrowed.text_block_begin,
                    atoms: borrowed.atoms.clone(),
                    ascent: borrowed.ascent,
                    descent: borrowed.descent,
                }),
                fallback: RefLock::new(*self.0.fallback.borrow()),
                movie: self.0.movie.clone(),
            },
        ))
        .into()
    }

    fn id(self) -> CharacterId {
        0
    }

    fn movie(self) -> Arc<SwfMovie> {
        self.0.movie.clone()
    }

    fn replace_with(self, _context: &mut UpdateContext<'gc>, _id: CharacterId) {}

    fn render_self(self, context: &mut RenderContext<'_, 'gc>) {
        let line = self.0.line.borrow();
        let mut has_renderable_content = false;
        for lbox in line.html_line.boxes_iter() {
            if lbox.as_renderable_drawing().is_some() {
                has_renderable_content = true;
                break;
            }
            if let Some((text, _tf, font, params, _color)) = lbox.as_renderable_text(&line.text) {
                font.evaluate(
                    text,
                    Default::default(),
                    params,
                    &mut |_pos, _glyph_transform, glyph, _advance, _x| {
                        has_renderable_content |= glyph.renderable(context);
                    },
                );
                if has_renderable_content {
                    break;
                }
            }
        }
        if !has_renderable_content {
            if let Some(fallback) = *self.0.fallback.borrow() {
                fallback.render_self(context);
            }
            return;
        }

        let baseline = line.html_line.bounds().origin().y() + line.html_line.ascent();
        let (quality_offset, low_quality_line_offset) =
            if context.stage.quality() == StageQuality::Low {
                (Twips::from_pixels(2.0), Twips::new(15))
            } else {
                (Twips::ZERO, Twips::ZERO)
            };
        let line_offset = if line.atoms.first().is_some_and(|atom| atom.char_start > 0) {
            low_quality_line_offset
        } else {
            Twips::ZERO
        };
        context.transform_stack.push(&Transform {
            matrix: Matrix::translate(quality_offset, quality_offset + line_offset - baseline),
            ..Default::default()
        });

        for lbox in line.html_line.boxes_iter() {
            let origin = lbox.bounds().origin();
            let renderable = lbox.as_renderable_text(&line.text);
            let baseline_shift = match renderable.as_ref() {
                Some((_, tf, ..)) => match tf.baseline_shift {
                    Some(shift) => Twips::from_pixels(shift),
                    None => Twips::ZERO,
                },
                None => Twips::ZERO,
            };
            context.transform_stack.push(&Transform {
                matrix: Matrix::translate(origin.x(), origin.y() + baseline_shift),
                ..Default::default()
            });

            if let Some((text, _tf, font, params, color)) = renderable {
                let mut transform: Transform = Default::default();
                transform.color_transform.set_mult_color(color);
                font.evaluate(
                    text,
                    transform,
                    params,
                    &mut |_pos, glyph_transform, glyph, _advance, _x| {
                        if glyph.renderable(context) {
                            context.transform_stack.push(glyph_transform);
                            glyph.render(context);
                            context.transform_stack.pop();
                        }
                    },
                );
            }

            if let Some(drawing) = lbox.as_renderable_drawing() {
                drawing.render(context);
            }

            context.transform_stack.pop();
        }

        context.transform_stack.pop();
    }

    fn self_bounds(self, _mode: BoundsMode) -> Rectangle<Twips> {
        let line = self.0.line.borrow();
        Rectangle {
            x_min: Twips::ZERO,
            x_max: Twips::from_pixels(line.width() as f64),
            y_min: Twips::from_pixels(-(line.ascent() as f64)),
            y_max: Twips::from_pixels(line.descent() as f64),
        }
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

impl<'gc> TInteractiveObject<'gc> for FteTextLine<'gc> {
    fn raw_interactive(self) -> Gc<'gc, InteractiveObjectBase<'gc>> {
        HasPrefixField::as_prefix_gc(self.0)
    }

    fn as_displayobject(self) -> DisplayObject<'gc> {
        self.into()
    }

    fn filter_clip_event(
        self,
        _context: &mut UpdateContext<'gc>,
        _event: ClipEvent,
    ) -> ClipEventResult {
        ClipEventResult::NotHandled
    }

    fn event_dispatch(
        self,
        _context: &mut UpdateContext<'gc>,
        _event: ClipEvent<'gc>,
    ) -> ClipEventResult {
        ClipEventResult::NotHandled
    }

    fn mouse_pick_avm1(
        self,
        _context: &mut UpdateContext<'gc>,
        _point: Point<Twips>,
        _require_button_mode: bool,
    ) -> Option<InteractiveObject<'gc>> {
        None
    }

    fn mouse_pick_avm2(
        self,
        _context: &mut UpdateContext<'gc>,
        _point: Point<Twips>,
        _require_button_mode: bool,
    ) -> Avm2MousePick<'gc> {
        Avm2MousePick::Miss
    }

    fn mouse_cursor(self, _context: &mut UpdateContext<'gc>) -> MouseCursor {
        MouseCursor::Arrow
    }
}
