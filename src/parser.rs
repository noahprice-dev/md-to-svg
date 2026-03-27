use markdown::mdast::Node;

use crate::styles::{StyledBlock, StyledInline};

fn node_to_styled_inline(node: &Node) -> Vec<StyledInline> {
    match node {
        Node::Strong(strong) => {
            let mut blocks = Vec::new();
            // My understanding is that we take any number of children in this,
            // then recurse through the children updating the 'local' context, i.e. the context that has accumulated at the branch until that point
            // then return the final list.
            for child in &strong.children {
                blocks.push(StyledInline::Strong(node_to_styled_inline(child)))
            }
            blocks
        }
        Node::Emphasis(emph) => {
            // We are creating a new vec each time.... how do we collapse them?
            let mut blocks = Vec::new();

            for child in &emph.children {
                blocks.push(StyledInline::Emphasis(node_to_styled_inline(child)))
            }
            blocks
        }
        Node::Text(text) => {
            // * Cosmic will treat "\n" as a new line directive, which means "soft wraps",
            // * such as when a user has created a new line for clarity in the MD, will be seen as a Hard Break.
            // * To ensure we only handle expicit Line Breaks, we replace with a space character to continue the line gracefully,
            // * while not breaking up sentences that were formatted a certain way for the MD editor.
            // ? To note - CommonMark allows both implicit soft breaks with a `newline` character such as what we are removing here,
            // ? As well as the markdown typical "space space" ending a line.
            // ? Per spec 0.31.2:
            // ? A conforming parser may render a soft line break in HTML either as a line ending or as a space.
            // ? A renderer may also provide an option to render soft line breaks as hard line breaks.
            // ?
            // ? Therefore, we probably need to handle some cfg directive for this for perfect conformity. I am willing to remain opinionated for now.
            let normalized = text.value.trim_matches('\n').replace("\n", " ").to_string();

            // This is a leaf node. We can collapse into a new StyledBlock.
            vec![StyledInline::Text(normalized)]
        }
        Node::Html(html) => {
            let mut blocks = Vec::new();
            if html.value.to_ascii_lowercase() == "<br>" || html.value.to_lowercase() == "<br/>" {
                blocks.push(StyledInline::HardBreak);
            } else {
                // * Render the HTML inner content as-is, but warn the user.
                eprintln!(
                    "Warning: Unsupported HTML Tag {}. Rendering as plain text...",
                    html.value.clone()
                );
                blocks.push(StyledInline::Text(html.value.clone()));
            }
            blocks
        }
        Node::InlineCode(code) => {
            let normalized = code.value.trim_matches('\n').replace("\n", " ").to_string();

            // This is a leaf node. We can collapse into a new StyledBlock.
            vec![StyledInline::InlineCode(normalized)]
        }
        Node::Link(link) => todo!(),
        Node::Break(_) => {
            vec![StyledInline::HardBreak]
        }
        unknown => {
            //? Note that we cannot extract the Node variant name simply, so i am ignoring it for now.
            eprintln!(
                "Warning: Inline style for type {:?} not supported. Rendering as plain text... ",
                unknown
            );
            // ? Nodes do not have any single, common typing that I can read in. As well, I have lost the original markdown after parsing, so its impossible to just copy directly.
            vec![StyledInline::Text(format!("{:?}", unknown))]
        }
    }
}

// TODO update to properly pass node children down instead of node direct ala Heading, Paragraph
// TODO Investigate Header style incorrect - size & bold not present
pub fn node_to_styled_block(node: &Node, indent: u8) -> Vec<StyledBlock> {
    match node {
        Node::Root(root) => {
            let mut lines = Vec::new();
            for child in &root.children {
                lines.extend(node_to_styled_block(child, 0));
            }
            lines
        }
        Node::Heading(heading) => {
            let segments = heading
                .children
                .iter()
                .flat_map(|child| node_to_styled_inline(child))
                .collect();
            vec![StyledBlock::Header {
                segments,
                level: heading.depth,
            }]
        }
        Node::Paragraph(paragraph) => {
            let segments = paragraph
                .children
                .iter()
                .flat_map(|child| node_to_styled_inline(child))
                .collect();
            vec![StyledBlock::Paragraph { segments: segments }]
        }
        Node::Html(_) => {
            // * Html styling is handled at an inline-level and returned as a Text object.
            // * Aside from <br> we don't currently support ANY HTML.
            // * Therefore, we can render any non break HTML as a paragraph, alongside our warning.
            // * This is subject to change, and will require more advanced filtering at this later.
            // todo support HTML tables. This should be part of the table feature impl in a future version.
            let segments = node_to_styled_inline(node);
            vec![StyledBlock::Paragraph { segments: segments }]
        }
        Node::List(list) => {
            let mut lines = Vec::new();
            let mut counter = list.start.unwrap_or(1);

            for child in &list.children {
                // ? Node::List can only contain Node::ListItem
                if let Node::ListItem(item) = child {
                    for item_child in &item.children {
                        // * Check if this list item contains another container block (List, Blockquote)
                        // * or a Leaf Container (Paragraph, Heading, etc)
                        match item_child {
                            // * Check if we have a nested list
                            Node::List(_) => {
                                // Recurse and handle the new list's contents
                                lines.extend(node_to_styled_block(item_child, indent + 1))
                            }
                            Node::Blockquote(_) => {
                                todo!()
                            }
                            Node::Paragraph(paragraph) => {
                                // * Parse inline styles
                                let segments = paragraph
                                    .children
                                    .iter()
                                    .flat_map(|child| node_to_styled_inline(child))
                                    .collect();

                                // * Handle Bullet List vs Numbered List
                                if list.ordered {
                                    lines.push(StyledBlock::NumberedListItem {
                                        segments,
                                        number: counter,
                                        indent,
                                    });
                                    counter += 1; // increase for next list item.
                                } else {
                                    lines.push(StyledBlock::BulletListItem { segments, indent });
                                }
                            }
                            _ => todo!(),
                        }
                    }
                }
            }
            lines
        }
        Node::Blockquote(blockquote) => todo!(),
        Node::ThematicBreak(_) => {
            vec![StyledBlock::ThematicBreak]
        }
        Node::Image(img) => todo!(),

        unknown => {
            eprintln!(
                "Warning: Unsupported node type: {:#?}. Rendering as plain text..",
                unknown
            );

            let unknown_node_segments = node_to_styled_inline(unknown);

            vec![StyledBlock::Paragraph {
                segments: unknown_node_segments,
            }]
        }
    }
}
