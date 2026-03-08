use clap::Parser;
use cosmic_text::FontSystem;
use md_to_svg::cli::Cli;
use md_to_svg::pipeline;
use md_to_svg::{layout::SvgConfig, pipeline::write_svg_to_file};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Derive arguments from Parser
    let cli = Cli::parse();

    // * -- Set defaults --
    let svg_cfg = SvgConfig::default();
    let mut font_system = FontSystem::new(); // * Detect system fonts and load

    let text_tags =
        pipeline::process_md_to_svg(&Path::new(&cli.input_path), &mut font_system, &svg_cfg)?;

    write_svg_to_file(&Path::new(&cli.output_path), text_tags)?;
    Ok(())
}
