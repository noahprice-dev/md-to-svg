use serde::{Deserialize, Deserializer, Serialize};
use std::{
    fs::{self},
    io::ErrorKind,
    path::PathBuf,
    str::FromStr,
};

use crate::MdToSvgError;

// TODO:
// ~~ Add config builder
// ~~ Move Config sub-struccts into independent structs with Flatten
// ? Create default config file
// ? Parse Cli as overrides to ConfigBuilder with suported defaults to unwrap Options
// ? Use `dirs` crate to derive config location agnostic to OS

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct SvgConfig {
    // SVG Canvas Options
    pub canvas_opts: CanvasConfig,
    // Font & Whitespace Options
    pub text_opts: TypographyConfig,
    // Header Scale & Margin Options
    pub header_opts: HeaderConfig,
}

impl SvgConfig {
    /// Space between lines inside of a paragraph.
    pub const fn calculate_line_height_px(&self) -> f32 {
        self.text_opts.font_size * self.text_opts.line_height_factor
    }
    /// Space between discrete paragraphs or blocks of text.
    pub const fn calculate_paragraph_spacing_px(&self) -> f32 {
        self.text_opts.font_size * self.text_opts.paragraph_spacing_factor
    }
}

impl Default for SvgConfig {
    // / Create an SvgConfig with default values.
    // / Notably: 800px high by 600px wide, font size 16px, no padding, white background.
    fn default() -> Self {
        SvgConfig {
            canvas_opts: CanvasConfig::default(),
            text_opts: TypographyConfig::default(),
            header_opts: HeaderConfig::default(),
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct PresetConfig {
    #[serde(
        rename(serialize = "canvas_opts", deserialize = "canvas"),
        skip_serializing_if = "Option::is_none"
    )]
    pub canvas: Option<CanvasOverride>,

    #[serde(
        rename(serialize = "text_opts", deserialize = "typography"),
        skip_serializing_if = "Option::is_none"
    )]
    pub typography: Option<TypographyOverride>,

    #[serde(
        rename(serialize = "header_opts", deserialize = "headers"),
        skip_serializing_if = "Option::is_none"
    )]
    pub headers: Option<HeaderOverride>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct CanvasConfig {
    pub width: f32,
    pub height: f32,
    pub bg_color: String,
    /// CSS-style padding: "10" (all), "10 20" (v h), "10 20 30" (t, h, b) or "10 20 10 20" (t r b l)
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

#[derive(Debug, PartialEq, Clone, Default, Serialize, Deserialize)]
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
impl TryFrom<&str> for Padding {
    type Error = MdToSvgError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse()
    }
}
impl Padding {
    /// Define a new Padding in clockwise order: Top, Right, Bottom, Left
    pub fn new(top: f32, right: f32, bottom: f32, left: f32) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }
    /// Define a new Padding for Vertical then Horizontal
    pub fn symmetric(v: f32, h: f32) -> Self {
        Self::new(v, h, v, h)
    }

    /// Define a new Padding where all sides are the same.
    pub fn all(padding: f32) -> Self {
        Self::new(padding, padding, padding, padding)
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct TypographyConfig {
    pub font_size: f32,
    // todo expose this as an option to the end user?
    // todo  Explain default is sans-serif.
    //#[arg(skip)]
    //pub font_family: Family,
    pub line_height_factor: f32, 
    pub paragraph_spacing_factor: f32,
    pub bullet_indent_em: f32,
    pub bullet_char: String,
}

impl Default for TypographyConfig {
    fn default() -> Self {
        Self {
            font_size: 16.,
            line_height_factor: 1.15,
            paragraph_spacing_factor: 1.2,
            bullet_indent_em: 1.5,
            bullet_char: String::from("•"),
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct HeaderConfig {
    pub header_scales: HeaderScales,
    /// Margin above header in px
    pub header_margin_top: f32,
    /// Margin below header in px
    pub header_margin_bot: f32,
}

impl Default for HeaderConfig {
    fn default() -> Self {
        Self {
            header_scales: HeaderScales::default(),
            header_margin_top: 10.,
            header_margin_bot: 10.,
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct HeaderScales {
    pub h1: f32,
    pub h2: f32,
    pub h3: f32,
    pub h4: f32,
    pub h5: f32,
    pub h6: f32,
}
impl HeaderScales {
    /// Access the font scaling for a specific Header Level by u8.
    /// The Markdown AST will only ever give us a value between 1 and 6 inclusive.
    pub fn scale_for_level(&self, level: u8) -> f32 {
        match level {
            1 => self.h1,
            2 => self.h2,
            3 => self.h3,
            4 => self.h4,
            5 => self.h5,
            6 => self.h6,
            _ => unreachable!("Expected a value between 1 and 6 inclusive"),
        }
    }
}
impl Default for HeaderScales {
    fn default() -> Self {
        Self {
            h1: 2.0,
            h2: 1.6,
            h3: 1.3,
            h4: 1.1,
            h5: 1.0,
            h6: 1.0,
        }
    }
}

// * -- Overrides --
#[derive(Debug, PartialEq, clap::Args, Serialize, Deserialize)]
pub struct CanvasOverride {
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<f32>,

    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<f32>,

    /// Background color declaration in Hex code. Must include '#'. (Default white | "#FFFFFF")
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bg_color: Option<String>,
    /// CSS-style padding: "10" (all), "10 20" (v h), "10 20 30" (t, h, b) or "10 20 10 20" (t r b l)
    #[arg(long)]
    #[serde(
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_padding_from_str",
        default
    )]
    pub padding: Option<Padding>,
}

#[derive(Debug, PartialEq, clap::Args, Serialize, Deserialize)]
pub struct TypographyOverride {
    /// Font Size in px (Default 16)
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_size: Option<f32>,

    /// Multiplier to adjust the vertical height of the line box (default 1.15)
    /// This is multiplied by the font-size to create spacing between glyphs within a paragraph.
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_height_factor: Option<f32>,

    /// Multipler to adjust the vertical distance between two discrete paragraphs or blocks after a Hard Line Break. (Default 2)
    #[arg(long = "p-space")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paragraph_spacing_em: Option<f32>,

    /// Bullet indentation in em units. (Default 2)
    #[arg(long = "b-indent")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bullet_indent_em: Option<f32>,

    /// String to use as a prefix 'bullet' for unordered lists. (Default "•")
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bullet_char: Option<String>,
}

#[derive(Debug, PartialEq, clap::Args, Serialize, Deserialize)]
pub struct HeaderOverride {
    #[arg(skip)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header_scales: Option<HeaderScales>,
    /// Margin above header in px. (Default 10.0)
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header_margin_top: Option<f32>,
    /// Margin below header in px. (Default 10.0)
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header_margin_bot: Option<f32>,
}

/// Load an SVG Preset Configuration from a path and return a `PresetConfig` to overwrite.
/// ## Errors
/// - If the path is invalid, will return MdToSvgError::ConfigNotFound
/// - If the file cannot be read for another reason (e.g. Busy, invalid permissions) then it will return MdToSvgError::ConfigNotReadable
/// - If the underlying TOML is invalid, will return a MdToSvgError::ConfigParseFailed with the underlying `toml` error.
pub fn load_preset_config(preset_path: PathBuf) -> Result<PresetConfig, MdToSvgError> {
    let raw_toml = fs::read_to_string(&preset_path).map_err(|err| match err.kind() {
        ErrorKind::NotFound => MdToSvgError::ConfigNotFound(preset_path),
        _ => MdToSvgError::ConfigNotReadable(preset_path, err),
    })?;

    Ok(toml::from_str::<PresetConfig>(&raw_toml)?)
}

/// Handle deserializing CSS Style padding declaration from strings.
fn deserialize_padding_from_str<'de, D: Deserializer<'de>>(
    d: D,
) -> Result<Option<Padding>, D::Error> {
    let s = String::deserialize(d)?;
    s.parse::<Padding>()
        .map(Some)
        .map_err(serde::de::Error::custom)
}

#[cfg(test)]
mod tests {
    use config::Config;

    use crate::config::{HeaderScales, Padding, PresetConfig, SvgConfig, TypographyOverride};

    // * --- Overrides ---
    // * Typography Overrides
    #[test]
    fn typography_override_replaces_single_some_value() {
        // * Arrange
        // Type Override with FontSize
        let typography_override = TypographyOverride {
            font_size: Some(24.0),
            line_height_factor: None,
            paragraph_spacing_em: None,
            bullet_indent_em: None,
            bullet_char: None,
        };

        let preset_config = PresetConfig {
            typography: Some(typography_override),
            canvas: None,
            headers: None,
        };
        let default_config = SvgConfig::default();

        // * Act
        let combined_settings = Config::builder()
            .add_source(
                config::Config::try_from(&default_config)
                    .expect("Should be able to add default SvgConfig to ConfigBuilder"),
            )
            .add_source(
                config::Config::try_from(&preset_config)
                    .expect("Should be able to add modified PresetConfig to ConfigBuilder"),
            )
            .build()
            .expect("Should be able to build config from PresetConfig & Default SvgConfig.");

        // * Assert
        println!("Basic Settings:{:#?}", &combined_settings);
        // Verify the font size has changed
        assert_eq!(
            combined_settings.get::<f32>("text_opts.font_size").unwrap(),
            preset_config.typography.unwrap().font_size.unwrap()
        );
        //?  Verify no other values have changed.
        assert_eq!(
            combined_settings
                .get::<f32>("text_opts.bullet_indent_em")
                .unwrap(),
            default_config.text_opts.bullet_indent_em
        );
    }

    // * --- Padding ---
    #[test]
    fn padding_from_str_parses_one_value_applies_to_all_sides() {
        let padding: Padding = "10".parse().unwrap();

        assert_eq!(padding, Padding::all(10.0));
    }

    #[test]
    fn padding_from_str_parses_two_values_applies_to_vertical_horizontal() {
        let padding: Padding = "10 20".parse().unwrap();

        assert_eq!(padding, Padding::symmetric(10.0, 20.0));
    }

    #[test]
    fn padding_from_str_parses_three_values_applies_to_top_horizontal_bottom() {
        let padding: Padding = "10 20 30".parse().unwrap();

        assert_eq!(padding, Padding::new(10.0, 20.0, 30.0, 20.0));
    }

    #[test]
    fn padding_from_str_parses_four_values_applies_each_side_independently() {
        let padding: Padding = "10 20 30 40".parse().unwrap();

        assert_eq!(padding, Padding::new(10.0, 20.0, 30.0, 40.0));
    }
    
    // * --- Header Scales ---
    
    #[test]
    
    fn scale_for_level_returns_correct_scale_for_all_valid_levels() {
        // Create a HeaderScales
        let header_scales = HeaderScales::default();
        
        assert_eq!(header_scales.scale_for_level(1), 2.0);
        assert_eq!(header_scales.scale_for_level(2), 1.6);
        assert_eq!(header_scales.scale_for_level(3), 1.3);
        assert_eq!(header_scales.scale_for_level(4), 1.1);
        assert_eq!(header_scales.scale_for_level(5), 1.0);
        assert_eq!(header_scales.scale_for_level(6), 1.0);
        
        
    }
    
    #[test]
    #[should_panic(expected = "Expected a value between 1 and 6 inclusive")]
    fn scale_for_level_panics_on_invalid_level() {
        let header_scales = HeaderScales::default();
        
        header_scales.scale_for_level(99);
    }
    
}
