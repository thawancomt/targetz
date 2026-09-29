use sqlx::prelude::FromRow;

#[derive(Debug, Clone, PartialEq, Eq, FromRow)]
pub struct Document {
    pub id: i64,
    pub hash: String,
    pub extension: String,
    pub mtime: i64,
    pub original_name: String,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, FromRow)]
pub struct ProjectDocument {
    pub project_id: i64,
    pub document_id: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, FromRow)]
pub struct ProjectDocumentCustomer {
    pub project_id: i64,
    pub customer_id: i64,
    pub document_id: i64,
    pub status: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CustomerDocumentStatus {
    NotConfirmed,
    Confirmed,
}

impl CustomerDocumentStatus {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::NotConfirmed => "not_confirmed",
            Self::Confirmed => "confirmed",
        }
    }
}

impl TryFrom<&str> for CustomerDocumentStatus {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "not_confirmed" => Ok(Self::NotConfirmed),
            "confirmed" => Ok(Self::Confirmed),
            other => Err(format!("Invalid customer document status: {}", other)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, FromRow)]
pub struct ProjectDocumentStats {
    pub project_id: i64,
    pub project_name: String,
    pub document_count: i64,
    pub customer_count: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, FromRow)]
pub struct DocumentWithCustomers {
    pub id: i64,
    pub hash: String,
    pub extension: String,
    pub mtime: i64,
    pub original_name: String,
    pub path: String,
    // Comma-separated list of customer IDs, could be parsed as needed
    pub customer_ids: Option<String>,
}

impl DocumentWithCustomers {
    /// Distinct customer ids parsed from the comma-separated `customer_ids`.
    pub fn customer_id_set(&self) -> std::collections::HashSet<&str> {
        self.customer_ids
            .as_deref()
            .into_iter()
            .flat_map(|ids| ids.split(','))
            .map(str::trim)
            .filter(|id| !id.is_empty())
            .collect()
    }

    pub fn customer_count(&self) -> usize {
        self.customer_id_set().len()
    }
}
