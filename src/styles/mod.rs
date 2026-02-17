use cosmic_text::{Style, Weight};
use markdown::mdast::Node;
use regex::{Regex, RegexSet};
use std::{collections::HashMap, vec};



#[derive(Debug, Clone)]
pub struct StyledBlock {
    pub text: String,
    pub weight: Weight,
    pub style: Style,
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
        }
    }
}

// todo incomplete list of missing variants:
// - Table
// - Blockquote
#[derive(Clone, Debug)]
pub enum StyledLine {
    Header {
        segments: Vec<StyledBlock>,
        level: u8,
    },
    BulletListItem {
        segments: Vec<StyledBlock>,
        indent: u8,
    },
    NumberedListItem {
        segments: Vec<StyledBlock>,
        number: u32,
        indent: u8,
    },
    Paragraph {
        segments: Vec<StyledBlock>,
    },
    Code {
        text: String,
    },
    Blank,
}

// note: Strikethrough, Under/overline and other text decorations aren't supported in default Markdown and aren't showing in my node as unique text.
// For now, I don't care to adapt these cases. However, they could likely be addded as further StyleContext cases later on.
// ? it seems that these items are supported in GFM (Github Flavored Markdown) - for now, I would like to handle pure Markdown.
#[derive(Clone, Copy, Debug)]
pub struct StyleContext {
    pub bold: bool,
    pub italic: bool,
}

impl StyleContext {
    pub fn default() -> Self {
        StyleContext {
            bold: false,
            italic: false,
        }
    }
}

