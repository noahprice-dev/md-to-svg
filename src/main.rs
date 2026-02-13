use std::error::Error;
use std::fmt::format;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::vec;
use std::{collections::HashMap, path::Path};
use md_to_svg::types::parse_block_type;
use md_to_svg::types::Block;

use regex::{Regex, RegexSet};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read in a Markdown file.
    let blocks = parse_markdown("./test.md")?;
    println!("Blocks Found: {:#?}", blocks);
    Ok(())
}

fn parse_markdown<P>(filename: P) -> Result<Vec<Block>, Box<dyn std::error::Error>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    let blocks = BufReader::new(file)
        .lines()
        // ? Perform our word-wrap on the Line
        .map(|line_result| line_result.map(|line| parse_block_type(line)))
        .collect();

    match blocks {
        Ok(blocks) => Ok(blocks),
        Err(e) => Err(Box::new(e)),
    }
}
