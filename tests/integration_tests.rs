use md_to_svg::config::SvgConfig;
use md_to_svg::layout::{LayoutItem, process_layouts};
use md_to_svg::styles::{StyledBlock, StyledInline};

use crate::common::{create_default_test_font_system, normalize_svg_for_comparison};

pub mod common;

#[test]
fn process_layouts_creates_svg() {
    let mut font_system = create_default_test_font_system();
    let cfg = SvgConfig::default();

    let paragraph_text = "Hello World".to_string();

    let styled_block = StyledBlock::Paragraph {
        segments: vec![StyledInline::Text(paragraph_text.clone())],
    };

    let layout_item = LayoutItem::Block(styled_block.into_layout_block(&cfg, &mut font_system));

    let svg_lines = process_layouts(vec![layout_item], &cfg);

    assert_eq!(svg_lines.len(), 1);
    assert!(svg_lines[0].contains(&paragraph_text));
}

#[test]
fn process_layout_line_long_soft_wrap_produces_multiple_lines() {
    let mut font_system = create_default_test_font_system();
    let cfg = SvgConfig::default();

    let paragraph_text = "'In the stillest moment of your night, if it were truly denied you to create, would you die?' And if your answer is yes, you have no choice. That is your choice, because there is no euphoria in a different place for an artist.".to_string();

    let styled_block = StyledBlock::Paragraph {
        segments: vec![StyledInline::Text(paragraph_text.clone())],
    };

    let layout_item = LayoutItem::Block(styled_block.into_layout_block(&cfg, &mut font_system));

    let svg_lines = process_layouts(vec![layout_item], &cfg);

    assert_eq!(svg_lines.len(), 3);
}

#[test]
fn process_layout_line_long_hard_break_produces_multiple_lines() {
    let mut font_system = create_default_test_font_system();
    let cfg = SvgConfig::default();

    let line_one = "Slip like Freudian".to_string();
    let line_two = "Your first and last step to playing yourself like accordion".to_string();
    let styled_block = StyledBlock::Paragraph {
        segments: vec![
            StyledInline::Text(line_one.clone()),
            StyledInline::HardBreak,
            StyledInline::Text(line_two.clone()),
        ],
    };

    let layout_item = LayoutItem::Block(styled_block.into_layout_block(&cfg, &mut font_system));

    let svg_lines = process_layouts(vec![layout_item], &cfg);

    assert_eq!(svg_lines.len(), 2);
    assert!(svg_lines[0].contains(&line_one));
    assert!(svg_lines[1].contains(&line_two));
}

#[test]
fn process_layouts_soft_wrap_preserves_text_content() {
    let mut font_system = create_default_test_font_system();
    let cfg = SvgConfig::default();

    let expected = r#"This is some long text that will wrap, but also includes some special characters such as "Quotes", < > & '"#.to_string();

    let styled_block = StyledBlock::Paragraph {
        segments: vec![StyledInline::Text(expected.clone())],
    };

    let layout_item = LayoutItem::Block(styled_block.into_layout_block(&cfg, &mut font_system));

    let svg_lines = process_layouts(vec![layout_item], &cfg);

    let actual = svg_lines
        .iter()
        .map(|line| normalize_svg_for_comparison(line))
        .collect::<Vec<String>>()
        .join(" ");

    assert!(actual.contains(&expected));
}

#[test]
fn process_layouts_hard_break_preserves_text_content() {
    let mut font_system = create_default_test_font_system();
    let cfg = SvgConfig::default();

    let paragraph_text_upper = r#"This is some long text that will wrap"#.to_string();
    let paragraph_text_lower =
        r#"but also includes some special characters such as "Quotes", < > & '"#.to_string();

    let expected = [paragraph_text_upper.as_str(), paragraph_text_lower.as_str()].join(" ");

    let styled_block = StyledBlock::Paragraph {
        segments: vec![
            StyledInline::Text(paragraph_text_upper.clone()),
            StyledInline::HardBreak,
            StyledInline::Text(paragraph_text_lower.clone()),
        ],
    };

    let layout_item = LayoutItem::Block(styled_block.into_layout_block(&cfg, &mut font_system));

    let svg_lines = process_layouts(vec![layout_item], &cfg);

    let actual = svg_lines
        .iter()
        .map(|line| normalize_svg_for_comparison(line))
        .collect::<Vec<String>>()
        .join(" ");

    assert!(actual.contains(&expected));
}

#[test]
fn process_layouts_handles_thematic_break() {
    let mut font_system = create_default_test_font_system();
    let cfg = SvgConfig::default();
    
    let styled_block = StyledBlock::Paragraph { segments: vec![StyledInline::Text("Hello World".to_string())] };
    let layout_items = vec![
        LayoutItem::Block(
            styled_block.into_layout_block(&cfg, &mut font_system)),
        LayoutItem::ThematicBreak {
                left: cfg.canvas_opts.padding.left,
                right: cfg.canvas_opts.width - cfg.canvas_opts.padding.right,
            }];

    let svg_lines = process_layouts(layout_items, &cfg);
    
    assert_eq!(svg_lines.len(), 2);
    
    assert!(svg_lines[0].contains("Hello World"));
    assert!(svg_lines[1].contains("<line"))
}
