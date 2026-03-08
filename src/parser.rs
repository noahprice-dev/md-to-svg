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
            // * such as when a user has created a new line for clarity in the MD, will be seen as a Hard Break.
            // * To ensure we only handle expicit Line Breaks, we replace with a space character to continue the line gracefully,
            // * while not breaking up words.
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
                // * Render the HTML as-is.
                eprintln!("Warning: Unsupported HTML Tag {}. Rendering as plain text...", html.value.clone());
                
                blocks.push(StyledSegment::Text(StyledBlock::new(html.value.clone(), context)));
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
        unknown => {
            //? Note that we cannot extract the Node variant name simply, so i am ignoring it for now.
            eprintln!(
                "Warning: Inline style {:?} not supported. Rendering as plain text... ",
                unknown
            );
            // ? Nodes do not have any single, common typing that I can read in. As well, I have lost the original markdown after parsing, so its impossible to just copy directly.
            vec![StyledSegment::Text(StyledBlock::new(
                format!("{:?}", unknown),
                context,
            ))]
        }
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
        Node::Html(_) => {
            // * Html styling is handled at an inline-level and returned as a Text object.
            // * Aside from <br> we don't currently support ANY HTML.
            // * Therefore, we can render any non break HTML as a paragraph, alongside our warning.
            // * This is subject to change, and will require more advanced filtering at the block level later.
            // todo support HTML tables. This should be part of the table feature impl in a future version.
            let segments = parse_inline_styles(node, StyleContext::default());
            vec![StyledLine::Paragraph { segments: segments }]
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

        unknown => {
            eprintln!(
                "Warning: Unsupported node type: {:#?}. Rendering as plain text..",
                unknown
            );
            let unknown_node_segments = parse_inline_styles(unknown, StyleContext::default());
            vec![StyledLine::Paragraph {
                segments: unknown_node_segments,
            }]
        }
    }
}
