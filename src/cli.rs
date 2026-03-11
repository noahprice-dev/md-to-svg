use std::path::PathBuf;

use clap::{Parser};
use serde::{Deserialize, Serialize};

use crate::config::{CanvasConfig, HeaderConfig, TypographyConfig};

#[derive(Parser, Serialize, Deserialize)]
#[command(version, about, long_about=None)]
pub struct Cli {
    /// Path to input Markdown file
    pub input_path: PathBuf,
    /// Path to write output SVG file
    pub output_path: PathBuf,
    pub preset: Option<PathBuf>,
    
    #[command(flatten)]
    #[serde(rename="canvas",skip_serializing_if = "Option::is_none")]
    pub canvas_opts: Option<CanvasConfig>,
    
    #[command(flatten)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_opts: Option<TypographyConfig>,
    
    #[command(flatten)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header_opts: Option<HeaderConfig>
}