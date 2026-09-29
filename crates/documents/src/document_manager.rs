use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use directories::ProjectDirs;
use sqlx::{Pool, Sqlite, Transaction};
use thiserror::Error;
use uuid::Uuid;

use crate::models::Document;

#[derive(Debug, Error)]
pub enum UploadDocumentError {
    #[error("Failed to resolve application data directory: {0}")]
    DataDirResolution(String),

    #[error("Source file unreadable at {path}: {reason}")]
    SourceFileUnreadable { path: PathBuf, reason: String },

    #[error("Failed to hash source file at {path}: {reason}")]
    HashFailure { path: PathBuf, reason: String },

    #[error("Failed to copy file from {from} to {to}: {reason}")]
    FileCopyFailed {
        from: PathBuf,
        to: PathBuf,
        reason: String,
    },

    #[error("Failed to remove source file after copying at {path}: {reason}")]
    SourceFileRemovalFailed { path: PathBuf, reason: String },

    #[error("File with hash {hash} already exists in project {project_id}")]
    DuplicateHashInProject { hash: String, project_id: i64 },

    #[error("Database error during document upload: {0}")]
    DatabaseFailure(String),

    #[error("Compensation failed for {path}: {reason}")]
    CompensationFailed { path: PathBuf, reason: String },
}

#[derive(Debug, Error)]
pub enum ReplaceDocumentError {
    #[error("Document with id {0} was not found")]
    DocumentNotFound(i64),

    #[error("Replacement file has identical hash ({hash}) as existing document {document_id}")]
    IdenticalHash { document_id: i64, hash: String },

    #[error("Failed to resolve application data directory: {0}")]
    DataDirResolution(String),

    #[error("Source file unreadable at {path}: {reason}")]
    SourceFileUnreadable { path: PathBuf, reason: String },

    #[error("Failed to hash source file at {path}: {reason}")]
    HashFailure { path: PathBuf, reason: String },

    #[error("Failed to copy file from {from} to {to}: {reason}")]
    FileCopyFailed {
        from: PathBuf,
        to: PathBuf,
        reason: String,
    },

    #[error("Failed to remove source file at {path}: {reason}")]
    SourceFileRemovalFailed { path: PathBuf, reason: String },

    #[error("Database error during document replacement: {0}")]
    DatabaseFailure(String),

    #[error("Failed to remove old document file at {path}: {reason}")]
    OldFileRemovalFailed { path: PathBuf, reason: String },

    #[error("Compensation failed for {path}: {reason}")]
    CompensationFailed { path: PathBuf, reason: String },
}

#[derive(Debug, Error)]
pub enum DeleteDocumentError {
    #[error("Document with id {0} was not found")]
    DocumentNotFound(i64),

    #[error("Database error during document deletion: {0}")]
    DatabaseFailure(String),

    #[error("Failed to delete physical file at {path}: {reason}")]
    FileRemovalFailed { path: PathBuf, reason: String },
}

#[derive(Debug, Error)]
pub enum GetDocumentError {
    #[error("Document with id {0} was not found")]
    DocumentNotFound(i64),

    #[error("Database error when fetching document: {0}")]
    DatabaseFailure(String),
}

#[derive(Debug, Error)]
pub enum DocumentManagerError {
    #[error("Data directory initialization failed: {0}")]
    DataDirInit(String),

    #[error(transparent)]
    Upload(#[from] UploadDocumentError),

    #[error(transparent)]
    Replace(#[from] ReplaceDocumentError),

    #[error(transparent)]
    Delete(#[from] DeleteDocumentError),

    #[error(transparent)]
    Get(#[from] GetDocumentError),
}

#[derive(Debug)]
pub enum DocumentManagerEvent {
    UploadDocumentStart {
        project_id: i64,
        file_count: usize,
    },
    UploadDocumentSuccess {
        project_id: i64,
        document_id: i64,
        path: String,
    },
    UploadDocumentFailure {
        project_id: i64,
        path: PathBuf,
        error: String,
    },
    ReplaceDocumentStart {
        project_id: i64,
        old_document_id: i64,
        new_file: PathBuf,
    },
    ReplaceDocumentSuccess {
        project_id: i64,
        new_document_id: i64,
        old_document_id: i64,
    },
    ReplaceDocumentFailure {
        project_id: i64,
        old_document_id: i64,
        error: String,
    },
    DeleteDocumentStart {
        document_id: i64,
    },
    DeleteDocumentSuccess {
        document_id: i64,
    },
    DeleteDocumentFailure {
        document_id: i64,
        error: String,
    },
    GetDocumentStart {
        document_id: i64,
    },
    GetDocumentSuccess {
        document_id: i64,
    },
    GetDocumentFailure {
        document_id: i64,
        error: String,
    },
}

#[derive(Debug)]
pub struct UploadFailure {
    pub original_path: PathBuf,
    pub original_name: String,
    pub error: UploadDocumentError,
}

#[derive(Debug)]
pub struct UploadDocumentsResult {
    pub successes: Vec<Document>,
    pub failures: Vec<UploadFailure>,
}

pub struct DocumentDraft {
    pub hash: String,
    pub extension: String,
    pub mtime: i64,
    pub original_name: String,
    pub path: String,
}

#[derive(Clone, Debug)]
pub struct DocumentManager {
    pool: Pool<Sqlite>,
    data_dir: PathBuf,
}

impl DocumentManager {
    /// Base data directory resolved from system ProjectDirs. Stored document
    /// paths are relative to it.
    pub fn default_data_dir() -> Result<PathBuf, DocumentManagerError> {
        let proj_dirs = ProjectDirs::from("software", "whatever", "targetz").ok_or_else(|| {
            DocumentManagerError::DataDirInit("Failed to resolve project directories".to_string())
        })?;
        Ok(proj_dirs.data_dir().to_path_buf())
    }

    /// Construct DocumentManager resolving the base data directory from system ProjectDirs
    pub fn new(pool: Pool<Sqlite>) -> Result<Self, DocumentManagerError> {
        Self::with_data_dir(pool, Self::default_data_dir()?)
    }

    /// Construct DocumentManager with an explicit data directory path
    pub fn with_data_dir(
        pool: Pool<Sqlite>,
        data_dir: PathBuf,
    ) -> Result<Self, DocumentManagerError> {
        if !data_dir.exists() {
            std::fs::create_dir_all(&data_dir).map_err(|err| {
                DocumentManagerError::DataDirInit(format!(
                    "Failed to create data dir {}: {}",
                    data_dir.display(),
                    err
                ))
            })?;
        }
        Ok(Self { pool, data_dir })
    }

    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    /// Internal function to insert a document and link it to the project inside an open transaction
    pub async fn insert_document_tx(
        tx: &mut Transaction<'_, Sqlite>,
        doc: &DocumentDraft,
        project_id: i64,
    ) -> Result<Document, sqlx::Error> {
        let inserted = sqlx::query!(
            r#"
            INSERT INTO documents (hash, extension, mtime, original_name, path)
            VALUES (?, ?, ?, ?, ?)
            RETURNING id, hash, extension, mtime, original_name, path
            "#,
            doc.hash,
            doc.extension,
            doc.mtime,
            doc.original_name,
            doc.path
        )
        .fetch_one(&mut **tx)
        .await?;

        sqlx::query!(
            r#"
            INSERT INTO project_document (project_id, document_id)
            VALUES (?, ?)
            "#,
            project_id,
            inserted.id
        )
        .execute(&mut **tx)
        .await?;

        Ok(Document {
            id: inserted.id,
            hash: inserted.hash,
            extension: inserted.extension,
            mtime: inserted.mtime,
            original_name: inserted.original_name,
            path: inserted.path,
        })
    }

    /// Streaming Blake3 hash calculation without loading whole file into memory
    pub fn compute_hash_streaming(path: &Path) -> Result<String, std::io::Error> {
        let file = File::open(path)?;
        let mut reader = BufReader::with_capacity(64 * 1024, file);
        let mut hasher = blake3::Hasher::new();
        let mut buffer = [0u8; 64 * 1024];

        loop {
            let bytes_read = reader.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }

        Ok(hasher.finalize().to_hex().to_string())
    }

    /// Upload documents sequentially, each in its own transaction
    pub async fn upload_documents(
        &self,
        files: &[PathBuf],
        project_id: i64,
    ) -> UploadDocumentsResult {
        tracing::info!(
            event = ?DocumentManagerEvent::UploadDocumentStart {
                project_id,
                file_count: files.len(),
            },
            "Starting upload of {} documents for project {}",
            files.len(),
            project_id
        );

        let mut successes = Vec::new();
        let mut failures = Vec::new();

        for file_path in files {
            match self.upload_single_document(file_path, project_id).await {
                Ok(doc) => {
                    tracing::info!(
                        event = ?DocumentManagerEvent::UploadDocumentSuccess {
                            project_id,
                            document_id: doc.id,
                            path: doc.path.clone(),
                        },
                        "Uploaded document id {} for project {}",
                        doc.id,
                        project_id
                    );
                    successes.push(doc);
                }
                Err(err) => {
                    let original_name = file_path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown")
                        .to_string();

                    tracing::error!(
                        event = ?DocumentManagerEvent::UploadDocumentFailure {
                            project_id,
                            path: file_path.clone(),
                            error: err.to_string(),
                        },
                        "Failed to upload document {}: {}",
                        file_path.display(),
                        err
                    );

                    failures.push(UploadFailure {
                        original_path: file_path.clone(),
                        original_name,
                        error: err,
                    });
                }
            }
        }

        UploadDocumentsResult {
            successes,
            failures,
        }
    }

    async fn upload_single_document(
        &self,
        file_path: &Path,
        project_id: i64,
    ) -> Result<Document, UploadDocumentError> {
        let metadata = std::fs::metadata(file_path).map_err(|e| {
            UploadDocumentError::SourceFileUnreadable {
                path: file_path.to_path_buf(),
                reason: e.to_string(),
            }
        })?;

        let mtime = metadata
            .modified()
            .map_err(|e| UploadDocumentError::SourceFileUnreadable {
                path: file_path.to_path_buf(),
                reason: e.to_string(),
            })?
            .duration_since(UNIX_EPOCH)
            .map_err(|e| UploadDocumentError::SourceFileUnreadable {
                path: file_path.to_path_buf(),
                reason: e.to_string(),
            })?
            .as_secs() as i64;

        let original_name = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let extension = file_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        // 1. Calculate Blake3 hash streaming
        let hash = Self::compute_hash_streaming(file_path).map_err(|e| {
            UploadDocumentError::HashFailure {
                path: file_path.to_path_buf(),
                reason: e.to_string(),
            }
        })?;

        // 2. Check if a document with this hash is already in the project
        let existing = sqlx::query!(
            r#"
            SELECT d.id
            FROM documents d
            INNER JOIN project_document pd ON d.id = pd.document_id
            WHERE pd.project_id = ? AND d.hash = ?
            LIMIT 1
            "#,
            project_id,
            hash
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| UploadDocumentError::DatabaseFailure(e.to_string()))?;

        if existing.is_some() {
            return Err(UploadDocumentError::DuplicateHashInProject { hash, project_id });
        }

        // 3. Copy file into data dir under UUIDv7 name, then remove source
        let file_uuid = Uuid::now_v7();
        let relative_name = if extension.is_empty() {
            file_uuid.to_string()
        } else {
            format!("{}.{}", file_uuid, extension)
        };
        let target_path = self.data_dir.join(&relative_name);

        std::fs::copy(file_path, &target_path).map_err(|e| {
            UploadDocumentError::FileCopyFailed {
                from: file_path.to_path_buf(),
                to: target_path.clone(),
                reason: e.to_string(),
            }
        })?;

        // 4. Open transaction and insert
        let mut tx = match self.pool.begin().await {
            Ok(tx) => tx,
            Err(e) => {
                Self::compensate_copied_file(&target_path);
                return Err(UploadDocumentError::DatabaseFailure(e.to_string()));
            }
        };

        let draft = DocumentDraft {
            hash,
            extension,
            mtime,
            original_name,
            path: relative_name,
        };

        match Self::insert_document_tx(&mut tx, &draft, project_id).await {
            Ok(doc) => {
                if let Err(e) = tx.commit().await {
                    Self::compensate_copied_file(&target_path);
                    return Err(UploadDocumentError::DatabaseFailure(e.to_string()));
                }
                Ok(doc)
            }
            Err(e) => {
                let _ = tx.rollback().await;
                Self::compensate_copied_file(&target_path);
                Err(UploadDocumentError::DatabaseFailure(e.to_string()))
            }
        }
    }

    /// Replace an existing document with a new file
    pub async fn replace_document(
        &self,
        old_document_id: i64,
        new_file: &Path,
        project_id: i64,
    ) -> Result<Document, ReplaceDocumentError> {
        tracing::info!(
            event = ?DocumentManagerEvent::ReplaceDocumentStart {
                project_id,
                old_document_id,
                new_file: new_file.to_path_buf(),
            },
            "Starting replace of document {} in project {}",
            old_document_id,
            project_id
        );

        let metadata = std::fs::metadata(new_file).map_err(|e| {
            ReplaceDocumentError::SourceFileUnreadable {
                path: new_file.to_path_buf(),
                reason: e.to_string(),
            }
        })?;

        let mtime = metadata
            .modified()
            .map_err(|e| ReplaceDocumentError::SourceFileUnreadable {
                path: new_file.to_path_buf(),
                reason: e.to_string(),
            })?
            .duration_since(UNIX_EPOCH)
            .map_err(|e| ReplaceDocumentError::SourceFileUnreadable {
                path: new_file.to_path_buf(),
                reason: e.to_string(),
            })?
            .as_secs() as i64;

        let original_name = new_file
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let extension = new_file
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        // 1. Hash new file streaming
        let new_hash = Self::compute_hash_streaming(new_file).map_err(|e| {
            ReplaceDocumentError::HashFailure {
                path: new_file.to_path_buf(),
                reason: e.to_string(),
            }
        })?;

        // 2. Copy new file into data dir under UUIDv7 name
        let file_uuid = Uuid::now_v7();
        let relative_name = if extension.is_empty() {
            file_uuid.to_string()
        } else {
            format!("{}.{}", file_uuid, extension)
        };
        let new_target_path = self.data_dir.join(&relative_name);

        std::fs::copy(new_file, &new_target_path).map_err(|e| {
            ReplaceDocumentError::FileCopyFailed {
                from: new_file.to_path_buf(),
                to: new_target_path.clone(),
                reason: e.to_string(),
            }
        })?;

        if let Err(e) = std::fs::remove_file(new_file) {
            let _ = std::fs::remove_file(&new_target_path);
            return Err(ReplaceDocumentError::SourceFileRemovalFailed {
                path: new_file.to_path_buf(),
                reason: e.to_string(),
            });
        }

        // 3. Execute atomic transaction
        let mut tx = match self.pool.begin().await {
            Ok(tx) => tx,
            Err(e) => {
                Self::compensate_copied_file(&new_target_path);
                return Err(ReplaceDocumentError::DatabaseFailure(e.to_string()));
            }
        };

        // Find old document
        let old_doc = match sqlx::query!(
            r#"
            SELECT d.id, d.hash, d.extension, d.mtime, d.original_name, d.path
            FROM documents d
            INNER JOIN project_document pd ON d.id = pd.document_id
            WHERE pd.project_id = ? AND d.id = ?
            "#,
            project_id,
            old_document_id
        )
        .fetch_optional(&mut *tx)
        .await
        {
            Ok(Some(row)) => row,
            Ok(None) => {
                let _ = tx.rollback().await;
                Self::compensate_copied_file(&new_target_path);
                return Err(ReplaceDocumentError::DocumentNotFound(old_document_id));
            }
            Err(e) => {
                let _ = tx.rollback().await;
                Self::compensate_copied_file(&new_target_path);
                return Err(ReplaceDocumentError::DatabaseFailure(e.to_string()));
            }
        };

        // Check if hash equals old document's hash
        if old_doc.hash == new_hash {
            let _ = tx.rollback().await;
            Self::compensate_copied_file(&new_target_path);
            return Err(ReplaceDocumentError::IdenticalHash {
                document_id: old_document_id,
                hash: new_hash,
            });
        }

        let draft = DocumentDraft {
            hash: new_hash,
            extension,
            mtime,
            original_name,
            path: relative_name,
        };

        // Insert new document into documents and project_document
        let new_document = match Self::insert_document_tx(&mut tx, &draft, project_id).await {
            Ok(doc) => doc,
            Err(e) => {
                let _ = tx.rollback().await;
                Self::compensate_copied_file(&new_target_path);
                return Err(ReplaceDocumentError::DatabaseFailure(e.to_string()));
            }
        };

        // Migrate project_document_customer foreign key references to new document id
        if let Err(e) = sqlx::query!(
            r#"
            UPDATE project_document_customer
            SET document_id = ?
            WHERE project_id = ? AND document_id = ?
            "#,
            new_document.id,
            project_id,
            old_document_id
        )
        .execute(&mut *tx)
        .await
        {
            let _ = tx.rollback().await;
            Self::compensate_copied_file(&new_target_path);
            return Err(ReplaceDocumentError::DatabaseFailure(e.to_string()));
        }

        // Delete old document from documents (cascades remove old project_document pivot)
        if let Err(e) = sqlx::query!(
            r#"
            DELETE FROM documents WHERE id = ?
            "#,
            old_document_id
        )
        .execute(&mut *tx)
        .await
        {
            let _ = tx.rollback().await;
            Self::compensate_copied_file(&new_target_path);
            return Err(ReplaceDocumentError::DatabaseFailure(e.to_string()));
        }

        // Commit transaction
        if let Err(e) = tx.commit().await {
            Self::compensate_copied_file(&new_target_path);
            return Err(ReplaceDocumentError::DatabaseFailure(e.to_string()));
        }

        // After successful commit: delete old physical file
        let old_file_path = self.data_dir.join(&old_doc.path);
        if let Err(e) = std::fs::remove_file(&old_file_path) {
            tracing::warn!(
                "Failed to delete old document physical file at {}: {}. Left for prune service.",
                old_file_path.display(),
                e
            );
        }

        tracing::info!(
            event = ?DocumentManagerEvent::ReplaceDocumentSuccess {
                project_id,
                new_document_id: new_document.id,
                old_document_id,
            },
            "Replaced document {} with {} in project {}",
            old_document_id,
            new_document.id,
            project_id
        );

        Ok(new_document)
    }

    /// Delete document, cascade clean pivots, and delete physical file
    pub async fn delete_document(&self, id: i64) -> Result<(), DeleteDocumentError> {
        tracing::info!(
            event = ?DocumentManagerEvent::DeleteDocumentStart { document_id: id },
            "Starting deletion of document {}",
            id
        );

        let doc = sqlx::query!(
            r#"
            SELECT id, hash, extension, mtime, original_name, path
            FROM documents
            WHERE id = ?
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DeleteDocumentError::DatabaseFailure(e.to_string()))?
        .ok_or(DeleteDocumentError::DocumentNotFound(id))?;

        sqlx::query!(
            r#"
            DELETE FROM documents WHERE id = ?
            "#,
            id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DeleteDocumentError::DatabaseFailure(e.to_string()))?;

        // Delete physical file
        let physical_path = self.data_dir.join(&doc.path);
        if let Err(e) = std::fs::remove_file(&physical_path) {
            tracing::warn!(
                "Failed to remove physical file {} after deleting document {}: {}. Left for prune service.",
                physical_path.display(),
                id,
                e
            );
        }

        tracing::info!(
            event = ?DocumentManagerEvent::DeleteDocumentSuccess { document_id: id },
            "Deleted document {}",
            id
        );

        Ok(())
    }

    /// Get a document by its id
    pub async fn get_document(&self, id: i64) -> Result<Document, GetDocumentError> {
        tracing::info!(
            event = ?DocumentManagerEvent::GetDocumentStart { document_id: id },
            "Fetching document {}",
            id
        );

        let row = sqlx::query!(
            r#"
            SELECT id, hash, extension, mtime, original_name, path
            FROM documents
            WHERE id = ?
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| GetDocumentError::DatabaseFailure(e.to_string()))?
        .ok_or(GetDocumentError::DocumentNotFound(id))?;

        tracing::info!(
            event = ?DocumentManagerEvent::GetDocumentSuccess { document_id: id },
            "Fetched document {}",
            id
        );

        Ok(Document {
            id: row.id,
            hash: row.hash,
            extension: row.extension,
            mtime: row.mtime,
            original_name: row.original_name,
            path: row.path,
        })
    }

    fn compensate_copied_file(path: &Path) {
        if path.exists() {
            if let Err(e) = std::fs::remove_file(path) {
                tracing::error!(
                    "Compensation deletion failed for {}: {}. Left for prune service.",
                    path.display(),
                    e
                );
            }
        }
    }
}

impl DocumentManager {
    pub async fn get_project_document_stats(
        &self,
    ) -> Result<Vec<crate::models::ProjectDocumentStats>, sqlx::Error> {
        let stats = sqlx::query_as!(
            crate::models::ProjectDocumentStats,
            r#"
            SELECT
                p.id as project_id,
                p.name as project_name,
                COUNT(DISTINCT pd.document_id) as document_count,
                COUNT(DISTINCT pdc.customer_id) as customer_count
            FROM projects p
            LEFT JOIN project_document pd ON p.id = pd.project_id
            LEFT JOIN project_document_customer pdc ON p.id = pdc.project_id
            GROUP BY p.id
            ORDER BY p.name ASC
            "#
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(stats)
    }

    /// Persists discovered document -> customer relations for `project_id`.
    ///
    /// New links are stored as `not_confirmed`. Links that already exist are
    /// left untouched, so a customer the user already confirmed is never reset.
    /// Runs in a single transaction. Returns how many new links were created.
    pub async fn save_relations(
        &self,
        project_id: i64,
        relations: &crate::relation_manager::RelationsResult,
    ) -> Result<u64, sqlx::Error> {
        let mut tx = self.pool.begin().await?;
        let mut inserted = 0;

        for (document_id, customers) in &relations.relations {
            for customer in customers {
                let result = sqlx::query(
                    "INSERT OR IGNORE INTO project_document_customer \
                     (project_id, customer_id, document_id, status) \
                     VALUES (?, ?, ?, ?)",
                )
                .bind(project_id)
                .bind(customer.id)
                .bind(document_id)
                .bind(crate::models::CustomerDocumentStatus::NotConfirmed.as_str())
                .execute(&mut *tx)
                .await?;
                inserted += result.rows_affected();
            }
        }

        tx.commit().await?;
        Ok(inserted)
    }

    pub async fn get_project_documents(
        &self,
        project_id: i64,
    ) -> Result<Vec<crate::models::DocumentWithCustomers>, sqlx::Error> {
        let docs = sqlx::query_as!(
            crate::models::DocumentWithCustomers,
            r#"
            SELECT
                d.id,
                d.hash,
                d.extension,
                d.mtime,
                d.original_name,
                d.path,
                CAST(GROUP_CONCAT(pdc.customer_id) AS TEXT) as customer_ids
            FROM documents d
            INNER JOIN project_document pd ON d.id = pd.document_id
            LEFT JOIN project_document_customer pdc ON d.id = pdc.document_id AND pd.project_id = pdc.project_id
            WHERE pd.project_id = ?
            GROUP BY d.id
            ORDER BY d.original_name ASC
            "#,
            project_id
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(docs)
    }
}
