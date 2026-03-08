use std::path::PathBuf;

use clap::{Parser};

#[derive(Parser)]
#[command(version, about, long_about=None)]
pub struct Cli {
    /// Path to input Markdown file
    pub input_path: PathBuf,
    
    /// Path to write output SVG file
    pub output_path: PathBuf,
}