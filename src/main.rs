use cosmic_text::Attrs;
use cosmic_text::BorrowedWithFontSystem;
use cosmic_text::Buffer;
use cosmic_text::FontSystem;
use cosmic_text::Metrics;
use cosmic_text::Shaping;
use markdown::mdast::Node;
use markdown::{ParseOptions};
use std::fs::{self};
use std::{path::Path};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read in a Markdown file.
    parse_markdown("./test.md");
    Ok(())
}

fn parse_markdown<P>(filename: P)
where
    P: AsRef<Path>,
{
    //let file = File::open(filename).unwrap();
    let raw_md = fs::read_to_string(filename).unwrap(); // Understand the free-ing that is done here when we do .lines()
    let raw_md = raw_md.lines();
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

    // Break MD lines into Vec<string> where each String is a Line-wrapped Line.
    let mut wrappedline_groups: Vec<Vec<String>> = Vec::new();
    for line in raw_md {
        wrappedline_groups.push(wrap_line(line, &mut buffer, &attrs));
    }

    // for (i, l) in wrappedline_groups.iter().enumerate() {
    //     println!("Line {:#?}: |{:#?}|", i, l);
    // }

    // Process each Line-Wrapped Line as a group into an AST for each line.
    // This ensures line-level styling (i.e. List) is preserved at the root, while maintaining our visual line-breaks.
    let mut asts: Vec<Node> = Vec::new();

    for group in wrappedline_groups {
        asts.push(convert_line_group_to_styledblock(group, &opts));
    }

    //println!("Line {:#?}: |{:#?}|", 0, asts[0]);
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

//todo Process each line in the line_group, take the first index' Root's immediate child and parse the Type of the child.
//todo this will represent the entire Line Type.
/// For each line Group that we pass, convert the whole group into a collection of StyledBlocks.
/// ? How are we going to pass the position from Cosmic?
fn convert_line_group_to_styledblock(line_group: Vec<String>, opts: &ParseOptions) -> Node {
    let ast = markdown::to_mdast(&line_group[0], opts)
        .expect("Should be able to parse regular markdown without erroring.");

    println!("AST Node: {:?}", ast);
    println!("Line Text: {:?}", line_group[0]);

    ast
}
