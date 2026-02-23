use core::panic;
use cosmic_text::{
    Attrs, Buffer, Family, FontSystem, LayoutRun, Metrics, Style, Weight,
    skrifa::raw::tables::svg::{self, Svg},
};
use std::{char, collections::HashMap, thread::current};

use crate::styles::{StyledBlock, StyledLine};

pub struct SvgConfig {
    // SVG Canvas Options
    pub width: f32,
    pub height: f32,
    // ? Support CSS Style padding args
    // ? This is for the actual SVG, not relevant to the Cosmic text.
    pub top_padding: f32,
    pub right_padding: f32,
    pub bottom_padding: f32,
    pub left_padding: f32,

    // Font Details
    pub font_size: f32,
    // todo expose this as an option to the end user?
    // todo  Explain default is sans-serif.
    //pub font_family: Family,

    // Bullet Style Options
    pub bullet_indent_em: f32, // default 1.5 or 2.0 ->
    pub bullet_char: char,

    // Header Style Options
    pub header_scales: HashMap<u8, f32>,
    pub header_margin_top: f32,
    pub header_margin_bot: f32,

    pub bg_color: String,
}
#[derive(Debug)]
pub struct LayoutLine {
    pub buffer: Buffer,
    pub segments: Vec<StyledBlock>,
    pub prefix_len: usize,
    pub indent_offset: f32,
}
impl SvgConfig {
    pub fn default() -> Self {
        SvgConfig {
            width: 600.0,
            height: 800.0,
            top_padding: 0.0,
            right_padding: 0.0,
            bottom_padding: 0.0,
            left_padding: 0.0,
            font_size: 16.0,
            bullet_indent_em: 1.5,
            bullet_char: char::from_u32(0x2022).expect("Should be able to unwrap the character •"),
            header_scales: HashMap::from([
                (1, 2.0),
                (2, 1.6),
                (3, 1.3),
                (4, 1.1),
                (5, 1.0),
                (6, 1.0),
            ]),
            header_margin_top: 0.0,
            header_margin_bot: 0.0,
            bg_color: String::from("#FFFFFF"),
        }
    }
}

pub struct SegmentRange {
    segment_idx: usize,
    start_byte: usize,
    end_byte: usize,
}

pub struct TSpan {
    text: String,
    x: f32,
    y: f32,
    weight: Weight,
    style: Style,
}

pub fn styled_line_to_layout(
    line: &StyledLine,
    font_system: &mut FontSystem,
    cfg: &SvgConfig,
) -> LayoutLine {
    // * Arrange our default available width based on the overall SVG size minus any L/R padding.
    let mut available_width = cfg.width - (cfg.left_padding + cfg.right_padding);

    match line {
        StyledLine::Paragraph { segments } => {
            let mut buffer = Buffer::new(
                font_system,
                Metrics::new(cfg.font_size, cfg.font_size * 1.5),
            );
            let rich_text: Vec<(&str, Attrs<'_>)> = segments
                .iter()
                .map(|block| {
                    let text = block.text.as_str();
                    let attrs = Attrs::new().weight(block.weight).style(block.style);

                    (text, attrs)
                })
                .collect();

            buffer.set_rich_text(
                font_system,
                rich_text,
                &Attrs::new().family(Family::SansSerif),
                cosmic_text::Shaping::Advanced,
                None,
            );
            buffer.set_size(font_system, Some(cfg.width), Some(f32::MAX));

            // buffer.lines.iter().enumerate().for_each(|(i, line)| {
            //     println!("Line {} | Text: {}", i, line.clone().into_text());
            //     println!("Line {} | Text: {:#?}", i, line.attrs_list().spans());
            // });

            LayoutLine {
                buffer,
                segments: segments.clone(),
                prefix_len: 0,
                indent_offset: 0.0,
            }
        }
        StyledLine::Header { segments, level } => {
            // ? Depending on the Header indentation, we will need to increase the size of our font.
            // ? We store the multipler in a hash-table in our config.
            let scaled_font_size =
                cfg.font_size * cfg.header_scales.get(level).copied().unwrap_or(1.0);

            let mut buffer = Buffer::new(
                font_system,
                Metrics::new(scaled_font_size, scaled_font_size * 1.5),
            );

            let rich_text: Vec<(&str, Attrs<'_>)> = segments
                .iter()
                .map(|block| {
                    let text = block.text.as_str();
                    // ? Headers are Bold by convention
                    // todo add config for this.
                    let attrs = Attrs::new().weight(Weight::BOLD).style(block.style);

                    (text, attrs)
                })
                .collect();

            buffer.set_rich_text(
                font_system,
                rich_text,
                &Attrs::new().family(Family::SansSerif),
                cosmic_text::Shaping::Advanced,
                None,
            );

            buffer.set_size(font_system, Some(cfg.width), Some(f32::MAX));

            LayoutLine {
                buffer,
                segments: segments.clone(),
                prefix_len: 0,
                indent_offset: 0.0,
            }
        }
        StyledLine::BulletListItem { segments, indent } => {
            let mut buffer = Buffer::new(
                font_system,
                Metrics::new(cfg.font_size, cfg.font_size * 1.5),
            );

            // * Calculate our indent by taking the number of indents and multiplying it by a unit size
            // * Our Unit Size is based on the font_size multiplied by an em value, default 1.5.
            let indent_size = *indent as f32 * (cfg.bullet_indent_em * cfg.font_size);

            let prefix = format!("{} ", cfg.bullet_char);

            // * Update our available_width based on the indent and prefix-length
            available_width = available_width - indent_size;

            let styled_segments: Vec<(String, Attrs)> = segments
                .iter()
                .enumerate()
                .map(|(i, block)| {
                    // We need to insert the bullet character here, however we can't simply use format!().as_str
                    // as the String returned by format!() doesn't live long enough when it leaves the styled_segments scope.
                    // Therefore, we need to have `styled_segments` own the full String and later have `buffer` borrow it for it's final form.
                    let text = if i == 0 {
                        format!("{}{}", &prefix, &block.text)
                    } else {
                        block.text.clone()
                    };

                    let attrs = Attrs::new().weight(block.weight).style(block.style);

                    (text, attrs)
                })
                .collect();

            // Convert our segments back into &str, Attrs to satisfy Cosmic API
            let rich_text: Vec<(&str, Attrs)> = styled_segments
                .iter()
                .map(|(text, attrs)| (text.as_str(), attrs.clone()))
                .collect();

            buffer.set_rich_text(
                font_system,
                rich_text,
                &Attrs::new().family(Family::SansSerif),
                cosmic_text::Shaping::Advanced,
                None,
            );

            buffer.set_size(font_system, Some(available_width), Some(f32::MAX));

            LayoutLine {
                buffer,
                segments: segments.clone(),
                prefix_len: prefix.len(),
                indent_offset: indent_size,
            }
        }
        _ => panic! {"Type not yet implemented! {:#?}", &line.get_type()},
    }
}

pub fn process_layouts(layouts: Vec<LayoutLine>, cfg: &SvgConfig, starting_y: f32) -> Vec<String> {
    // * Return value
    let mut svg_lines: Vec<String> = Vec::new();

    // * Store mutuable vars for our moving offsets.
    let mut cumulative_y_offset = starting_y;

    // For each layout in our document,
    // Process it into an SVG.
    // The returned values are the formatted SVG, and the last character index in the line and the current Y offset.
    // We use this offset to start looking through format segments, as our Run.glyph.start values will reset each run.
    for layout in layouts {
        let (svgs, updated_y) = process_layout_line(layout, cumulative_y_offset, cfg);
        svg_lines.extend(svgs);

        cumulative_y_offset += updated_y;
    }

    // Return final SVG collection.
    svg_lines
}

/// y_cursor == Padding or other start offset.
fn process_layout_line(layout: LayoutLine, y_cursor: f32, cfg: &SvgConfig) -> (Vec<String>, f32) {
    let mut svg_elements: Vec<String> = Vec::new();
    // * Build our Segment Map for this LayoutLine.
    let segment_ranges = build_segment_ranges(&layout.segments);

    let mut cumulative_y = y_cursor;
    let mut cumulative_byte_offset: usize = 0;

    // * For each run in our buffer, we need to process it in a few ways:
    // * 1. Check if there is a prefix on this run.
    // * Since prefixes are stripped at the time we parse the AST, we inserted it when we transform into a Styled Line.
    // * Therefore we need to handle it separately here.
    // * 2. Check if the new character we are processing belongs to a different segment
    // * 3. If there has been a change, crunch the previous characters as a TSPAN with the styles in that segment, and start a new string buffer.
    // * 4. Update the Y positions
    // * 5. Add the run to our output buffer.
    for (run_idx, run) in layout.buffer.layout_runs().enumerate() {
        let baseline_y = y_cursor + run.line_y;

        let (tspans, bytes_in_run, updated_y) = process_run(
            run,
            &segment_ranges,
            &layout.segments,
            layout.prefix_len,
            cumulative_byte_offset,
            baseline_y,
            cfg,
            layout.indent_offset,
        );
        println!("Run #{}", run_idx);

        svg_elements.push(tspans_to_svg(&tspans));
        // cumulative_byte_offset += bytes_in_run;
        cumulative_y += updated_y;
    }

    (svg_elements, cumulative_y)
}

fn process_run(
    run: LayoutRun,
    segment_ranges: &Vec<SegmentRange>,
    segments: &Vec<StyledBlock>,
    prefix_len: usize,
    byte_offset: usize,
    y_cursor: f32,
    cfg: &SvgConfig,
    indent_offset: f32,
) -> (Vec<TSpan>, usize, f32) {
    let mut tspans: Vec<TSpan> = vec![];
    // * Handle prefixes
    // ? Since we inserted our prefix, it isn't going to be part of our StyledSegment.
    // ? Therefore we need to process & emit it separately.
    if prefix_len > 0 && byte_offset == 0 {
        let mut prefix_text = String::new();

        // * Collect all glyphs that are part of the prefix
        for glyph in run.glyphs.iter().take_while(|g| g.start < prefix_len) {
            prefix_text.push_str(&run.text[glyph.start..glyph.end]);
        }

        if !prefix_text.is_empty() {
            tspans.push(TSpan {
                text: prefix_text,
                x: cfg.left_padding + indent_offset,
                y: y_cursor,
                weight: Weight::NORMAL,
                style: Style::Normal,
            });
        }
    }

    let mut current_text: String = String::new(); // * Create a string buffer that holds all characters in a given segment range.
    let mut current_segment_idx: Option<usize> = None;
    let mut current_x = cfg.left_padding + indent_offset; // handle starting offset for the line of text.

    // * Start investigating all 'glyphs' - or Unicode Codepoints.
    // ? It is important to note these are not necessarily entire characters or grapheme clusters.
    for glyph in run.glyphs.iter() {
        if glyph.start < prefix_len {
            continue; // Skip prefix glyphs
        }
        println!("Byte Offset: {}", byte_offset);
        // Adjust the starting byte position to account for the prefix.
        let adjusted_byte_pos = (glyph.start - prefix_len) + byte_offset;
        println!("Adjusted Byte Pos: {}", adjusted_byte_pos);
        // Find which segment this glyph belongs to.
        let segment_idx = segment_ranges
            .iter()
            .find(|range| {
                adjusted_byte_pos >= range.start_byte && adjusted_byte_pos <= range.end_byte
            })
            .map(|range| range.segment_idx)
            .expect(&format!(
                "Should be able to find a glyph at index {}.",
                adjusted_byte_pos
            ));



        // Check if we have moved into a different segment.
        if let Some(prev_idx) = current_segment_idx {
            if prev_idx != segment_idx {
                // Emit the accumulated text as a TSpan with styling from the previous segment.
                let prev_segment = &segments[prev_idx];

                tspans.push(TSpan {
                    text: current_text.clone(),
                    x: current_x,
                    y: y_cursor,
                    weight: prev_segment.weight,
                    style: prev_segment.style,
                });

                // Reset text buffer and update our X to move inline with all previous characters.
                current_text.clear();
                current_x = cfg.left_padding + indent_offset + glyph.x;
            }
        } else {
            // We haven't added a content glyph yet, so start.
            current_x = cfg.left_padding + indent_offset + glyph.x;
        }
        println!("Segment:{}", segments[segment_idx].text);
        println!(
            "Glyph {}|{} at byte offset {}",
            run.text.chars().nth(glyph.start).unwrap(),
            glyph.start,
            adjusted_byte_pos
        );
        // Add this glyph to our text buffer.
        let ch = &run.text[glyph.start..glyph.end];
        current_text.push_str(ch);
        println!("current_text: {}", current_text);
        println!("---");
        current_segment_idx = Some(segment_idx);
    }

    // At the end of the run, if we have any text remaining in our buffer, crunch it.
    if !current_text.is_empty() {
        if let Some(seg_idx) = current_segment_idx {
            let segment = &segments[seg_idx];
            tspans.push(TSpan {
                text: current_text.clone(),
                x: current_x,
                y: y_cursor,
                weight: segment.weight,
                style: segment.style,
            });
        }
    }

    (tspans, run.text.len(), y_cursor)
}

/// Convert a `Tspan` into a raw SVG string wrapped by a <text> tag.
fn tspans_to_svg(tspans: &[TSpan]) -> String {
    let tspan_strings: Vec<String> = tspans
        .iter()
        .map(|ts| {
            let weight_attr = if ts.weight == Weight::BOLD {
                r#" font-weight="bold" "#
            } else {
                ""
            };

            let style_attr = match ts.style {
                Style::Italic => r#" font-style="italic""#,
                Style::Oblique => r#" font-style="oblique""#,
                Style::Normal => "",
            };

            // Return a formatted SVG String.
            format!(
                r#"<tspan x="{}" y="{}"{}{}>{}</tspan>"#,
                ts.x,
                ts.y,
                weight_attr,
                style_attr,
                html_escape(&ts.text)
            )
        })
        .collect();

    // todo handle custom font-size & font family
    format!(
        r#"<text font-family="sans-serif" font-size="16">{}</text>"#,
        tspan_strings.join("")
    )
}
///  Precompute the text range of our StyledBlock text as a byte range, which matches with the Cosmic Glyph start/end indices.
fn build_segment_ranges(segments: &Vec<StyledBlock>) -> Vec<SegmentRange> {
    let mut ranges = Vec::new();

    let mut current_pos = 0;

    for (seg_idx, segment) in segments.iter().enumerate() {
        // todo ? Do we need to replace these characters here, or is there somewhere sooner we can handle it after the Cosmic shaping.
        let seg_len = segment.text.replace("\r\n", "").len();
        println!("Segment Length: {}", seg_len);
        println!("Segment Text: {}", segment.text);

        ranges.push(SegmentRange {
            segment_idx: seg_idx,
            start_byte: current_pos,
            end_byte: current_pos + seg_len - 1,
        });
        current_pos += seg_len
    }
    ranges
}

fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
