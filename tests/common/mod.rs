use std::path::Path;

use cosmic_text::{FontSystem, fontdb::Database};

/// Create a simple FontSystem with default Sans-Serif font derived from tests/fonts.
pub fn create_default_test_font_system() -> FontSystem {
    // Create a default, empty FontDB
    let mut db = Database::new();
    // * Load Noto Sans from tests/fonts/
    // ? We load all fonts in this directory instead of loading the individual font variants (Italic, Bold etc)
    // ? Our default case is to access all of these, rather than loading specific fonts for each test.
    // ? we could break this out to accept a Weight/Style variant struct as an arg, and match accordingly later on if we need.
    db.load_fonts_dir(Path::new("tests/fonts/"));
    
    // override default Sans-Serif on db.
    db.set_sans_serif_family("Noto Sans"); // ! I don't know how to validate this...
    
    let font_sys = FontSystem::new_with_locale_and_db("en-US".to_string(), db);
    font_sys
}