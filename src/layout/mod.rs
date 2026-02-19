use core::panic;
use cosmic_text::{Attrs, Buffer, Family, FontSystem, Metrics, Style, Weight};
use std::{char, collections::HashMap};

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
}
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

// todo 
pub fn layout_line_to_svg(layout: LayoutLine, cfg: &SvgConfig) -> Vec<String> {
    let mut svg_elements = Vec::new();
    
    // Build byte position → segment mapping once
    let segment_ranges = build_segment_ranges(&layout.segments);
    
    for run in layout.buffer.layout_runs() {
        let line_y = cfg.top_padding + run.line_y;
        let mut tspans = Vec::new();
        
        // ═══════════════════════════════════════════════════════
        // STEP 1: Handle prefix (if present in this run)
        // ═══════════════════════════════════════════════════════
        if layout.prefix_len > 0 {
            let mut prefix_text = String::new();
            
            // Collect all glyphs that are part of the prefix
            for glyph in run.glyphs.iter().take_while(|g| g.start < layout.prefix_len) {
                prefix_text.push_str(&run.text[glyph.start..glyph.end]);
            }
            
            if !prefix_text.is_empty() {
                tspans.push(TSpan {
                    text: prefix_text,
                    x: cfg.left_padding + layout.indent_offset,
                    y: line_y,
                    weight: Weight::NORMAL,
                    style: Style::Normal,
                });
            }
        }
        
        // ═══════════════════════════════════════════════════════
        // STEP 2: Process content glyphs, grouping by segment
        // ═══════════════════════════════════════════════════════
        let mut current_text = String::new();
        let mut current_segment_idx: Option<usize> = None;
        let mut current_x = cfg.left_padding + layout.indent_offset;
        
        // todo Could we shorten the range of our loop instead of validating we are in the right location?
        for glyph in run.glyphs.iter() {
            // Skip prefix glyphs (already handled above)
            if glyph.start < layout.prefix_len {
                continue;
            }
            
            // Adjust byte position to account for prefix
            // ! validate this
            let adjusted_byte_pos = glyph.start - layout.prefix_len;
            
            // Find which segment this glyph belongs to
            let segment_idx = segment_ranges.iter()
                .find(|range| {
                    adjusted_byte_pos >= range.start_byte 
                    && adjusted_byte_pos < range.end_byte
                })
                .map(|range| range.segment_idx)
                .unwrap_or(0);  // Fallback to first segment
            
            // Check if we've moved to a different segment
            if let Some(prev_idx) = current_segment_idx {
                if prev_idx != segment_idx {
                    // Emit the accumulated tspan for previous segment
                    let prev_segment = &layout.segments[prev_idx];
                    tspans.push(TSpan {
                        text: current_text.clone(),
                        x: current_x,
                        y: line_y,
                        weight: prev_segment.weight,
                        style: prev_segment.style,
                    });
                    
                    // Start new tspan for new segment
                    current_text.clear();
                    current_x = cfg.left_padding + layout.indent_offset + glyph.x;
                }
            } else {
                // First content glyph - set initial x position
                current_x = cfg.left_padding + layout.indent_offset + glyph.x;
            }
            
            // Accumulate this glyph's character(s)
            let ch = &run.text[glyph.start..glyph.end];
            current_text.push_str(ch);
            current_segment_idx = Some(segment_idx);
        }
        
        // Emit final tspan (if any content was accumulated)
        if !current_text.is_empty() {
            if let Some(seg_idx) = current_segment_idx {
                let segment = &layout.segments[seg_idx];
                tspans.push(TSpan {
                    text: current_text,
                    x: current_x,
                    y: line_y,
                    weight: segment.weight,
                    style: segment.style,
                });
            }
        }
        
        // ═══════════════════════════════════════════════════════
        // STEP 3: Convert TSpans to SVG string
        // ═══════════════════════════════════════════════════════
        // todo implement tspans to svg.
        let svg_line = tspans_to_svg(&tspans);
        svg_elements.push(svg_line);
    }
    
    svg_elements
}

fn tspans_to_svg(tspans: &[TSpan]) -> String {
    let tspan_strings: Vec<String> = tspans.iter()
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
                r#"<tspan x-"{}" y="{}"{}{}>{}</tspan>"#,
                ts.x,
                ts.y,
                weight_attr,
                style_attr,
                html_escape(&ts.text)
            )
        }).collect();

        // todo handle custom font-size & font family
        format!(r#"<text font-family="sans-serif"" font-size="16">{}</text>"#, tspan_strings.join(""))
}
///  Precompute the text range of our StyledBlock text as a byte range, which matches with the Cosmic Glyph start/end indices.
fn build_segment_ranges(segments: &Vec<StyledBlock>) -> Vec<SegmentRange> {
    let mut ranges = Vec::new();

    let mut current_pos = 0;

    for (seg_idx, segment) in segments.iter().enumerate() {
        let seg_len = segment.text.len();
        ranges.push(SegmentRange {
            segment_idx: seg_idx,
            start_byte: current_pos,
            end_byte: current_pos + seg_len,
        });
        current_pos += seg_len
    }
    ranges
}

// [x] Extract layout from Cosmic Text buffers (buffer.layout_runs())
// [x] Track y-positions as you stack lines vertically
// [x] Apply x-offsets for indentation
// [x] Convert to SVG <text> and <tspan> elements

fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}