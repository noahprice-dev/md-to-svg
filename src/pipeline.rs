use cosmic_text::FontSystem;
use derive_builder::Builder;
use markdown::ParseOptions;
use std::{
    collections::HashMap,
    fs::{self, File},
    io::{BufWriter, ErrorKind, Write},
    path::Path,
};

use crate::{
    MdToSvgError,
    layout::{LayoutResult, process_layouts, styled_line_to_layout},
    parser::parse_blocks,
};

pub fn process_md_to_svg(
    input_path: &Path,
    font_system: &mut FontSystem,
    cfg: &SvgConfig,
) -> Result<String, MdToSvgError> {
    // Read a file and handle invalid file path, unreadable file.

    let raw_text = fs::read_to_string(input_path).map_err(|err| match err.kind() {
        ErrorKind::NotFound => {
            return MdToSvgError::InputNotFound(input_path.to_path_buf());
        }
        // ? Generic case, something went wrong in the input, but we haven't specified it in this match statement.
        _ => return MdToSvgError::InputUnreadable(err),
    })?;

    let md_text = raw_text
        .replace("\r\n", "\n")
        .replace("<em>", "*")
        .replace("</em>", "*")
        .replace("<strong>", "**")
        .replace("</strong>", "**");

    // * Note - per the `markdown` documentation, this cannot fail using standard parse options.
    // * It should only fail if JSX/MDX is enabled, AND that parsing fails.
    let root = markdown::to_mdast(&md_text, &ParseOptions::default())?;

    let styled_lines = parse_blocks(&root, 0);

    println!("Styled Lines: {:#?}", styled_lines);
    let layout_lines = styled_lines
        .into_iter() //? note into_iter consumes the original!
        .map(|styled_line| styled_line_to_layout(styled_line, font_system, cfg))
        .collect::<Vec<LayoutResult>>();

    let text_tags = process_layouts(layout_lines, cfg);

    let svg_file = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
        <svg
        viewBox="0 0 {width} {height}"
        width="{width}"
        height="{height}"
        version="1.1"
        xmlns="http://www.w3.org/2000/svg">
            <rect width="{width}" height="{height}" fill="{bg}"/>
            {text_tags}
            </svg>"#,
        width = cfg.width,
        height = cfg.height,
        bg = cfg.bg_color,
        text_tags = text_tags.join("\n")
    );

    Ok(svg_file)
}

pub fn write_svg_to_file(
    output_path: &std::path::Path,
    svg_data: String,
) -> Result<(), MdToSvgError> {
    let outfile = File::create(output_path).map_err(|err| match err.kind() {
        ErrorKind::NotFound => MdToSvgError::OutputNotFound(output_path.to_path_buf()),
        _ => MdToSvgError::OutputNotWritable(err),
    })?;

    let mut writer = BufWriter::new(outfile);

    writeln!(writer, "{}", svg_data).unwrap();

    // ? Do we need a more robust error check on the flush here?
    writer
        .flush()
        .map_err(|err| MdToSvgError::OutputNotWritable(err))?;

    Ok(())
}

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
