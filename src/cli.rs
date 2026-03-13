use std::path::PathBuf;

use clap::{Parser};
use serde::{Deserialize, Serialize};

use crate::config::{CanvasConfig, CanvasOverride, HeaderConfig, HeaderOverride, TypographyConfig, TypographyOverride};

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
