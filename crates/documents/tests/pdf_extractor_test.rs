use std::fs::File;
use std::io::Write;
use documents::extractor::pdf_extractor::{PdfExtractor, PdfExtractorError};
use tempfile::tempdir;

#[test]
fn test_pdf_extractor_mock_scanned() {
    let temp = tempdir().unwrap();
    let scanned_path = temp.path().join("scan.pdf");
    let mut f = File::create(&scanned_path).unwrap();
    f.write_all(b"[OCR_REQUIRED] Scanned image without text layer").unwrap();

    let result = PdfExtractor::extract(&scanned_path);
    match result {
        Err(PdfExtractorError::PdfNeedsOcr { pages }) => {
            assert_eq!(pages, vec![1]);
        }
        other => panic!("Expected PdfNeedsOcr, got: {:?}", other),
    }
}
