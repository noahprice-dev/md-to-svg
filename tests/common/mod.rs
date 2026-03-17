use std::path::PathBuf;

use cosmic_text::{
    Family, FontSystem, Style, Weight,
    fontdb::{Database, Query},
};

/// Create a simple FontSystem with default Sans-Serif font derived from tests/fonts.
pub fn create_default_test_font_system() -> FontSystem {
    // Create a default, empty FontDB
    let mut db = Database::new();
    // * Load Noto Sans from tests/fonts/
    // ? We load individual static font files because the `dir` level loader fails quietly.
    db.load_font_file(get_fixture_path(&["tests", "fonts", "NotoSans-Bold.ttf"]))
        .expect("Bold test font should load successfully.");
    db.load_font_file(get_fixture_path(&["tests", "fonts", "NotoSans-BoldItalic.ttf"]))
        .expect("BoldItalic test font should load successfully.");
    db.load_font_file(get_fixture_path(&["tests", "fonts", "NotoSans-Italic.ttf"]))
        .expect("Italic test font should load successfully.");
    db.load_font_file(get_fixture_path(&["tests", "fonts", "NotoSans-Regular.ttf"]))
        .expect("Regular test font should load successfully.");

    // override default Sans-Serif on db.
    db.set_sans_serif_family("Noto Sans");
    
    let validate_noto = db.query(&Query {
        families: &[Family::Name("Noto Sans")],
        weight: Weight::NORMAL,
        stretch: cosmic_text::Stretch::Normal,
        style: Style::Normal,
    });
    
    let validate_serif = db.query(&Query {
        families: &[Family::SansSerif],
        weight: Weight::NORMAL,
        stretch: cosmic_text::Stretch::Normal,
        style: Style::Normal,
    });
    
    // * Validate that NotoSans has actually been set as the SansSerif type.
    assert_eq!(validate_noto, validate_serif);
    
    let font_sys = FontSystem::new_with_locale_and_db("en-US".to_string(), db);
    font_sys
}

pub fn normalize_svg_for_comparison(text: &str) -> String {
    let doc = roxmltree::Document::parse(&text).unwrap();
    
    let mut tspan_contents = vec![];
    for node in doc.descendants(){
        if node.tag_name().name() == "tspan" {
            tspan_contents.push(node.text().expect("TSpan should not be empty."));
        }
    }
    tspan_contents.concat()
}

pub fn get_fixture_path(sub_paths: &[&str]) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    
    path.extend(sub_paths);
    path
}