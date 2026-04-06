use core::f32;
use cosmic_text::{Buffer, FamilyOwned, LayoutRun, Style, Weight};

use crate::config::SvgConfig;

// This should use a From impl that takes in a StyledSpan and adjusts accordingly?
/// A range of text with a specific style associated with `cosmic_text` style types.
#[derive(Debug, Clone, PartialEq)]
pub enum LayoutInline {
    Text {
        text: String,
        weight: Weight,
        style: Style,
        family: FamilyOwned,
    },
    HardBreak, // No data
    InlineLink {
        link_text: Vec<LayoutInline>,
        url: String,
        title: Option<String>,
    },
}
// TODO - introduce separate type for nested variants to ensure we can only have valid text inside of Links.
impl LayoutInline {
    pub fn raw_text(&self) -> String {
        match self {
            LayoutInline::Text { text, .. } => text.clone(),
            LayoutInline::HardBreak => "\n".to_string(),
            LayoutInline::InlineLink { link_text, .. } => {
                link_text.iter().map(|child| child.raw_text()).collect()
            }
        }
    }
}

/// Shaped buffer, ready for SVG conversion
#[derive(Debug)]
pub struct LayoutBlock {
    pub buffer: Buffer,
    pub segments: Vec<LayoutInline>,
    pub font_size: f32,
    pub prefix_len: usize,
    pub indent_offset: f32,
    pub margin_top: f32,
    pub margin_bottom: f32,
}

/// A discrete unit of work for the layout engine.
/// Created by collapsing a StyleBlock into specific layout and rendering instructions.
/// This may be a LayoutBlock or a non-text structural element.
pub enum LayoutItem {
    Block(LayoutBlock),
    ThematicBreak { left: f32, right: f32 }, //? Support for creative thematic breaks?
    Blank { height: f32 },
}

impl LayoutItem {
    pub fn margin_top(&self) -> f32 {
        match self {
            LayoutItem::Block(lyt) => lyt.margin_top,
            LayoutItem::ThematicBreak { .. } => 0.0, // Fixed space
            LayoutItem::Blank { .. } => 0.0,         // Fixed space
        }
    }

    pub fn margin_bottom(&self) -> f32 {
        match self {
            LayoutItem::Block(lyt) => lyt.margin_bottom,
            LayoutItem::ThematicBreak { .. } => 0.0, // Fixed space
            LayoutItem::Blank { .. } => 0.0,         // Fixed space
        }
    }
}

/// A range of glyph indices in the source text that carry a consistent style.
///
// ? *This is used to map the result of Cosmic's shaping to the parsed styles, as this is lost during transformation.*
#[derive(Debug)]
pub struct StyledInlineRange {
    segment_idx: usize,
    start_byte: usize,
    end_byte: usize,
    child_idx: Option<usize>,
}

/// Complete definition of a Text Span SVG element.
// TODO - introduce separate type for nested variants to ensure we can only have valid variants by type.
#[derive(Clone)]
pub enum TspanDefinition {
    Text {
        text: String,
        font_size: f32,
        weight: Weight,
        style: Style,
        family: FamilyOwned,
    },
    InlineLink {
        text: Vec<TspanDefinition>,
        url: String,
        title: Option<String>,
    },
}

#[derive(Default)]
struct RunState {
    current_text: String,
    current_segment_idx: Option<usize>,
    current_child_idx: Option<usize>,
    /// Styled Link Text, without metadata
    link_tspans: Vec<TspanDefinition>,
}
impl RunState {
    // TODO: font_size belongs on LayoutInline::Text - relevant when implementing sub/superscript.
    // TODO - reword this to be less robotic.
    /// Move forward in the text state.
    /// Compares the current state versus the inccoming state, optionally returning a TSpanDefinition if a segment change is detected.
    /// In the case of a child segment changing within a segment, format and store the child range to be returned with other children on segment change.
    /// 
    fn advance(
        &mut self,
        new_char: &str,
        new_segment_idx: usize,
        new_child_idx: Option<usize>,
        segments: &[LayoutInline],
        font_size: f32,
    ) -> Option<TspanDefinition> {
        // todo rename
        let result = match self.current_segment_idx {
            // First glyph in the layout. Initialize State
            None => None
            ,
            // * Same segment
            Some(idx) if idx == new_segment_idx => {
                // ? If we are not in a Link, then our current and new child will both be None.
                // ? If we are in a link, one of these must be Some.
                // * Same Segment, new child. We have a link with different styles of text within.
                // * Flush current child text to `link_tspans`
                if self.current_child_idx != new_child_idx { // ? Do we flip this?
                    // ? Child boundary - flush text into self.link_tspans
                    if self.current_child_idx.is_some() {
                        // Get styling of current child:
                        // ? We know that only InlineLink segments will have a child, however our type system doesn't yet clarify that,
                        // ? so we need to do some weird pattern matching to fit this.
                        match &segments[idx] {
                            LayoutInline::InlineLink { link_text, .. } => {
                                if let LayoutInline::Text {
                                    text: _,
                                    weight,
                                    style,
                                    family,
                                } = &link_text[self.current_child_idx.expect(
                                    "InlineLink variants off LayoutInline must be of type Text.",
                                )] {
                                    // * Update link spans for this segment
                                    self.link_tspans.push(TspanDefinition::Text {
                                        text: self.current_text.clone(),
                                        font_size,
                                        weight: weight.clone(),
                                        style: style.clone(),
                                        family: family.clone(),
                                    });
                                }
                                // ? Partial flush. Reset our current text as this is not done unconditionally.
                                self.current_text.clear();

                                return None
                            }
                            // ? Unreachable, in theory - type system needs to back this.
                            _ => unreachable!(
                                "Child indices can only be found on Text variants of LayoutInline."
                            ),
                        }
                    } else {
                        // * No previous child
                        None
                    }
                } else {
                    // * Same segment, same child - accumulate and wait for the next change
                    None
                }
            }
            // * New Segment - emit prior segment
            Some(_) => {
                self.flush(segments, font_size)
            }
        };
        // Unconditional state updates.
        self.current_text.push_str(new_char);
        self.current_child_idx = new_child_idx;
        self.current_segment_idx = Some(new_segment_idx);

        result
    }

    /// Emits the current accumulated state as a `TspanDefinition`
    /// Clears `current_text` and `link_tspans` as part of emission.
    /// Returns `None` if there is nothing to emit.
    fn flush(&mut self, segments: &[LayoutInline], font_size: f32) -> Option<TspanDefinition> {
        // Check if we have anything to process
        // ? If current_text is not empty, we should be able to assume that `current_segment_idx` cannot be None, as it is a required field when we add any text.
        // ? The inverse is not true. If `current_text` is empty, we cannot confidently say that `current_segment_idx` is None:
        // ? the text is cleared during each child and segment transition.
        // ? This logic holds if we are transitioning segments or forcing a final output.
        let result: Option<TspanDefinition> =
            if self.current_segment_idx.is_some() && !self.current_text.is_empty() {
                // TODO evaluate if `clone` is neccessary here.
                let segment = segments[self
                    .current_segment_idx
                    .expect("segment_idx must be Some when current_text is non-empty.")]
                .clone();

                match segment {
                    LayoutInline::Text {
                        text: _,
                        weight,
                        style,
                        family,
                    } => Some(TspanDefinition::Text {
                        text: self.current_text.clone(),
                        font_size,
                        weight,
                        style,
                        family: family.clone(),
                    }),
                    LayoutInline::InlineLink {
                        link_text: link_lyt_inlines,
                        url,
                        title,
                    } => {
                        // Crunch current text into a TspanDefinition.
                        // ? The `link_text` vector of InlineLink cannot store another InlineLink due to behaviour defined outside of this function.
                        // ? This is a common point of friction for me - it feels like we are defining behaviour that makes this invalid state impossible,
                        // ? rather than relying on the type-system to guarantee it.
                        if let LayoutInline::Text {
                            text,
                            weight,
                            style,
                            family,
                        } = link_lyt_inlines[self.current_child_idx?].clone()
                        {
                            self.link_tspans.push(TspanDefinition::Text {
                                text,
                                font_size,
                                weight,
                                style,
                                family: family.clone(),
                            });
                        }
                        // emit
                        Some(TspanDefinition::InlineLink {
                            text: self.link_tspans.clone(),
                            url,
                            title,
                        })
                    }
                    // This feels weak. but I don't know how to describe or enforce this.
                    // The behaviour of `build_styled_inline_ranges` skips HardBreak in the counter and returns "\n".
                    _ => unreachable!(
                        "Non-text InlineStyles should not be parsed as a StyledInlineRange"
                    ),
                }
            } else {
                None
            };
        self.current_text.clear();
        self.link_tspans.clear();

        result
    }
    
}

pub fn process_layouts(layouts: Vec<LayoutItem>, cfg: &SvgConfig) -> Vec<String> {
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
            LayoutItem::Block(lyt) => {
                let (svgs, updated_y) = process_layout_line(lyt, cumulative_y_offset, cfg);
                svg_lines.extend(svgs);

                // * After each LayoutLine, we insert a gap to represent a line between paragraphs.
                // * We perform "margin collapsing" - attempting to mirror the CSS behaviour.
                let next_margin_top = iter.peek().map(|next| next.margin_top()).unwrap_or(0.0);

                let gap = layout.margin_bottom().max(next_margin_top);

                cumulative_y_offset = updated_y + gap;
            }
            LayoutItem::ThematicBreak { left, right } => {
                svg_lines.extend(vec![create_thematic_break(
                    *left,
                    *right,
                    cumulative_y_offset,
                )]);

                cumulative_y_offset = cumulative_y_offset + cfg.calculate_paragraph_spacing_px();
            }
            LayoutItem::Blank { height } => {
                cumulative_y_offset = cumulative_y_offset + height;
            }
        }
        // Return final SVG collection.
    }

    svg_lines
}

fn process_layout_line(layout: &LayoutBlock, y_cursor: f32, cfg: &SvgConfig) -> (Vec<String>, f32) {
    let mut svg_elements: Vec<String> = Vec::new();
    let mut cumulative_y = y_cursor; //the baseline 'leading' (led-ing), if ya nasty

    // * Build our Segment Map for this LayoutLine.
    let segment_ranges = build_styled_inline_ranges(&layout.segments);

    let mut run_byte_offset: usize = 0;

    for (_idx, run) in layout.buffer.layout_runs().enumerate() {
        let baseline_y = y_cursor + run.line_y;

        let full_text: &String = &layout.segments.iter().map(|seg| seg.raw_text()).collect();

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
    segment_ranges: &Vec<StyledInlineRange>,
    segments: &Vec<LayoutInline>,
    prefix_len: usize,
    font_size: f32,
    run_byte_offset: usize,
) -> Vec<TspanDefinition> {
    let mut tspans: Vec<TspanDefinition> = vec![];

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
            tspans.push(TspanDefinition::Text {
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

        // TODO Can we collapse this into a single function to return TspanDefinition?
        // Check if we have moved into a different segment.
        if let Some(prev_idx) = current_segment_idx {
            if prev_idx != segment_idx {
                // Emit the accumulated text as a TSpan with styling from the previous segment.
                let (weight, style, family) = match &segments[prev_idx] {
                    LayoutInline::Text {
                        text: _,
                        weight,
                        style,
                        family,
                    } => (weight, style, family),
                    LayoutInline::InlineLink { link_text, url, title } => {
                        todo!()
                    }
                    _ => unreachable!("Non-text segment shouldn't have a Range"),
                };

                tspans.push(TspanDefinition {
                    text: current_text.clone(),
                    font_size: font_size,
                    weight: *weight,
                    style: *style,
                    family: family.clone(),
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
            // Emit the accumulated text as a TSpan with styling from the previous segment.
            let (weight, style, family) = match &segments[seg_idx] {
                // TODO - this needs to handle InlineLinks
                LayoutInline::Text {
                    text: _,
                    weight,
                    style,
                    family,
                } => (weight, style, family),
                _ => unreachable!("Non-text segment shouldn't have a Range"),
            };
            tspans.push(TspanDefinition {
                text: current_text.clone(),
                font_size: font_size,
                weight: *weight,
                style: *style,
                family: family.clone(),
            });
        }
    }

    tspans
}

/// Convert a `Tspan` into a raw SVG string  by a <text> tag.
fn tspans_to_svg(tspans: &[TspanDefinition], x: f32, y: f32) -> String {
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

/// Create a Horizontal Line/Thematic Break.
fn create_thematic_break(left: f32, right: f32, y: f32) -> String {
    // define svg: </hr> at position
    format!(r#"<line x1="{left}" y1="{y}" x2="{right}" y2="{y}" stroke="black" />"#)
}

///  Precompute the text range of our StyledBlock text as a byte range, which matches with the Cosmic Glyph start/end indices.
fn build_styled_inline_ranges(segments: &Vec<LayoutInline>) -> Vec<StyledInlineRange> {
    let mut ranges = Vec::new();
    let mut current_pos = 0;

    for (seg_idx, segment) in segments.iter().enumerate() {
        //println!("Current Pos: {}", current_pos);
        match segment {
            LayoutInline::Text { text, .. } => {
                let seg_len = text.len();

                ranges.push(StyledInlineRange {
                    segment_idx: seg_idx,
                    start_byte: current_pos,
                    end_byte: current_pos + seg_len,
                });

                current_pos += seg_len;
            }

            LayoutInline::HardBreak => {
                // No style - advance cursor.
                current_pos += 1;
            }

            LayoutInline::InlineLink { link_text, .. } => {
                let seg_len = link_text.len();

                ranges.push(StyledInlineRange {
                    segment_idx: seg_idx,
                    start_byte: current_pos,
                    end_byte: current_pos + seg_len,
                });

                current_pos += seg_len;
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
    use crate::layout::{LayoutInline, TspanDefinition, build_styled_inline_ranges, tspans_to_svg};
    use cosmic_text::{FamilyOwned, Style, Weight};

    // * --- tspan_to_svg ---
    #[test]
    fn tspans_to_svg_preserves_bold_weight_includes_attribute() {
        // * Arrange
        // Create a simple input text, no styling.
        let input_text = "Hello World".to_string();
        let attr = r#"font-weight="bold""#.to_string();
        // Form a TSpan with some default X/Y.
        let tspan = TspanDefinition {
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
        let tspan = TspanDefinition {
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
        let tspan = TspanDefinition {
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
        let tspan = TspanDefinition {
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
        let tspan = TspanDefinition {
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

        let tspan = TspanDefinition {
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
        let segment = vec![LayoutInline::Text {
            text: seg_text.clone(),
            weight: Weight::NORMAL,
            style: Style::Normal,
            family: FamilyOwned::SansSerif,
        }];

        // * Act
        let segment_ranges = build_styled_inline_ranges(&segment);

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
            LayoutInline::Text {
                text: first_segment_text.clone(),
                weight: Weight::NORMAL,
                style: Style::Normal,
                family: FamilyOwned::SansSerif,
            },
            LayoutInline::Text {
                text: second_segment_text.clone(),
                weight: Weight::NORMAL,
                style: Style::Normal,
                family: FamilyOwned::SansSerif,
            },
        ];

        let segment_ranges = build_styled_inline_ranges(&segments);

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
            LayoutInline::Text {
                text: first_segment_text.clone(),
                weight: Weight::NORMAL,
                style: Style::Normal,
                family: FamilyOwned::SansSerif,
            },
            LayoutInline::HardBreak,
            LayoutInline::Text {
                text: second_segment_text.clone(),
                weight: Weight::NORMAL,
                style: Style::Normal,
                family: FamilyOwned::SansSerif,
            },
        ];

        let segment_ranges = build_styled_inline_ranges(&segments);

        // * assert
        assert_eq!(segment_ranges.len(), 2); // Remains 2! Only count Text segments.
        assert_eq!(segment_ranges[0].start_byte, 0);
        assert_eq!(segment_ranges[0].end_byte, 6); // Does not modify the contents of Segment 1!
        assert_eq!(segment_ranges[1].start_byte, 7); // Offset by 1 from Hard Break.
    }
}