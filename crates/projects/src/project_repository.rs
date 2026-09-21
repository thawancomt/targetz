use shared::{
    app_errors::AppRepositoryError,
    customer::{Customer, Persisted},
    project::{Project, ProjectDraft, ProjectRow},
};
use sqlx::{Pool, Sqlite};

#[derive(Debug, Clone)]
pub struct ProjectRepository {
    pool: Pool<Sqlite>,
}

impl ProjectRepository {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool: pool.clone() }
    }

    pub async fn create_project(
        &self,
        project: ProjectDraft,
    ) -> Result<Project<Persisted>, AppRepositoryError> {
        let new_project = sqlx::query_as!(
            ProjectRow,
            r#"
                INSERT INTO projects (
                    name,
                    description,
                    codename,
                    current_version,
                    budget,
                    status,
                    site_url,
                    start_date,
                    target_deadline
                )
                VALUES (
                    $1, $2, $3, $4, $5, $6, $7, $8, $9
                )
                RETURNING *
            "#,
            project.name,
            project.description,
            project.codename,
            project.current_version,
            project.budget,
            project.status.as_str(),
            project.site_url,
            project.start_date,
            project.target_deadline,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppRepositoryError::FailedToCreate(e.to_string()))?;

        Ok(new_project.into())
    }
    pub async fn delete_project(&self, project_id: i64) -> Result<(), AppRepositoryError> {
        let _result = sqlx::query!(r#"DELETE FROM projects WHERE projects.id = ?"#, project_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AppRepositoryError::FailedToDelete(project_id.to_string(), e.to_string()));

        Ok(())
    }
    pub fn link_customer(
        &self,
        _customer: Customer,
        _project_id: i64,
    ) -> Result<(), AppRepositoryError> {
        todo!()
    }
    pub fn unlink_customer(
        &self,
        _customer: Customer,
        _project_id: i64,
    ) -> Result<(), AppRepositoryError> {
        todo!()
    }
    pub fn update_status(&self) -> Result<(), AppRepositoryError> {
        todo!()
    }

    pub async fn get_projects(&self) -> Result<Vec<Project<Persisted>>, AppRepositoryError> {
        let result = sqlx::query_as!(
            Project::<Persisted>,
            r#"
                SELECT * FROM projects
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppRepositoryError::FailedToFetch(e.to_string()))?;

        Ok(result)
    }
}
