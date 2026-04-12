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
        // Inline Link:
        // i.e. `[Example](www.example.com)`
        Node::Link(link) => {
            // ? link text can be styled
            let link_text = link
                .children
                .iter()
                .flat_map(|child| node_to_styled_inline(child))
                .collect();

            vec![StyledInline::InlineLink {
                text: link_text,
                url: link.url.clone(),
                title: link.title.clone(),
            }]
        }
        Node::LinkReference(link_ref) =>
        // ? link text can be styled
        {
            let link_text = link_ref
                .children
                .iter()
                .flat_map(|child| node_to_styled_inline(child))
                .collect();

            // ? We need to reconnect this to a Definition block at the upper layer when we move into a Layout.
            // ? I dislike including the type Definition as a Node layer right now,
            // ? though perhaps it can be treated at the pipeline level similarly to how we handle StyledBlock::ThematicBreak.
            // ? We would need to process and store links and definitions separately from the Block type.
            // ? Tricky to tie them back, but certainly possible. I would like to avoid certain
            // ? side-effects like mutable objects outside of the scope - that feels nasty.
            vec![StyledInline::LinkReference {
                text: link_text,
                identifier: link_ref.identifier.clone(),
            }]
        }
        Node::Image(_img) => todo!(),
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
        Node::Blockquote(_blockquote) => todo!(),
        Node::ThematicBreak(_) => {
            vec![StyledBlock::ThematicBreak]
        }
        Node::Definition(def) => vec![StyledBlock::Definition(def.clone())], // ? Does this need a custom type to ignore data?
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

#[cfg(test)]
mod tests {
    use markdown::ParseOptions;
    use pretty_assertions::assert_eq;
    use crate::{
        parser::node_to_styled_block,
        styles::{StyledBlock, StyledInline},
    };

    #[test]
    fn parse_inline_link_produces_styled_inline_inline_link() {
        let text = "[Example](www.example.com)";

        let root = markdown::to_mdast(text, &ParseOptions::default())
            .expect("Should be able to parse &str as Markdown");

        let result = node_to_styled_block(&root, 0);

        assert_eq!(
            result,
            vec![StyledBlock::Paragraph {
                segments: vec![StyledInline::InlineLink {
                    text: vec![StyledInline::Text("Example".to_string())],
                    url: "www.example.com".to_string(),
                    title: None
                }]
            }]
        )
    }

    #[test]
    fn parse_inline_link_preserves_url_and_title() {
        let text = r#"[Example](www.example.com "My Title")"#;

        let root = markdown::to_mdast(text, &ParseOptions::default())
            .expect("Should be able to parse &str as Markdown");

        let result = node_to_styled_block(&root, 0);

        assert_eq!(
            result,
            vec![StyledBlock::Paragraph {
                segments: vec![StyledInline::InlineLink {
                    text: vec![StyledInline::Text("Example".to_string())],
                    url: "www.example.com".to_string(),
                    title: Some("My Title".to_string())
                }]
            }]
        )
    }

    #[test]
    fn parse_inline_link_without_url_preserves_empty_url() {
        let text = r#"[Example]()"#;

        let root = markdown::to_mdast(text, &ParseOptions::default())
            .expect("Should be able to parse &str as Markdown");

        let result = node_to_styled_block(&root, 0);

        assert_eq!(
            result,
            vec![StyledBlock::Paragraph {
                segments: vec![StyledInline::InlineLink {
                    text: vec![StyledInline::Text("Example".to_string())],
                    url: "".to_string(),
                    title: None,
                }]
            }]
        )
    }

    #[test]
    fn parse_inline_link_without_url_with_title_preserves_empty_url_maintains_title() {
        let text = r#"[Example](<> "Title")"#;

        let root = markdown::to_mdast(text, &ParseOptions::default())
            .expect("Should be able to parse &str as Markdown");

        let result = node_to_styled_block(&root, 0);

        assert_eq!(
            result,
            vec![StyledBlock::Paragraph {
                segments: vec![StyledInline::InlineLink {
                    text: vec![StyledInline::Text("Example".to_string())],
                    url: "".to_string(),
                    title: Some("Title".to_string()),
                }]
            }]
        )
    }

    #[test]
    fn parse_inline_link_with_styled_text_preserves_child_structure() {
        let text = r#"[*Example*](www.example.com)"#;

        let root = markdown::to_mdast(text, &ParseOptions::default())
            .expect("Should be able to parse &str as Markdown");

        let result = node_to_styled_block(&root, 0);

        assert_eq!(
            result,
            vec![StyledBlock::Paragraph {
                segments: vec![StyledInline::InlineLink {
                    text: vec![StyledInline::Emphasis(vec![StyledInline::Text(
                        "Example".to_string()
                    )])],
                    url: "www.example.com".to_string(),
                    title: None,
                }]
            }]
        )
    }

    // #[test]
    // fn parse_reference_link_produces_styled_inline_link_reference() {
    //     todo!()
    // }
    
    #[test]
    fn parse_empty_shortcut_reference_renders_as_plain_text() {
        let text = r#"[foo][]"#;

        let root = markdown::to_mdast(text, &ParseOptions::default())
            .expect("Should be able to parse &str as Markdown");

        let result = node_to_styled_block(&root, 0);

        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0],
            StyledBlock::Paragraph {
                segments: vec![StyledInline::Text("[foo][]".to_string())]
            }
        );
    }
}
