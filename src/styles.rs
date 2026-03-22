use cosmic_text::{FamilyOwned, Style, Weight};

use crate::layout::LayoutInline;

#[derive(Debug, Clone, PartialEq)]
// TODO (1.1) - Support for strikethrough, sub/superscript.
/// ? Family defaults to SansSerif
// TODO (1.1) Make family configurable.
// TODO use bool style flags.

/// A span of text with a consistent text-style.
pub struct StyledSpan {
    pub text: String,
    pub weight: bool,
    pub style: bool,
    pub family: bool,
}

impl StyledSpan {
    // TODO remove and replace with `layout` level Transform step.
}

/// Defines a text container that may optionally
/// contain further differently styled text or non-text structural definitions.
// TODO rename to StyledInline (STRUCTURE PRESERVING, PRODUCED BY PARSER)
#[derive(Clone, Debug, PartialEq)]
pub enum StyledInline {
    Text(StyledSpan),
    Emphasis(Vec<StyledInline>),
    Strong(Vec<StyledInline>),
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
                    text: span.text,
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
///
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
    Link {
        // ? Can be formatted text.
        text: Vec<StyledSpan>,
        url: String,
        title: Option<String>,
    },
    Image {
        // ? Cannot be formatted text.
        description: Option<String>,
        url: String,
        title: Option<String>,
    },
    ThematicBreak,
    Blank,
}

impl StyledBlock {
    pub fn get_type(&self) -> String {
        match self {
            StyledBlock::Header { .. } => String::from("Header"),
            StyledBlock::BulletListItem { .. } => String::from("Bullet List Item"),
            StyledBlock::NumberedListItem { .. } => String::from("Numbered List Item"),
            StyledBlock::Paragraph { .. } => String::from("Paragraph"),
            StyledBlock::Blockquote { .. } => String::from("Blockquote"),
            StyledBlock::Link { .. } => String::from("Link"),
            StyledBlock::Image { .. } => String::from("Image"),
            StyledBlock::ThematicBreak => String::from("Horizontal Rule"),
            StyledBlock::Blank => String::from("Blank"),
        }
    }
}

// note: Strikethrough, Under/overline and other text decorations aren't supported in default Markdown and aren't showing in my node as unique text.
// For now, I don't care to adapt these cases. However, they could likely be addded as further StyleContext cases later on.
// ? it seems that these items are supported in GFM (Github Flavored Markdown) - for now, I would like to handle pure Markdown.
// todo make private
#[derive(Clone, Copy, Debug)]
pub struct StyleContext {
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
