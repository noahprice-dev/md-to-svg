use std::path::PathBuf;

use clap::{Parser};
use serde::{Serialize};

use crate::config::{CanvasOverride,  HeaderOverride, TypographyOverride};

#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
pub struct Cli {
    /// Path to input Markdown file
    pub input_path: PathBuf,
    /// Path to write output SVG file
    pub output_path: PathBuf,
    pub preset: Option<PathBuf>,
    
    #[command(flatten)]
    pub overrides: ConfigOverrides
}

#[derive(clap::Args, Serialize, Debug)]
pub struct ConfigOverrides {
    #[command(flatten)]
    pub canvas_opts: CanvasOverride,
    
    #[command(flatten)]
    pub text_opts: TypographyOverride,
    
    #[command(flatten)]
    pub header_opts: HeaderOverride
}