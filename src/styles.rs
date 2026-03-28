use cosmic_text::{Attrs, Buffer, FamilyOwned, FontSystem, Metrics, Style, Weight};

use crate::{
    config::SvgConfig,
    layout::{LayoutBlock, LayoutInline},
};

// TODO (1.1) - Support for strikethrough, sub/superscript.
// TODO (1.1) Make family configurable. (Elsewhere...)
/// ? Family defaults to SansSerif
/// Defines a text container that may optionally
/// contain further differently styled text or non-text structural definitions.
#[derive(Clone, Debug, PartialEq)]
pub enum StyledInline {
    Text(String),
    Emphasis(Vec<StyledInline>),
    Strong(Vec<StyledInline>),
    InlineCode(String),
    Link {
        text: Vec<StyledInline>,
        url: String,
        title: Option<String>,
    },
    HardBreak,
}
impl StyledInline {
    /// Parse an InlineStyle into a set of flat LayoutInlines.
    /// This function consumes the original StyledInline and the children of the original object.
    pub fn into_layout_inline(self) -> Vec<LayoutInline> {
        // StyledInline::Text(StyledSpan::new(text, ctx))
        self.transform(StyleContext::default())
    }
    /// These collections are flattened into a LayoutInline by parsing the style associated
    /// with each container type, then recursively flattening any children containers and recording
    /// the style of said container. The final returned value is a `LayoutInline::Text` that has the appropriate styling over the provided range.
    /// In the case of Links - we store the returned vector for the formatted text inside of the InlineLink variant, along with the metadata
    /// provided during AST parsing.
    ///
    /// For example:
    /// ```
    /// <i><b>This text is Strong and Emphasized</i></b>
    /// ```
    /// Becomes
    /// LayoutInline::Text{text: "This text is Strong and Emphasized", weight: Weight::Bold, style: Style::Italic, family, ...
    /// }
    fn transform(self, ctx: StyleContext) -> Vec<LayoutInline> {
        match self {
            StyledInline::Text(span) => {
                // Final/Leaf case
                // Convert StyleContext into real `cosmic` types
                vec![LayoutInline::Text {
                    text: span,
                    weight: if ctx.bold {
                        Weight::BOLD
                    } else {
                        Weight::NORMAL
                    },
                    style: if ctx.italic {
                        Style::Italic
                    } else {
                        Style::Normal
                    },
                    family: if ctx.monospace {
                        FamilyOwned::Monospace
                    } else {
                        FamilyOwned::SansSerif
                    },
                }]
            }
            StyledInline::Emphasis(styled_inlines) => {
                let ctx = StyleContext {
                    italic: true,
                    ..ctx
                };
                styled_inlines
                    .into_iter()
                    .flat_map(|child| child.transform(ctx))
                    .collect()
            }
            StyledInline::Strong(styled_inlines) => {
                let ctx = StyleContext { bold: true, ..ctx };
                styled_inlines
                    .into_iter()
                    .flat_map(|child| child.transform(ctx))
                    .collect()
            }
            StyledInline::InlineCode(code) => {
                // ? `monospace` formatting overrides prior context as it has higher precedence.
                // ? Per commonmark spec: 0.31.2 section 6.1 Code Spans:
                // ? > Code span backticks have higher precedence than any other inline constructs except HTML tags and autolinks.

                // todo add doctest here.
                let ctx = StyleContext {
                    monospace: true,
                    bold: false,
                    italic: false,
                };
                // Needs to recurse, but does not contain a Vec<StyledInline>. So we need to wrap it in a Text variant.
                vec![StyledInline::Text(code)]
                    .into_iter()
                    .flat_map(|child| child.transform(ctx))
                    .collect()
            }
            StyledInline::Link { text, url, title } => {
                vec![LayoutInline::InlineLink {
                    link_text: text
                        .into_iter()
                        .flat_map(|child| child.transform(ctx))
                        .collect(),
                    url,
                    title,
                }]
            }
            StyledInline::HardBreak => vec![LayoutInline::HardBreak],
        }
    }
}

// todo incomplete list of missing variants:
// TODO (1.1) Tables via HTML/GFM
// - Blockquote
// - Link
// - Image
/// Block level definitions.
/// Provides styling directives for specific line level containers (Lists, Blockquote)
/// As well as Block leaves such as Paragraph or Headers and non-text structural elements such as Thematic Break (Horizontal Rule)
/// This is a container or abstract type that loosely wraps style data.
#[derive(Clone, Debug)]
pub enum StyledBlock {
    Header {
        segments: Vec<StyledInline>,
        level: u8,
    },
    BulletListItem {
        segments: Vec<StyledInline>,
        indent: u8,
    },
    NumberedListItem {
        segments: Vec<StyledInline>,
        number: u32,
        indent: u8,
    },
    Paragraph {
        segments: Vec<StyledInline>,
    },
    Blockquote {
        text: String,
    },
    ThematicBreak,
}

impl StyledBlock {
    pub fn into_layout_block(self, cfg: &SvgConfig, font_system: &mut FontSystem) -> LayoutBlock {
        match self {
            StyledBlock::Paragraph { segments } => {
                let available_width = cfg.canvas_opts.width
                    - (cfg.canvas_opts.padding.left + cfg.canvas_opts.padding.right);

                let layout_lines: Vec<LayoutInline> = segments
                    .into_iter()
                    .flat_map(|line| line.into_layout_inline())
                    .collect();

                let buffer = create_buffer(
                    &layout_lines,
                    cfg.text_opts.font_size,
                    cfg.calculate_line_height_px(),
                    available_width,
                    font_system,
                    None,
                );
                LayoutBlock {
                    buffer,
                    segments: layout_lines,
                    font_size: cfg.text_opts.font_size,
                    prefix_len: 0,
                    indent_offset: 0.0,
                    margin_top: cfg.calculate_paragraph_spacing_px(),
                    margin_bottom: cfg.calculate_paragraph_spacing_px(),
                }
            }
            StyledBlock::Header { segments, level } => {
                let scaled_font_size =
                    cfg.text_opts.font_size * cfg.header_opts.header_scales.scale_for_level(level);
                println!("Scaled Font Size (into layout block): {}", scaled_font_size);

                let available_width = cfg.canvas_opts.width
                    - (cfg.canvas_opts.padding.left + cfg.canvas_opts.padding.right);

                let layout_lines: Vec<LayoutInline> = segments
                    .into_iter()
                    .flat_map(|line| line.into_layout_inline())
                    .collect();

                let buffer = create_buffer(
                    &layout_lines,
                    scaled_font_size,
                    scaled_font_size * cfg.text_opts.line_height_factor,
                    available_width,
                    font_system,
                    None,
                );
                LayoutBlock {
                    buffer,
                    segments: layout_lines,
                    font_size: scaled_font_size,
                    prefix_len: 0,
                    indent_offset: 0.0,
                    margin_top: cfg.calculate_paragraph_spacing_px(),
                    margin_bottom: cfg.calculate_paragraph_spacing_px(),
                }
            }
            StyledBlock::BulletListItem { segments, indent } => {
                let indent_size = if cfg.text_opts.indent_first_bullet {
                    (indent + 1) as f32 * (cfg.text_opts.bullet_indent_em * cfg.text_opts.font_size)
                } else {
                    indent as f32 * (cfg.text_opts.bullet_indent_em * cfg.text_opts.font_size)
                };

                // * Update our available_width based on the indent and padding.
                let available_width = cfg.canvas_opts.width
                    - (cfg.canvas_opts.padding.left + cfg.canvas_opts.padding.right)
                    - indent_size;

                let prefix = format!("{} ", cfg.text_opts.bullet_char);

                let layout_lines: Vec<LayoutInline> = segments
                    .into_iter()
                    .flat_map(|line| line.into_layout_inline())
                    .collect();

                let buffer = create_buffer(
                    &layout_lines,
                    cfg.text_opts.font_size,
                    cfg.calculate_line_height_px(),
                    available_width,
                    font_system,
                    Some(&prefix),
                );
                LayoutBlock {
                    buffer,
                    segments: layout_lines,
                    font_size: cfg.text_opts.font_size,
                    prefix_len: prefix.len(),
                    indent_offset: indent_size,
                    margin_top: cfg.calculate_paragraph_spacing_px(),
                    margin_bottom: cfg.calculate_paragraph_spacing_px(),
                }
            }
            StyledBlock::NumberedListItem {
                segments,
                number,
                indent,
            } => {
                let indent_size = if cfg.text_opts.indent_first_bullet {
                    (indent + 1) as f32 * (cfg.text_opts.bullet_indent_em * cfg.text_opts.font_size)
                } else {
                    indent as f32 * (cfg.text_opts.bullet_indent_em * cfg.text_opts.font_size)
                };

                let available_width = cfg.canvas_opts.width
                    - (cfg.canvas_opts.padding.left + cfg.canvas_opts.padding.right)
                    - indent_size;
                let prefix = format!("{}. ", number);

                let layout_lines: Vec<LayoutInline> = segments
                    .into_iter()
                    .flat_map(|line| line.into_layout_inline())
                    .collect();

                let buffer = create_buffer(
                    &layout_lines,
                    cfg.text_opts.font_size,
                    cfg.calculate_line_height_px(),
                    available_width,
                    font_system,
                    Some(&prefix),
                );
                LayoutBlock {
                    buffer,
                    segments: layout_lines,
                    font_size: cfg.text_opts.font_size,
                    prefix_len: prefix.len(),
                    indent_offset: indent_size,
                    margin_top: cfg.calculate_paragraph_spacing_px(),
                    margin_bottom: cfg.calculate_paragraph_spacing_px(),
                }
            }
            StyledBlock::ThematicBreak => todo!(),
            _ => todo!(),
        }
    }

    // TODO remove? This is just used for debugging andd printing in-progress types..
    pub fn get_type(&self) -> String {
        match self {
            StyledBlock::Header { .. } => String::from("Header"),
            StyledBlock::BulletListItem { .. } => String::from("Bullet List Item"),
            StyledBlock::NumberedListItem { .. } => String::from("Numbered List Item"),
            StyledBlock::Paragraph { .. } => String::from("Paragraph"),
            StyledBlock::Blockquote { .. } => String::from("Blockquote"),
            // StyledBlock::Link { .. } => String::from("Link"),
            // StyledBlock::Image { .. } => String::from("Image"),
            StyledBlock::ThematicBreak => String::from("Horizontal Rule"),
        }
    }
}

// note: Strikethrough, Under/overline and other text decorations aren't supported in default Markdown and aren't showing in my node as unique text.
// For now, I don't care to adapt these cases. However, they could likely be addded as further StyleContext cases later on.
// ? it seems that these items are supported in GFM (Github Flavored Markdown) - for now, I would like to handle pure Markdown.
// todo make private
#[derive(Clone, Copy, Debug)]
struct StyleContext {
    pub bold: bool,
    pub italic: bool,
    pub monospace: bool,
}

impl StyleContext {
    pub fn default() -> Self {
        StyleContext {
            bold: false,
            italic: false,
            monospace: false,
        }
    }
}

fn create_buffer(
    inlines: &[LayoutInline],
    font_size: f32,
    line_height: f32,
    width: f32,
    font_system: &mut FontSystem,
    prefix: Option<&str>,
) -> cosmic_text::Buffer {
    let mut buffer = Buffer::new(font_system, Metrics::new(font_size, line_height));

    //  * Define an interator instruction for the optional prefix
    let prefix_spans: Option<(&str, Attrs<'_>)> = prefix.as_deref().map(|p| (p, Attrs::new()));

    // * Define an iterator instruction for our content strings
    let layout_spans = inlines.iter().flat_map(|inline| match inline {
        LayoutInline::Text {
            text,
            weight,
            style,
            family,
        } => {
            vec![(
                text.as_str(),
                Attrs::new()
                    .weight(*weight)
                    .style(*style)
                    .family(family.as_family()),
            )]
        }
        // * We have previously removed soft-breaks from the Markdown document, since these are not implicit
        // * We need to inform Cosmic on where to insert real HardBreaks from the original doc.
        LayoutInline::HardBreak => {
            vec![("\n", Attrs::new())]
        }
        // TODO
        LayoutInline::InlineLink {
            link_text,
            url,
            title,
        } => todo!(),
    });

    // * Join both iterators. We only add the prefix spans if we have Some((&str, Attrs))
    let all_spans: Vec<(&str, Attrs<'_>)> = prefix_spans.into_iter().chain(layout_spans).collect();

    buffer.set_rich_text(
        font_system,
        all_spans,
        &Attrs::new(),
        cosmic_text::Shaping::Advanced,
        None,
    );
    buffer.set_size(font_system, Some(width), Some(f32::MAX));

    buffer
}

#[cfg(test)]
mod tests {
    use cosmic_text::{
        FamilyOwned, FontSystem, Style, Weight, fontdb::Database, skrifa::raw::tables::layout,
    };
    use std::{path::Path, result};

    use crate::{
        config::SvgConfig,
        layout::{LayoutBlock, LayoutInline, LayoutItem},
        styles::{StyledBlock, StyledInline, create_buffer},
    };

    /// Create a simple FontSystem with default Sans-Serif font derived from tests/fonts.
    fn create_default_test_font_system() -> FontSystem {
        // Create a default, empty FontDB
        let mut db = Database::new();
        // * Load Noto Sans from tests/fonts/
        // ? We load all fonts in this directory instead of loading the individual font variants (Italic, Bold etc)
        // ? Our default case is to access all of these, rather than loading specific fonts for each test.
        // ? we could break this out to accept a Weight/Style variant struct as an arg, and match accordingly later on if we need.
        db.load_fonts_dir(Path::new("tests/fonts/"));

        // override default Sans-Serif on db.
        db.set_sans_serif_family("Noto Sans");

        let font_sys = FontSystem::new_with_locale_and_db("en-US".to_string(), db);
        font_sys
    }

    // TODO RENAME
    #[test]
    fn styled_block_paragraph_transforms_to_layout_block() {
        // * Arrange
        let mut font_system = create_default_test_font_system();
        let cfg = SvgConfig::default();

        let paragraph_text = "This is some paragraph text.".to_string();

        let styled_segments = vec![StyledInline::Text(paragraph_text.clone())];
        let layout_segments: Vec<LayoutInline> = styled_segments
            .iter()
            .flat_map(|seg| seg.clone().into_layout_inline())
            .collect();

        // Create a basic StyledLine
        let styled_line_para = StyledBlock::Paragraph {
            segments: styled_segments.clone(),
        };

        // * Act
        let layout_result = styled_line_para.into_layout_block(&cfg, &mut font_system);
        let expected: LayoutBlock = LayoutBlock {
            buffer: create_buffer(
                &vec![LayoutInline::Text {
                    text: paragraph_text.clone(),
                    weight: Weight::NORMAL,
                    style: Style::Normal,
                    family: FamilyOwned::SansSerif,
                }],
                cfg.text_opts.font_size,
                cfg.calculate_line_height_px(),
                cfg.canvas_opts.width,
                &mut font_system,
                None,
            ),
            segments: layout_segments,
            font_size: cfg.text_opts.font_size,
            prefix_len: 0,
            indent_offset: 0.0,
            margin_top: cfg.calculate_paragraph_spacing_px(),
            margin_bottom: cfg.calculate_paragraph_spacing_px(),
        };
        // * Assert
        assert_eq!(layout_result.font_size, expected.font_size);
        assert_eq!(layout_result.segments, expected.segments);
        assert_eq!(layout_result.margin_top, expected.margin_top);
        assert_eq!(layout_result.margin_bottom, expected.margin_bottom);
        assert_eq!(layout_result.prefix_len, expected.prefix_len);
        assert_eq!(layout_result.indent_offset, expected.indent_offset);
    }

    #[test]
    fn into_layout_block_handles_hard_break() {
        // * Arrange
        let mut font_system = create_default_test_font_system();
        let cfg = SvgConfig::default();

        let paragraph_text = "This is some paragraph text.".to_string();

        let styled_segments = vec![
            StyledInline::Text(paragraph_text.clone()),
            StyledInline::HardBreak,
        ];
        let layout_segments: Vec<LayoutInline> = styled_segments
            .iter()
            .flat_map(|seg| seg.clone().into_layout_inline())
            .collect();

        // Create a basic StyledLine
        let styled_line_para = StyledBlock::Paragraph {
            segments: styled_segments.clone(),
        };

        // * Act
        let layout_result = styled_line_para.into_layout_block(&cfg, &mut font_system);
        let expected: LayoutBlock = LayoutBlock {
            buffer: create_buffer(
                &vec![
                    LayoutInline::Text {
                        text: paragraph_text.clone(),
                        weight: Weight::NORMAL,
                        style: Style::Normal,
                        family: FamilyOwned::SansSerif,
                    },
                    LayoutInline::HardBreak,
                ],
                cfg.text_opts.font_size,
                cfg.calculate_line_height_px(),
                cfg.canvas_opts.width,
                &mut font_system,
                None,
            ),
            segments: layout_segments,
            font_size: cfg.text_opts.font_size,
            prefix_len: 0,
            indent_offset: 0.0,
            margin_top: cfg.calculate_paragraph_spacing_px(),
            margin_bottom: cfg.calculate_paragraph_spacing_px(),
        };
        // * Assert
        assert_eq!(layout_result.segments, expected.segments);
    }

    #[test]
    fn into_layout_block_header_applies_cfg_font_scale() {
        // * Arrange
        let mut font_system = create_default_test_font_system();
        let cfg = SvgConfig::default();

        let header_text = "# Header".to_string();

        let segments = vec![StyledInline::Text(header_text.clone())];

        let styled_block_paragraph = StyledBlock::Header {
            segments: segments.clone(),
            level: 1,
        };

        // * Act
        let layout_result = styled_block_paragraph.into_layout_block(&cfg, &mut font_system);

        // * Assert
        assert_eq!(
            layout_result.font_size,
            cfg.text_opts.font_size * cfg.header_opts.header_scales.scale_for_level(1)
        );
    }

    #[test]
    fn into_layout_block_bullet_list_item_applies_prefix_length() {
        // * Arrange
        let mut font_system = create_default_test_font_system();
        let cfg = SvgConfig::default();

        let bullet_item_text = "- Bullet Item".to_string();

        let segments = vec![StyledInline::Text(bullet_item_text)];

        // Create a basic StyledLine
        let styled_block_bullet_item = StyledBlock::BulletListItem {
            segments: segments.clone(),
            indent: 0,
        };

        // * Act
        let layout_result = styled_block_bullet_item.into_layout_block(&cfg, &mut font_system);

        // * Assert
        assert_eq!(layout_result.prefix_len, 4); //? Character "•" is 3 bytes, followed by a single white-space = 4 bytes total.
    }

    #[test]
    fn into_layout_block_bullet_list_item_applies_indent_offset() {
        // * Arrange
        let mut font_system = create_default_test_font_system();
        let cfg = SvgConfig::default();

        let bullet_item_text = "- Bullet Item".to_string();

        let segments = vec![StyledInline::Text(bullet_item_text)];

        // ? Assume item is 3 layers deep.
        let styled_block_bullet_item = StyledBlock::BulletListItem {
            segments: segments.clone(),
            indent: 3, // ? Derived from AST
        };

        // * Act
        let layout_result = styled_block_bullet_item.into_layout_block(&cfg, &mut font_system);

        // * Assert
        assert_eq!(
            layout_result.indent_offset,
            4.0 * (cfg.text_opts.bullet_indent_em * cfg.text_opts.font_size) // ? List Items are indented by 1 unit by default - so indent = indent + 1 * indent_size
        );
    }

    #[test]
    fn styled_line_to_numbered_list_item_applies_prefix_length() {
        // * Arrange
        let mut font_system = create_default_test_font_system();
        let cfg = SvgConfig::default();

        let list_item_text = "6. Numbered Item".to_string();

        let segments = vec![StyledInline::Text(list_item_text)];

        // Create a basic StyledLine
        let styled_block_numbered_item = StyledBlock::NumberedListItem {
            segments: segments.clone(),
            number: 6, // ? Derived from AST
            indent: 0,
        };

        // * Act
        let layout_result = styled_block_numbered_item.into_layout_block(&cfg, &mut font_system);

        // * Assert
        assert_eq!(layout_result.prefix_len, 3); // ? prefixes for NumberedList are "#. " - 3 bytes: ASCII #, period, space.
    }

    #[test]
    fn styled_line_to_numbered_list_item_applies_indent_offset() {
        // * Arrange
        let mut font_system = create_default_test_font_system();
        let cfg = SvgConfig::default();

        let list_item_text = "2. Nested Number Item".to_string();

        let segments = vec![StyledInline::Text(list_item_text)];

        // Create a basic StyledLine
        let styled_block_numbered_item = StyledBlock::NumberedListItem {
            segments: segments.clone(),
            number: 2,
            indent: 2,
        };

        // * Act
        let layout_result = styled_block_numbered_item.into_layout_block(&cfg, &mut font_system);

        // * Assert

        assert_eq!(
            layout_result.indent_offset,
            3.0 * (cfg.text_opts.bullet_indent_em * cfg.text_opts.font_size) // ? List Items are indented by 1 unit by default - so indent = indent + 1 * indent_size
        );
    }

    // TODO revisit
    // #[test]
    // fn styled_line_blank_returns_line_height() {
    //     // * Arrange
    //     let mut font_system = create_default_test_font_system();
    //     let cfg = SvgConfig::default();

    //     // Create a blank StyledLine
    //     let styled_line_para = StyledBlock::Blank;

    //     // * Act
    //     let layout_result = styled_line_to_layout(styled_line_para, &mut font_system, &cfg);

    //     // * Assert
    //     // Assert LayoutResult is the Line variant (not Blank)
    //     assert!(matches!(layout_result, LayoutItem::Blank { height: _ }));
    //     let LayoutItem::Blank { height } = layout_result else {
    //         panic!("Expected LayoutResult::Blank");
    //     };
    //     assert_eq!(height, cfg.calculate_paragraph_spacing_px());
    // }

    #[test]
    fn into_layout_block_inline_code_overrides_parent_styles() {
        // * Arrange
        todo!()
    }
}
