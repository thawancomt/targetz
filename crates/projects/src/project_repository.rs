use shared::{
    app_errors::AppRepositoryError,
    customer::{Customer, Persisted, Stakeholder},
    project::{Project, ProjectDraft, ProjectHistoryRow, ProjectRow, ProjectStatus},
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
    pub async fn update_status(
        &self,
        project: Project,
        status: ProjectStatus,
    ) -> Result<(Project<Persisted>, ProjectHistoryRow), AppRepositoryError> {
        let project_snapshot = project.clone();

        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|e| AppRepositoryError::FailedToCreate(e.to_string()))?;

        let updated_project = sqlx::query_as!(
            Project::<Persisted>,
            r#"
                UPDATE projects
                SET status = ?
                WHERE projects.id = ?
                RETURNING *
            "#,
            status.as_str(),
            &project.id
        )
        .fetch_one(transaction.as_mut())
        .await
        .map_err(|e| AppRepositoryError::FailedToFetch(e.to_string()))?;

        let new_history = sqlx::query_as!(
            ProjectHistoryRow,
            r#"
                INSERT INTO project_status_history (
                    from_status, to_status, project_id
                )
                VALUES (?,?,?)

                RETURNING *
            "#,
            &project_snapshot.status.as_str(),
            &status.as_str(),
            &project.id
        )
        .fetch_one(transaction.as_mut())
        .await
        .map_err(|e| AppRepositoryError::FailedToCreatePivot(e.to_string()))?;

        transaction
            .commit()
            .await
            .map_err(|e| AppRepositoryError::FailedToCreate(e.to_string()))?;

        Ok((updated_project, new_history))
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

    pub async fn get_project_history(
        &self,
        project_id: i64,
    ) -> Result<Vec<ProjectHistoryRow>, AppRepositoryError> {
        let result = sqlx::query_as!(
            ProjectHistoryRow,
            r#"
                SELECT * FROM project_status_history p WHERE p.project_id = ?
            "#,
            &project_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppRepositoryError::FailedToCreate(e.to_string()))?;

        Ok(result)
    }

    pub async fn get_stakeholders(
        &self,
        project_id: i64,
    ) -> Result<Vec<Stakeholder>, AppRepositoryError> {
        let result = sqlx::query_as!(
            Stakeholder,
            r#"
                SELECT c.*, role
                    FROM project_customer pc
                JOIN customers c
                    ON pc.customer_id = c.id

                WHERE pc.project_id = ?
            "#,
            &project_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppRepositoryError::FailedToFetch(e.to_string()))?;

        Ok(result)
    }
}
