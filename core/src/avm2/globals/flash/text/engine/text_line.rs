//! Natives for the flash.text.engine.TextLine class.

use crate::avm2::activation::Activation;
use crate::avm2::error::Error;
use crate::avm2::value::Value;
use crate::display_object::FteLine;
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
