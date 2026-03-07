use cosmic_text::FontSystem;
use md_to_svg::{layout::SvgConfig, pipeline::write_svg_to_file};
use std::io::{BufWriter, Write};
use md_to_svg::pipeline

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let svg_cfg = SvgConfig::default();

    // * Detect system fonts
    let mut font_system = FontSystem::new();
    
    //todo
    let text_tags = pipeline::process_md_to_svg(input_path, &mut font_system, &svg_cfg)?;
    //todo
    write_svg_to_file(output_path, text_tags);
    Ok(())
}