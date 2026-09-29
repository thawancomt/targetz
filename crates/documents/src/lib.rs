pub mod document_manager;
pub mod extractor;
pub mod models;
pub mod relation_manager;

pub use document_manager::{
    DeleteDocumentError, DocumentDraft, DocumentManager, DocumentManagerError,
    DocumentManagerEvent, GetDocumentError, ReplaceDocumentError, UploadDocumentError,
    UploadDocumentsResult, UploadFailure,
};
pub use extractor::{DocumentExtractorService, ExtractorError, pdf_extractor::PdfExtractor};
pub use models::{CustomerDocumentStatus, Document, ProjectDocument, ProjectDocumentCustomer};
pub use relation_manager::{
    DocumentProcessingError, RelationManager, RelationManagerError, RelationManagerEvent,
    RelationsResult,
};
pub mod ui;
