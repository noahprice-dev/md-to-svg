use std::{
    io::{self},
    path::PathBuf,
};

use markdown::message::Message;

#[derive(Debug, thiserror::Error)]
pub enum MdToSvgError {
    #[error("input file not found: {0}")]
    InputNotFound(PathBuf),

    #[error("input file not readable: {0}")]
    InputUnreadable(PathBuf, io::Error),

    #[error("markdown parse failed: {reason}")]
    MarkdownParseFailed { reason: String },

    #[error("output file not found: {0}")]
    OutputNotFound(PathBuf),

    #[error("output file not writeable: {0}")]
    OutputNotWritable(PathBuf, io::Error),

    #[error("HTML parse failed: {0}")]
    HtmlParseFailed(String),

    #[error("md-to-svg configuration not found: {0}")]
    ConfigNotFound(PathBuf),

    #[error("Configuration file at path: {0} not readable: {1}")]
    ConfigNotReadable(PathBuf, io::Error),

    #[error("Failed to parse configuration file")]
    ConfigParseFailed(#[from] toml::de::Error),
}

impl From<Message> for MdToSvgError {
    fn from(msg: Message) -> Self {
        Self::MarkdownParseFailed { reason: msg.reason }
    }
}
