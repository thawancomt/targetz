use sqlx::prelude::FromRow;

use crate::customer::Persisted;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectStatus {
    Started,
    Finished,
    Prospecting,
    Propousing,
    Refactoring,
}

impl ProjectStatus {
    pub const fn as_str(&self) -> &'static str {
        match self {
            ProjectStatus::Finished => "Finished",
            ProjectStatus::Propousing => "Propousing",
            ProjectStatus::Started => "Started",
            ProjectStatus::Refactoring => "Refactoring",
            ProjectStatus::Prospecting => "Prospecting",
        }
    }
}

impl From<String> for ProjectStatus {
    fn from(value: String) -> Self {
        match value.as_str() {
            "Finished" => ProjectStatus::Finished,
            "Propousing" => ProjectStatus::Propousing,
            "Started" => ProjectStatus::Started,
            "Refactoring" => ProjectStatus::Refactoring,
            "Prospecting" => ProjectStatus::Prospecting,
            _ => ProjectStatus::Started,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Project<State = i64> {
    pub id: State,
    pub name: String,
    pub current_version: String,
    pub created_at: String,
    pub status: ProjectStatus,
    pub description: Option<String>,
    pub start_date: Option<String>,
    pub site_url: Option<String>,
    pub codename: Option<String>,
    pub target_deadline: Option<String>,
    pub updated_at: Option<String>,
    pub budget: Option<f64>,
}

pub struct ProjectDraft {
    pub name: String,
    pub current_version: String,
    pub status: ProjectStatus,
    pub description: Option<String>,
    pub start_date: Option<String>,
    pub site_url: Option<String>,
    pub codename: Option<String>,
    pub target_deadline: Option<String>,
    pub budget: Option<f64>,
}

pub struct ProjectRow {
    pub id: i64,
    pub name: String,
    pub current_version: String,
    pub created_at: String,
    pub status: String,
    pub description: Option<String>,
    pub start_date: Option<String>,
    pub site_url: Option<String>,
    pub codename: Option<String>,
    pub target_deadline: Option<String>,
    pub updated_at: Option<String>,
    pub budget: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct ProjectHistoryRow {
    pub id: Option<i64>,
    pub from_status: String,
    pub to_status: String,
    pub project_id: i64,
    pub note: Option<String>,
    pub change_ask_by: Option<String>,
    pub reason: Option<String>,
    pub impact_on_target_deadline: Option<String>,
    pub created_at: String,
}

impl From<ProjectRow> for Project<Persisted> {
    fn from(value: ProjectRow) -> Self {
        Project {
            id: value.id,
            name: value.name,
            current_version: value.current_version,
            created_at: value.created_at,
            status: ProjectStatus::from(value.status),
            budget: value.budget,
            codename: value.codename,
            description: value.description,
            site_url: value.site_url,
            start_date: value.start_date,
            target_deadline: value.target_deadline,
            updated_at: value.updated_at,
        }
    }
}
