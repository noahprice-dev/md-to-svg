use core::{f32, panic};
use cosmic_text::{
    Attrs, Buffer, Family, FamilyOwned, FontSystem, LayoutRun, Metrics, Style, Weight,
};

use crate::{
    config::SvgConfig,
    styles::{StyledLine, StyledSegment},
};

#[derive(Debug)]
pub struct LayoutLine {
    pub buffer: Buffer,
    pub segments: Vec<StyledSegment>,
    pub font_size: f32,
    pub prefix_len: usize,
    pub indent_offset: f32,
    pub margin_top: f32,
    pub margin_bottom: f32,
}

pub enum LayoutResult {
    Line(LayoutLine),
    Blank { height: f32 },
}

impl LayoutResult {
    pub fn margin_top(&self) -> f32 {
        match self {
            LayoutResult::Line(lyt) => lyt.margin_top,
            LayoutResult::Blank { .. } => 0.0, // Fixed space
        }
    }

    pub fn margin_bottom(&self) -> f32 {
        match self {
            LayoutResult::Line(lyt) => lyt.margin_bottom,
            LayoutResult::Blank { .. } => 0.0, // Fixed space
        }
    }
}

#[derive(Debug)]
pub struct SegmentRange {
    segment_idx: usize,
    start_byte: usize,
    end_byte: usize,
}

pub struct TSpan {
    text: String,
    font_size: f32,
    weight: Weight,
    style: Style,
    family: FamilyOwned,
}

pub fn styled_line_to_layout(
    line: StyledLine,
    font_system: &mut FontSystem,
    cfg: &SvgConfig,
) -> LayoutResult {
    // * Arrange our default available width based on the overall SVG size minus any L/R padding.

    match line {
        StyledLine::Paragraph { segments } => {
            let available_width = cfg.canvas_opts.width
                - (cfg.canvas_opts.padding.left + cfg.canvas_opts.padding.right);

            let mut buffer = Buffer::new(
                font_system,
                Metrics::new(cfg.text_opts.font_size, cfg.get_line_spacing_factor()),
            );

            let styled_segments: Vec<(&str, Attrs)> = segments
                .iter()
                .flat_map(|seg| {
                    match seg {
                        StyledSegment::Text(block) => {
                            //styled_blocks.push(block.clone());
                            vec![(
                                block.text.as_str(),
                                Attrs::new()
                                    .weight(block.weight)
                                    .style(block.style)
                                    .family(block.family.as_family()),
                            )]
                        }
                        StyledSegment::HardBreak => {
                            // * Since we are updating how we are drawing in Cosmic,
                            // * we also need to reflect that in how we draw with SVG by inserting the same lines.
                            vec![("\n", Attrs::new())]
                        }
                    }
                })
                .collect();

            let _full_text: String = styled_segments.iter().map(|(text, _)| *text).collect();

            let rich_text: Vec<(&str, Attrs)> = styled_segments
                .iter()
                .map(|(text, attrs)| (*text, attrs.clone()))
                .collect();

            buffer.set_rich_text(
                font_system,
                rich_text,
                &Attrs::new().family(Family::SansSerif),
                cosmic_text::Shaping::Advanced,
                None,
            );
            buffer.set_size(font_system, Some(available_width), Some(f32::MAX));

            LayoutResult::Line(LayoutLine {
                buffer,
                segments: segments.clone(),
                font_size: cfg.text_opts.font_size,
                prefix_len: 0,
                indent_offset: 0.0,
                margin_top: cfg.get_paragraph_spacing_factor(),
                margin_bottom: cfg.get_paragraph_spacing_factor(),
            })
        }

        StyledLine::Header { segments, level } => {
            // ? Depending on the Header level, we will scale our font-size.
            let scaled_font_size =
                cfg.text_opts.font_size * cfg.header_opts.header_scales.scale_for_level(level);

            let available_width = cfg.canvas_opts.width
                - (cfg.canvas_opts.padding.left + cfg.canvas_opts.padding.right);

            // * Define the size of a the Line Box for this line.
            let mut buffer = Buffer::new(
                font_system,
                Metrics::new(
                    scaled_font_size,
                    scaled_font_size * cfg.text_opts.line_height_factor,
                ),
            );

            let styled_segments: Vec<(&str, Attrs)> = segments
                .iter()
                .flat_map(|block| match block {
                    StyledSegment::Text(block) => {
                        vec![(
                            block.text.as_str(),
                            Attrs::new().weight(block.weight).style(block.style),
                        )]
                    }
                    StyledSegment::HardBreak => {
                        vec![("\n", Attrs::new())]
                    }
                })
                .collect();

            let rich_text: Vec<(&str, Attrs)> = styled_segments
                .iter()
                .map(|(text, attrs)| (*text, attrs.clone()))
                .collect();

            buffer.set_rich_text(
                font_system,
                rich_text,
                &Attrs::new().family(Family::SansSerif),
                cosmic_text::Shaping::Advanced,
                None,
            );

            buffer.set_size(font_system, Some(available_width), Some(f32::MAX));

            LayoutResult::Line(LayoutLine {
                buffer,
                segments: segments.clone(),
                font_size: scaled_font_size,
                prefix_len: 0,
                indent_offset: 0.0,
                margin_top: cfg.header_opts.header_margin_top,
                margin_bottom: cfg.header_opts.header_margin_bot,
            })
        }

        StyledLine::BulletListItem { segments, indent } => {
            // * Define the size of a the Line Box for this line.
            let mut buffer = Buffer::new(
                font_system,
                Metrics::new(cfg.text_opts.font_size, cfg.get_line_spacing_factor()),
            );

            // * Calculate our indent by taking the number of indents and multiplying it by a unit size
            // * Our Unit Size is based on the font_size multiplied by an em value, default 1.5.
            let indent_size =
                indent as f32 * (cfg.text_opts.bullet_indent_em * cfg.text_opts.font_size);

            // * Update our available_width based on the indent and padding.
            let available_width = cfg.canvas_opts.width
                - (cfg.canvas_opts.padding.left + cfg.canvas_opts.padding.right)
                - indent_size;

            let prefix = format!("{} ", cfg.text_opts.bullet_char);

            let styled_segments: Vec<(String, Attrs)> = segments
                .iter()
                .enumerate()
                .flat_map(|(i, block)| {
                    match block {
                        StyledSegment::Text(block) => {
                            let text = if i == 0 {
                                // We need to insert the bullet character here, however we can't simply use format!().as_str
                                // as the String returned by format!() doesn't live long enough when it leaves the styled_segments scope.
                                // Therefore, we need to have `styled_segments` own the full String and later have `buffer` borrow it for it's final form.
                                format!("{}{}", &prefix, &block.text)
                            } else {
                                block.text.clone()
                            };

                            let attrs = Attrs::new().weight(block.weight).style(block.style);

                            vec![(text, attrs)]
                        }
                        StyledSegment::HardBreak => {
                            let text = "\n".to_string();
                            let attrs = Attrs::new();
                            vec![(text, attrs)]
                        }
                    }
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

            LayoutResult::Line(LayoutLine {
                buffer,
                segments: segments.clone(),
                font_size: cfg.text_opts.font_size,
                prefix_len: prefix.len(),
                indent_offset: indent_size,
                margin_top: cfg.get_paragraph_spacing_factor(),
                margin_bottom: cfg.get_paragraph_spacing_factor(),
            })
        }

        StyledLine::NumberedListItem {
            segments,
            number,
            indent,
        } => {
            // * Define the size of a the Line Box for this line.
            let mut buffer = Buffer::new(
                font_system,
                Metrics::new(cfg.text_opts.font_size, cfg.get_line_spacing_factor()),
            );

            // * Calculate our indent by taking the number of indents and multiplying it by a unit size
            // * Our Unit Size is based on the font_size multiplied by an em value, default 1.5.
            let indent_size =
                indent as f32 * (cfg.text_opts.bullet_indent_em * cfg.text_opts.font_size);

            // * Update our available_width based on the indent and prefix-length
            let available_width = cfg.canvas_opts.width
                - (cfg.canvas_opts.padding.left + cfg.canvas_opts.padding.right)
                - indent_size;

            let prefix = format!("{}. ", number);

            let styled_segments: Vec<(String, Attrs)> = segments
                .iter()
                .enumerate()
                .flat_map(|(i, block)| {
                    match block {
                        StyledSegment::Text(block) => {
                            let text = if i == 0 {
                                // We need to insert the bullet character here, however we can't simply use format!().as_str
                                // as the String returned by format!() doesn't live long enough when it leaves the styled_segments scope.
                                // Therefore, we need to have `styled_segments` own the full String and later have `buffer` borrow it for it's final form.
                                format!("{}{}", &prefix, &block.text)
                            } else {
                                block.text.clone()
                            };

                            let attrs = Attrs::new().weight(block.weight).style(block.style);

                            vec![(text, attrs)]
                        }
                        StyledSegment::HardBreak => {
                            let text = "\n".to_string();
                            let attrs = Attrs::new();
                            vec![(text, attrs)]
                        }
                    }
                })
                .collect();

            // * Cosmic Text expects a vector of 'spans' which are a collection of Strings and the attributes for that String.
            let rich_text: Vec<(&str, Attrs)> = styled_segments
                .iter()
                .map(|(text, attrs)| ((text.as_str()), attrs.clone()))
                .collect();

            buffer.set_rich_text(
                font_system,
                rich_text,
                &Attrs::new().family(Family::SansSerif),
                cosmic_text::Shaping::Advanced,
                None,
            );

            // * Our buffer size is limited by Width as we want accurate word-wrapping to the canvas size.
            // * Our height is unbounded because we are not testing for vertical space, as each line is run on a different Buffer.
            buffer.set_size(font_system, Some(available_width), Some(f32::MAX));

            LayoutResult::Line(LayoutLine {
                buffer,
                segments: segments.clone(),
                font_size: cfg.text_opts.font_size,
                prefix_len: prefix.len(),
                indent_offset: indent_size,
                margin_top: cfg.get_paragraph_spacing_factor(),
                margin_bottom: cfg.get_paragraph_spacing_factor(),
            })
        }
        StyledLine::Blank => LayoutResult::Blank {
            height: cfg.get_paragraph_spacing_factor(),
        },
        _ => panic! {"StyledLine to Layout has not yet implemented: {:#?}", &line.get_type()},
    }
}

pub fn process_layouts(layouts: Vec<LayoutResult>, cfg: &SvgConfig) -> Vec<String> {
    // * Return value
    let mut svg_lines: Vec<String> = Vec::new();

    // * Store mutuable vars for our moving offsets.
    let mut cumulative_y_offset = cfg.canvas_opts.padding.top;
    // For each layout in our document,
    // Process it into an SVG.
    // The returned values are the formatted SVG, and the last character index in the line and the current Y offset.
    // We use this offset to start looking through format segments, as our Run.glyph.start values will reset each run.

    let mut iter = layouts.into_iter().peekable();

    while let Some(layout) = iter.next() {
        match &layout {
            LayoutResult::Line(lyt) => {
                let (svgs, updated_y) = process_layout_line(lyt, cumulative_y_offset, cfg);
                svg_lines.extend(svgs);

                // * After each LayoutLine, we insert a gap to represent a line between paragraphs.
                // * We perform "margin collapsing" - attempting to mirror the CSS behaviour.
                let next_margin_top = iter.peek().map(|next| next.margin_top()).unwrap_or(0.0);

                let gap = layout.margin_bottom().max(next_margin_top);

                cumulative_y_offset = updated_y + gap;
            }

            LayoutResult::Blank { height } => {
                cumulative_y_offset = cumulative_y_offset + height;
            }
        }
        // Return final SVG collection.
    }
    svg_lines
}

fn process_layout_line(layout: &LayoutLine, y_cursor: f32, cfg: &SvgConfig) -> (Vec<String>, f32) {
    let mut svg_elements: Vec<String> = Vec::new();
    let mut cumulative_y = y_cursor;

    // * Build our Segment Map for this LayoutLine.
    let segment_ranges = build_segment_ranges(&layout.segments);

    let mut run_byte_offset: usize = 0;

    for (_idx, run) in layout.buffer.layout_runs().enumerate() {
        let baseline_y = y_cursor + run.line_y;

        let full_text: &String = &layout
            .segments
            .iter()
            .map(|seg| match seg {
                StyledSegment::Text(block) => block.text.clone(),
                StyledSegment::HardBreak => "\n".to_string(),
            })
            .collect();

        let current_x = cfg.canvas_opts.padding.left + layout.indent_offset; // handle starting offset for the line of text.

        let tspans = process_run(
            &run,
            &segment_ranges,
            &layout.segments,
            layout.prefix_len,
            layout.font_size,
            run_byte_offset,
        );

        cumulative_y = baseline_y;

        // * Cosmic will hold the full line text of a soft-wrapped line in all runs within the layout.
        // * Lines with a HardBreak, or newline, at the end will only have the line they are writing's content.
        // * Therefore, we use a run_byte_offset for wrapped runs, and do not for soft-wrapped runs.
        run_byte_offset =
            if full_text.as_bytes().get(run_byte_offset + run.glyphs.len()) == Some(&b'\n') {
                run_byte_offset + run.glyphs.len() + 1
            } else {
                0
            };

        svg_elements.push(tspans_to_svg(&tspans, current_x, run.line_y + y_cursor));
    }

    (svg_elements, cumulative_y)
}

fn process_run(
    run: &LayoutRun,
    segment_ranges: &Vec<SegmentRange>,
    segments: &Vec<StyledSegment>,
    prefix_len: usize,
    font_size: f32,
    run_byte_offset: usize,
) -> Vec<TSpan> {
    let mut tspans: Vec<TSpan> = vec![];

    // * Handle prefixes
    // ? Since we inserted our prefix, it isn't going to be part of our StyledSegment.
    // ? Therefore we need to process & emit it separately.
    if prefix_len > 0 {
        let mut prefix_text = String::new();

        // * Collect all glyphs that are part of the prefix
        for glyph in run.glyphs.iter().take_while(|g| g.start < prefix_len) {
            prefix_text.push_str(&run.text[glyph.start..glyph.end]);
        }

        if !prefix_text.is_empty() {
            tspans.push(TSpan {
                text: prefix_text,
                font_size: font_size,
                weight: Weight::NORMAL,
                style: Style::Normal,
                family: FamilyOwned::SansSerif,
            });
        }
    }

    let mut current_text: String = String::new(); // * Create a string buffer that holds all characters in a given segment range.
    let mut current_segment_idx: Option<usize> = None;

    // * Start investigating all 'glyphs' - or Unicode Codepoints.
    // ? It is important to note these are not necessarily entire characters or grapheme clusters.
    for glyph in run.glyphs.iter() {
        if glyph.start < prefix_len {
            continue; // Skip prefix glyphs
        }

        // Adjust the starting byte position to account for the prefix & our prior glyphs in the run.
        let adjusted_byte_pos = (glyph.start - prefix_len) + run_byte_offset;

        // Find which segment this glyph belongs to.
        let segment_range = segment_ranges.iter().find(|range| {
            adjusted_byte_pos >= range.start_byte && adjusted_byte_pos < range.end_byte
        });

        // If we don't get a segment, we are on a Line Break.
        let segment_idx = match segment_range {
            Some(seg) => seg.segment_idx,
            None => {
                //println!("Current text: {}", current_text);
                unreachable!(
                    "Glyph at index {} has no matching segment range - \
                    build_segment_ranges produced incomplete coverage",
                    adjusted_byte_pos
                );
            }
        };

        // Check if we have moved into a different segment.
        if let Some(prev_idx) = current_segment_idx {
            if prev_idx != segment_idx {
                // Emit the accumulated text as a TSpan with styling from the previous segment.
                let prev_segment = match &segments[prev_idx] {
                    StyledSegment::Text(block) => block,
                    _ => unreachable!("Non-text segment shouldn't have a Range"),
                };

                tspans.push(TSpan {
                    text: current_text.clone(),
                    font_size: font_size,
                    weight: prev_segment.weight,
                    style: prev_segment.style,
                    family: prev_segment.family.clone(),
                });

                // Reset text buffer and update our X to move inline with all previous characters.
                current_text.clear();
            }
        }
        let ch = &run.text[glyph.start..glyph.end];
        current_text.push_str(ch);
        current_segment_idx = Some(segment_idx);
    }

    // At the end of the run, if we have any text remaining in our buffer, crunch it.
    if !current_text.is_empty() {
        if let Some(seg_idx) = current_segment_idx {
            let segment = match &segments[seg_idx] {
                StyledSegment::Text(block) => block,
                _ => unreachable!("Non-text segment shouldn't have a Range"),
            };
            tspans.push(TSpan {
                text: current_text.clone(),
                font_size: font_size,
                weight: segment.weight,
                style: segment.style,
                family: segment.family.clone(),
            });
        }
    }

    tspans
}

/// Convert a `Tspan` into a raw SVG string  by a <text> tag.
fn tspans_to_svg(tspans: &[TSpan], x: f32, y: f32) -> String {
    let tspan_strings: Vec<String> = tspans
        .iter()
        .map(|ts| {
            let weight_attr = if ts.weight == Weight::BOLD {
                r#"font-weight="bold""#
            } else {
                ""
            };

            let style_attr = match ts.style {
                Style::Italic => r#"font-style="italic""#,
                Style::Oblique => r#"font-style="oblique""#,
                Style::Normal => "",
            };

            let font_family = match &ts.family {
                FamilyOwned::Name(smol_str) => {
                    format!(r#"font-family="{}, sans-serif""#, smol_str)
                }
                FamilyOwned::SansSerif => r#"font-family="sans-serif""#.to_string(),
                FamilyOwned::Serif => r#"font-family="serif""#.to_string(),
                FamilyOwned::Cursive => r#"font-family="cursive""#.to_string(),
                FamilyOwned::Fantasy => r#"font-family="fantasy""#.to_string(),
                FamilyOwned::Monospace => r#"font-family="monospace""#.to_string(),
            };

            let font_size = format!(r#"font-size="{}px""#, ts.font_size);
            // Return a formatted SVG String.
            format!(
                r#"<tspan {} {} {} {}>{}</tspan>"#, // TODO collapse whitespace properly for unused attrs
                font_size,
                weight_attr,
                style_attr,
                font_family,
                html_escape(&ts.text)
            )
        })
        .collect();
    format!(
        r#"<text x="{}" y="{}" font-family="sans-serif">{}</text>"#,
        x,
        y,
        tspan_strings.join("")
    )
}
///  Precompute the text range of our StyledBlock text as a byte range, which matches with the Cosmic Glyph start/end indices.
fn build_segment_ranges(segments: &Vec<StyledSegment>) -> Vec<SegmentRange> {
    let mut ranges = Vec::new();
    let mut current_pos = 0;

    for (seg_idx, segment) in segments.iter().enumerate() {
        //println!("Current Pos: {}", current_pos);
        match segment {
            StyledSegment::Text(block) => {
                let seg_len = block.text.len();

                ranges.push(SegmentRange {
                    segment_idx: seg_idx,
                    start_byte: current_pos,
                    end_byte: current_pos + seg_len,
                });

                current_pos += seg_len;
            }

            StyledSegment::HardBreak => {
                // No style - advance cursor.
                current_pos += 1;
            }
        }
    }
    // println!("Current Pos: {}", current_pos);
    // println!("Ranges: {:?}", ranges);
    ranges
}

fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use crate::{
        layout::{
            LayoutResult, SvgConfig, TSpan, build_segment_ranges, styled_line_to_layout,
            tspans_to_svg,
        },
        styles::{StyledBlock, StyledLine, StyledSegment},
    };
    use cosmic_text::{FamilyOwned, Style, Weight};

    // * --- utilities ---
    use std::path::Path;

    use cosmic_text::{FontSystem, fontdb::Database};

    /// Create a simple FontSystem with default Sans-Serif font derived from tests/fonts.
    pub fn create_default_test_font_system() -> FontSystem {
        // Create a default, empty FontDB
        let mut db = Database::new();
        // * Load Noto Sans from tests/fonts/
        // ? We load all fonts in this directory instead of loading the individual font variants (Italic, Bold etc)
        // ? Our default case is to access all of these, rather than loading specific fonts for each test.
        // ? we could break this out to accept a Weight/Style variant struct as an arg, and match accordingly later on if we need.
        db.load_fonts_dir(Path::new("tests/fonts/"));

        // override default Sans-Serif on db.
        db.set_sans_serif_family("Noto Sans"); // ! I don't know how to validate this...

        let font_sys = FontSystem::new_with_locale_and_db("en-US".to_string(), db);
        font_sys
    }
    // * --- tspan_to_svg ---
    #[test]
    fn tspans_to_svg_preserves_bold_weight_includes_attribute() {
        // * Arrange
        // Create a simple input text, no styling.
        let input_text = "Hello World".to_string();
        let attr = r#"font-weight="bold""#.to_string();
        // Form a TSpan with some default X/Y.
        let tspan = TSpan {
            text: input_text.clone(),
            font_size: 16.0,
            weight: Weight::BOLD,
            style: Style::Normal,
            family: FamilyOwned::SansSerif,
        };
        // * Act
        // call `tspans_to_svg` with the created TSpan, X, Y
        let svg_string = tspans_to_svg(&[tspan], 0.0, 16.0);
        // * Assert
        // CreatedTspanStr contains our simple input text.
        assert!(svg_string.contains(&input_text));
        assert!(svg_string.contains(&attr));
    }

    #[test]
    fn tspans_to_svg_normal_weight_omits_attribute() {
        // * Arrange
        // Create a simple input text, no styling.
        let input_text = "Hello World".to_string();
        let attr = r#"font-weight="bold""#.to_string();
        // Form a TSpan with some default X/Y.
        let tspan = TSpan {
            text: input_text.clone(),
            font_size: 16.0,
            weight: Weight::NORMAL,
            style: Style::Normal,
            family: FamilyOwned::SansSerif,
        };
        // * Act
        // call `tspans_to_svg` with the created TSpan, X, Y
        let svg_string = tspans_to_svg(&[tspan], 0.0, 16.0);
        // * Assert
        // CreatedTspanStr contains our simple input text.
        assert!(svg_string.contains(&input_text));
        assert!(!svg_string.contains(&attr));
    }

    #[test]
    fn tspans_to_svg_italic_style_includes_attribute() {
        // * Arrange
        // Create a simple input text, no styling.
        let input_text = "Hello World".to_string();
        let attr = r#"font-style="italic""#.to_string();
        // Form a TSpan with some default X/Y.
        let tspan = TSpan {
            text: input_text.clone(),
            font_size: 16.0,
            weight: Weight::NORMAL,
            style: Style::Italic,
            family: FamilyOwned::SansSerif,
        };
        // * Act
        // call `tspans_to_svg` with the created TSpan, X, Y
        let svg_string = tspans_to_svg(&[tspan], 0.0, 16.0);
        // * Assert
        // CreatedTspanStr contains our simple input text.
        assert!(svg_string.contains(&input_text));
        assert!(svg_string.contains(&attr));
    }

    #[test]
    fn tspans_to_svg_normal_style_omits_attribute() {
        // * Arrange
        // Create a simple input text, no styling.
        let input_text = "Hello World".to_string();
        let attr = r#"font-style="italic""#.to_string();
        // Form a TSpan with some default X/Y.
        let tspan = TSpan {
            text: input_text.clone(),
            font_size: 16.0,
            weight: Weight::NORMAL,
            style: Style::Normal,
            family: FamilyOwned::SansSerif,
        };
        // * Act
        // call `tspans_to_svg` with the created TSpan, X, Y
        let svg_string = tspans_to_svg(&[tspan], 0.0, 16.0);
        // * Assert
        // CreatedTspanStr contains our simple input text.
        assert!(svg_string.contains(&input_text));
        assert!(!svg_string.contains(&attr));
    }

    #[test]
    fn tspans_to_svg_text_content_matches_input() {
        // * Arrange
        // Create a simple input text, no styling.
        let input_text = "Hello World".to_string();

        // Form a TSpan with some defaults
        let tspan = TSpan {
            text: input_text.clone(),
            font_size: 16.0,
            weight: Weight::NORMAL,
            style: Style::Normal,
            family: FamilyOwned::SansSerif,
        };

        // * Act
        // call `tspans_to_svg` with the created TSpan, X, Y
        let svg_string = tspans_to_svg(&[tspan], 0.0, 16.0);
        // * Assert
        // CreatedTspanStr contains our simple input text.
        assert!(svg_string.contains(&input_text));
    }

    #[test]
    fn tspans_to_svg_escapes_html_characters() {
        // * Arrange
        let input_text = r#"<&>""#.to_string();
        let compare_text = "&lt;&amp;&gt;&quot;";

        let tspan = TSpan {
            text: input_text.clone(),
            font_size: 16.0,
            weight: Weight::NORMAL,
            style: Style::Normal,
            family: FamilyOwned::SansSerif,
        };

        let svg_string = tspans_to_svg(&[tspan], 0.0, 16.0);
        assert!(svg_string.contains(&compare_text));
    }
    // * --- build_segment_ranges ---
    #[test]
    fn build_segment_single_text_segment_has_correct_byte_offsets() {
        // * Arrange
        let seg_text = "Hello World".to_string();
        // Create a vector of StyledSegments.
        let segment = vec![StyledSegment::Text(StyledBlock {
            text: seg_text.clone(),
            weight: Weight::NORMAL,
            style: Style::Normal,
            family: FamilyOwned::SansSerif,
        })];

        // * Act
        let segment_ranges = build_segment_ranges(&segment);

        // * Assert
        assert_eq!(segment_ranges.len(), 1);
        let range = &segment_ranges[0];
        assert_eq!(range.start_byte, 0);
        assert_eq!(range.end_byte, 11);
    }

    #[test]
    fn build_segment_ranges_consecutive_segments_have_adjusted_offsets() {
        let first_segment_text = "Hello ".to_string();
        let second_segment_text = "World".to_string();

        let segments = vec![
            StyledSegment::Text(StyledBlock {
                text: first_segment_text.clone(),
                weight: Weight::NORMAL,
                style: Style::Normal,
                family: FamilyOwned::SansSerif,
            }),
            StyledSegment::Text(StyledBlock {
                text: second_segment_text.clone(),
                weight: Weight::NORMAL,
                style: Style::Normal,
                family: FamilyOwned::SansSerif,
            }),
        ];

        let segment_ranges = build_segment_ranges(&segments);

        // * assert
        assert_eq!(segment_ranges.len(), 2);
        assert_eq!(segment_ranges[0].start_byte, 0);
        assert_eq!(segment_ranges[0].end_byte, 6);
        assert_eq!(segment_ranges[1].start_byte, 6);
    }

    #[test]
    fn build_segment_ranges_hard_break_advances_offset_by_one() {
        let first_segment_text = "Hello ".to_string();
        let second_segment_text = "World".to_string();

        let segments = vec![
            StyledSegment::Text(StyledBlock {
                text: first_segment_text.clone(),
                weight: Weight::NORMAL,
                style: Style::Normal,
                family: FamilyOwned::SansSerif,
            }),
            StyledSegment::HardBreak,
            StyledSegment::Text(StyledBlock {
                text: second_segment_text.clone(),
                weight: Weight::NORMAL,
                style: Style::Normal,
                family: FamilyOwned::SansSerif,
            }),
        ];

        let segment_ranges = build_segment_ranges(&segments);

        // * assert
        assert_eq!(segment_ranges.len(), 2); // Remains 2! Only count Text segments.
        assert_eq!(segment_ranges[0].start_byte, 0);
        assert_eq!(segment_ranges[0].end_byte, 6); // Does not modify the contents of Segment 1!
        assert_eq!(segment_ranges[1].start_byte, 7); // Offset by 1 from Hard Break.
    }

    // * --- styled_line_to_layout ---
    #[test]
    fn styled_line_to_layout_paragraph() {
        // * Arrange
        let mut font_system = create_default_test_font_system();
        let cfg = SvgConfig::default();

        let paragraph_text = "This is some paragraph text.".to_string();

        let segments = vec![StyledSegment::Text(StyledBlock {
            text: paragraph_text.clone(),
            weight: Weight::NORMAL,
            style: Style::Normal,
            family: FamilyOwned::SansSerif,
        })];

        // Create a basic StyledLine
        let styled_line_para = StyledLine::Paragraph {
            segments: segments.clone(),
        };

        // * Act
        let layout_result = styled_line_to_layout(styled_line_para, &mut font_system, &cfg);

        // * Assert
        // Assert LayoutResult is the Line variant (not Blank)
        assert!(matches!(layout_result, LayoutResult::Line(_)));
        let LayoutResult::Line(line) = layout_result else {
            panic!("Expected LayoutResult::Line");
        };
        // Assert segments match the input StyledLine's segments
        assert_eq!(line.segments, segments);
        // Assert font_size matches cfg.text_opts.font_size
        assert_eq!(line.font_size, cfg.text_opts.font_size);
        // Assert margin_top matches cfg.paragraph_spacing()
        assert_eq!(line.margin_top, cfg.get_paragraph_spacing_factor());
        // Assert margin_bottom matches cfg.paragraph_spacing()
        assert_eq!(line.margin_bottom, cfg.get_paragraph_spacing_factor());
        // Assert prefix_len is 0 (paragraphs have no prefix)
        assert_eq!(line.prefix_len, 0);
        // Assert indent_offset is 0.0 (paragraphs have no indent)
        assert_eq!(line.indent_offset, 0.0);
    }

    #[test]
    fn styled_line_to_layout_paragraph_handles_hard_break() {
        // * Arrange
        let mut font_system = create_default_test_font_system();
        let cfg = SvgConfig::default();

        let paragraph_text = "This is some paragraph text.".to_string();

        let segments = vec![
            StyledSegment::Text(StyledBlock {
                text: paragraph_text.clone(),
                weight: Weight::NORMAL,
                style: Style::Normal,
                family: FamilyOwned::SansSerif,
            }),
            StyledSegment::HardBreak,
        ];

        // Create a basic StyledLine
        let styled_line_para = StyledLine::Paragraph {
            segments: segments.clone(),
        };

        // * Act
        let layout_result = styled_line_to_layout(styled_line_para, &mut font_system, &cfg);

        // * Assert
        // Assert LayoutResult is the Line variant (not Blank)
        assert!(matches!(layout_result, LayoutResult::Line(_)));
        let LayoutResult::Line(line) = layout_result else {
            panic!("Expected LayoutResult::Line");
        };
        // Assert segments match the input StyledLine's segments
        assert_eq!(line.segments, segments);
        // Assert font_size matches cfg.text_opts.font_size
        assert_eq!(line.font_size, cfg.text_opts.font_size);
        // Assert margin_top matches cfg.paragraph_spacing()
        assert_eq!(line.margin_top, cfg.get_paragraph_spacing_factor());
        // Assert margin_bottom matches cfg.paragraph_spacing()
        assert_eq!(line.margin_bottom, cfg.get_paragraph_spacing_factor());
        // Assert prefix_len is 0 (paragraphs have no prefix)
        assert_eq!(line.prefix_len, 0);
        // Assert indent_offset is 0.0 (paragraphs have no indent)
        assert_eq!(line.indent_offset, 0.0);
    }

    #[test]
    fn styled_line_to_layout_header_applies_font_scale() {
        // * Arrange
        let mut font_system = create_default_test_font_system();
        let cfg = SvgConfig::default();

        let header_text = "# Header".to_string();

        let segments = vec![StyledSegment::Text(StyledBlock {
            text: header_text.clone(),
            weight: Weight::BOLD,
            style: Style::Normal,
            family: FamilyOwned::SansSerif,
        })];

        // Create a basic StyledLine
        let styled_line_para = StyledLine::Header {
            segments: segments.clone(),
            level: 1,
        };

        // * Act
        let layout_result = styled_line_to_layout(styled_line_para, &mut font_system, &cfg);

        // * Assert
        // Assert LayoutResult is the Line variant (not Blank)
        assert!(matches!(layout_result, LayoutResult::Line(_)));
        let LayoutResult::Line(line) = layout_result else {
            panic!("Expected LayoutResult::Line");
        };

        assert_eq!(line.segments, segments);
        assert_eq!(
            line.font_size,
            cfg.text_opts.font_size * cfg.header_opts.header_scales.scale_for_level(1)
        );
        assert_eq!(line.margin_top, cfg.header_opts.header_margin_top);
        assert_eq!(line.margin_bottom, cfg.header_opts.header_margin_bot);
        assert_eq!(line.prefix_len, 0);
        assert_eq!(line.indent_offset, 0.0);
    }

    #[test]
    fn styled_line_to_bullet_list_item_applies_prefix_length() {
        // * Arrange
        let mut font_system = create_default_test_font_system();
        let cfg = SvgConfig::default();

        let header_text = "- Bullet Item".to_string();

        let segments = vec![StyledSegment::Text(StyledBlock {
            text: header_text.clone(),
            weight: Weight::NORMAL,
            style: Style::Normal,
            family: FamilyOwned::SansSerif,
        })];

        // Create a basic StyledLine
        let styled_line_para = StyledLine::BulletListItem {
            segments: segments.clone(),
            indent: 0,
        };

        // * Act
        let layout_result = styled_line_to_layout(styled_line_para, &mut font_system, &cfg);

        // * Assert
        // Assert LayoutResult is the Line variant (not Blank)
        assert!(matches!(layout_result, LayoutResult::Line(_)));
        let LayoutResult::Line(line) = layout_result else {
            panic!("Expected LayoutResult::Line");
        };

        assert_eq!(line.segments, segments);
        assert_eq!(line.font_size, cfg.text_opts.font_size);
        assert_eq!(line.prefix_len, 4); //? Character "•" is 3 bytes, followed by a single white-space = 4 bytes total.
        assert_eq!(line.indent_offset, 0.0);
        assert_eq!(line.margin_top, cfg.get_paragraph_spacing_factor());
        assert_eq!(line.margin_bottom, cfg.get_paragraph_spacing_factor());
    }

    #[test]
    fn styled_line_to_bullet_list_item_applies_indent_offset() {
        // * Arrange
        let mut font_system = create_default_test_font_system();
        let cfg = SvgConfig::default();

        let header_text = "- Bullet Item".to_string();

        let segments = vec![StyledSegment::Text(StyledBlock {
            text: header_text.clone(),
            weight: Weight::NORMAL,
            style: Style::Normal,
            family: FamilyOwned::SansSerif,
        })];

        // Create a basic StyledLine
        let styled_line_para = StyledLine::BulletListItem {
            segments: segments.clone(),
            indent: 3,
        };

        // * Act
        let layout_result = styled_line_to_layout(styled_line_para, &mut font_system, &cfg);

        // * Assert
        // Assert LayoutResult is the Line variant (not Blank)
        assert!(matches!(layout_result, LayoutResult::Line(_)));
        let LayoutResult::Line(line) = layout_result else {
            panic!("Expected LayoutResult::Line");
        };

        assert_eq!(line.segments, segments);
        assert_eq!(line.font_size, cfg.text_opts.font_size);
        assert_eq!(line.prefix_len, 4); //? Character "•" is 3 bytes, followed by a single white-space = 4 bytes total.
        assert_eq!(
            line.indent_offset,
            3.0 * (cfg.text_opts.bullet_indent_em * cfg.text_opts.font_size)
        );
        assert_eq!(line.margin_top, cfg.get_paragraph_spacing_factor());
        assert_eq!(line.margin_bottom, cfg.get_paragraph_spacing_factor());
    }

    #[test]
    fn styled_line_to_numbered_list_item_applies_prefix_length() {
        // * Arrange
        let mut font_system = create_default_test_font_system();
        let cfg = SvgConfig::default();

        let header_text = "- Bullet Item".to_string();

        let segments = vec![StyledSegment::Text(StyledBlock {
            text: header_text.clone(),
            weight: Weight::NORMAL,
            style: Style::Normal,
            family: FamilyOwned::SansSerif,
        })];

        let list_item_number = 6;

        // Create a basic StyledLine
        let styled_line_para = StyledLine::NumberedListItem {
            segments: segments.clone(),
            number: list_item_number,
            indent: 0,
        };

        // * Act
        let layout_result = styled_line_to_layout(styled_line_para, &mut font_system, &cfg);

        // * Assert
        // Assert LayoutResult is the Line variant (not Blank)
        assert!(matches!(layout_result, LayoutResult::Line(_)));
        let LayoutResult::Line(line) = layout_result else {
            panic!("Expected LayoutResult::Line");
        };

        assert_eq!(line.segments, segments);
        assert_eq!(line.font_size, cfg.text_opts.font_size);
        assert_eq!(line.prefix_len, 3); // * prefixes for NumberedList are "#. " - 3 bytes: ASCII #, period, space.
        assert_eq!(line.indent_offset, 0.0);
        assert_eq!(line.margin_top, cfg.get_paragraph_spacing_factor());
        assert_eq!(line.margin_bottom, cfg.get_paragraph_spacing_factor());
    }

    #[test]
    fn styled_line_to_numbered_list_item_applies_indent_offset() {
        // * Arrange
        let mut font_system = create_default_test_font_system();
        let cfg = SvgConfig::default();

        let header_text = "- Bullet Item".to_string();

        let segments = vec![StyledSegment::Text(StyledBlock {
            text: header_text.clone(),
            weight: Weight::NORMAL,
            style: Style::Normal,
            family: FamilyOwned::SansSerif,
        })];

        // Create a basic StyledLine
        let styled_line_para = StyledLine::NumberedListItem {
            segments: segments.clone(),
            number: 2,
            indent: 2,
        };

        // * Act
        let layout_result = styled_line_to_layout(styled_line_para, &mut font_system, &cfg);

        // * Assert
        // Assert LayoutResult is the Line variant (not Blank)
        assert!(matches!(layout_result, LayoutResult::Line(_)));
        let LayoutResult::Line(line) = layout_result else {
            panic!("Expected LayoutResult::Line");
        };

        assert_eq!(line.segments, segments);
        assert_eq!(line.font_size, cfg.text_opts.font_size);
        assert_eq!(line.prefix_len, 3); // * prefixes for NumberedList are "#. " - 3 bytes: ASCII #, period, space.
        assert_eq!(
            line.indent_offset,
            2.0 * (cfg.text_opts.bullet_indent_em * cfg.text_opts.font_size)
        );
        assert_eq!(line.margin_top, cfg.get_paragraph_spacing_factor());
        assert_eq!(line.margin_bottom, cfg.get_paragraph_spacing_factor());
    }

    #[test]
    fn styled_line_blank_returns_line_height() {
        // * Arrange
        let mut font_system = create_default_test_font_system();
        let cfg = SvgConfig::default();

        // Create a blank StyledLine
        let styled_line_para = StyledLine::Blank;

        // * Act
        let layout_result = styled_line_to_layout(styled_line_para, &mut font_system, &cfg);

        // * Assert
        // Assert LayoutResult is the Line variant (not Blank)
        assert!(matches!(layout_result, LayoutResult::Blank { height: _ }));
        let LayoutResult::Blank { height } = layout_result else {
            panic!("Expected LayoutResult::Blank");
        };
        assert_eq!(height, cfg.get_paragraph_spacing_factor());
    }
}
