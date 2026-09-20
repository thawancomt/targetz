use shared::{
    customer::{Customer, Draft, Persisted},
    customer_interaction::{Interaction, InteractionRow},
};
use sqlx::{Pool, Sqlite};

#[derive(Debug, Clone)]
pub struct InteractionRepository {
    pool: Pool<Sqlite>,
    pub customer: Option<Customer<Persisted>>,
}

#[derive(Debug, thiserror::Error)]
pub enum InteractionRepositoryError {
    #[error("Failed to create interaction : {0}")]
    CreateError(String),

    #[error("Failed to create the pivot record  : {0}")]
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

        let customer = self.customer.clone();

        let Interaction {
            id: _,
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

        println!("new id: {}", result.id);

        let _pivot_record = sqlx::query!(
            r#"
                INSERT INTO customer_interaction (
                    customer_id,
                    interaction_id
                )
                VALUES (?, ?)
            "#,
            customer.unwrap().id,
            result.id
        )
        .fetch_optional(transaction.as_mut())
        .await
        .map_err(|e| InteractionRepositoryError::PivotCreateError(e.to_string()))?;

        transaction
            .commit()
            .await
            .map_err(|e| InteractionRepositoryError::CreateError(e.to_string()))?;
        Ok(result.into())
    }

    pub async fn get_interactions_for_customer(
        &self,
    ) -> Result<Vec<Interaction>, InteractionRepositoryError> {
        let customer = match self.customer.as_ref() {
            Some(data) => data.clone(),
            None => {
                return Err(InteractionRepositoryError::CreateError(
                    "no customer target".to_string(),
                ));
            }
        };

        let result = sqlx::query_as!(
            InteractionRow,
            r#"
                SELECT ii.*
                FROM customer_interaction ci
                INNER JOIN customers cc
                    ON ci.customer_id = cc.id
                INNER JOIN interaction ii
                    ON ci.interaction_id = ii.id

                WHERE cc.id = ?
            "#,
            customer.id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| InteractionRepositoryError::CreateError(e.to_string()))?;

        Ok(result
            .into_iter()
            .map(|interaction| interaction.into())
            .collect())
    }
}
