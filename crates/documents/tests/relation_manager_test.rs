use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use documents::{
    DocumentExtractorService, DocumentManager, DocumentProcessingError, ExtractorError,
    RelationManager,
};
use shared::customer::Customer;
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

fn make_customer(
    id: i64,
    name: &str,
    email: &str,
    phone: &str,
    site: Option<&str>,
    insta: Option<&str>,
) -> Customer<i64> {
    Customer {
        id,
        name: name.to_string(),
        email: email.to_string(),
        phone_number: phone.to_string(),
        address: None,
        instagram_url: insta.map(|s| s.to_string()),
        site_url: site.map(|s| s.to_string()),
        is_client: false,
        contacted: false,
        created_at: String::new(),
    }
}

#[tokio::test]
async fn test_scanned_pdf_fails_while_text_documents_succeed() {
    let pool = setup_test_db().await;
    let data_temp = tempdir().unwrap();
    let source_temp = tempdir().unwrap();

    sqlx::query!("INSERT INTO projects (id, name, current_version) VALUES (1, 'Project 1', '1.0')")
        .execute(&pool)
        .await
        .unwrap();

    let doc_mgr = DocumentManager::with_data_dir(pool.clone(), data_temp.path().to_path_buf())
        .expect("Failed to init DocumentManager");
    let extractor = DocumentExtractorService::new();
    let relation_mgr = RelationManager::new(doc_mgr.clone(), extractor);

    // File 1: Text file mentioning customer
    let file1 = create_temp_file(
        &source_temp,
        "contract.txt",
        b"Signed by joao@company.com on 2026-09-29",
    );

    // File 2: Scanned PDF requiring OCR
    let file2_scanned = create_temp_file(
        &source_temp,
        "scanned_doc.pdf",
        b"[OCR_REQUIRED] Binary image data",
    );

    let upload_res = doc_mgr.upload_documents(&[file1, file2_scanned], 1).await;
    assert_eq!(upload_res.successes.len(), 2);

    let doc1_id = upload_res.successes[0].id;
    let doc2_id = upload_res.successes[1].id;

    let customer = make_customer(1, "Joao", "joao@company.com", "", None, None);

    let relations_res = relation_mgr
        .get_relations(vec![doc1_id, doc2_id], vec![customer.clone()])
        .await
        .expect("get_relations failed");

    // doc1 succeeds and finds Joao
    assert!(relations_res.relations.contains_key(&doc1_id));
    assert_eq!(relations_res.relations[&doc1_id].len(), 1);
    assert_eq!(relations_res.relations[&doc1_id][0].id, customer.id);

    // doc2 is in failures
    assert!(!relations_res.relations.contains_key(&doc2_id));
    assert!(relations_res.failures.contains_key(&doc2_id));

    match &relations_res.failures[&doc2_id] {
        DocumentProcessingError::ExtractionFailed(ExtractorError::PdfNeedsOcr { pages }) => {
            assert_eq!(*pages, vec![1]);
        }
        other => panic!("Expected ExtractionFailed(PdfNeedsOcr), got: {:?}", other),
    }
}

#[tokio::test]
async fn test_targeted_matching_rules() {
    let pool = setup_test_db().await;
    let data_temp = tempdir().unwrap();
    let source_temp = tempdir().unwrap();

    sqlx::query!("INSERT INTO projects (id, name, current_version) VALUES (1, 'Project 1', '1.0')")
        .execute(&pool)
        .await
        .unwrap();

    let doc_mgr = DocumentManager::with_data_dir(pool.clone(), data_temp.path().to_path_buf())
        .expect("Failed to init DocumentManager");
    let extractor = DocumentExtractorService::new();
    let relation_mgr = RelationManager::new(doc_mgr.clone(), extractor);

    // Text contains:
    // - Email token: maria.joao@gmail.com (should NOT match joao@gmail.com)
    // - Email token: ana@gmail.com (should match ana@gmail.com)
    // - Phone with Portugal prefix: +351 912 345 678 (should match 912345678)
    // - URL token: targetz.com (should match https://targetz.com or targetz.com)
    let text = b"Contacts:\nEmail: maria.joao@gmail.com\nSecondary: ANA@GMAIL.COM\nPhone: +351 912 345 678\nWeb: targetz.com";
    let file = create_temp_file(&source_temp, "doc.txt", text);

    let doc = doc_mgr
        .upload_documents(&[file], 1)
        .await
        .successes
        .into_iter()
        .next()
        .unwrap();

    let c_joao = make_customer(1, "Joao", "joao@gmail.com", "", None, None);
    let c_ana = make_customer(2, "Ana", "ana@gmail.com", "", None, None);
    let c_carlos = make_customer(3, "Carlos", "", "912345678", None, None);
    let c_site = make_customer(4, "Site", "", "", Some("targetz.com"), None);
    let c_empty = make_customer(5, "Empty", "", "", None, None);

    let customers = vec![
        c_joao.clone(),
        c_ana.clone(),
        c_carlos.clone(),
        c_site.clone(),
        c_empty.clone(),
    ];

    let relations_res = relation_mgr
        .get_relations(vec![doc.id], customers)
        .await
        .expect("get_relations failed");

    let matched = &relations_res.relations[&doc.id];
    let matched_ids: Vec<i64> = matched.iter().map(|c| c.id).collect();

    // c_joao must NOT match (maria.joao@gmail.com is distinct from joao@gmail.com)
    assert!(
        !matched_ids.contains(&1),
        "joao@gmail.com should not match inside maria.joao@gmail.com"
    );

    // c_ana matches (case-insensitive email)
    assert!(
        matched_ids.contains(&2),
        "ana@gmail.com should match ANA@GMAIL.COM"
    );

    // c_carlos matches (+351 912 345 678 normalized matches 912345678)
    assert!(
        matched_ids.contains(&3),
        "912345678 should match +351 912 345 678"
    );

    // c_site matches URL
    assert!(matched_ids.contains(&4), "targetz.com should match URL");

    // c_empty must NOT match
    assert!(
        !matched_ids.contains(&5),
        "Empty customer identifiers should never match"
    );
}

#[tokio::test]
async fn test_matching_determinism_across_runs() {
    let pool = setup_test_db().await;
    let data_temp = tempdir().unwrap();
    let source_temp = tempdir().unwrap();

    sqlx::query!("INSERT INTO projects (id, name, current_version) VALUES (1, 'Project 1', '1.0')")
        .execute(&pool)
        .await
        .unwrap();

    let doc_mgr = DocumentManager::with_data_dir(pool.clone(), data_temp.path().to_path_buf())
        .expect("Failed to init DocumentManager");
    let extractor = DocumentExtractorService::new();
    let relation_mgr = RelationManager::new(doc_mgr.clone(), extractor);

    let text = b"Customer 1: cust1@example.com, Customer 2: +351 922 222 222, Customer 3: https://cust3.org";
    let file = create_temp_file(&source_temp, "multi.txt", text);

    let doc = doc_mgr
        .upload_documents(&[file], 1)
        .await
        .successes
        .into_iter()
        .next()
        .unwrap();

    let mut customers = Vec::new();
    for i in 1..=50 {
        customers.push(make_customer(
            i,
            &format!("Cust {}", i),
            &format!("cust{}@example.com", i),
            &format!("9222222{:02}", i % 100),
            Some(&format!("https://cust{}.org", i)),
            None,
        ));
    }

    // Run multiple times
    let run1 = relation_mgr
        .get_relations(vec![doc.id], customers.clone())
        .await
        .unwrap();
    let run2 = relation_mgr
        .get_relations(vec![doc.id], customers.clone())
        .await
        .unwrap();

    let ids1: Vec<i64> = run1.relations[&doc.id].iter().map(|c| c.id).collect();
    let ids2: Vec<i64> = run2.relations[&doc.id].iter().map(|c| c.id).collect();

    assert_eq!(ids1, ids2, "Customer matching must be 100% deterministic");
}
