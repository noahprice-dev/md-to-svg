use std::path::PathBuf;

use clap::{Parser};
use serde::Serialize;

use crate::config::{CanvasConfig, HeaderConfig, TypographyConfig};

#[derive(Parser, Serialize)]
#[command(version, about, long_about=None)]
pub struct Cli {
    /// Path to input Markdown file
    pub input_path: PathBuf,
    /// Path to write output SVG file
    pub output_path: PathBuf,
    
    #[command(flatten)]
    #[serde(rename(deserialize = "canvas"))]
    pub canvas_opts: CanvasConfig,
    
    #[command(flatten)]
    #[serde(rename(deserialize = "text-style"))]
    pub text_opts: TypographyConfig,
    
    #[command(flatten)]
    #[serde(rename="headers")]
    pub header_opts: HeaderConfig
}