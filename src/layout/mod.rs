use core::{f32, panic};
use cosmic_text::{Attrs, Buffer, Family, FontSystem, LayoutRun, Metrics, Style, Weight};
use std::{char, collections::HashMap};

use crate::styles::{StyledLine, StyledSegment};

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
    /// Space between discrete text blocks.
    pub line_height_factor: f32,
    /// Space between lines inside of a paragraph.
    pub list_item_spacing_em: f32,

    // Bullet Style Options
    pub bullet_indent_em: f32, // default 1.5 or 2.0 ->
    pub bullet_char: char,

    // Header Style Options
    pub header_scales: HashMap<u8, f32>,
    pub header_margin_top: f32,
    pub header_margin_bot: f32,

    pub bg_color: String,
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
            line_height_factor: 1.5,
            list_item_spacing_em: 0.6,
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
    /// Space between discrete text blocks.
    pub const fn line_height(&self) -> f32 {
        self.font_size * self.line_height_factor
    }
    /// Space between lines inside of a paragraph.
    pub const fn list_item_spacing(&self) -> f32 {
        self.font_size * self.list_item_spacing_em
    }
}

#[derive(Debug)]
pub struct LayoutLine {
    pub buffer: Buffer,
    pub segments: Vec<StyledSegment>,
    pub font_size: f32,
    pub prefix_len: usize,
    pub indent_offset: f32,
    pub margin_top : f32,
    pub margin_bottom : f32
}

pub enum LayoutResult {
    Line(LayoutLine),
    Blank { height: f32 },
}

impl LayoutResult {
    pub fn margin_top(&self) -> f32 {
        match self {
            LayoutResult::Line(lyt) => lyt.margin_top,
            LayoutResult::Blank { .. } => 0.0 // Fixed space
        }
    }
    
    pub fn margin_bottom (&self) -> f32 {
        match self {
            LayoutResult::Line(lyt) => lyt.margin_bottom,
            LayoutResult::Blank { .. } => 0.0 // Fixed space
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
}

pub fn styled_line_to_layout(
    line: StyledLine,
    font_system: &mut FontSystem,
    cfg: &SvgConfig,
) -> LayoutResult {
    // * Arrange our default available width based on the overall SVG size minus any L/R padding.

    match line {
        StyledLine::Paragraph { segments } => {
            let available_width = cfg.width - (cfg.left_padding + cfg.right_padding);
            
            let mut buffer =
                Buffer::new(font_system, Metrics::new(cfg.font_size, cfg.line_height()));

            let styled_segments: Vec<(&str, Attrs)> = segments
                .iter()
                .flat_map(|seg| {
                    match seg {
                        StyledSegment::Text(block) => {
                            //styled_blocks.push(block.clone());
                            vec![(
                                block.text.as_str(),
                                Attrs::new().weight(block.weight).style(block.style),
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
                font_size: cfg.font_size,
                prefix_len: 0,
                indent_offset: 0.0,
                margin_top: cfg.line_height(),
                margin_bottom: cfg.line_height()
            })
        }

        StyledLine::Header { segments, level } => {
            // ? Depending on the Header indentation, we will need to increase the size of our font.
            // ? We store the multipler in a hash-table in our config.
            let scaled_font_size = cfg.font_size * cfg.header_scales[&level];

            let available_width = cfg.width - (cfg.left_padding + cfg.right_padding);

            let mut buffer = Buffer::new(
                font_system,
                Metrics::new(scaled_font_size, scaled_font_size * cfg.line_height_factor),
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
                margin_top: cfg.header_margin_top,
                margin_bottom: cfg.header_margin_bot
            })
        }

        StyledLine::BulletListItem { segments, indent } => {
            let mut buffer = Buffer::new(
                font_system,
                // todo This *looks* right, but is not exactly what I want.
                // todo The original configuration should be changed rather than using the wrong var to achieve the right effect.
                Metrics::new(cfg.font_size, cfg.list_item_spacing()),
            );

            // * Calculate our indent by taking the number of indents and multiplying it by a unit size
            // * Our Unit Size is based on the font_size multiplied by an em value, default 1.5.
            let indent_size = indent as f32 * (cfg.bullet_indent_em * cfg.font_size);

            // * Update our available_width based on the indent and prefix-length
            let available_width = cfg.width - (cfg.left_padding + cfg.right_padding) - indent_size;

            let prefix = format!("{} ", cfg.bullet_char);

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
                font_size: cfg.font_size,
                prefix_len: prefix.len(),
                indent_offset: indent_size,
                margin_top: cfg.list_item_spacing(),
                margin_bottom: cfg.list_item_spacing()
                
            })
        }

        StyledLine::NumberedListItem {
            segments,
            number,
            indent,
        } => {
            let mut buffer = Buffer::new(
                font_system,
                Metrics::new(cfg.font_size, cfg.list_item_spacing()),
            );

            // * Calculate our indent by taking the number of indents and multiplying it by a unit size
            // * Our Unit Size is based on the font_size multiplied by an em value, default 1.5.
            let indent_size = indent as f32 * (cfg.bullet_indent_em * cfg.font_size);

            // * Update our available_width based on the indent and prefix-length
            let available_width = cfg.width - (cfg.left_padding + cfg.right_padding) - indent_size;

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
                font_size: cfg.font_size,
                prefix_len: prefix.len(),
                indent_offset: indent_size,
                margin_top: cfg.list_item_spacing(),
                margin_bottom: cfg.list_item_spacing()
            })
        }
        StyledLine::Blank => {
            LayoutResult::Blank {height: cfg.line_height()}
        }
        _ => panic! {"StyledLine to Layout has not yet implemented: {:#?}", &line.get_type()},
    }
}

pub fn process_layouts(layouts: Vec<LayoutResult>, cfg: &SvgConfig) -> Vec<String> {
    // * Return value
    let mut svg_lines: Vec<String> = Vec::new();

    // * Store mutuable vars for our moving offsets.
    let mut cumulative_y_offset = cfg.top_padding;
    // For each layout in our document,
    // Process it into an SVG.
    // The returned values are the formatted SVG, and the last character index in the line and the current Y offset.
    // We use this offset to start looking through format segments, as our Run.glyph.start values will reset each run.
    
    let mut iter = layouts.into_iter().peekable();
    
    while let Some(layout) = iter.next(){
        match &layout {
            LayoutResult::Line(lyt) => {
                let (svgs, updated_y) = process_layout_line(lyt, cumulative_y_offset, cfg);
                svg_lines.extend(svgs);

                // * After each LayoutLine, we insert a gap to represent a line between paragraphs.
                // * We perform "margin collapsing" - attempting to mirror the CSS behaviour.
                let next_margin_top = iter.peek()
                .map(|next| next.margin_top())
                .unwrap_or(0.0);
            
                let gap = layout.margin_bottom().max(next_margin_top);
                cumulative_y_offset = updated_y + gap;
            }
            
            LayoutResult::Blank{height} => {
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

    
    for (idx, run) in layout.buffer.layout_runs().enumerate() {
        let baseline_y = y_cursor + run.line_y;
        //println!("Layout Segments: {:#?}", layout.segments);
        
        println!(
            "Run index: {} | text: {:?} | run.text length {}",
            idx,
            run.text,
            run.text.len()
        );

        
        
        let full_text: &String = &layout
            .segments
            .iter()
            .map(|seg| match seg {
                StyledSegment::Text(block) => block.text.clone(),
                StyledSegment::HardBreak => "\n".to_string(),
            })
            .collect();

        let current_x = cfg.left_padding + layout.indent_offset; // handle starting offset for the line of text.

        let tspans = process_run(
            &run,
            &segment_ranges,
            &layout.segments,
            layout.prefix_len,
            layout.font_size,
            run_byte_offset,
        );
        
        cumulative_y = baseline_y;
        run_byte_offset += run.text.len()
            + if full_text.as_bytes().get(run_byte_offset + run.text.len()) == Some(&b'\n') {
                1
            } else {
                0
            };

        svg_elements.push(tspans_to_svg(&tspans, current_x, cumulative_y));
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
        let adjusted_byte_pos = glyph.start + run_byte_offset - prefix_len;

        // Find which segment this glyph belongs to.
        let segment_range = segment_ranges.iter().find(|range| {
            adjusted_byte_pos >= range.start_byte && adjusted_byte_pos < range.end_byte
        });

        // If we don't get a segment, we are on a Line Break.
        let segment_idx = match segment_range {
            Some(seg) => seg.segment_idx,
            None => continue,
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
                });

                // Reset text buffer and update our X to move inline with all previous characters.
                current_text.clear();
            }
        }

        // Add this glyph to our text buffer.
        println!(
            "Glyph Character: {} | Start: {}, Glyph End: {}",
            &run.text[glyph.start..glyph.end],
            glyph.start,
            glyph.end
        );
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
                r#" font-weight="bold" "#
            } else {
                ""
            };

            let style_attr = match ts.style {
                Style::Italic => r#" font-style="italic""#,
                Style::Oblique => r#" font-style="oblique""#,
                Style::Normal => "",
            };

            let font_size = format!(r#" font-size="{}px""#, ts.font_size);
            // Return a formatted SVG String.
            format!(
                r#"<tspan {}{}{}>{}</tspan>"#,
                font_size,
                weight_attr,
                style_attr,
                html_escape(&ts.text)
            )
        })
        .collect();

    // todo handle custom font-size & font family
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
                println!("Segment text: {:?}", block.text);
                let seg_len = block.text.len();

                ranges.push(SegmentRange {
                    segment_idx: seg_idx,
                    start_byte: current_pos,
                    end_byte: current_pos + seg_len,
                });

                current_pos += seg_len;
            }

            // todo Investigate this breaking text with '\'.
            StyledSegment::HardBreak => {
                // No style - advance cursor.
                current_pos += 1;
            }
        }
    }
    println!("Current Pos: {}", current_pos);
    println!("Ranges: {:?}", ranges);
    ranges
}

fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
