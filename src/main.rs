use cosmic_text::FontSystem;
use markdown::ParseOptions;
use md_to_svg::layout::SvgConfig;
use md_to_svg::layout::process_layouts;
use md_to_svg::layout::styled_line_to_layout;
use md_to_svg::parser::parse_blocks;
use std::fs::File;
use std::fs::{self};
use std::io::{BufWriter, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let svg_cfg = SvgConfig::default();

    // * Detect system fonts
    let mut font_system = FontSystem::new();

    // * Read in our Markdown.
    let md_text = fs::read_to_string("./test.md").unwrap()
     // * Normalize Line Endings and convert inline HTML into Markdown syntax.
        .replace("\r\n", "\n")
        .replace("<em>", "*")
        .replace("</em>", "*")
        .replace("<strong>", "**")
        .replace("</strong>", "**");
    
    

    // * ParseOptions modifies how to parse different flavours of markdown, and which flavours to support.
    // todo Right now we just handle vanilla. We could expand into GFM later.
    let opts = ParseOptions::default();
    let full_md_ast = markdown::to_mdast(&md_text, &opts).unwrap();

    let mut svg_lines: Vec<String> = Vec::new();
    let mut layout_lines = Vec::new();
    // * Handle processing
    
    // todo Refactor to a function
    // todo fix height being updated iteratively.
    if let Some(root_child) = full_md_ast.children() {
        //println!("{:#?}", full_md_ast);
        for child in root_child {
            for style_line in parse_blocks(child, 0) {
                layout_lines.push(styled_line_to_layout(
                    style_line,
                    &mut font_system,
                    &svg_cfg,
                ));
            }
        }
    }
    
    //let mut buffer = Buffer::new(&mut font_system, Metrics::new(24., 24. * 1.5));
    //buffer.set_text(&mut font_system, "This is a long paragraph that\nwraps across multiple lines.", &Attrs::new(), cosmic_text::Shaping::Advanced, None);
    //buffer.set_size(&mut font_system, Some(f32::MAX), Some(f32::MAX));
    //
    //buffer.lines.iter().for_each(|line| println!("Buffer line:{}", line.text()));
    
    svg_lines.extend(process_layouts(layout_lines, &svg_cfg));
    write_svg_to_file("./outputs/test.svg", svg_lines, &svg_cfg);

    Ok(())
}

fn write_svg_to_file(path: &str, lines: Vec<String>, cfg: &SvgConfig) {
    //let fp = "./output.svg";
    let outfile = File::create(path).expect(&format!(
        "Should be able to open or create a file at {}",
        path
    ));

    let mut writer = BufWriter::new(outfile);
    let prelude = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
    <svg
    viewBox="0 0 {width:?} {height:?}"
    width="{width:?}"
    height="{height:?}"
    version="1.1"
    xmlns="http://www.w3.org/2000/svg">
    <rect width="{width:?}" height="{height:?}" fill="{bg:}"/>"#,
        width = cfg.width,
        height = cfg.height,
        bg = cfg.bg_color
    );

    let finish = r#"</svg>"#;
    writeln!(writer, "{}", prelude).expect(&format!("Should be able to write to file at {}", path));
    for line in lines {
        writeln!(writer, "{}", line).expect(&format!(
            "Should be able to write SVG Line to file at {}",
            path
        ));
    }
    writeln!(writer, "{}", finish).unwrap();

    writer.flush().expect("Should be able to flush BufWriter");
}
