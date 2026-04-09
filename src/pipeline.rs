use std::{
    collections::HashMap, fs::{self, File}, io::{BufWriter, ErrorKind, Write}, path::Path
};

use cosmic_text::FontSystem;
use markdown::{ParseOptions, mdast::Definition};

use crate::{
    MdToSvgError,
    config::SvgConfig,
    layout::{LayoutItem, process_layouts},
    parser::node_to_styled_block, styles::StyledBlock,
};

pub fn process_md_to_svg(
    input_path: &Path,
    font_system: &mut FontSystem,
    cfg: &SvgConfig,
) -> Result<String, MdToSvgError> {
    // Read a file and handle invalid file path, unreadable file.

    let raw_text = fs::read_to_string(input_path).map_err(|err| match err.kind() {
        ErrorKind::NotFound => MdToSvgError::InputNotFound(input_path.to_path_buf()),
        // ? Generic case, something went wrong in the input, but we haven't specified it in this match statement.
        _ => MdToSvgError::InputUnreadable(input_path.to_path_buf(), err),
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
    
    
    let styled_blocks: Vec<crate::styles::StyledBlock> = node_to_styled_block(&root, 0);
    
    let definitions: HashMap<String, Definition> = styled_blocks
    .iter()
    .filter_map(|block| match block {
        StyledBlock::Definition(def) => Some((def.identifier.clone(), def.clone())),
        _ => None,
    })
    .collect();

    
    let layout_items = styled_blocks
        .into_iter()
        .filter_map(|block| match block {
            // Handle non-textual structural items..
            StyledBlock::Definition(_) => None,
            StyledBlock::ThematicBreak => Some(LayoutItem::ThematicBreak {
                left: cfg.canvas_opts.padding.left,
                right: cfg.canvas_opts.width - cfg.canvas_opts.padding.right,
            }),
            _ => Some(LayoutItem::Block(block.into_layout_block(cfg, font_system, &definitions)))})
            .collect();

    let text_tags = process_layouts(layout_items, cfg);

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
        width = cfg.canvas_opts.width,
        height = cfg.canvas_opts.height,
        bg = cfg.canvas_opts.bg_color,
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
        _ => MdToSvgError::OutputNotWritable(output_path.to_path_buf(), err),
    })?;

    let mut writer = BufWriter::new(outfile);

    writeln!(writer, "{}", svg_data).unwrap();

    // ? Do we need a more robust error check on the flush here?
    writer
        .flush()
        .map_err(|err| MdToSvgError::OutputNotWritable(output_path.to_path_buf(), err))?;

    Ok(())
}
