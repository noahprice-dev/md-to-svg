use cosmic_text::{FamilyOwned, Style, Weight};
use md_to_svg::layout::{SvgConfig, process_layouts, styled_line_to_layout};
use md_to_svg::styles::{StyledBlock, StyledLine, StyledSegment};

use crate::common::create_default_test_font_system;

mod common;

#[test]
fn process_layout_line_paragraph() {
    let mut font_system = create_default_test_font_system();
    let cfg = SvgConfig::default();

    let paragraph_text = "Hello World".to_string();

    let styled_line = StyledLine::Paragraph {
        segments: vec![StyledSegment::Text(StyledBlock {
            text: paragraph_text.clone(),
            weight: Weight::NORMAL,
            style: Style::Normal,
            family: FamilyOwned::SansSerif,
        })],
    };

    let layout_result = styled_line_to_layout(styled_line, &mut font_system, &cfg);

    let svg_lines = process_layouts(vec![layout_result], &cfg);

    assert_eq!(svg_lines.len(), 1);
    assert!(svg_lines[0].contains(&paragraph_text));
}

#[test]
fn process_layout_line_long_soft_wrap_produces_multiple_lines() {
    let mut font_system = create_default_test_font_system();
    let cfg = SvgConfig::default();

    let paragraph_text = "In the stillest moment of your night, if it were truly denied you to create, would you die?' And if your answer is yes, you have no choice. That is your choice, because there is no euphoria in a different place for an artist.".to_string();

    let styled_line = StyledLine::Paragraph {
        segments: vec![StyledSegment::Text(StyledBlock {
            text: paragraph_text.clone(),
            weight: Weight::NORMAL,
            style: Style::Normal,
            family: FamilyOwned::SansSerif,
        })],
    };

    let layout_result = styled_line_to_layout(styled_line, &mut font_system, &cfg);

    let svg_lines = process_layouts(vec![layout_result], &cfg);

    assert_eq!(svg_lines.len(), 3);
}

#[test]
fn process_layout_line_long_hard_break_produces_multiple_lines() {
    let mut font_system = create_default_test_font_system();
    let cfg = SvgConfig::default();

    let styled_line = StyledLine::Paragraph {
        segments: vec![
            StyledSegment::Text(StyledBlock {
                text: "Slip like Freudian".to_string(),
                weight: Weight::NORMAL,
                style: Style::Normal,
                family: FamilyOwned::SansSerif,
            }),
            StyledSegment::HardBreak,
            StyledSegment::Text(StyledBlock {
                text: "Your first and last step to playing yourself like accordion".to_string(),
                weight: Weight::NORMAL,
                style: Style::Normal,
                family: FamilyOwned::SansSerif,
            }),
        ],
    };

    let layout_result = styled_line_to_layout(styled_line, &mut font_system, &cfg);

    let svg_lines = process_layouts(vec![layout_result], &cfg);

    assert_eq!(svg_lines.len(), 2);
}

#[test]
fn process_layouts_soft_wrap_preserves_text_content() {}
#[test]
fn process_layouts_hard_break_preserves_text_content() {}
