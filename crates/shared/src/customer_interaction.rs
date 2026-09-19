use sqlx::prelude::FromRow;

#[derive(Debug, Clone)]
pub enum InteractionStatus {
    Contacted,
    NoResponse,
    NewClient,
    Refused,
}

impl InteractionStatus {
    pub fn as_str(&self) -> &str {
        match self {
            InteractionStatus::Contacted => "Contacted",
            InteractionStatus::NewClient => "New client",
            InteractionStatus::NoResponse => "No response from customer",
            InteractionStatus::Refused => "Refused the contact or deal",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "Contacted" => Self::Contacted,
            "New client" => Self::NewClient,
            "No response from customer" => Self::NoResponse,
            "Refused the contact or deal" => Self::Refused,
            _ => Self::NoResponse,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Interaction<State = i64> {
    pub id: State,
    pub interaction_date: String,
    pub status: InteractionStatus,
    pub note: Option<String>,
}

#[derive(Debug, FromRow)]
pub struct InteractionRow {
    pub id: i64,
    pub interaction_date: String,
    pub status: String,
    pub note: Option<String>,
}

impl From<InteractionRow> for Interaction<i64> {
    fn from(row: InteractionRow) -> Self {
        Self {
            id: row.id,
            interaction_date: row.interaction_date,
            status: InteractionStatus::from_str(&row.status),
            note: row.note,
        }
    }
}
