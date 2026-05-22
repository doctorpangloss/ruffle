use ruffle_macros::istr;

use crate::avm2::activation::Activation;
use crate::avm2::error::{Error, make_error_2175};
use crate::avm2::function::FunctionArgs;
use crate::avm2::globals::flash::display::display_object::initialize_for_allocator;
use crate::avm2::globals::methods::flash_text_engine_content_element as element_methods;
use crate::avm2::globals::slots::flash_text_engine_content_element as element_slots;
use crate::avm2::globals::slots::flash_text_engine_element_format as format_slots;
use crate::avm2::globals::slots::flash_text_engine_font_description as font_desc_slots;
use crate::avm2::globals::slots::flash_text_engine_text_block as block_slots;
use crate::avm2::globals::slots::flash_text_engine_text_line as line_slots;
use crate::avm2::object::{Object, TObject};
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use crate::avm2_stub_method;
use crate::display_object::{EditText, TDisplayObject};
use crate::html::TextFormat;
use crate::string::{WStr, WString};

#[allow(dead_code)]
fn next_line_start(previous: Option<(usize, usize)>) -> usize {
    match previous {
        Some((begin, raw_length)) => begin + raw_length,
        None => 0,
    }
}

#[allow(dead_code)]
fn resolve_text_content<'gc>(
    activation: &mut Activation<'_, 'gc>,
    text_block: Object<'gc>,
    previous_text_line: Option<Object<'gc>>,
) -> Result<Option<(usize, crate::string::AvmString<'gc>, Object<'gc>)>, Error<'gc>> {
    let content = text_block.get_slot(block_slots::_CONTENT);
    if matches!(content, Value::Null) {
        return Ok(None);
    }

    let full_text = {
        let txt = content
            .call_method(element_methods::GET_TEXT, &[], activation)
            .unwrap_or_else(|_| istr!("").into());
        if matches!(txt, Value::Null) {
            return Ok(None);
        }
        txt.coerce_to_string(activation)
            .expect("Guaranteed by AS bindings")
    };

    let start = next_line_start(match previous_text_line {
        Some(prev) => Some((
            prev.get_slot(line_slots::_TEXT_BLOCK_BEGIN_INDEX)
                .coerce_to_i32(activation)? as usize,
            prev.get_slot(line_slots::_RAW_TEXT_LENGTH)
                .coerce_to_i32(activation)? as usize,
        )),
        None => None,
    });

    if start >= full_text.len() {
        text_block.set_slot(
            block_slots::_TEXT_LINE_CREATION_RESULT,
            istr!("complete").into(),
            activation,
        )?;
        return Ok(None);
    }

    Ok(Some((start, full_text, content.as_object().unwrap())))
}

#[allow(dead_code)]
fn finite_baseline_shift(value: f64) -> Option<f64> {
    value.is_finite().then_some(value)
}

#[allow(dead_code)]
fn format_from_content<'gc>(
    activation: &mut Activation<'_, 'gc>,
    content: Object<'gc>,
) -> Result<(TextFormat, f64, f64), Error<'gc>> {
    let mut format = TextFormat {
        font: Some(WString::from_utf8("_sans")),
        size: Some(12.0),
        color: Some(swf::Color::from_rgb(0, 0xff)),
        ..Default::default()
    };

    let Some(ef) = content.get_slot(element_slots::_ELEMENT_FORMAT).as_object() else {
        return Err(make_error_2175(activation));
    };

    let color = ef.get_slot(format_slots::_COLOR).coerce_to_u32(activation)?;
    format.color = Some(swf::Color::from_rgb(color & 0xff_ffff, 0xff));

    let size = ef
        .get_slot(format_slots::_FONT_SIZE)
        .coerce_to_number(activation)?;
    format.size = Some(size);

    let tracking_left = ef
        .get_slot(format_slots::_TRACKING_LEFT)
        .coerce_to_number(activation)?;
    let tracking_right = ef
        .get_slot(format_slots::_TRACKING_RIGHT)
        .coerce_to_number(activation)?;
    format.letter_spacing = Some(tracking_left + tracking_right);

    let kerning = ef
        .get_slot(format_slots::_KERNING)
        .coerce_to_string(activation)?;
    format.kerning = Some(kerning.to_utf8_lossy() != "off");

    let baseline_shift = ef
        .get_slot(format_slots::_BASELINE_SHIFT)
        .coerce_to_number(activation)?;
    format.baseline_shift = finite_baseline_shift(baseline_shift);

    if let Value::Object(fd) = ef.get_slot(format_slots::_FONT_DESCRIPTION) {
        let name = fd
            .get_slot(font_desc_slots::_FONT_NAME)
            .coerce_to_string(activation)?;
        format.font = Some(WString::from(name.as_wstr()));

        let weight = fd
            .get_slot(font_desc_slots::_FONT_WEIGHT)
            .coerce_to_string(activation)?;
        format.bold = Some(weight.to_utf8_lossy() == "bold");

        let posture = fd
            .get_slot(font_desc_slots::_FONT_POSTURE)
            .coerce_to_string(activation)?;
        format.italic = Some(posture.to_utf8_lossy() == "italic");
    }

    Ok((format, tracking_left, tracking_right))
}

#[allow(dead_code)]
fn typographic_case_transform(case: &str, text: &WStr) -> Option<WString> {
    let transformed = match case {
        "uppercase" => text.to_utf8_lossy().to_uppercase(),
        "lowercase" => text.to_utf8_lossy().to_lowercase(),
        _ => return None,
    };
    let transformed = WString::from_utf8(&transformed);
    (transformed.len() == text.len()).then_some(transformed)
}

#[allow(dead_code)]
fn collect_runs<'gc>(
    activation: &mut Activation<'_, 'gc>,
    content: Object<'gc>,
    out: &mut Vec<(WString, TextFormat, f64, f64)>,
) -> Result<(), Error<'gc>> {
    let is_group =
        content.instance_class().name().local_name().to_utf8_lossy() == "GroupElement";

    if is_group {
        let count_name = crate::string::AvmString::new_utf8(activation.gc(), "elementCount");
        let get_at_name = crate::string::AvmString::new_utf8(activation.gc(), "getElementAt");
        let count = Value::from(content)
            .get_public_property(count_name, activation)?
            .coerce_to_i32(activation)?;
        for i in 0..count {
            let child = Value::from(content).call_public_property(
                get_at_name,
                FunctionArgs::from_slice(&[Value::from(i)]),
                activation,
            )?;
            if let Some(child) = child.as_object() {
                collect_runs(activation, child, out)?;
            }
        }
    } else {
        let text = Value::from(content)
            .call_method(element_methods::GET_TEXT, &[], activation)
            .unwrap_or_else(|_| istr!("").into())
            .coerce_to_string(activation)?;
        let (format, tracking_left, tracking_right) = format_from_content(activation, content)?;
        let mut run_text = WString::from(text.as_wstr());

        if let Some(ef) = content.get_slot(element_slots::_ELEMENT_FORMAT).as_object() {
            let tc = ef
                .get_slot(format_slots::_TYPOGRAPHIC_CASE)
                .coerce_to_string(activation)?;
            if let Some(transformed) =
                typographic_case_transform(tc.to_utf8_lossy().as_ref(), &run_text)
            {
                run_text = transformed;
            }
        }
        out.push((run_text, format, tracking_left, tracking_right));
    }
    Ok(())
}

pub fn create_text_line<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    avm2_stub_method!(activation, "flash.text.TextBlock", "createTextLine");

    let previous_text_line = args.try_get_object(0);
    let width = args.get_f64(1);

    let content = this.get_slot(block_slots::_CONTENT);

    let content = if matches!(content, Value::Null) {
        return Ok(Value::Null);
    } else {
        content
    };

    let text = match previous_text_line {
        Some(_) => {
            // Some SWFs rely on eventually getting `null` from createLineText.
            // TODO: Support multiple lines
            this.set_slot(
                block_slots::_TEXT_LINE_CREATION_RESULT,
                istr!("complete").into(),
                activation,
            )?;
            return Ok(Value::Null);
        }
        // Get the content element's text property (it's a getter).
        // TODO: GraphicElement?
        None => {
            let txt = content
                .call_method(element_methods::GET_TEXT, &[], activation)
                .unwrap_or_else(|_| istr!("").into());

            if matches!(txt, Value::Null) {
                // FP returns a null TextLine when `o` is null- note that
                // `o` is already coerced to a String because of the AS bindings.
                return Ok(Value::Null);
            } else {
                txt.coerce_to_string(activation)
                    .expect("Guaranteed by AS bindings")
            }
        }
    };

    let class = activation.avm2().classes().textline;
    let movie = activation.caller_movie_or_root();

    // FIXME: TextLine should be its own DisplayObject
    let display_object: EditText =
        EditText::new_fte(activation.context, movie, 0.0, 0.0, width, 15.0);

    display_object.set_text(text.as_wstr(), activation.context);

    // FIXME: This needs to use `intrinsic_bounds` to measure the width
    // of the provided text, and set the width of the EditText to that.
    // Some games depend on this (e.g. Realm Grinder).

    let content = content.as_object().unwrap();
    let element_format = content.get_slot(element_slots::_ELEMENT_FORMAT).as_object();

    apply_format(activation, display_object, text.as_wstr(), element_format)?;

    let instance = initialize_for_allocator(activation.context, display_object.into(), class);

    instance.set_slot(line_slots::_TEXT_BLOCK, this.into(), activation)?;

    instance.set_slot(line_slots::_SPECIFIED_WIDTH, args.get_value(1), activation)?;

    instance.set_slot(
        line_slots::_RAW_TEXT_LENGTH,
        Value::from_usize_lossy(text.len()),
        activation,
    )?;

    this.set_slot(
        block_slots::_TEXT_LINE_CREATION_RESULT,
        istr!("success").into(),
        activation,
    )?;

    this.set_slot(block_slots::_FIRST_LINE, instance.into(), activation)?;

    Ok(instance.into())
}

fn apply_format<'gc>(
    activation: &mut Activation<'_, 'gc>,
    display_object: EditText<'gc>,
    text: &WStr,
    element_format: Option<Object<'gc>>,
) -> Result<(), Error<'gc>> {
    if let Some(element_format) = element_format {
        // TODO: Support more ElementFormat properties
        let color = element_format
            .get_slot(format_slots::_COLOR)
            .coerce_to_u32(activation)?;
        let size = element_format
            .get_slot(format_slots::_FONT_SIZE)
            .coerce_to_number(activation)?;

        let (font, bold, italic, is_device_font) = if let Value::Object(font_description) =
            element_format.get_slot(format_slots::_FONT_DESCRIPTION)
        {
            (
                Some(
                    font_description
                        .get_slot(font_desc_slots::_FONT_NAME)
                        .coerce_to_string(activation)?
                        .as_wstr()
                        .into(),
                ),
                Some(
                    &font_description
                        .get_slot(font_desc_slots::_FONT_WEIGHT)
                        .coerce_to_string(activation)?
                        == b"bold",
                ),
                Some(
                    &font_description
                        .get_slot(font_desc_slots::_FONT_POSTURE)
                        .coerce_to_string(activation)?
                        == b"italic",
                ),
                &font_description
                    .get_slot(font_desc_slots::_FONT_LOOKUP)
                    .coerce_to_string(activation)?
                    == b"device",
            )
        } else {
            (None, None, None, true)
        };

        let format = TextFormat {
            color: Some(swf::Color::from_rgb(color, 0xFF)),
            size: Some(size),
            font,
            bold,
            italic,
            ..TextFormat::default()
        };

        display_object.set_is_device_font(activation.context, is_device_font);
        display_object.set_text_format(0, text.len(), format.clone(), activation.context);
        display_object.set_new_text_format(format);
    } else {
        display_object.set_is_device_font(activation.context, true);
    }

    display_object.set_word_wrap(true, activation.context);

    let measured_text = display_object.measure_text(activation.context);

    display_object.set_height(activation.context, measured_text.1.to_pixels());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_line_start_resumes_after_the_previous_line() {
        assert_eq!(next_line_start(Some((10, 7))), 17);
        assert_eq!(next_line_start(None), 0);
    }

    #[test]
    fn baseline_shift_keeps_finite_values_and_drops_nan() {
        assert_eq!(finite_baseline_shift(3.5), Some(3.5));
        assert_eq!(finite_baseline_shift(-2.0), Some(-2.0));
        assert_eq!(finite_baseline_shift(f64::NAN), None);
    }

    #[test]
    fn typographic_case_transforms_only_when_length_is_preserved() {
        let abc = WString::from_utf8("abc");
        assert_eq!(
            typographic_case_transform("uppercase", &abc).unwrap(),
            WString::from_utf8("ABC")
        );
        assert!(typographic_case_transform("caps", &abc).is_none());
        let sharp_s = WString::from_utf8("stra\u{00DF}e");
        assert!(typographic_case_transform("uppercase", &sharp_s).is_none());
    }
}
