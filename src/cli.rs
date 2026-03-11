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
    pub canvas_opts: CanvasConfig,
    #[command(flatten)]
    pub text_opts: TypographyConfig,
    #[command(flatten)]
    pub header_opts: HeaderConfig
}