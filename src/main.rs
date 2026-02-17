use cosmic_text::Attrs;
use cosmic_text::BorrowedWithFontSystem;
use cosmic_text::Buffer;
use cosmic_text::FontSystem;
use cosmic_text::Metrics;
use cosmic_text::Shaping;
use markdown::mdast::Node;
use markdown::{ParseOptions};
use md_to_svg::parser::parse_blocks;
use std::fs::File;
use std::fs::{self};
use std::io::{BufWriter, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let raw_md = fs::read_to_string("./test.md").unwrap();
    // Detect system fonts
    let mut font_system = FontSystem::new();
    // Define font size & line height of our buffer.
    let metrics = Metrics::new(14.0, 20.0);

    // Instantiate our Buffer. We will perform shaping & layout for our strings.
    let mut buffer = Buffer::new(&mut font_system, metrics);
    // Add our font_system to the Buffer thru borrowing "for convvenient method calls"
    let mut buffer = buffer.borrow_with(&mut font_system);

    // ? Note that this defines the allowable space to write into. If we are unable to fit the full string, it will do as much as possible then stop.
    buffer.set_size(Some(600.0), None);


    // Attributes handle the styling and font family, among other things. Defaults to a sans-serif
    let attrs = Attrs::new();
    let opts = ParseOptions::default();
    let full_md_ast = markdown::to_mdast(&raw_md, &opts).unwrap();
    write_lines_to_file(vec![full_md_ast.clone()]);

    if let Some(root_child) = full_md_ast.children() {
        for child in root_child {
            println!("Child: {:#?}",parse_blocks(child, 0));
        }
}
    //parse_markdown("./test.md");
    Ok(())
}

// Pass the buffer with Size into this function alongside each Line from the Markdown.
// On the line perform markdown::to_ast
// Return the Node tree.
fn wrap_line(
    line: &str,
    buffer: &mut BorrowedWithFontSystem<'_, Buffer>,
    attrs: &Attrs<'_>,
) -> Vec<String> {
    // Update our buffer with the Line string.
    buffer.set_text(line, attrs, Shaping::Advanced, None);
    buffer.shape_until_scroll(false); // Do not limit rendered text to fit Canvas height.

    let mut wrapped_lines: Vec<String> = Vec::new();

    // Retrieve the Layed out Lines
    for run in buffer.layout_runs() {
        // Get the first and last glyph of our entire run.
        let first_glyph = run.glyphs.first().map(|g| g.start).unwrap_or(0);
        let last_glyph = run.glyphs.last().map(|g| g.end).unwrap_or(0);

        // Get the line for this run. Note that if we do not have an inherent linebreak (\n) we always have 0.
        if let Some(buffer_line) = buffer.lines.get(run.line_i) {
            // Get the full input string.
            let full_paragraph_text = buffer_line.text();
            let visual_line_string = &full_paragraph_text[first_glyph..last_glyph];
            wrapped_lines.push(visual_line_string.to_string())
        }
    }

    wrapped_lines
}

fn write_lines_to_file(asts: Vec<Node>) {
    // get our file - note this will overwrite the contents each time
    // currently it is desirable to do this. just be aware.
    let fp = "./test_buffer_out.txt";
    let file = File::create(fp).expect("Should be able to open file for writing.");

    let mut writer = BufWriter::new(file);

    for line in asts {
        writeln!(writer, "{:#?}", line).unwrap();
    }

    writer.flush().unwrap();
}