use cosmic_text::Attrs;
use cosmic_text::Buffer;
use cosmic_text::FontSystem;
use cosmic_text::Metrics;
use cosmic_text::Shaping;
use markdown::{ParseOptions, to_mdast};
use md_to_svg::styles::Block;
use md_to_svg::styles::parse_block_type;
use std::error::Error;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Read};
use std::vec;
use std::{collections::HashMap, path::Path};

use regex::{Regex, RegexSet};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read in a Markdown file.
    parse_markdown("./test.md");
    Ok(())
}

fn parse_markdown<P>(filename: P)
where
    P: AsRef<Path>,
{
    //let file = File::open(filename).unwrap();
    let raw_md = fs::read_to_string(filename).unwrap();

    // Detect system fonts
    let mut font_system = FontSystem::new();
    // Define font size & line height of our buffer.
    let metrics = Metrics::new(14.0, 20.0);

    // Instantiate our Buffer. We will perform shaping & layout for our strings.
    let mut buffer = Buffer::new(&mut font_system, metrics);

    // Add our font_system to the Buffer thru borrowing "for convvenient method calls"
    let mut buffer = buffer.borrow_with(&mut font_system);

    // ? Note that this defines the allowable space to write into. If we are unable to fit the full string, it will do as much as possible then stop.
    buffer.set_size(Some(800.0), None);

    // Attributes handle the styling and font family, among other things. Defaults to a sans-serif
    let attrs = Attrs::new();

    // Use advanced since we don't control the input text, or which font is being displayed (inherently)
    buffer.set_text("Hi this is a string with a very *long* layout that will **require** wrapping hopefully here are some ***names** of books Tales from Obojima and From Hell and Paprika and Homonculus", &attrs, Shaping::Advanced, None);
    buffer.shape_until_scroll(false);
    let mut lines: Vec<String> = Vec::new();

    // Inspect the output runs
    for run in buffer.layout_runs() {
        // Get the first and last glyph of our entire run.
        let first_glyph = run.glyphs.first().map(|g| g.start).unwrap_or(0);
        let last_glyph = run.glyphs.last().map(|g| g.end).unwrap_or(0);

        // Get the line for this run. Note that if we do not have an inherent linebreak (\n) we always have 0.
        if let Some(buffer_line) = buffer.lines.get(run.line_i) {
            // Get the full input string.
            let full_paragraph_text = buffer_line.text();

            let visual_line_string = &full_paragraph_text[first_glyph..last_glyph];
            lines.push(visual_line_string.to_string());
        }
    }

    for (i, line) in lines.iter().enumerate() {
        println!("Line {}: |{}|", i, line);
    }
}
