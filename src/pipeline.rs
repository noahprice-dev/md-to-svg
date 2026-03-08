use std::{
    fs::{self, File},
    io::{BufWriter, ErrorKind, Write},
    path::Path,
};

use cosmic_text::FontSystem;
use markdown::ParseOptions;
use roxmltree::Node;

use crate::{
    MdToSvgError,
    layout::{LayoutResult, SvgConfig, process_layouts, styled_line_to_layout},
    parser::parse_blocks,
    styles::StyledLine,
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

    let styled_lines  = parse_blocks(&root, 0);
    
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
