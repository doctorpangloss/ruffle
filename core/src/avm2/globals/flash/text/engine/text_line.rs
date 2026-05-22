//! Natives for the flash.text.engine.TextLine class.

use crate::avm2::activation::Activation;
use crate::avm2::error::Error;
use crate::avm2::parameters::ParametersExt;
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

#[cfg(test)]
mod tests {
    use super::baseline_position;

    #[test]
    fn baseline_position_maps_the_named_baselines() {
        assert_eq!(baseline_position("roman", 16.0, 4.0), 0.0);
        assert_eq!(baseline_position("ascent", 16.0, 4.0), -16.0);
        assert_eq!(baseline_position("descent", 16.0, 4.0), 4.0);
        assert_eq!(baseline_position("ideographicCenter", 16.0, 4.0), -6.0);
        assert_eq!(baseline_position("nonsense", 16.0, 4.0), 0.0);
    }
}
