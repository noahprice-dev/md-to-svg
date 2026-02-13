
use std::collections::HashMap;

use regex::{Regex, RegexSet};

pub struct Options {
    // SVG Canvas Options
    width: usize,
    height: usize,
    // ? Support CSS Style padding args
    top_padding: usize,
    right_padding: usize,
    bottom_padding: usize,
    left_padding: usize,

    // Font Details
    font_size: usize,
    font_family: String,

    // Bullet Style Options
    bullet_indent: usize,
    bullet_char: String,

    // Header Style Options
    header_scales: HashMap<usize, f32>,
    header_margin_top: f32,
    header_margin_bot: f32,
}
pub enum Style {
    Bold,
    Italic,
    Code,
}
pub struct Tspan(String);

#[derive(Debug)]
pub enum Block {
    Header { header_level: usize, text: String },
    Paragraph(String),
    Bullet { indent_level: usize, text: String },
    Numbered { indent_level: usize, text: String },
    Blank,
}

impl Block {
    pub fn to_tspan(self) -> Tspan {
        match self {
            Block::Header { header_level, text } => todo!(),
            Block::Paragraph(_) => todo!(),
            Block::Bullet { indent_level, text } => todo!(),
            Block::Numbered { indent_level, text } => todo!(),
            Block::Blank => todo!(),
        }
    }
}

pub fn parse_block_type(raw_line: String) -> Block {
    let patterns = vec![
        r"^(#{1,3})\s+(.*)",     // Header
        r"^(\s*)(\d+)\.\s+(.*)", // Numbered list (1., 2. etc)
        r"^(\s*)([-*+])\s+(.*)", // Bullet list (matches - | * | +)
    ];

    let set = RegexSet::new(&patterns).expect("Markdown RegEx patterns should be valid.");
    let regexes: Vec<Regex> = patterns
        .iter()
        .map(|p| Regex::new(p).expect("Should be able to convert pattern to RegEx"))
        .collect();

    let line = raw_line.trim_end();

    if line.is_empty() {
        return Block::Blank;
    }

    let block = match set.matches(line).into_iter().next() {
        // * Headers
        Some(0) => {
            let caps = regexes[0]
                .captures(line)
                .expect("Matched on Header, should have found Header-like block");
            let level = caps[1].len(); // ? The header size is controlled by the number of hashes in a row. 1 = h1, 2=h2, etc.
            let text = &caps[2]; //? Everything after # ends
            Block::Header {
                header_level: level,
                text: text.to_string(),
            }
        }
        // * Numbered List
        Some(1) => {
            let caps = regexes[1]
                .captures(line)
                .expect("Matched on Numbered List, should have found NumberedList-like block");
            let indent = caps[1].len();

            let text = &caps[3]; //? Everything after # ends
            Block::Numbered {
                indent_level: indent,
                text: text.to_string(),
            }
        }
        // * Bullet
        Some(2) => {
            let caps = regexes[2]
                .captures(line)
                .expect("Matched on Bullet List, should have found BulletList-like block");
            let indent = caps[1].len() / 4; // ? 4 spaces/1 tab = 1 indent

            let text = &caps[3]; //? Everything after # ends
            Block::Bullet {
                indent_level: indent,
                text: text.to_string(),
            }
        }
        _ => Block::Paragraph(line.to_string()),
    };
    block
}

pub fn parse_inline_style(block: Block) -> Vec<Tspan> {
    todo!()
}
