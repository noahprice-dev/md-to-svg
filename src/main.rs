use clap::Parser;
use config::Config;
use cosmic_text::FontSystem;
use md_to_svg::cli::Cli;
use md_to_svg::config::{SvgConfig, load_preset_config};
use md_to_svg::pipeline;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // * Get default values or Svg Export
    let default_config = SvgConfig::default();

    // Derive arguments from Parser
    let cli = Cli::parse();

    let _cfg_preset = &cli
        .preset
        .clone()
        .map(|path| load_preset_config(path))
        .transpose()?;

    let settings = Config::builder()
    .add_source(config::Config::try_from(&default_config)?)
    .add_source(config::Config::try_from(&_cfg_preset)?)
    .add_source(config::Config::try_from(&cli)?)
    .build()?;

    let svg_cfg: SvgConfig = settings.try_deserialize().unwrap();
    let mut font_system = FontSystem::new(); // * Detect system fonts and load

    println!("{:#?}", svg_cfg);

    let text_tags =
        pipeline::process_md_to_svg(&Path::new(&cli.input_path), &mut font_system, &svg_cfg)?;

    pipeline::write_svg_to_file(&Path::new(&cli.output_path), text_tags)?;
    Ok(())
}
