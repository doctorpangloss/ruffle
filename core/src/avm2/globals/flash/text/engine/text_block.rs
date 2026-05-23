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
use crate::display_object::{FteLine, FteTextLine};
use crate::font::FontType;
use crate::html::{FormatSpans, LayoutLine, TextFormat, lower_from_text_spans};
use crate::string::{WStr, WString};
use swf::Twips;

fn next_line_start(previous: Option<(usize, usize)>) -> usize {
    match previous {
        Some((begin, raw_length)) => begin + raw_length,
        None => 0,
    }
}

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

fn finite_baseline_shift(value: f64) -> Option<f64> {
    value.is_finite().then_some(value)
}

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

fn typographic_case_transform(case: &str, text: &WStr) -> Option<WString> {
    let transformed = match case {
        "uppercase" => text.to_utf8_lossy().to_uppercase(),
        "lowercase" => text.to_utf8_lossy().to_lowercase(),
        _ => return None,
    };
    let transformed = WString::from_utf8(&transformed);
    (transformed.len() == text.len()).then_some(transformed)
}

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

fn paragraph_len(tail: &WStr) -> usize {
    match tail.iter().position(|u| u == 0x2028 || u == 0x2029) {
        Some(pos) => pos + 1,
        None => tail.len(),
    }
}

fn clip_run(
    run_start: usize,
    run_end: usize,
    lo_bound: usize,
    hi_bound: usize,
) -> Option<(usize, usize)> {
    let lo = run_start.max(lo_bound);
    let hi = run_end.min(hi_bound);
    (lo < hi).then_some((lo, hi))
}

fn lay_out_first_line<'gc>(
    activation: &mut Activation<'_, 'gc>,
    full_text: crate::string::AvmString<'gc>,
    start: usize,
    content: Object<'gc>,
    width: f64,
) -> Result<Option<(LayoutLine<'gc>, WString, usize)>, Error<'gc>> {
    let tail = &full_text[start..];
    let para_len = paragraph_len(tail);
    let line_end = start + para_len;

    let mut runs: Vec<(WString, TextFormat, f64, f64)> = Vec::new();
    collect_runs(activation, content, &mut runs)?;

    let mut remaining = WString::new();
    let mut run_spans: Vec<(usize, usize, &TextFormat, f64, f64)> = Vec::new();
    let mut run_off = 0usize;
    for (run_text, run_fmt, tracking_left, tracking_right) in &runs {
        let run_start = run_off;
        run_off += run_text.len();
        if let Some((lo, hi)) = clip_run(run_start, run_off, start, line_end) {
            run_spans.push((lo - start, hi - start, run_fmt, *tracking_left, *tracking_right));
            remaining.push_str(&run_text[lo - run_start..hi - run_start]);
        }
    }
    if runs.is_empty() {
        remaining = WString::from(&tail[..para_len]);
    }

    let base = match runs.first() {
        Some((_, fmt, ..)) => fmt.clone(),
        None => format_from_content(activation, content)?.0,
    };
    let mut spans = FormatSpans::from_text(remaining.clone(), base);
    for (from, to, fmt, ..) in &run_spans {
        spans.set_text_format(*from, *to, fmt);
    }

    let requested_width = if width >= 1_000_000.0 {
        None
    } else {
        Some(Twips::from_pixels(width))
    };

    let movie = activation.caller_movie_or_root();
    let layout = lower_from_text_spans(
        &spans,
        activation.context,
        movie,
        requested_width,
        false,
        true,
        FontType::Device,
        true,
    );

    let Some(mut first) = layout.lines().first().cloned() else {
        return Ok(None);
    };

    let (line_lo, line_hi) = (first.start(), first.end());
    let mut leading = 0.0;
    let mut trailing = 0.0;
    for &(from, to, _, tracking_left, tracking_right) in &run_spans {
        if from <= line_lo && line_lo < to {
            leading = tracking_left;
        }
        if from < line_hi && line_hi <= to {
            trailing = tracking_right;
        }
    }
    first.trim_edge_tracking(Twips::from_pixels(leading), Twips::from_pixels(trailing));

    Ok(Some((first, remaining, start)))
}

pub fn create_text_line<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let previous_text_line = args.try_get_object(0);
    let width = args.get_f64(1);

    let Some((start, full_text, content)) =
        resolve_text_content(activation, this, previous_text_line)?
    else {
        return Ok(Value::Null);
    };

    let Some((html_line, text, begin)) =
        lay_out_first_line(activation, full_text, start, content, width)?
    else {
        this.set_slot(
            block_slots::_TEXT_LINE_CREATION_RESULT,
            istr!("complete").into(),
            activation,
        )?;
        return Ok(Value::Null);
    };

    let fte_line = FteLine::new(html_line, text, begin);
    let raw_text_length = fte_line.raw_text_length();
    let movie = activation.caller_movie_or_root();

    let fte = FteTextLine::new(activation.context, movie, fte_line);
    let class = activation.avm2().classes().textline;
    let instance = initialize_for_allocator(activation.context, fte.into(), class);

    instance.set_slot(line_slots::_TEXT_BLOCK, this.into(), activation)?;
    instance.set_slot(line_slots::_SPECIFIED_WIDTH, args.get_value(1), activation)?;
    instance.set_slot(
        line_slots::_RAW_TEXT_LENGTH,
        (raw_text_length as i32).into(),
        activation,
    )?;
    instance.set_slot(
        line_slots::_TEXT_BLOCK_BEGIN_INDEX,
        (start as i32).into(),
        activation,
    )?;

    if let Some(prev) = previous_text_line {
        prev.set_slot(line_slots::_NEXT_LINE, instance.into(), activation)?;
        instance.set_slot(line_slots::_PREVIOUS_LINE, prev.into(), activation)?;
    } else {
        this.set_slot(block_slots::_FIRST_LINE, instance.into(), activation)?;
    }
    this.set_slot(block_slots::_LAST_LINE, instance.into(), activation)?;

    this.set_slot(
        block_slots::_TEXT_LINE_CREATION_RESULT,
        istr!("success").into(),
        activation,
    )?;

    Ok(instance.into())
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
    fn paragraph_len_stops_after_the_first_hard_break() {
        assert_eq!(paragraph_len(&WString::from_utf8("hello")), 5);
        assert_eq!(paragraph_len(&WString::from_utf8("ab\u{2028}cd")), 3);
        assert_eq!(paragraph_len(&WString::from_utf8("x\u{2029}y")), 2);
    }

    #[test]
    fn clip_run_intersects_a_run_with_the_line() {
        assert_eq!(clip_run(0, 10, 3, 7), Some((3, 7)));
        assert_eq!(clip_run(0, 3, 5, 9), None);
        assert_eq!(clip_run(5, 5, 0, 9), None);
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
