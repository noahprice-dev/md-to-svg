use markdown::mdast::Node;
use regex::Regex;

use crate::styles::{StyleContext, StyledBlock, StyledLine, StyledSegment};

fn parse_inline_styles(node: &Node, mut context: StyleContext) -> Vec<StyledSegment> {
    match node {
        Node::Strong(strong) => {
            context.bold = true;
            let mut blocks = Vec::new();
            // My understanding is that we take any number of children in this,
            // then recurse through the children updating the 'local' context, i.e. the context that has accumulated at the branch until that point
            // then return the final list.
            for child in &strong.children {
                blocks.extend(parse_inline_styles(child, context.clone()));
            }
            blocks
        }

        Node::Emphasis(emph) => {
            context.italic = true;
            // We are creating a new vec each time.... how do we collapse them?
            let mut blocks = Vec::new();

            for child in &emph.children {
                blocks.extend(parse_inline_styles(child, context.clone()));
            }
            blocks
        }

        Node::Text(text) => {
            // * Cosmic will treat "\n" as a new line directive, which means that "soft wraps",
            // * such as when the user enters a
            let normalized = text.value.trim_matches('\n').replace("\n", " ").to_string();

            // This is a final node. We can collapse into a new StyledBlock.
            vec![StyledSegment::Text(StyledBlock::new(normalized, context))]
        }

        Node::Paragraph(para) => {
            let mut blocks = Vec::new();

            // Pass through to handle styling..
            for child in &para.children {
                blocks.extend(parse_inline_styles(child, context));
            }
            blocks
        }

        Node::Heading(heading) => {
            let mut blocks = Vec::new();
            // Pass through to handle styling..
            context.bold = true;
            for child in &heading.children {
                blocks.extend(parse_inline_styles(child, context));
            }
            blocks
        }
        Node::Html(html) => {
            let mut blocks = Vec::new();
            if html.value.to_ascii_lowercase() == "<br>" || html.value.to_lowercase() == "<br/>" {
                blocks.extend(vec![StyledSegment::HardBreak]);
            } else {
                // investigate foreignObject Tag.
                // Convert HTML to... text? die?
            }
            blocks
        }

        Node::InlineCode(code) => {
            println!("Inline code");
            context.monospace = true;
            let normalized = code.value.trim_matches('\n').replace("\n", " ").to_string();

            // This is a final node. We can collapse into a new StyledBlock.
            vec![StyledSegment::Text(StyledBlock::new(normalized, context))]
        }
        Node::Break(_) => {
            vec![StyledSegment::HardBreak]
        }

        _ => panic!("InlineStyle Type not yet implemented: {:?} ", node),
    }
}

pub fn parse_blocks(node: &Node, indent: u8) -> Vec<StyledLine> {
    match node {
        Node::Root(root) => {
            let mut lines = Vec::new();
            for child in &root.children {
                lines.extend(parse_blocks(child, 0));
            }
            lines
        }
        Node::Heading(heading) => {
            // * Process all inline styling of the Node
            let segments = parse_inline_styles(node, StyleContext::default());
            vec![StyledLine::Header {
                segments,
                level: heading.depth,
            }]
        }
        Node::Paragraph(_) => {
            // * Process all inline styling of the Node
            let segments = parse_inline_styles(node, StyleContext::default());
            vec![StyledLine::Paragraph { segments: segments }]
        }
        Node::Html(html) => match extract_html_tag(&html.value).as_deref() {
            Some("br") => vec![StyledLine::Blank],
            Some(unknown) => {
                eprintln!("Warning: Unsupported HTML Tag <{}> - skipping..", unknown);
                vec![]
            }
            None => vec![],
        },
        Node::List(list) => {
            let mut lines = Vec::new();
            let mut counter = list.start.unwrap_or(1);

            for child in &list.children {
                if let Node::ListItem(item) = child {
                    for item_child in &item.children {
                        match item_child {
                            Node::Paragraph(_) => {
                                // * Parse inline styles
                                let segments =
                                    parse_inline_styles(item_child, StyleContext::default());
                                // * Handle Bullet List vs Numbered List
                                if list.ordered {
                                    lines.push(StyledLine::NumberedListItem {
                                        segments,
                                        number: counter,
                                        indent,
                                    });
                                    counter += 1; // increase for next list item.
                                } else {
                                    lines.push(StyledLine::BulletListItem { segments, indent });
                                }
                            }
                            // * Check if we have a nested list
                            Node::List(_) => {
                                // Recurse and handle the new list's contents
                                lines.extend(parse_blocks(item_child, indent + 1))
                            }

                            _ => todo!(),
                        }
                    }
                }
            }
            lines
        }

        _ => panic!("ParseBlocks has not yet implemented: {:?} ", node),
    }
}

/// Retrieve the first HTML tag in a line to understand if we can parse it or not.\
/// ### Note
/// This is an extremely naive RegEx parse. If we require more complex HTML parsing, turn to another crate before adding more regex.
fn extract_html_tag(tag: &str) -> Option<String> {
    let re = Regex::new(r#"<([a-zA-Z][a-zA-Z0-9]*)"#).unwrap();
    let caps = re.captures(&tag);
    if let Some(tag) = caps {
        Some(tag.get_match().as_str().to_string().to_ascii_lowercase())
    } else {
        None
    }
}
