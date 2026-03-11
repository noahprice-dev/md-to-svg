use clap::Parser;
use config::{Config, File};
use cosmic_text::FontSystem;
use md_to_svg::cli::Cli;
use md_to_svg::pipeline;
use md_to_svg::config::SvgConfig;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Derive arguments from Parser
    let cli = Cli::parse();

    let settings = Config::builder()
    .add_source(File::with_name("md2svg_config"))
    .add_source(config::Config::try_from(&cli).unwrap_or_default())
    .build()
    .unwrap();

    let mut font_system = FontSystem::new(); // * Detect system fonts and load
    
    let svg_cfg: SvgConfig = settings.try_deserialize().unwrap();
    
    let text_tags =
        pipeline::process_md_to_svg(&Path::new(&cli.input_path), &mut font_system, &svg_cfg)?;

    pipeline::write_svg_to_file(&Path::new(&cli.output_path), text_tags)?;
    Ok(())
}
