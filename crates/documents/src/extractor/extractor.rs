use std::path::Path;
use thiserror::Error;
use crate::extractor::pdf_extractor::{PdfExtractor, PdfExtractorError};

#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum ExtractorError {
    #[error("PDF requires OCR for pages: {pages:?}")]
    PdfNeedsOcr { pages: Vec<usize> },

    #[error("PDF extraction error: {0}")]
    PdfError(String),

    #[error("Unsupported file extension: {0}")]
    UnsupportedExtension(String),

    #[error("File is missing an extension: {0}")]
    MissingExtension(String),

    #[error("Failed to read file at {path}: {reason}")]
    IoError { path: String, reason: String },
}

impl From<PdfExtractorError> for ExtractorError {
    fn from(err: PdfExtractorError) -> Self {
        match err {
            PdfExtractorError::PdfNeedsOcr { pages } => ExtractorError::PdfNeedsOcr { pages },
            PdfExtractorError::InspectionFailed { path: _, reason } => {
                ExtractorError::PdfError(reason)
            }
            PdfExtractorError::ExtractionFailed { path: _, reason } => {
                ExtractorError::PdfError(reason)
            }
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct DocumentExtractorService;

impl DocumentExtractorService {
    pub fn new() -> Self {
        Self
    }

    /// Receives a file path, figures out how to read it, and returns the raw text.
    pub fn extract_text(&self, path: &Path) -> Result<String, ExtractorError> {
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase())
            .ok_or_else(|| ExtractorError::MissingExtension(path.display().to_string()))?;

        match extension.as_str() {
            "pdf" => PdfExtractor::extract(path).map_err(Into::into),
            "txt" | "md" | "json" | "csv" | "html" => {
                std::fs::read_to_string(path).map_err(|err| ExtractorError::IoError {
                    path: path.display().to_string(),
                    reason: err.to_string(),
                })
            }
            unsupported => Err(ExtractorError::UnsupportedExtension(unsupported.to_string())),
        }
    }
}
