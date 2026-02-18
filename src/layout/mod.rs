use core::num;
use std::{char, collections::HashMap, fmt::format};

use cosmic_text::{Attrs, Buffer, Family, FontSystem, Metrics, Weight};

use crate::styles::StyledLine;

pub struct SvgConfig {
    // SVG Canvas Options
    pub width: f32,
    pub height: f32,
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
    pub bullet_indent_em: f32, // default 1.5 or 2.0 ->
    pub bullet_char: char,

    // Header Style Options
    pub header_scales: HashMap<u8, f32>,
    pub header_margin_top: f32,
    pub header_margin_bot: f32,
}

impl SvgConfig {
    pub fn default() -> Self {
        SvgConfig {
            width: 600.0,
            height: 800.0,
            top_padding: 0,
            right_padding: 0,
            bottom_padding: 0,
            left_padding: 0,
            font_size: 16.0,
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
        }
    }
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
        StyledLine::BulletListItem { segments, indent } => {
            // * Arrange our canvas width based on the indentation of the line.
            // * We calculate by multipling the font_size by some em to get a unit indent space
            // * Then we multiply that by the number of necessary indentations.
            // * Finally, subtract that from the overall canvas width, so when we position it later it wraps appropriately.
            let indent_px = *indent as f32 * (cfg.bullet_indent_em * cfg.font_size);
            let available_width = cfg.width - indent_px;

            let mut buffer = Buffer::new(
                font_system,
                Metrics::new(cfg.font_size, cfg.font_size * 1.5),
            );

            let styled_segments: Vec<(String, Attrs)> = segments
                .iter()
                .enumerate()
                .map(|(i, block)| {
                    // We need to insert the bullet character here, however we can't simply use format!().as_str
                    // as the String returned by format!() doesn't live long enough when it leaves the styled_segments scope.
                    // Therefore, we need to have `styled_segments` own the full String and later have `buffer` borrow it for it's final form.
                    let text = if i == 0 {
                        format!("{}  {}", cfg.bullet_char, &block.text)
                    } else {
                        block.text.clone()
                    };

                    // ? Headers are Bold by convention
                    // todo add config for this.
                    let attrs = Attrs::new().weight(Weight::BOLD).style(block.style);

                    (text, attrs)
                })
                .collect();

            // Convert our segments back into &str, Attrs to satisfy Cosmic API
            let rich_text: Vec<(&str, Attrs)> = styled_segments
                .iter()
                .map(|(text, attrs)| (text.as_str(), attrs.clone()))
                .collect();

            buffer.set_rich_text(
                font_system,
                rich_text,
                &Attrs::new().family(Family::SansSerif),
                cosmic_text::Shaping::Advanced,
                None,
            );

            buffer.set_size(font_system, Some(available_width), Some(f32::MAX));
            buffer
        }
        StyledLine::NumberedListItem {
            segments,
            indent,
            number,
        } => {
            // * Arrange our canvas width based on the indentation of the line.
            // * We calculate by multipling the font_size by some em to get a unit indent space
            // * Then we multiply that by the number of necessary indentations.
            // * Finally, subtract that from the overall canvas width, so when we position it later it wraps appropriately.
            let indent_px = *indent as f32 * (cfg.bullet_indent_em * cfg.font_size);
            let available_width = cfg.width - indent_px;

            let mut buffer = Buffer::new(
                font_system,
                Metrics::new(cfg.font_size, cfg.font_size * 1.5),
            );

            let styled_segments: Vec<(String, Attrs)> = segments
                .iter()
                .enumerate()
                .map(|(i, block)| {
                    // We need to insert the bullet character here, however we can't simply use format!().as_str
                    // as the String returned by format!() doesn't live long enough when it leaves the styled_segments scope.
                    // Therefore, we need to have `styled_segments` own the full String and later have `buffer` borrow it for it's final form.
                    let text = if i == 0 {
                        format!("{}. {}", number, &block.text)
                    } else {
                        block.text.clone()
                    };

                    // ? Headers are Bold by convention
                    // todo add config for this.
                    let attrs = Attrs::new().weight(Weight::BOLD).style(block.style);

                    (text, attrs)
                })
                .collect();

            // Convert our segments back into &str, Attrs to satisfy Cosmic API
            let rich_text: Vec<(&str, Attrs)> = styled_segments
                .iter()
                .map(|(text, attrs)| (text.as_str(), attrs.clone()))
                .collect();

            buffer.set_rich_text(
                font_system,
                rich_text,
                &Attrs::new().family(Family::SansSerif),
                cosmic_text::Shaping::Advanced,
                None,
            );

            buffer.set_size(font_system, Some(available_width), Some(f32::MAX));
            buffer
        }
        // catch ourselves
        _ => panic!("StyledLine type: {:#?} not implemented!", line),
    }
}
