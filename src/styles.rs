use cosmic_text::{FamilyOwned, Style, Weight};



#[derive(Debug, Clone, PartialEq)]
// TODO (1.1) - Support for strikethrough, sub/superscript.
/// ? Family defaults to SansSerif
pub struct StyledLeafBlock{
    pub text: String,
    pub weight: Weight,
    pub style: Style,
    pub family: FamilyOwned,
}
impl StyledLeafBlock {
    pub fn new(text: String, ctx: StyleContext) -> Self {
        StyledLeafBlock {
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

/// A Styled Segment is a run of text within some Container Block with some consistent style
#[derive(Clone, Debug, PartialEq)]
pub enum StyledSegment {
    Text(StyledLeafBlock),
    HardBreak
}
impl StyledSegment {
    pub fn text(text: String, ctx: StyleContext) -> Self {
        StyledSegment::Text(StyledLeafBlock::new(text, ctx))
    }
}
// todo incomplete list of missing variants:
// TODO (1.1) Tables via HTML/GFM
// - Blockquote
// - Link
// - Image
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
    Blockquote{
        text: String,
    },
    Link{
        // ? Can be formatted text.
        text: Vec<StyledLeafBlock>,
        url: String,
        title: Option<String>,  
    },
    Image {
        // ? Cannot be formatted text.
        description: Option<String>,
        url: String,
        title: Option<String>
    },
    ThematicBreak,
    Blank,
}

impl StyledLine {
    pub fn get_type(&self) -> String {
        match self {
            StyledLine::Header { .. } => String::from("Header"),
            StyledLine::BulletListItem {..} => String::from("Bullet List Item"),
            StyledLine::NumberedListItem { .. } => String::from("Numbered List Item"),
            StyledLine::Paragraph { .. } => String::from("Paragraph"),
            StyledLine::Blockquote { .. } => String::from("Blockquote"),
            StyledLine::Link { ..} => String::from("Link"),
            StyledLine::Image{..} => String::from("Image"),
            StyledLine::ThematicBreak => String::from("Horizontal Rule"),
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

