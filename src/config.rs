use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs::{self, File}, io::{self, ErrorKind, Read}, path::PathBuf, str::FromStr};

use crate::MdToSvgError;

// TODO:
// ~~ Add config builder
// ~~ Move Config sub-struccts into independent structs with Flatten
// ? Create default config file
// ? Parse Cli as overrides to ConfigBuilder with suported defaults to unwrap Options
// ? Use `dirs` crate to derive config location agnostic to OS
#[derive(Debug, Deserialize)]
pub struct PresetConfig {
    pub canvas: CanvasConfig,
    pub typography: TypographyConfig,
    pub headers: HeaderConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SvgConfig {
    // SVG Canvas Options
    pub canvas_opts: CanvasConfig,
    // Font & Whitespace Options
    pub text_opts: TypographyConfig,
    // Header Scale & Margin Options
    pub header_opts: HeaderConfig,
}

impl SvgConfig {
    // / Create an SvgConfig with default values.
    // / Notably: 800px high by 600px wide, font size 16px, no padding, white background.
    pub fn new() -> Self {
        SvgConfig {
            canvas_opts: CanvasConfig::default(),
            text_opts: TypographyConfig::default(),
            header_opts: HeaderConfig::default(),
        }
    }
    /// Space between discrete text blocks.
    pub const fn get_line_height(&self) -> f32 {
        self.text_opts.font_size * self.text_opts.line_height_factor
    }
    /// Space between lines inside of a paragraph.
    pub const fn get_paragraph_spacing(&self) -> f32 {
        self.text_opts.font_size * self.text_opts.paragraph_spacing_em
    }
}

impl From<PresetConfig> for SvgConfig {
    fn from(preset_config: PresetConfig) -> Self {
        Self { canvas_opts: preset_config.canvas, text_opts: preset_config.typography, header_opts: preset_config.headers }
    }
}

#[derive(Debug, Clone, clap::Args, Serialize, Deserialize)]
pub struct CanvasConfig {
    #[arg(long, default_value_t = CanvasConfig::default().width)]
    pub width: f32,
    #[arg(long, default_value_t = CanvasConfig::default().height)]
    pub height: f32,
    #[arg(long, default_value_t = CanvasConfig::default().bg_color)]
    pub bg_color: String,
    /// CSS-style padding: "10" (all), "10 20" (v h), "10 20 30" (t, h, b) or "10 20 10 20" (t r b l)
    #[arg(long, default_value = "0")]
    pub padding: Padding,
}

impl Default for CanvasConfig {
    fn default() -> Self {
        Self {
            width: 600.,
            height: 800.,
            bg_color: "#FFFFFF".to_string(),
            padding: Padding::default(),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct Padding {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl FromStr for Padding {
    type Err = MdToSvgError;
    fn from_str(s: &str) -> Result<Self, MdToSvgError> {
        let parts: Vec<f32> = s
            .split_whitespace()
            .map(|v| {
                v.parse().map_err(|_| MdToSvgError::MarkdownParseFailed {
                    reason: format!("Invalid Number: {}", v),
                })
            })
            .collect::<Result<_, _>>()?;

        match parts.as_slice() {
            [all] => Ok(Self::all(*all)),
            [tb, lr] => Ok(Self::symmetric(*tb, *lr)),
            [t, lr, b] => Ok(Self::new(*t, *lr, *b, *lr)),
            [t, r, b, l] => Ok(Self::new(*t, *r, *b, *l)),
            _ => Err(MdToSvgError::MarkdownParseFailed {
                reason: "Padding must have 1, 2, 3 or 4 values.".to_string(),
            }),
        }
    }
}

impl Padding {
    pub fn new(top: f32, right: f32, bottom: f32, left: f32) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }
    pub fn symmetric(v: f32, h: f32) -> Self {
        Self::new(v, h, v, h)
    }
    pub fn all(padding: f32) -> Self {
        Self::new(padding, padding, padding, padding)
    }
}

#[derive(Debug, Clone, clap::Args, Serialize, Deserialize)]
pub struct TypographyConfig {
    #[arg(long, default_value_t = TypographyConfig::default().font_size)]
    pub font_size: f32,
    
    // todo expose this as an option to the end user?
    // todo  Explain default is sans-serif.
    //#[arg(skip)]
    //pub font_family: Family,
    
    #[arg(long, default_value_t = TypographyConfig::default().line_height_factor)]
    pub line_height_factor: f32,
    /// Space between lines inside of a paragraph.
    #[arg(long, default_value_t = TypographyConfig::default().paragraph_spacing_em)]
    pub paragraph_spacing_em: f32,
    /// Bullet indentation in em units.
    #[arg(long, default_value_t = TypographyConfig::default().bullet_indent_em)]
    pub bullet_indent_em: f32,
    
    #[arg(long, default_value_t = TypographyConfig::default().bullet_char)]
    pub bullet_char: char,
}

impl Default for TypographyConfig {
    fn default() -> Self {
        Self {
            font_size: 16.,
            line_height_factor: 1.5,
            paragraph_spacing_em: 0.6,
            bullet_indent_em: 1.5,
            bullet_char: char::from_u32(0x2022).expect("Should be able to unwrap the character •"),
        }
    }
}
#[derive(Debug, Clone, clap::Args, Serialize, Deserialize)]
pub struct HeaderConfig {
    #[arg(skip)]
    pub header_scales: HashMap<u8, f32>,
    /// Margin above header in px
    #[arg(long, default_value_t = TypographyConfig::default().line_height_factor)]
    pub header_margin_top: f32,
    /// Margin below header in px
    #[arg(long, default_value_t = TypographyConfig::default().line_height_factor)]
    pub header_margin_bot: f32,
}

impl Default for HeaderConfig {
    fn default() -> Self {
        Self {
            header_scales: HashMap::from([
                (1, 2.0),
                (2, 1.6),
                (3, 1.3),
                (4, 1.1),
                (5, 1.0),
                (6, 1.0),
            ]),
            header_margin_top: 0.,
            header_margin_bot: 0.,
        }
    }
}

/// Load an SVG Preset Configuration from a path.
/// ## Errors
/// - If the path is invalid, will return MdToSvgError::ConfigNotFound
/// - If the file cannot be read for another reason (e.g. Busy, invalid permissions) then it will return MdToSvgError::ConfigNotReadable
/// - If the underlying TOML is invalid, will return a MdToSvgError::ConfigParseFailed with the underlying `toml` error.
pub fn load_preset_config(preset_path: PathBuf) -> Result<SvgConfig, MdToSvgError> {
    let raw_toml = fs::read_to_string(&preset_path).map_err(|err| match err.kind() {
        ErrorKind::NotFound => {
            MdToSvgError::ConfigNotFound(preset_path)
        },
        _=> MdToSvgError::ConfigNotReadable(preset_path, err)
    })?;
    let preset_config = toml::from_str::<PresetConfig>(&raw_toml)?;
    
    Ok(SvgConfig::from(preset_config))
    
}


// * -- Overrides --
#[derive(clap::Args, Serialize)]
pub struct CanvasOverride {}

#[derive(clap::Args, Serialize)]
pub struct TypographyOverride {
    /// Font Size. (default 16)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub font_size: Option<f32>,
    
    /// Space between discrete text blocks.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub line_height_factor: Option<f32>,
    
    /// Spacing between lines within a block.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long="pspace")]
    pub paragraph_spacing_em: Option<f32>,
    
    /// Bullet indentation in em units.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long="bindent")]
    pub bullet_indent_em: Option<f32>,
    
    /// Character to use as unordered list prefix. Requires single character.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub bullet_char: Option<char>
}

#[derive(clap::Args, Serialize)]
pub struct HeaderOverride {}