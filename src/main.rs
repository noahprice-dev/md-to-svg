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
    let cfg_preset = &cli
        .preset_path
        .clone()
        .map(|path| load_preset_config(path))
        .transpose()?;

    let mut builder = Config::builder().add_source(
        config::Config::try_from(&default_config)
            .expect("BUG: default config should always serialize"),
    );

    if let Some(preset) = cfg_preset {
        builder = builder.add_source(config::Config::try_from(preset).expect(
            "BUG: PresetConfig should always be serializable if load_preset_config succeeded.",
        ));
    }

    let settings = builder
        .add_source(config::Config::try_from(&cli.overrides).expect("BUG: CLI Overrides should always serialize"))
        .build()?;
    let svg_cfg: SvgConfig = settings.try_deserialize().unwrap();

    let mut font_system = FontSystem::new(); // * Detect system fonts and load

    let text_tags =
        pipeline::process_md_to_svg(&Path::new(&cli.input_path), &mut font_system, &svg_cfg)?;

    pipeline::write_svg_to_file(&Path::new(&cli.output_path), text_tags)?;
    Ok(())
}
