use crate::customer::Persisted;

pub enum ProjectStatus {
    Started,
    Finished,
    Prospecting,
    Propousing,
    Refactoring,
}

impl ProjectStatus {
    pub const fn as_str(&self) -> &str {
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
        match value.to_lowercase().as_str() {
            "finished" => ProjectStatus::Finished,
            "propousing" => ProjectStatus::Finished,
            "started" => ProjectStatus::Finished,
            "refactoring" => ProjectStatus::Finished,
            _ => ProjectStatus::Started,
        }
    }
}

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
