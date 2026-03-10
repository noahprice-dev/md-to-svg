use std::collections::HashMap;

use derive_builder::Builder;


#[derive(Builder)]
#[builder(setter(into, strip_option))]
pub struct SvgConfig {
    // SVG Canvas Options
    #[builder(default = 600.)]
    pub width: f32,
    #[builder(default = 800.)]
    pub height: f32,
    // ? Support CSS Style padding args
    // ? This is for the actual SVG, not relevant to the Cosmic text.
    pub top_padding: f32,
    pub right_padding: f32,
    pub bottom_padding: f32,
    pub left_padding: f32,
    // Font Details
    pub font_size: f32,
    // todo expose this as an option to the end user?
    // todo  Explain default is sans-serif.
    //pub font_family: Family,
    /// Space between discrete text blocks.
    pub line_height_factor: f32,
    /// Space between lines inside of a paragraph.
    pub paragraph_spacing_em: f32,

    // Bullet Style Options
    pub bullet_indent_em: f32, // default 1.5 or 2.0 ->
    pub bullet_char: char,

    // Header Style Options
    pub header_scales: HashMap<u8, f32>,
    pub header_margin_top: f32,
    pub header_margin_bot: f32,

    pub bg_color: String,
}

impl SvgConfig {
    /// Create an SvgConfig with default values.
    /// Notably: 800px high by 600px wide, font size 16px, no padding, white background.
    pub fn new() -> Self {
        SvgConfig {
            width: 600.0,
            height: 800.0,
            top_padding: 0.0,
            right_padding: 0.0,
            bottom_padding: 0.0,
            left_padding: 0.0,
            font_size: 16.0,
            line_height_factor: 1.5,
            paragraph_spacing_em: 0.6,
            bullet_indent_em: 1.5,
            bullet_char: char::from_u32(0x2022).expect("Should be able to unwrap the character •"),
            header_scales: HashMap::from([
                (1, 2.0),
                (2, 1.6),
                (3, 1.3),
                (4, 1.1),
                (5, 1.0),
                (6, 1.0),
            ]),
            header_margin_top: 0.0,
            header_margin_bot: 0.0,
            bg_color: String::from("#FFFFFF"),
        }
    }
    /// Space between discrete text blocks.
    pub const fn get_line_height(&self) -> f32 {
        self.font_size * self.line_height_factor
    }
    /// Space between lines inside of a paragraph.
    pub const fn get_paragraph_spacing(&self) -> f32 {
        self.font_size * self.paragraph_spacing_em
    }
    
}