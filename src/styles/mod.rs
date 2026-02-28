use cosmic_text::{FamilyOwned, Style, Weight};



#[derive(Debug, Clone)]
// todo support strikethrough - this passes down to the SVG directly.
/// Settings for a set of glyphs to be rendered.
/// Family defaults to SansSerif
// todo can we augment this to use Config defaults as a default? Alternatively use an
pub struct StyledBlock{
    pub text: String,
    pub weight: Weight,
    pub style: Style,
    pub family: FamilyOwned,
}
impl StyledBlock {
    pub fn new(text: String, ctx: StyleContext) -> Self {
        StyledBlock {
            text: text,
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
            
        }
    }
}

#[derive(Clone, Debug)]
pub enum StyledSegment {
    Text(StyledBlock),
    HardBreak
}
impl StyledSegment {
    pub fn text(text: String, ctx: StyleContext) -> Self {
        StyledSegment::Text(StyledBlock::new(text, ctx))
    }
}
// todo incomplete list of missing variants:
// - Table
// - Blockquote
#[derive(Clone, Debug)]
pub enum StyledLine {
    Header {
        segments: Vec<StyledSegment>,
        level: u8,
    },
    BulletListItem {
        segments: Vec<StyledSegment>,
        indent: u8,
    },
    NumberedListItem {
        segments: Vec<StyledSegment>,
        number: u32,
        indent: u8,
    },
    Paragraph {
        segments: Vec<StyledSegment>,
    },
    InlineCode {
        text: String,
    },
    Blank,
}

impl StyledLine {
    pub fn get_type(&self) -> String {
        match self {
            StyledLine::Header { .. } => String::from("Header"),
            StyledLine::BulletListItem {..} => String::from("Bullet List Item"),
            StyledLine::NumberedListItem { .. } => String::from("Numbered List Item"),
            StyledLine::Paragraph { .. } => String::from("Paragraph"),
            StyledLine::InlineCode { ..} => String::from("Code"),
            StyledLine::Blank => String::from("Blank"),
        }
    }
}

// note: Strikethrough, Under/overline and other text decorations aren't supported in default Markdown and aren't showing in my node as unique text.
// For now, I don't care to adapt these cases. However, they could likely be addded as further StyleContext cases later on.
// ? it seems that these items are supported in GFM (Github Flavored Markdown) - for now, I would like to handle pure Markdown.
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
            monospace: false
        }
    }
}

