use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use documents::{
    CustomerDocumentStatus, DocumentManager, ReplaceDocumentError, UploadDocumentError,
};
use sqlx::Pool;
use sqlx::Sqlite;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use tempfile::tempdir;

async fn setup_test_db() -> Pool<Sqlite> {
    let connect_opts = SqliteConnectOptions::new()
        .filename(":memory:")
        .foreign_keys(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(connect_opts)
        .await
        .expect("Failed to connect to in-memory sqlite");

    sqlx::migrate!("../../migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    pool
}

fn create_temp_file(dir: &tempfile::TempDir, name: &str, content: &[u8]) -> PathBuf {
    let path = dir.path().join(name);
    let mut file = File::create(&path).expect("Failed to create temp file");
    file.write_all(content).expect("Failed to write content");
    path
}

#[tokio::test]
async fn test_batch_upload_with_duplicate_fails_duplicate_only_and_cleans_disk() {
    let pool = setup_test_db().await;
    let data_temp = tempdir().unwrap();
    let source_temp = tempdir().unwrap();

    // Create a dummy project in DB
    sqlx::query!(
        "INSERT INTO projects (id, name, current_version) VALUES (1, 'Project Alpha', '1.0')"
    )
    .execute(&pool)
    .await
    .unwrap();

    let doc_mgr = DocumentManager::with_data_dir(pool.clone(), data_temp.path().to_path_buf())
        .expect("Failed to init DocumentManager");

    let file1 = create_temp_file(&source_temp, "file1.txt", b"First unique document content");
    let file2 = create_temp_file(&source_temp, "file2.txt", b"Second unique document content");
    // Duplicate of file1
    let file3_dup = create_temp_file(&source_temp, "file3.txt", b"First unique document content");

    let res = doc_mgr
        .upload_documents(&[file1, file2, file3_dup], 1)
        .await;

    // file1 and file2 succeed, file3_dup fails
    assert_eq!(res.successes.len(), 2);
    assert_eq!(res.failures.len(), 1);

    let failure = &res.failures[0];
    assert_eq!(failure.original_name, "file3.txt");
    match &failure.error {
        UploadDocumentError::DuplicateHashInProject { project_id, .. } => {
            assert_eq!(*project_id, 1);
        }
        other => panic!("Expected DuplicateHashInProject, got: {:?}", other),
    }

    // Verify exactly 2 files in data_dir
    let entries: Vec<_> = std::fs::read_dir(data_temp.path())
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    assert_eq!(
        entries.len(),
        2,
        "Only successful uploads should remain on disk"
    );
}

#[tokio::test]
async fn test_same_file_in_different_projects_allowed() {
    let pool = setup_test_db().await;
    let data_temp = tempdir().unwrap();
    let source_temp = tempdir().unwrap();

    sqlx::query!("INSERT INTO projects (id, name, current_version) VALUES (1, 'Project 1', '1.0')")
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query!("INSERT INTO projects (id, name, current_version) VALUES (2, 'Project 2', '1.0')")
        .execute(&pool)
        .await
        .unwrap();

    let doc_mgr = DocumentManager::with_data_dir(pool.clone(), data_temp.path().to_path_buf())
        .expect("Failed to init DocumentManager");

    let file_p1 = create_temp_file(
        &source_temp,
        "contract.pdf",
        b"Same file content across projects",
    );
    let file_p2 = create_temp_file(
        &source_temp,
        "contract_copy.pdf",
        b"Same file content across projects",
    );

    let res1 = doc_mgr.upload_documents(&[file_p1], 1).await;
    assert_eq!(res1.successes.len(), 1);

    let res2 = doc_mgr.upload_documents(&[file_p2], 2).await;
    assert_eq!(res2.successes.len(), 1);

    // Two independent documents and two physical files
    assert_ne!(res1.successes[0].id, res2.successes[0].id);
    assert_ne!(res1.successes[0].path, res2.successes[0].path);

    let entries: Vec<_> = std::fs::read_dir(data_temp.path())
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    assert_eq!(entries.len(), 2);
}

#[tokio::test]
async fn test_replace_document_success() {
    let pool = setup_test_db().await;
    let data_temp = tempdir().unwrap();
    let source_temp = tempdir().unwrap();

    sqlx::query!("INSERT INTO projects (id, name, current_version) VALUES (1, 'Project 1', '1.0')")
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query!(
        "INSERT INTO customers (id, name, email, phone_number) VALUES (10, 'Cust A', 'a@test.com', '912345678')"
    )
    .execute(&pool)
    .await
    .unwrap();

    let doc_mgr = DocumentManager::with_data_dir(pool.clone(), data_temp.path().to_path_buf())
        .expect("Failed to init DocumentManager");

    let initial_file = create_temp_file(&source_temp, "initial.txt", b"Version 1 content");
    let upload_res = doc_mgr.upload_documents(&[initial_file], 1).await;
    let old_doc = upload_res.successes.into_iter().next().unwrap();
    let old_physical_path = data_temp.path().join(&old_doc.path);
    assert!(old_physical_path.exists());

    // Link customer to old document
    sqlx::query!(
        "INSERT INTO project_document_customer (project_id, customer_id, document_id, status) VALUES (1, 10, ?, 'confirmed')",
        old_doc.id
    )
    .execute(&pool)
    .await
    .unwrap();

    // Replace with new file
    let new_file = create_temp_file(&source_temp, "updated.txt", b"Version 2 updated content");
    let replaced_doc = doc_mgr
        .replace_document(old_doc.id, &new_file, 1)
        .await
        .expect("Replacement failed");

    // Old document is deleted
    assert!(doc_mgr.get_document(old_doc.id).await.is_err());
    // Old physical file is deleted
    assert!(!old_physical_path.exists());

    // New physical file exists
    let new_physical_path = data_temp.path().join(&replaced_doc.path);
    assert!(new_physical_path.exists());

    // Customer relation is migrated to new document id with status preserved
    let cust_pivot = sqlx::query!(
        "SELECT document_id, status FROM project_document_customer WHERE project_id = 1 AND customer_id = 10"
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(cust_pivot.document_id, replaced_doc.id);
    assert_eq!(cust_pivot.status, "confirmed");
}

#[tokio::test]
async fn test_failed_replace_with_identical_hash_leaves_old_intact() {
    let pool = setup_test_db().await;
    let data_temp = tempdir().unwrap();
    let source_temp = tempdir().unwrap();

    sqlx::query!("INSERT INTO projects (id, name, current_version) VALUES (1, 'Project 1', '1.0')")
        .execute(&pool)
        .await
        .unwrap();

    let doc_mgr = DocumentManager::with_data_dir(pool.clone(), data_temp.path().to_path_buf())
        .expect("Failed to init DocumentManager");

    let initial_file = create_temp_file(&source_temp, "file.txt", b"Static identical content");
    let upload_res = doc_mgr.upload_documents(&[initial_file], 1).await;
    let old_doc = upload_res.successes.into_iter().next().unwrap();
    let old_physical_path = data_temp.path().join(&old_doc.path);

    // Try replacing with file with identical content
    let dup_file = create_temp_file(&source_temp, "file_clone.txt", b"Static identical content");
    let err = doc_mgr
        .replace_document(old_doc.id, &dup_file, 1)
        .await
        .unwrap_err();

    match err {
        ReplaceDocumentError::IdenticalHash { document_id, .. } => {
            assert_eq!(document_id, old_doc.id);
        }
        other => panic!("Expected IdenticalHash error, got: {:?}", other),
    }

    // Old document still exists in DB
    let fetched = doc_mgr.get_document(old_doc.id).await.unwrap();
    assert_eq!(fetched.id, old_doc.id);

    // Old physical file still exists on disk
    assert!(old_physical_path.exists());

    // No leftover compensation files
    let entries: Vec<_> = std::fs::read_dir(data_temp.path())
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    assert_eq!(entries.len(), 1, "Only old physical file should exist");
}

#[tokio::test]
async fn test_delete_document_removes_pivots_and_physical_file() {
    let pool = setup_test_db().await;
    let data_temp = tempdir().unwrap();
    let source_temp = tempdir().unwrap();

    sqlx::query!("INSERT INTO projects (id, name, current_version) VALUES (1, 'Project 1', '1.0')")
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query!(
        "INSERT INTO customers (id, name, email, phone_number) VALUES (10, 'Cust A', 'a@test.com', '912345678')"
    )
    .execute(&pool)
    .await
    .unwrap();

    let doc_mgr = DocumentManager::with_data_dir(pool.clone(), data_temp.path().to_path_buf())
        .expect("Failed to init DocumentManager");

    let file = create_temp_file(&source_temp, "doc.txt", b"Document to be deleted");
    let doc = doc_mgr
        .upload_documents(&[file], 1)
        .await
        .successes
        .into_iter()
        .next()
        .unwrap();

    let physical_path = data_temp.path().join(&doc.path);
    assert!(physical_path.exists());

    // Link customer
    sqlx::query!(
        "INSERT INTO project_document_customer (project_id, customer_id, document_id, status) VALUES (1, 10, ?, 'not_confirmed')",
        doc.id
    )
    .execute(&pool)
    .await
    .unwrap();

    // Delete
    doc_mgr.delete_document(doc.id).await.unwrap();

    // Verify row deleted from documents
    assert!(doc_mgr.get_document(doc.id).await.is_err());

    // Verify cascade deleted from project_document and project_document_customer
    let pd_count = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM project_document WHERE document_id = ?",
        doc.id
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(pd_count, 0);

    let pdc_count = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM project_document_customer WHERE document_id = ?",
        doc.id
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(pdc_count, 0);

    // Physical file deleted
    assert!(!physical_path.exists());
}

#[tokio::test]
async fn test_mentioned_customers_list_and_confirm() {
    let pool = setup_test_db().await;
    let data_temp = tempdir().unwrap();
    let source_temp = tempdir().unwrap();

    sqlx::query!("INSERT INTO projects (id, name, current_version) VALUES (1, 'Project 1', '1.0')")
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query!(
        "INSERT INTO customers (id, name, email, phone_number) VALUES (10, 'Cust A', 'a@test.com', '912345678')"
    )
    .execute(&pool)
    .await
    .unwrap();

    let doc_mgr = DocumentManager::with_data_dir(pool.clone(), data_temp.path().to_path_buf())
        .expect("Failed to init DocumentManager");

    let file = create_temp_file(&source_temp, "brief.txt", b"Mentions Cust A");
    let doc = doc_mgr
        .upload_documents(&[file], 1)
        .await
        .successes
        .into_iter()
        .next()
        .unwrap();

    sqlx::query!(
        "INSERT INTO project_document_customer (project_id, customer_id, document_id, status) VALUES (1, 10, ?, 'not_confirmed')",
        doc.id
    )
    .execute(&pool)
    .await
    .unwrap();

    let mentions = doc_mgr.get_mentioned_customers(1).await.unwrap();
    assert_eq!(mentions.len(), 1);
    assert_eq!(mentions[0].customer_name, "Cust A");
    assert_eq!(mentions[0].document_name, "brief.txt");
    assert!(!mentions[0].is_confirmed());

    let updated = doc_mgr
        .set_customer_document_status(1, 10, doc.id, CustomerDocumentStatus::Confirmed)
        .await
        .unwrap();
    assert!(updated);

    let mentions = doc_mgr.get_mentioned_customers(1).await.unwrap();
    assert!(mentions[0].is_confirmed());

    let missing = doc_mgr
        .set_customer_document_status(1, 99, doc.id, CustomerDocumentStatus::NotConfirmed)
        .await
        .unwrap();
    assert!(!missing);
}
