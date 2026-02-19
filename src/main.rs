use cosmic_text::FontSystem;
use markdown::ParseOptions;
use md_to_svg::layout::SvgConfig;
use md_to_svg::layout::layout_line_to_svg;
use md_to_svg::layout::styled_line_to_layout;
use md_to_svg::parser::parse_blocks;
use std::fs::File;
use std::fs::{self};
use std::io::{BufWriter, Write};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let svg_cfg = SvgConfig::default();

    // Detect system fonts
    let mut font_system = FontSystem::new();

    // * Read in our Markdown.
    let raw_md = fs::read_to_string("./test.md").unwrap();

    // ? ParseOptions modifies how to parse different flavours of markdown, and which flavours to support.
    // todo Right now we just handle vanilla. We could expand into GFM later.
    let opts = ParseOptions::default();
    let full_md_ast = markdown::to_mdast(&raw_md, &opts).unwrap();

    let mut svg_out: Vec<String> = Vec::new();

    // * Handle processing
    if let Some(root_child) = full_md_ast.children() {
        for child in root_child {
            let styled_lines = parse_blocks(child, 0);
            for line in styled_lines {
                let layout_line = styled_line_to_layout(&line, &mut font_system, &svg_cfg);
                svg_out.extend(layout_line_to_svg(layout_line, &svg_cfg));
            }
        }
    }

    let fp = "./output.svg";
    let outfile = File::create(Path::new("./output.svg")).expect(&format!(
        "Should be able to open or create a file at {}",
        fp
    ));

    let mut writer = BufWriter::new(outfile);
    for line in svg_out {
        match writeln!(writer, "{}", line) {
            Ok(_) => continue,
            Err(e) => panic!("Problem writing to {fp:?}: {e:?}")
        }
    }
    writer.flush().expect("Should be able to flush BufWriter");


    Ok(())
}