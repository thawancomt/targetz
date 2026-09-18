use shared::{
    customer::{Customer, Draft, Persisted},
    customer_interaction::{Interaction, InteractionRow, InteractionStatus},
};
use sqlx::{Pool, Sqlite};

#[derive(Debug, Clone)]
pub struct InteractionRepository {
    pool: Pool<Sqlite>,
    pub customer: Option<Customer<Persisted>>,
}

#[derive(Debug, thiserror::Error)]
pub enum InteractionRepositoryError {
    #[error("Failed to create interaction")]
    CreateError(String),

    #[error("Failed to create the pivot record ")]
    PivotCreateError(String),
}

impl InteractionRepository {
    pub fn new(pool: Pool<Sqlite>, customer: Option<Customer<Persisted>>) -> Self {
        Self { customer, pool }
    }

    pub async fn create_interaction(
        &self,
        interaction: Interaction<Draft>,
    ) -> Result<Interaction, InteractionRepositoryError> {
        if self.customer.is_none() {
            return Err(InteractionRepositoryError::CreateError(
                "Missing customer target".to_string(),
            ));
        };

        let Interaction {
            id,
            interaction_date,
            status,
            note,
        } = interaction;

        let mut transaction = self.pool.begin().await.map_err(|e| {
            InteractionRepositoryError::CreateError(format!(
                "Failed to aquire the transaction {}",
                e.to_string()
            ))
        })?;

        let result = sqlx::query_as!(
            InteractionRow,
            r#"
                INSERT INTO interaction (
                    interaction_date,
                    status ,
                    note
                )
                VALUES (
                    ?, ?, ?
                )
                RETURNING
                    id,
                    interaction_date,
                    status ,
                    note
            "#,
            interaction_date,
            status.as_str(),
            note
        )
        .fetch_one(transaction.as_mut())
        .await
        .map_err(|e| InteractionRepositoryError::CreateError(e.to_string()))?;

        let _pivot_record = sqlx::query!(
            r#"
                INSERT INTO customer_interaction (
                    customer_id,
                    interaction_id
                )
                VALUES (?, ?)
            "#,
            result.id,
            0
        )
        .fetch_one(transaction.as_mut())
        .await
        .map_err(|e| InteractionRepositoryError::PivotCreateError(e.to_string()))?;

        transaction
            .commit()
            .await
            .map_err(|e| InteractionRepositoryError::CreateError(e.to_string()))?;

        Ok(result.into())
    }
}
