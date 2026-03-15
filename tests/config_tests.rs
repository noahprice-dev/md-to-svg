use config::ConfigBuilder;
use md_to_svg::config::{CanvasConfig, Padding, SvgConfig, load_preset_config};

mod common;

#[test]
fn partial_preset_overrides_specified_fields_and_preserves_defaults() {
    let override_cfg = load_preset_config("tests/fixures/only_padding.toml".into()).unwrap();
    let combined_settings = config::Config::builder()
        .add_source(config::Config::try_from(&SvgConfig::default()).expect("Should be able to load default SvgConfig"))
        .add_source(config::Config::try_from(&override_cfg).expect("Should be able to load incomplete PresetConfig"))
        .build().unwrap();
    
    let result: SvgConfig = combined_settings.try_deserialize().unwrap();
    
    let expected = SvgConfig {
        canvas_opts: CanvasConfig {
            padding: Padding::new(10.0, 20.0, 10.0, 20.0),
            ..CanvasConfig::default()
        },
        ..SvgConfig::default()
    };
    
    assert_eq!(result, expected);
}
