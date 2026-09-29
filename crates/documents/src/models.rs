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
