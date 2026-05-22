//! Natives for the flash.text.engine.TextLine class.

use crate::avm2::activation::Activation;
use crate::avm2::error::Error;
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use crate::display_object::{Atom, FteLine};
use std::cell::Ref;

fn fte_line<'gc>(this: Value<'gc>) -> Option<Ref<'gc, FteLine<'gc>>> {
    let line = this.as_object()?.as_display_object()?.as_fte_text_line()?;
    Some(line.line())
}

pub fn get_text_width<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let Some(line) = fte_line(this) else {
        return Ok(0.0.into());
    };
    Ok((line.text_width() as f64).into())
}

pub fn get_text_height<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let Some(line) = fte_line(this) else {
        return Ok(0.0.into());
    };
    Ok(((line.ascent() + line.descent()) as f64).into())
}

pub fn get_ascent<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let Some(line) = fte_line(this) else {
        return Ok(12.0.into());
    };
    Ok((line.ascent() as f64).into())
}

pub fn get_descent<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let Some(line) = fte_line(this) else {
        return Ok(3.0.into());
    };
    Ok((line.descent() as f64).into())
}

pub fn get_atom_count<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let Some(line) = fte_line(this) else {
        return Ok(0.into());
    };
    Ok((line.atoms().len() as i32).into())
}

fn baseline_position(baseline: &str, ascent: f32, descent: f32) -> f32 {
    match baseline {
        "roman" => 0.0,
        "ascent" | "ideographicTop" => -ascent,
        "descent" | "ideographicBottom" => descent,
        "ideographicCenter" => (descent - ascent) / 2.0,
        _ => 0.0,
    }
}

pub fn get_baseline_position<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let baseline = args.get_value(0).coerce_to_string(activation)?;
    let (ascent, descent) = match fte_line(this) {
        Some(line) => (line.ascent(), line.descent()),
        None => (12.0, 3.0),
    };
    let position = baseline_position(baseline.to_utf8_lossy().as_ref(), ascent, descent);
    Ok((position as f64).into())
}

fn atom_at<'a>(line: &'a FteLine, index: i32) -> Option<&'a Atom> {
    if index < 0 {
        return None;
    }
    line.atoms().get(index as usize)
}

fn atom_index_for_char(char_index: i32, first_char_start: usize, atom_count: usize) -> i32 {
    if char_index < 0 || (char_index as usize) < first_char_start {
        return -1;
    }
    let offset = char_index as usize - first_char_start;
    if offset < atom_count {
        offset as i32
    } else {
        -1
    }
}

pub fn get_atom_bidi_level<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let index = args.get_i32(0);
    let Some(line) = fte_line(this) else {
        return Ok(0.into());
    };
    let Some(atom) = atom_at(&line, index) else {
        return Ok(0.into());
    };
    Ok((atom.bidi_level as i32).into())
}

pub fn get_atom_index_at_char_index<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let char_index = args.get_i32(0);
    let Some(line) = fte_line(this) else {
        return Ok((-1).into());
    };
    let index = match line.atoms().first() {
        Some(first) => atom_index_for_char(char_index, first.char_start, line.atoms().len()),
        None => -1,
    };
    Ok(index.into())
}

pub fn get_atom_text_block_begin_index<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let index = args.get_i32(0);
    let Some(line) = fte_line(this) else {
        return Ok((-1).into());
    };
    let Some(atom) = atom_at(&line, index) else {
        return Ok((-1).into());
    };
    Ok((atom.char_start as i32).into())
}

pub fn get_atom_text_block_end_index<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let index = args.get_i32(0);
    let Some(line) = fte_line(this) else {
        return Ok((-1).into());
    };
    let Some(atom) = atom_at(&line, index) else {
        return Ok((-1).into());
    };
    Ok((atom.char_end as i32).into())
}

pub fn get_atom_word_boundary_on_left<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let index = args.get_i32(0);
    let Some(line) = fte_line(this) else {
        return Ok(false.into());
    };
    let Some(atom) = atom_at(&line, index) else {
        return Ok(false.into());
    };
    Ok(atom.word_boundary_on_left.into())
}

fn atom_bounds(this: Value<'_>, index: i32) -> (f64, f64, f64, f64) {
    let Some(line) = fte_line(this) else {
        return (0.0, 0.0, 0.0, 0.0);
    };
    let Some(atom) = atom_at(&line, index) else {
        return (0.0, 0.0, 0.0, 0.0);
    };
    (
        atom.x as f64,
        -line.ascent() as f64,
        atom.width as f64,
        (line.ascent() + line.descent()) as f64,
    )
}

pub fn get_atom_bounds<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let index = args.get_i32(0);
    let (x, y, width, height) = atom_bounds(this, index);
    let class = activation.avm2().classes().rectangle;
    class.construct(activation, &[x.into(), y.into(), width.into(), height.into()])
}

pub fn get_atom_center<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let index = args.get_i32(0);
    let Some(line) = fte_line(this) else {
        return Ok(0.0.into());
    };
    let Some(atom) = atom_at(&line, index) else {
        return Ok(0.0.into());
    };
    Ok(((atom.x + atom.width / 2.0) as f64).into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atom_index_for_char_offsets_from_the_lines_first_atom() {
        assert_eq!(atom_index_for_char(4, 4, 3), 0);
        assert_eq!(atom_index_for_char(6, 4, 3), 2);
        assert_eq!(atom_index_for_char(3, 4, 3), -1);
        assert_eq!(atom_index_for_char(7, 4, 3), -1);
        assert_eq!(atom_index_for_char(-1, 4, 3), -1);
    }

    #[test]
    fn baseline_position_maps_the_named_baselines() {
        assert_eq!(baseline_position("roman", 16.0, 4.0), 0.0);
        assert_eq!(baseline_position("ascent", 16.0, 4.0), -16.0);
        assert_eq!(baseline_position("descent", 16.0, 4.0), 4.0);
        assert_eq!(baseline_position("ideographicCenter", 16.0, 4.0), -6.0);
        assert_eq!(baseline_position("nonsense", 16.0, 4.0), 0.0);
    }
}
