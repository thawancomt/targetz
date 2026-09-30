use shared::{
    app_errors::AppRepositoryError,
    customer::{Customer, Persisted, Stakeholder},
    project::{Project, ProjectDraft, ProjectHistoryRow, ProjectRow, ProjectStatus},
};
use sqlx::{Pool, QueryBuilder, Sqlite};

/// A project a customer is a stakeholder of, with the customer's role in it.
#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
pub struct CustomerProject {
    pub project_id: i64,
    pub project_name: String,
    pub status: String,
    pub role: String,
}

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
    pub async fn link_stakeholders(
        &self,
        customers: Vec<Stakeholder>,
        project_id: i64,
    ) -> Result<(), AppRepositoryError> {
        let mut query: QueryBuilder<Sqlite> = sqlx::QueryBuilder::new(
            r#"
                INSERT INTO project_customer (project_id, customer_id, role)
            "#,
        );

        query.push_values(customers, |mut b, stakeholder| {
            println!("{}", stakeholder.role.as_str());
            b.push_bind(project_id)
                .push_bind(stakeholder.id)
                .push_bind(stakeholder.role.as_str());
        });

        query.push(
            " ON CONFLICT (project_id, customer_id) DO UPDATE SET \
                 role = excluded.role",
        );

        let result = query
            .build()
            .execute(&self.pool)
            .await
            .map_err(|e| AppRepositoryError::FailedToCreatePivot(e.to_string()))?;

        println!("{}", result.rows_affected());

        Ok(())
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

    pub async fn get_projects_for_customer(
        &self,
        customer_id: i64,
    ) -> Result<Vec<CustomerProject>, AppRepositoryError> {
        sqlx::query_as::<_, CustomerProject>(
            r#"
                SELECT p.id AS project_id, p.name AS project_name, p.status AS status, pc.role AS role
                FROM project_customer pc
                JOIN projects p ON p.id = pc.project_id
                WHERE pc.customer_id = ?
                ORDER BY p.name ASC
            "#,
        )
        .bind(customer_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppRepositoryError::FailedToFetch(e.to_string()))
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

    pub async fn update_project(
        &self,
        project_id: i64,
        project_draft: ProjectDraft,
    ) -> Result<Project<Persisted>, AppRepositoryError> {
        let result = sqlx::query_as!(
            Project::<Persisted>,
            r#"
                UPDATE projects
                SET
                    name = COALESCE(?, name),
                    description = COALESCE(?, description),
                    codename = COALESCE(?, codename),
                    current_version = COALESCE(?, current_version),
                    budget = COALESCE(?, budget),
                    status = COALESCE(?, status),
                    site_url = COALESCE(?, site_url),
                    start_date = COALESCE(?, start_date),
                    target_deadline = COALESCE(?, target_deadline)
                RETURNING *
            "#,
            project_draft.name,
            project_draft.description,
            project_draft.codename,
            project_draft.current_version,
            project_draft.budget,
            project_draft.status.as_str(),
            project_draft.site_url,
            project_draft.start_date,
            project_draft.target_deadline,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppRepositoryError::FailedToUpdate(e.to_string()))?;

        Ok(result)
    }
}
