use std::collections::HashMap;

use cosmic_text::{Attrs, Buffer, Family, FontSystem, Metrics, Weight};

use crate::styles::StyledLine;

pub struct SvgConfig {
    // SVG Canvas Options
    pub width: f32,
    pub height: usize,
    // ? Support CSS Style padding args
    // ? This is for the actual SVG, not relevant to the Cosmic text.
    pub top_padding: usize,
    pub right_padding: usize,
    pub bottom_padding: usize,
    pub left_padding: usize,

    // Font Details
    pub font_size: f32,
    // todo expose this as an option to the end user?
    // todo  Explain default is sans-serif.
    //pub font_family: Family,

    // Bullet Style Options
    pub bullet_indent: usize,
    pub bullet_char: String,

    // Header Style Options
    pub header_scales: HashMap<u8, f32>,
    pub header_margin_top: f32,
    pub header_margin_bot: f32,
}

pub fn styled_line_to_buffer(
    line: &StyledLine,
    font_system: &mut FontSystem,
    cfg: &SvgConfig,
) -> Buffer {
    match line {
        StyledLine::Paragraph { segments } => {
            let mut buffer = Buffer::new(
                font_system,
                Metrics::new(cfg.font_size, cfg.font_size * 1.5),
            );
            let rich_text: Vec<(&str, Attrs<'_>)> = segments
                .iter()
                .map(|block| {
                    let text = block.text.as_str();
                    let attrs = Attrs::new().weight(block.weight).style(block.style);

                    (text, attrs)
                })
                .collect();

            buffer.set_rich_text(
                font_system,
                rich_text,
                &Attrs::new().family(Family::SansSerif),
                cosmic_text::Shaping::Advanced,
                None,
            );
            buffer.set_size(font_system, Some(cfg.width), Some(f32::MAX));
            buffer
        }
        StyledLine::Header { segments, level } => {
            // ? Depending on the Header indentation, we will need to increase the size of our font.
            // ? We store the multipler in a hash-table in our config.
            let scaled_font_size =
                cfg.font_size * cfg.header_scales.get(level).copied().unwrap_or(1.0);

            let mut buffer = Buffer::new(
                font_system,
                Metrics::new(scaled_font_size, scaled_font_size * 1.5),
            );

            let rich_text: Vec<(&str, Attrs<'_>)> = segments
                .iter()
                .map(|block| {
                    let text = block.text.as_str();
                    // ? Headers are Bold by convention
                    // todo add config for this.
                    let attrs = Attrs::new().weight(Weight::BOLD).style(block.style);

                    (text, attrs)
                })
                .collect();

            buffer.set_rich_text(
                font_system,
                rich_text,
                &Attrs::new().family(Family::SansSerif),
                cosmic_text::Shaping::Advanced,
                None,
            );

            buffer.set_size(font_system, Some(cfg.width), Some(f32::MAX));
            buffer
        }
        // catch ourselves
        _ => panic!("StyledLine type: {:#?} not implemented!", line),
    }
}
