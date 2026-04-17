use core::f32;
use cosmic_text::{Buffer, FamilyOwned, LayoutRun, Style, Weight};

use crate::config::SvgConfig;

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
#[derive(Debug, PartialEq)]
pub struct StyledInlineRange {
    segment_idx: usize,
    start_byte: usize,
    end_byte: usize,
    child_idx: Option<usize>,
}

/// Complete definition of a Text Span SVG element.
// TODO - introduce separate type for nested variants to ensure we can only have valid variants by type.
#[derive(Clone, Debug, PartialEq)]
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

impl TspanDefinition {
    pub fn to_svg_string(self) -> String {
        match self {
            // todo family can be handled at the text level and only inserted if the tspan is different.
            TspanDefinition::Text {
                text,
                font_size,
                weight,
                style,
                family,
            } => {
                let mut attrs: Vec<String> = Vec::new();

                if weight == Weight::BOLD {
                    attrs.push(r#"font-weight="bold""#.to_string());
                };

                match style {
                    Style::Italic => {
                        attrs.push(r#"font-style="italic""#.to_string());
                    }
                    Style::Oblique => {
                        attrs.push(r#"font-style="oblique""#.to_string());
                    }
                    Style::Normal => {}
                };

                attrs.push(format!(r#"font-size="{}px""#, font_size));

                match family {
                    FamilyOwned::Name(smol_str) => {
                        attrs.push(format!(r#"font-family="{}, sans-serif""#, smol_str));
                    }
                    FamilyOwned::SansSerif => {
                        attrs.push(r#"font-family="sans-serif""#.to_string());
                    }
                    FamilyOwned::Serif => {
                        attrs.push(r#"font-family="serif""#.to_string());
                    }
                    FamilyOwned::Cursive => {
                        attrs.push(r#"font-family="cursive""#.to_string());
                    }
                    FamilyOwned::Fantasy => {
                        attrs.push(r#"font-family="fantasy""#.to_string());
                    }
                    FamilyOwned::Monospace => {
                        attrs.push(r#"font-family="monospace""#.to_string());
                    }
                };

                format!(
                    r#"<tspan {}>{}</tspan>"#,
                    attrs.join(" "),
                    html_escape(&text)
                )
            }
            TspanDefinition::InlineLink { text, url, title } => {
                let escaped_url = html_escape(&url);

                let child_text = text
                    .into_iter()
                    .map(|ch| ch.to_svg_string())
                    .collect::<String>();
                let tspan = match title {
                    Some(escaped_title) => format!(
                        r#"<a href="{}"><title>{}</title>{}</a>"#,
                        escaped_url,
                        html_escape(&escaped_title),
                        child_text
                    ),

                    // html_escape
                    None => format!(r#"<a href="{}">{}</a>"#, escaped_url, child_text),
                };

                tspan
            }
        }
    }
}

#[derive(Default, Debug)]
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
            None => None,
            // * Same segment
            Some(seg_idx) if seg_idx == new_segment_idx => {
                // ? If we are not in a Link, then our current and new child will both be None.
                // ? If we are in a link, one of these must be Some.
                // * Same Segment, new child. We have a link with different styles of text within.
                // * Flush current child text to `link_tspans`
                if self.current_child_idx != new_child_idx {
                    // ? Do we flip this?
                    // ? Child boundary - flush text into self.link_tspans
                    if self.current_child_idx.is_some() {
                        // Get styling of current child:
                        // ? We know that only InlineLink segments will have a child, however our type system doesn't yet clarify that,
                        // ? so we need to do some weird pattern matching to fit this.
                        match &segments[seg_idx] {
                            LayoutInline::InlineLink { link_text, .. } => {
                                if let LayoutInline::Text {
                                    text: _,
                                    weight,
                                    style,
                                    family,
                                } = &link_text[self.current_child_idx.expect(
                                    "InlineLink.link_text must be of type Vec<LayoutInline::Text>.",
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

                                None
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
            Some(_) => self.flush(segments, font_size),
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

        svg_elements.push(tspans_to_svg(tspans, current_x, run.line_y + y_cursor));
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
        for glyph in run.glyphs.iter().take_while(|g| g.start < prefix_len) {
            prefix_text.push_str(&run.text[glyph.start..glyph.end]);
        }
        if !prefix_text.is_empty() {
            tspans.push(TspanDefinition::Text {
                text: prefix_text,
                font_size,
                weight: Weight::NORMAL,
                style: Style::Normal,
                family: FamilyOwned::SansSerif,
            });
        }
    }

    let mut run_state = RunState::default();

    // * Start investigating all 'glyphs' - or Unicode Codepoints.
    for glyph in run.glyphs.iter() {
        if glyph.start < prefix_len {
            continue; // Skip prefix glyphs
        }

        // Adjust the starting byte position to account for the prefix & our prior glyphs in the run.
        let adjusted_byte_pos = (glyph.start - prefix_len) + run_byte_offset;

        // Find which segment this glyph belongs to.
        // ? Same as before, but I unwrap it here instead of taking two steps.
        let segment_style_idx = segment_ranges
            .iter()
            .find(|range| {
                adjusted_byte_pos >= range.start_byte && adjusted_byte_pos < range.end_byte
            })
            .expect(
                format!(
                    "Glyph at index {} has no matching segment range - \
                    build_segment_ranges produced incomplete coverage. Accumulated text: {:#?}",
                    adjusted_byte_pos, run_state.current_text
                )
                .as_str(),
            );

        // ? Each glyph adds to the run state. If accumulating that glyph would cause an emission,
        // ? We add it to the function scope `tspans` vector for final return, and continue on.
        run_state
            .advance(
                &run.text[glyph.start..glyph.end],
                segment_style_idx.segment_idx,
                segment_style_idx.child_idx,
                segments,
                font_size,
            )
            .map(|tsp| tspans.push(tsp));
    }

    // At the end of the run, if we have any text remaining in our buffer, crunch it.
    run_state
        .flush(segments, font_size)
        .map(|tsp| tspans.push(tsp));

    tspans
}

/// Convert a `Tspan` into a raw SVG string  by a <text> tag.
fn tspans_to_svg(tspans: Vec<TspanDefinition>, x: f32, y: f32) -> String {
    let tspan_strings: Vec<String> = tspans.into_iter().map(|ts| ts.to_svg_string()).collect();
    format!(
        r#"<text x="{}" y="{}">{}</text>"#,
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
                // ? Rust Ranges are inclusive below and exclusive above
                // ? e.g.
                // let arr = [0, 1,   2, 3, 4]; arr[1..3]
                //          start^ end^
                // ? Don't consider '0' basing here. "hello " has 6 glyphs. Therefore, the start index is 0, and the last index is 6.
                // ? This means glyphs 0,1,2,3,4,5 are included.
                ranges.push(StyledInlineRange {
                    segment_idx: seg_idx,
                    start_byte: current_pos,

                    end_byte: current_pos + seg_len,
                    child_idx: None, // Text cannot have children.
                });

                current_pos += seg_len;
            }

            LayoutInline::HardBreak => {
                // No style - advance cursor.
                current_pos += 1;
            }

            LayoutInline::InlineLink { link_text, .. } => {
                for (i, text) in link_text.iter().enumerate() {
                    let child_len = text.raw_text().len();

                    ranges.push(StyledInlineRange {
                        segment_idx: seg_idx,
                        start_byte: current_pos,
                        end_byte: current_pos + child_len,
                        child_idx: Some(i),
                    });
                    current_pos += child_len;
                }
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
        config::SvgConfig,
        layout::{
            LayoutInline, RunState, StyledInlineRange, TspanDefinition, build_styled_inline_ranges,
            tspans_to_svg,
        },
    };
    use cosmic_text::{FamilyOwned, Style, Weight};
    use pretty_assertions::assert_eq;

    // * --- tspan_to_svg ---
    #[test]
    fn tspans_to_svg_preserves_bold_weight_includes_attribute() {
        // * Arrange
        let input_text = "Hello World".to_string();
        let attr = r#"font-weight="bold""#.to_string();

        let tspan = TspanDefinition::Text {
            text: input_text.clone(),
            font_size: 16.0,
            weight: Weight::BOLD,
            style: Style::Normal,
            family: FamilyOwned::SansSerif,
        };
        // * Act
        let svg_string = tspans_to_svg(vec![tspan], 0.0, 16.0);

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
        let tspan = TspanDefinition::Text {
            text: input_text.clone(),
            font_size: 16.0,
            weight: Weight::NORMAL,
            style: Style::Normal,
            family: FamilyOwned::SansSerif,
        };
        // * Act
        // call `tspans_to_svg` with the created TSpan, X, Y
        let svg_string = tspans_to_svg(vec![tspan], 0.0, 16.0);
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
        let tspan = TspanDefinition::Text {
            text: input_text.clone(),
            font_size: 16.0,
            weight: Weight::NORMAL,
            style: Style::Italic,
            family: FamilyOwned::SansSerif,
        };
        // * Act
        // call `tspans_to_svg` with the created TSpan, X, Y
        let svg_string = tspans_to_svg(vec![tspan], 0.0, 16.0);
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
        let tspan = TspanDefinition::Text {
            text: input_text.clone(),
            font_size: 16.0,
            weight: Weight::NORMAL,
            style: Style::Normal,
            family: FamilyOwned::SansSerif,
        };
        // * Act
        // call `tspans_to_svg` with the created TSpan, X, Y
        let svg_string = tspans_to_svg(vec![tspan], 0.0, 16.0);
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
        let tspan = TspanDefinition::Text {
            text: input_text.clone(),
            font_size: 16.0,
            weight: Weight::NORMAL,
            style: Style::Normal,
            family: FamilyOwned::SansSerif,
        };

        // * Act
        // call `tspans_to_svg` with the created TSpan, X, Y
        let svg_string = tspans_to_svg(vec![tspan], 0.0, 16.0);
        // * Assert
        // CreatedTspanStr contains our simple input text.
        assert!(svg_string.contains(&input_text));
    }

    #[test]
    fn tspans_to_svg_escapes_html_characters() {
        // * Arrange
        let input_text = r#"<&>""#.to_string();
        let compare_text = "&lt;&amp;&gt;&quot;";

        let tspan = TspanDefinition::Text {
            text: input_text.clone(),
            font_size: 16.0,
            weight: Weight::NORMAL,
            style: Style::Normal,
            family: FamilyOwned::SansSerif,
        };

        let svg_string = tspans_to_svg(vec![tspan], 0.0, 16.0);
        assert!(svg_string.contains(&compare_text));
    }

    #[test]
    fn tspans_to_svg_inline_link_emits_anchor_element() {
        let link_text = TspanDefinition::InlineLink {
            text: vec![TspanDefinition::Text {
                text: "Hello".to_string(),
                font_size: 16.0,
                weight: Weight::NORMAL,
                style: Style::Normal,
                family: FamilyOwned::SansSerif,
            }],
            url: "test/url".to_string(),
            title: None,
        };

        let result = tspans_to_svg(vec![link_text], 0.0, 16.0);
        println!("{:#?}", result);
        assert!(result.contains(r#"<a href="test/url""#));
        assert!(result.ends_with("</a></text>"));
        assert!(result.contains("<tspan"));
    }

    #[test]
    fn tspans_to_svg_inline_link_children_rendered_as_tspans() {
        let link_text = TspanDefinition::InlineLink {
            text: vec![TspanDefinition::Text {
                text: "Hello".to_string(),
                font_size: 16.0,
                weight: Weight::NORMAL,
                style: Style::Normal,
                family: FamilyOwned::SansSerif,
            }],
            url: "test/url".to_string(),
            title: None,
        };

        let result = tspans_to_svg(vec![link_text], 0.0, 16.0);

        let a_start = result.find("<a ").unwrap();
        let a_end = result.find("</a>").unwrap();
        let anchor_content = &result[a_start..a_end];
        assert!(anchor_content.contains("<tspan"));
        assert!(anchor_content.contains("Hello"));
    }

    #[test]
    fn tspans_to_svg_inline_link_preserves_title() {
        let link_text = TspanDefinition::InlineLink {
            text: vec![TspanDefinition::Text {
                text: "Hello".to_string(),
                font_size: 16.0,
                weight: Weight::NORMAL,
                style: Style::Normal,
                family: FamilyOwned::SansSerif,
            }],
            url: "test/url".to_string(),
            title: Some("My Title".to_string()),
        };

        let result = tspans_to_svg(vec![link_text], 0.0, 16.0);

        assert!(result.contains("<title>My Title</title>"));
    }

    #[test]
    fn tspans_to_svg_inline_link_no_title_omits_title_tag() {
        let link_text = TspanDefinition::InlineLink {
            text: vec![TspanDefinition::Text {
                text: "Hello".to_string(),
                font_size: 16.0,
                weight: Weight::NORMAL,
                style: Style::Normal,
                family: FamilyOwned::SansSerif,
            }],
            url: "test/url".to_string(),
            title: None,
        };

        let result = tspans_to_svg(vec![link_text], 0.0, 16.0);

        assert!(!result.contains("<title>"));
    }

    #[test]
    fn tspans_to_svg_inline_link_escapes_url() {
        let link_text = TspanDefinition::InlineLink {
            text: vec![TspanDefinition::Text {
                text: "Hello".to_string(),
                font_size: 16.0,
                weight: Weight::NORMAL,
                style: Style::Normal,
                family: FamilyOwned::SansSerif,
            }],
            url: "http://www.example.com?a=1&b=2".to_string(),
            title: None,
        };

        let result = tspans_to_svg(vec![link_text], 0.0, 16.0);
        println!("{}", result);
        assert!(result.contains("http://www.example.com?a=1&amp;b=2"));
    }

    #[test]
    fn tspans_to_svg_inline_link_escapes_title() {
        let link_text = TspanDefinition::InlineLink {
            text: vec![TspanDefinition::Text {
                text: "Hello".to_string(),
                font_size: 16.0,
                weight: Weight::NORMAL,
                style: Style::Normal,
                family: FamilyOwned::SansSerif,
            }],
            url: "test/url".to_string(),
            title: Some(r#" " & < > "#.to_string()),
        };

        let result = tspans_to_svg(vec![link_text], 0.0, 16.0);
        assert!(result.contains(" &quot; &amp; &lt; &gt;"));
    }

    // * --- to_svg_string * ---
    #[test]
    fn tspan_definition_text_produces_well_formed_tspan_element() {
        let tspan = TspanDefinition::Text {
            text: "Hello".to_string(),
            font_size: 16.0,
            weight: Weight::NORMAL,
            style: Style::Normal,
            family: FamilyOwned::SansSerif,
        };

        let result = tspan.to_svg_string();

        assert!(result.starts_with("<tspan"));
        assert!(result.ends_with("</tspan>"));
        assert!(result.contains(">Hello<"));
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

    #[test]
    fn build_styled_inline_ranges_text_before_link_produces_correct_ranges() {
        let segments = vec![
            LayoutInline::Text {
                text: "Hello ".to_string(),
                weight: Weight::NORMAL,
                style: Style::Normal,
                family: FamilyOwned::SansSerif,
            },
            LayoutInline::InlineLink {
                link_text: vec![LayoutInline::Text {
                    text: "World".to_string(),
                    weight: Weight::NORMAL,
                    style: Style::Normal,
                    family: FamilyOwned::SansSerif,
                }],
                url: "test/url".to_string(),
                title: None,
            },
        ];

        let expected = vec![
            StyledInlineRange {
                segment_idx: 0,
                start_byte: 0,
                end_byte: 6,
                child_idx: None,
            },
            StyledInlineRange {
                segment_idx: 1,
                start_byte: 6,
                end_byte: 11,
                child_idx: Some(0),
            },
        ];

        let result = build_styled_inline_ranges(&segments);

        assert_eq!(result, expected);
    }

    #[test]
    fn build_styled_inline_ranges_text_after_link_produces_correct_ranges() {
        let segments = vec![
            LayoutInline::InlineLink {
                link_text: vec![LayoutInline::Text {
                    text: "World".to_string(),
                    weight: Weight::NORMAL,
                    style: Style::Normal,
                    family: FamilyOwned::SansSerif,
                }],
                url: "test/url".to_string(),
                title: None,
            },
            LayoutInline::Text {
                text: "Hello ".to_string(),
                weight: Weight::NORMAL,
                style: Style::Normal,
                family: FamilyOwned::SansSerif,
            },
        ];

        let expected = vec![
            StyledInlineRange {
                segment_idx: 0,
                start_byte: 0,
                end_byte: 5,
                child_idx: Some(0),
            },
            StyledInlineRange {
                segment_idx: 1,
                start_byte: 5,
                end_byte: 11,
                child_idx: None,
            },
        ];

        let result = build_styled_inline_ranges(&segments);

        assert_eq!(result, expected);
    }

    #[test]
    fn build_styled_inline_ranges_mixed_formatting_children_produces_correct_ranges() {
        // * [**Bold** *Italic* Normal `monospace`][test/url]
        let segments = vec![LayoutInline::InlineLink {
            link_text: vec![
                LayoutInline::Text {
                    text: "Bold".to_string(),
                    weight: Weight::BOLD,
                    style: Style::Normal,
                    family: FamilyOwned::SansSerif,
                },
                LayoutInline::Text {
                    text: " ".to_string(),
                    weight: Weight::NORMAL,
                    style: Style::Normal,
                    family: FamilyOwned::SansSerif,
                },
                LayoutInline::Text {
                    text: "Italic".to_string(),
                    weight: Weight::NORMAL,
                    style: Style::Italic,
                    family: FamilyOwned::SansSerif,
                },
                LayoutInline::Text {
                    text: " ".to_string(),
                    weight: Weight::NORMAL,
                    style: Style::Normal,
                    family: FamilyOwned::SansSerif,
                },
                LayoutInline::Text {
                    text: "Normal".to_string(),
                    weight: Weight::NORMAL,
                    style: Style::Normal,
                    family: FamilyOwned::SansSerif,
                },
                LayoutInline::Text {
                    text: " ".to_string(),
                    weight: Weight::NORMAL,
                    style: Style::Normal,
                    family: FamilyOwned::SansSerif,
                },
                LayoutInline::Text {
                    text: "Monospace".to_string(),
                    weight: Weight::NORMAL,
                    style: Style::Normal,
                    family: FamilyOwned::Monospace,
                },
            ],
            url: "test/url".to_string(),
            title: None,
        }];

        let expected = vec![
            StyledInlineRange {
                segment_idx: 0,
                start_byte: 0,
                end_byte: 4,
                child_idx: Some(0),
            },
            StyledInlineRange {
                segment_idx: 0,
                start_byte: 4,
                end_byte: 5,
                child_idx: Some(1),
            },
            StyledInlineRange {
                segment_idx: 0,
                start_byte: 5,
                end_byte: 11,
                child_idx: Some(2),
            },
            StyledInlineRange {
                segment_idx: 0,
                start_byte: 11,
                end_byte: 12,
                child_idx: Some(3),
            },
            StyledInlineRange {
                segment_idx: 0,
                start_byte: 12,
                end_byte: 18,
                child_idx: Some(4),
            },
            StyledInlineRange {
                segment_idx: 0,
                start_byte: 18,
                end_byte: 19,
                child_idx: Some(5),
            },
            StyledInlineRange {
                segment_idx: 0,
                start_byte: 19,
                end_byte: 28,
                child_idx: Some(6),
            },
        ];

        let result = build_styled_inline_ranges(&segments);

        assert_eq!(result, expected);
    }

    #[test]
    fn build_styled_inline_ranges_consecutive_links_produce_correct_ranges() {
        // * [Hello][test/url] [World][another/url]
        let segments = vec![
            LayoutInline::InlineLink {
                link_text: vec![LayoutInline::Text {
                    text: "Hello ".to_string(),
                    weight: Weight::NORMAL,
                    style: Style::Normal,
                    family: FamilyOwned::SansSerif,
                }],
                url: "test/url".to_string(),
                title: None,
            },
            LayoutInline::Text {
                text: " ".to_string(),
                weight: Weight::NORMAL,
                style: Style::Normal,
                family: FamilyOwned::SansSerif,
            },
            LayoutInline::InlineLink {
                link_text: vec![LayoutInline::Text {
                    text: "World".to_string(),
                    weight: Weight::NORMAL,
                    style: Style::Normal,
                    family: FamilyOwned::SansSerif,
                }],
                url: "another/url".to_string(),
                title: None,
            },
        ];

        let expected = vec![
            StyledInlineRange {
                segment_idx: 0,
                start_byte: 0,
                end_byte: 6,
                child_idx: Some(0),
            },
            StyledInlineRange {
                segment_idx: 1,
                start_byte: 6,
                end_byte: 7,
                child_idx: None,
            },
            StyledInlineRange {
                segment_idx: 2,
                start_byte: 7,
                end_byte: 12,
                child_idx: Some(0),
            },
        ];

        let result = build_styled_inline_ranges(&segments);

        assert_eq!(result, expected);
    }

    // * -- Run State --
    #[test]
    fn run_state_advance_emits_on_segment_change() {
        let mut run_state = RunState::default();
        let cfg = SvgConfig::default();

        let segments = vec![
            LayoutInline::Text {
                text: "A".to_string(),
                weight: Weight::NORMAL,
                style: Style::Normal,
                family: FamilyOwned::SansSerif,
            },
            LayoutInline::Text {
                text: "B".to_string(),
                weight: Weight::BOLD,
                style: Style::Normal,
                family: FamilyOwned::SansSerif,
            },
        ];

        // Initial state - no return
        let result = run_state.advance("A", 0, None, &segments, cfg.text_opts.font_size);
        assert_eq!(result, None);

        // Cross segment boundary found, return previous segment.
        let result = run_state.advance("B", 1, None, &segments, cfg.text_opts.font_size);
        assert_eq!(
            result,
            Some(TspanDefinition::Text {
                text: "A".to_string(),
                font_size: cfg.text_opts.font_size,
                weight: Weight::NORMAL,
                style: Style::Normal,
                family: FamilyOwned::SansSerif
            })
        );
        // New segment added to existing text.
        assert_eq!(run_state.current_text, "B");
    }

    #[test]
    fn run_state_advance_accumulates_within_same_segment_returns_none() {
        let mut run_state = RunState::default();
        let cfg = SvgConfig::default();

        let segments = vec![LayoutInline::Text {
            text: "AB".to_string(),
            weight: Weight::NORMAL,
            style: Style::Normal,
            family: FamilyOwned::SansSerif,
        }];

        run_state.advance("A", 0, None, &segments, cfg.text_opts.font_size);

        // Same segment, no change
        let result = run_state.advance("B", 0, None, &segments, cfg.text_opts.font_size);
        assert_eq!(result, None);

        // Internal string buffer shows "AB"
        assert_eq!(run_state.current_text, "AB");
    }

    #[test]
    fn run_state_advance_partial_flush_when_child_segment_changes() {
        let mut run_state = RunState::default();
        let cfg = SvgConfig::default();

        let segments = vec![LayoutInline::InlineLink {
            link_text: vec![
                LayoutInline::Text {
                    text: "A".to_string(),
                    weight: Weight::NORMAL,
                    style: Style::Normal,
                    family: FamilyOwned::SansSerif,
                },
                LayoutInline::Text {
                    text: "B".to_string(),
                    weight: Weight::BOLD,
                    style: Style::Normal,
                    family: FamilyOwned::SansSerif,
                },
            ],
            url: "test/url".to_string(),
            title: None,
        }];

        // Initial state - no return
        let result = run_state.advance("A", 0, Some(0), &segments, cfg.text_opts.font_size);
        assert_eq!(result, None);

        // Advance within same segment, partial flush to internal link_tspans. None returned.
        let result = run_state.advance("B", 0, Some(1), &segments, cfg.text_opts.font_size);
        assert_eq!(result, None);
        assert_eq!(
            run_state.link_tspans,
            vec![TspanDefinition::Text {
                text: "A".to_string(),
                font_size: cfg.text_opts.font_size,
                weight: Weight::NORMAL,
                style: Style::Normal,
                family: FamilyOwned::SansSerif
            }]
        );
    }

    #[test]
    fn run_state_flush_returns_none_when_empty() {
        let mut run_state = RunState::default();
        let cfg = SvgConfig::default();

        // No state, nothing returned.
        let result = run_state.flush(&vec![], cfg.text_opts.font_size);

        assert_eq!(result, None);
    }

    #[test]
    fn run_state_advance_inline_link_emits_when_segment_changes() {
        let mut run_state = RunState::default();
        let cfg = SvgConfig::default();

        let segments = vec![
            LayoutInline::InlineLink {
                link_text: vec![
                    LayoutInline::Text {
                        text: "A".to_string(),
                        weight: Weight::NORMAL,
                        style: Style::Normal,
                        family: FamilyOwned::SansSerif,
                    },
                    LayoutInline::Text {
                        text: "B".to_string(),
                        weight: Weight::BOLD,
                        style: Style::Normal,
                        family: FamilyOwned::SansSerif,
                    },
                ],
                url: "test/url".to_string(),
                title: None,
            },
            LayoutInline::Text {
                text: "C".to_string(),
                weight: Weight::NORMAL,
                style: Style::Normal,
                family: FamilyOwned::SansSerif,
            },
        ];

        run_state.advance("A", 0, Some(0), &segments, cfg.text_opts.font_size);
        run_state.advance("B", 0, Some(1), &segments, cfg.text_opts.font_size);
        // Move into new Segment
        let result = run_state.advance("C", 1, None, &segments, cfg.text_opts.font_size);

        assert_eq!(
            result,
            Some(TspanDefinition::InlineLink {
                text: vec![
                    TspanDefinition::Text {
                        text: "A".to_string(),
                        font_size: cfg.text_opts.font_size,
                        weight: Weight::NORMAL,
                        style: Style::Normal,
                        family: FamilyOwned::SansSerif
                    },
                    TspanDefinition::Text {
                        text: "B".to_string(),
                        font_size: cfg.text_opts.font_size,
                        weight: Weight::BOLD,
                        style: Style::Normal,
                        family: FamilyOwned::SansSerif
                    }
                ],
                url: "test/url".to_string(),
                title: None
            })
        );
        assert_eq!(run_state.current_text, "C".to_string());
    }

    #[test]
    fn run_state_flush_emits_inline_link_with_all_children() {
        let mut run_state = RunState::default();
        let cfg = SvgConfig::default();

        let segments = vec![LayoutInline::InlineLink {
            link_text: vec![
                LayoutInline::Text {
                    text: "A".to_string(),
                    weight: Weight::NORMAL,
                    style: Style::Normal,
                    family: FamilyOwned::SansSerif,
                },
                LayoutInline::Text {
                    text: "B".to_string(),
                    weight: Weight::BOLD,
                    style: Style::Normal,
                    family: FamilyOwned::SansSerif,
                },
            ],
            url: "test/url".to_string(),
            title: None,
        }];

        run_state.advance("A", 0, Some(0), &segments, cfg.text_opts.font_size);
        run_state.advance("B", 0, Some(1), &segments, cfg.text_opts.font_size);

        // Flush state
        // ? This mimics behaviour where the last line of a run is a link.
        let result = run_state.flush(&segments, cfg.text_opts.font_size);
        assert_eq!(
            result,
            Some(TspanDefinition::InlineLink {
                text: vec![
                    TspanDefinition::Text {
                        text: "A".to_string(),
                        font_size: cfg.text_opts.font_size,
                        weight: Weight::NORMAL,
                        style: Style::Normal,
                        family: FamilyOwned::SansSerif
                    },
                    TspanDefinition::Text {
                        text: "B".to_string(),
                        font_size: cfg.text_opts.font_size,
                        weight: Weight::BOLD,
                        style: Style::Normal,
                        family: FamilyOwned::SansSerif
                    }
                ],
                url: "test/url".to_string(),
                title: None
            })
        );
        // Ensure our state has been zero'd.
        assert!(run_state.link_tspans.is_empty());
        assert!(run_state.current_text.is_empty());
    }
}
