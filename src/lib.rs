pub mod cli;
pub mod config;
pub mod error;
pub mod layout;
pub mod parser;
pub mod styles;
pub mod pipeline;

pub use error::MdToSvgError;
pub use pipeline::{process_md_to_svg, write_svg_to_file};

