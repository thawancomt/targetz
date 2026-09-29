use std::collections::HashMap;
use std::path::PathBuf;

use rayon::prelude::*;
use regex::Regex;
use shared::customer::{Customer, Persisted};
use thiserror::Error;

use crate::document_manager::{DocumentManager, GetDocumentError};
use crate::extractor::{DocumentExtractorService, ExtractorError};

#[derive(Debug, Error)]
pub enum DocumentProcessingError {
    #[error("Document {0} was not found")]
    DocumentNotFound(i64),

    #[error("Failed to retrieve document: {0}")]
    GetDocumentFailed(#[from] GetDocumentError),

    #[error("Extraction failed: {0}")]
    ExtractionFailed(#[from] ExtractorError),

    #[error("Document file not readable at {path}: {reason}")]
    FileUnreadable { path: PathBuf, reason: String },

    #[error("Regex error during customer matching: {0}")]
    RegexError(#[from] regex::Error),

    #[error("Task execution panicked or failed: {0}")]
    TaskJoinError(String),
}

#[derive(Debug, Error)]
pub enum RelationManagerError {
    #[error("Failed to spawn or join blocking task: {0}")]
    TaskJoinError(String),

    #[error(transparent)]
    Processing(#[from] DocumentProcessingError),
}

#[derive(Debug)]
pub enum RelationManagerEvent {
    GetRelationsStart {
        document_count: usize,
        customer_count: usize,
    },
    DocumentProcessingStart {
        document_id: i64,
    },
    DocumentProcessingSuccess {
        document_id: i64,
        customers_found: usize,
    },
    DocumentProcessingFailure {
        document_id: i64,
        error: String,
    },
    GetRelationsFinish {
        processed_count: usize,
        failure_count: usize,
    },
}

#[derive(Debug, Default)]
pub struct RelationsResult {
    pub relations: HashMap<i64, Vec<Customer<Persisted>>>,
    pub failures: HashMap<i64, DocumentProcessingError>,
}

#[derive(Clone)]
pub struct RelationManager {
    document_manager: DocumentManager,
    extractor: DocumentExtractorService,
}

impl RelationManager {
    pub fn new(document_manager: DocumentManager, extractor: DocumentExtractorService) -> Self {
        Self {
            document_manager,
            extractor,
        }
    }

    /// Read existing documents and discover which customers are mentioned in them
    pub async fn get_relations(
        &self,
        doc_ids: Vec<i64>,
        customers: Vec<Customer<Persisted>>,
    ) -> Result<RelationsResult, RelationManagerError> {
        tracing::info!(
            event = ?RelationManagerEvent::GetRelationsStart {
                document_count: doc_ids.len(),
                customer_count: customers.len(),
            },
            "Starting get_relations for {} documents and {} customers",
            doc_ids.len(),
            customers.len()
        );

        let mut result = RelationsResult::default();

        // Process documents sequentially
        for doc_id in doc_ids {
            tracing::info!(
                event = ?RelationManagerEvent::DocumentProcessingStart { document_id: doc_id },
                "Processing document {}",
                doc_id
            );

            // 1. Fetch document metadata (async)
            let document = match self.document_manager.get_document(doc_id).await {
                Ok(doc) => doc,
                Err(err) => {
                    let proc_err = match err {
                        GetDocumentError::DocumentNotFound(id) => {
                            DocumentProcessingError::DocumentNotFound(id)
                        }
                        other => DocumentProcessingError::GetDocumentFailed(other),
                    };

                    tracing::error!(
                        event = ?RelationManagerEvent::DocumentProcessingFailure {
                            document_id: doc_id,
                            error: proc_err.to_string(),
                        },
                        "Failed to get document {}: {}",
                        doc_id,
                        proc_err
                    );

                    result.failures.insert(doc_id, proc_err);
                    continue;
                }
            };

            let physical_path = self.document_manager.data_dir().join(&document.path);
            let extractor = self.extractor.clone();
            let customers_clone = customers.clone();

            // 2. CPU-bound extraction & matching runs inside spawn_blocking
            let process_result = tokio::task::spawn_blocking(move || {
                // Extract text
                let text = extractor.extract_text(&physical_path)?;

                // Match against all customers in parallel with Rayon
                let matched_customers = Self::match_customers_parallel(&text, &customers_clone);

                Ok::<Vec<Customer<Persisted>>, DocumentProcessingError>(matched_customers)
            })
            .await
            .map_err(|join_err| {
                RelationManagerError::TaskJoinError(format!(
                    "Blocking task join error for document {}: {}",
                    doc_id, join_err
                ))
            })?;

            match process_result {
                Ok(mut matched) => {
                    // Guarantee deterministic ordering by customer ID
                    matched.sort_by_key(|c| c.id);

                    tracing::info!(
                        event = ?RelationManagerEvent::DocumentProcessingSuccess {
                            document_id: doc_id,
                            customers_found: matched.len(),
                        },
                        "Document {} matched {} customers",
                        doc_id,
                        matched.len()
                    );

                    result.relations.insert(doc_id, matched);
                }
                Err(err) => {
                    tracing::error!(
                        event = ?RelationManagerEvent::DocumentProcessingFailure {
                            document_id: doc_id,
                            error: err.to_string(),
                        },
                        "Document {} failed processing: {}",
                        doc_id,
                        err
                    );

                    result.failures.insert(doc_id, err);
                }
            }
        }

        tracing::info!(
            event = ?RelationManagerEvent::GetRelationsFinish {
                processed_count: result.relations.len(),
                failure_count: result.failures.len(),
            },
            "Completed get_relations. Success: {}, Failures: {}",
            result.relations.len(),
            result.failures.len()
        );

        Ok(result)
    }

    /// Parallel matching of text against customer identifiers using Rayon
    fn match_customers_parallel(
        text: &str,
        customers: &[Customer<Persisted>],
    ) -> Vec<Customer<Persisted>> {
        customers
            .par_iter()
            .filter(|customer| Self::customer_matches_text(customer, text))
            .cloned()
            .collect()
    }

    /// Targeted matching rule for an individual customer
    pub fn customer_matches_text(customer: &Customer<Persisted>, text: &str) -> bool {
        // 1. Email matching: case-insensitive, word/token boundary
        let email = customer.email.trim();
        if !email.is_empty() && Self::matches_email(text, email) {
            return true;
        }

        // 2. URLs matching: site_url & instagram_url
        if let Some(ref site_url) = customer.site_url {
            let url = site_url.trim();
            if !url.is_empty() && Self::matches_url(text, url) {
                return true;
            }
        }

        if let Some(ref insta_url) = customer.instagram_url {
            let url = insta_url.trim();
            if !url.is_empty() && Self::matches_url(text, url) {
                return true;
            }
        }

        // 3. Phone matching: digits only, ignoring +351/351 country prefix
        let phone = customer.phone_number.trim();
        if !phone.is_empty() && Self::matches_phone(text, phone) {
            return true;
        }

        false
    }

    /// Email matching: case-insensitive, must not match inside larger token
    fn matches_email(text: &str, email: &str) -> bool {
        let pattern = format!(
            r"(?i)(?:^|[^a-zA-Z0-9_.+-]){}(?:$|[^a-zA-Z0-9_.+-])",
            regex::escape(email)
        );
        Regex::new(&pattern)
            .map(|re| re.is_match(text))
            .unwrap_or(false)
    }

    /// URL matching: case-insensitive, must not match inside larger token
    fn matches_url(text: &str, url: &str) -> bool {
        let pattern = format!(
            r"(?i)(?:^|[^a-zA-Z0-9_.-]){}(?:$|[^a-zA-Z0-9_.-])",
            regex::escape(url)
        );
        Regex::new(&pattern)
            .map(|re| re.is_match(text))
            .unwrap_or(false)
    }

    /// Normalize phone number to digits, ignoring Portugal +351 / 351 prefix
    pub fn normalize_phone_digits(raw: &str) -> String {
        let mut digits: String = raw.chars().filter(|c| c.is_ascii_digit()).collect();
        if digits.starts_with("351") && digits.len() > 9 {
            digits = digits[3..].to_string();
        }
        digits
    }

    /// Compare phone by digits only, ignoring +351 prefix and formatting
    fn matches_phone(text: &str, customer_phone: &str) -> bool {
        let target_digits = Self::normalize_phone_digits(customer_phone);
        if target_digits.is_empty() {
            return false;
        }

        // Find candidate phone number strings in the text
        // Looks for sequences of digits separated by spaces, dashes, dots, slashes, or parentheses
        let phone_candidate_regex = match Regex::new(
            r"(?:\+?351[\s./-]*)?(?:\(?\d{2,4}\)?[\s./-]*\d{2,4}[\s./-]*\d{2,4}|\d{8,15})",
        ) {
            Ok(re) => re,
            Err(_) => return false,
        };

        for mat in phone_candidate_regex.find_iter(text) {
            let candidate_str = mat.as_str();
            let candidate_digits = Self::normalize_phone_digits(candidate_str);
            if candidate_digits == target_digits {
                return true;
            }
        }

        false
    }
}
