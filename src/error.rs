use std::{io, path::PathBuf};

use markdown::message::Message;

#[derive(Debug, thiserror::Error)]
pub enum MdToSvgError {
    #[error("input file not found: {0}")]
    InputNotFound(PathBuf),
    #[error("input file not readable: {0}")]
    InputUnreadable(io::Error),
    #[error("markdown parse failed: {reason}")]
    ParseFailed { reason: String },
    #[error("output file not found: {0}")]
    OutputNotFound(PathBuf),
    #[error("output file not writeable: {0}")]
    OutputNotWritable(io::Error),
    #[error("HTML parse failed: {0}")]
    HtmlParseFailed(String),
    #[error("md-to-svg configuration load failed: {0}")]
    // Probably wants some additional context on whats wrong. The current point of failure is unwrapping the bullet character from bytes.
    ConfigLoadFailure(PathBuf),
}

impl From<Message> for MdToSvgError {
    fn from(msg: Message) -> Self {
        Self::ParseFailed { reason: msg.reason }
    }
}
