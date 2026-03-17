# Markdown to Svg Converter
A highly configurable Markdownt to SVG Converter written in Rust.

Supports:
- Text Wrapping
- Canvas size & Padding
- Headers, custom unordered-lists
- Full control over SVG settings in-line.

The intent of this tool is to be as configurable as possible for the end user. See the Roadmap section for a breakdown of currently supported features, as well as planned features for future releases.

## How to use

```bash
md_to_svg.exe [OPTIONS] <INPUT_PATH> <OUTPUT_PATH> [PRESET_PATH]
```

A path to a .TOML file can optionally be passed after the input and output fields to override the default settings. A preset file does not need to override each setting to be valid - you can choose which ones you wish to override on a case-by-case basis. 

Overrides are prioritized by a "last-wins" strategy in order:

- Default Settings
- Preset Configuration
- Command Line flags

This means you can provide a Preset to set a new baseline, but then tweak the result to fit your needs from the CLI at call time.

The following will convert a file using some Preset file, and then override the font-size value for the final result, but keep the rest of the Preset settings.

```toml
[canvas]
width = 1000
height = 1000

[typography]
font_size = 16
```

```bash
md_to_svg.exe "input_path.md" "output_path.svg" "path/to/preset.toml" --font-size 36
```

This tool supports [CSS-Style padding](https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties/padding#syntax). This will change the size of the text box within the entire SVG Canvas as defined by the Width and Height variables. Currently we only support pixel size padding declaration.

The following will apply to the SVG text box in a clockwise direction (i.e. Top, Right, Bottom, Left).
```
md_to_svg.exe "input_path.md" "output_path.svg" --padding 10 20 30 40
```

### Preset Configuration Files

Preset configuration files must be in the .TOML format. A valid Preset file does not need to be a complete override of the default settings. A full Config file for reference can be found [here](https://github.com/noahprice-dev/md-to-svg/blob/feat/docs/default_config.toml) which shows all default settings.

Note - overrididing Header Scales is only supported via Preset file. We do not support modifying the header-scales from the CLI.

## Installation

## Dependencies & Special Thanks
- `markdown_rs` for converting Markdown into an Abstract Syntax Tree
- `cosmic_text` for shaping and wrapping the resulting markdown.
- `config` for overrides and configurations
- `clap` for CLI
- `toml` for paring TOML files via `serde`

# Current Functionality
... table ...
## Roadmap
This tool is currently a WIP and moving towards a 1.0 release. The following sections iterate, in no particular order, what the milestones for the next release will be.
### 1.0 Release
- Docs
- Default configs & directory.
- Canvas sizing to height based on input text.
- Transparent backgrounds
[x] Complete Markdown to Svg pipeline.
[x] Custom styling configs

### 1.1
- Full OpenMark spec support
- Add String type wrapper for HexCode to verify typing.
- Debug renderer for Canvas sizing, etc.
- Vector-based text rendering for non-latin scripts
- Sub/Superscript, Strikethrough, text color styling

### Future Ideas
- Full GFM support with flags
