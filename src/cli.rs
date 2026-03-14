use std::path::PathBuf;

use clap::{Parser};
use serde::{Serialize};

use crate::config::{CanvasOverride,  HeaderOverride, TypographyOverride};

#[derive(Parser, Serialize)]
#[command(version, about, long_about=None)]
pub struct Cli {
    /// Path to input Markdown file
    pub input_path: PathBuf,
    /// Path to write output SVG file
    pub output_path: PathBuf,
    pub preset: Option<PathBuf>,
    
    #[command(flatten)]
    #[serde(flatten)]
    pub canvas_opts: CanvasOverride,
    
    #[command(flatten)]
    #[serde(flatten)]
    pub text_opts: TypographyOverride,
    
    #[command(flatten)]
    #[serde(flatten)]
    pub header_opts: HeaderOverride
}
