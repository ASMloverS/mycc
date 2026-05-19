use std::io::Write;

use crate::blame::BlameResult;
use crate::util::AppError;

pub mod html;
pub mod json;
pub mod markdown;
pub mod text;

pub trait Reporter {
    fn render(&self, result: &BlameResult, out: &mut dyn Write) -> Result<(), AppError>;
}

pub fn get_reporter(format: &str, no_color: bool) -> Result<Box<dyn Reporter>, AppError> {
    match format {
        "text" => Ok(Box::new(text::TextReporter { no_color })),
        "json" => Ok(Box::new(json::JsonReporter)),
        "md" => Ok(Box::new(markdown::MarkdownReporter)),
        "html" => Ok(Box::new(html::HtmlReporter)),
        _ => Err(crate::util::AppError::usage(format!(
            "unknown format: {}",
            format
        ))),
    }
}
