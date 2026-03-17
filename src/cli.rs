use std::path::PathBuf;

use clap::{Parser};
use serde::{Serialize};

use crate::config::{CanvasOverride,  HeaderOverride, TypographyOverride};

/// A highly configurable CLI tool to convert Markdown to SVG.
/// Priority in ascending order for overrides are Defaults -> Preset Config -> CLI Overrides.
/// A list of default settings is available at [X].
#[derive(Parser, Debug)]
#[command(version, about, long_about="A highly configurable CLI tool to convert Markdown to SVG.\nThe priority for overrides in ascending order are:\nDefaults -> Preset Config -> CLI Overrides.\nA list of default settings is available at [X].")]
pub struct Cli {
    /// Path to input Markdown file
    pub input_path: PathBuf,
    /// Path to write output SVG file
    pub output_path: PathBuf,
    /// Optional path to a Preset.toml file.
    pub preset_path: Option<PathBuf>,
    
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