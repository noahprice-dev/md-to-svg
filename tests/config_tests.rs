use std::path::PathBuf;

use md_to_svg::{
    MdToSvgError,
    config::{
        CanvasConfig, HeaderConfig, HeaderScales, Padding, SvgConfig, TypographyConfig,
        load_preset_config,
    },
};

use crate::common::get_normalized_fixture_path;
pub mod common;


#[test]
fn partial_preset_overrides_specified_fields_and_preserves_defaults() {
    let preset_cfg = load_preset_config(get_normalized_fixture_path(&["tests", "fixtures", "only_padding.toml"])).unwrap();
    let combined_settings = config::Config::builder()
        .add_source(
            config::Config::try_from(&SvgConfig::default())
                .expect("Should be able to load default SvgConfig"),
        )
        .add_source(
            config::Config::try_from(&preset_cfg)
                .expect("Should be able to load incomplete PresetConfig"),
        )
        .build()
        .unwrap();

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

#[test]
fn load_preset_config_returns_config_not_found_for_missing_file() {
    let invalid_path: PathBuf = "invalid/path".into();
    let preset_cfg = load_preset_config(invalid_path);

    assert!(matches!(preset_cfg, Err(MdToSvgError::ConfigNotFound(_))))
}

#[test]
fn load_preset_config_returns_parse_failed_for_invalid_toml() {
    let invalid_path: PathBuf = get_normalized_fixture_path(&["tests", "fixtures","invalid_cfg.toml"]);
    let preset_cfg = load_preset_config(invalid_path);
    assert!(matches!(
        preset_cfg,
        Err(MdToSvgError::ConfigParseFailed(_))
    ))
}

#[test]
fn complete_preset_overrides_all_fields() {
    let preset_cfg = load_preset_config(get_normalized_fixture_path(&["tests", "fixtures","complete_override.toml"])).unwrap();
    let combined_settings = config::Config::builder()
        .add_source(
            config::Config::try_from(&SvgConfig::default())
                .expect("Should be able to load default SvgConfig"),
        )
        .add_source(
            config::Config::try_from(&preset_cfg)
                .expect("Should be able to load complete PresetConfig"),
        )
        .build()
        .unwrap();
    let result: SvgConfig = combined_settings.try_deserialize().unwrap();

    let expected = SvgConfig {
        canvas_opts: CanvasConfig {
            width: 1200.0,
            height: 900.0,
            bg_color: "#02acac".to_string(),
            padding: Padding {
                top: 0.0,
                right: 10.0,
                bottom: 20.0,
                left: 30.0,
            },
        },
        text_opts: TypographyConfig {
            font_size: 36.,
            line_height_factor: 2.,
            paragraph_spacing_em: 1.2,
            bullet_indent_em: 2.,
            bullet_char: String::from(">"),
        },
        header_opts: HeaderConfig {
            header_scales: HeaderScales {
                h1: 3.0,
                h2: 2.5,
                h3: 2.0,
                h4: 1.5,
                h5: 1.2,
                h6: 0.8,
            },
            header_margin_top: 50.,
            header_margin_bot: 50.,
        },
    };
    
    pretty_assertions::assert_eq!(result, expected);
}

#[test]
fn preset_padding_parses_alll_css_shorthand_forms() {
    
}