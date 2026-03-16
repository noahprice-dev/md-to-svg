# Markdown to Svg Converter

## How to use

### CLI Options

### Preset Configuration Files

## Installation

## Dependencies & Special Thanks
- `markdown_rs` for converting Markdown into an Abstract Syntax Tree
- `cosmic_text` for shaping and wrapping the resulting markdown.
- `config` for overrides and configurations
- `clap` for CLI
- `toml` for paring TOML files via `serde`

## Contributing

### 1.0 Release
- Docs
- Default configs & directory.
- Canvas sizing to height based on input text.
[x] Complete Markdown to Svg pipeline.
[x] Custom styling configs

### 1.1 Roadmap
- Full OpenMark spec support
- Add String type wrapper for HexCode to verify typing.
- Debug renderer for Canvas sizing, etc.
- Vector-based text rendering for non-latin scripts
- Sub/Superscript, Strikethrough, text color styling

### Future Ideas
- Full GFM support with flags
- 