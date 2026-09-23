use shared::customer::{Customer, Draft, Persisted};
use sqlx::{Pool, Sqlite};

// 1. Estados em nível de TIPO

#[derive(Debug, thiserror::Error)]
pub enum CustomerRepositoryError {
    #[error("Failed to create customer: {0}")]
    CreateCustomerFailed(String),

    #[error("Failed to fetch customer: {0}")]
    FetchFailed(String),

    #[error("Failed to delete customer: {0}")]
    DeleteFailed(String),

    #[error("Failed to patch customer: {0}")]
    PatchError(String),
}

// 2. Struct genérica no campo `id`

#[derive(Debug, Clone)]
pub struct CustomerRepository {
    pool: Pool<Sqlite>,
}

impl CustomerRepository {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }

    pub async fn create_customer(
        &self,
        customer: Customer<Draft>,
    ) -> Result<Customer<Persisted>, CustomerRepositoryError> {
        let result = sqlx::query_as!(
            Customer::<Persisted>,
            r#"
            INSERT INTO customers (
                name,
                email,
                address,
                phone_number,
                instagram_url,
                site_url,
                contacted,
                is_client
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING
                id as "id: Persisted",
                name,
                email,
                phone_number,
                address,
                instagram_url,
                site_url,
                is_client,
                contacted,
                created_at
            "#,
            customer.name,
            customer.email,
            customer.address,
            customer.phone_number,
            customer.instagram_url,
            customer.site_url,
            customer.contacted,
            customer.is_client,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| CustomerRepositoryError::CreateCustomerFailed(e.to_string()))?;

        Ok(result)
    }

    pub async fn get_customers(&self) -> Result<Vec<Customer<Persisted>>, CustomerRepositoryError> {
        let result = sqlx::query_as!(
            Customer::<Persisted>,
            r#"
                SELECT * from customers
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CustomerRepositoryError::FetchFailed(e.to_string()))?;

        Ok(result)
    }

    pub async fn delete_customer(&self, customer_id: i64) -> Result<(), CustomerRepositoryError> {
        sqlx::query!(
            r#"
                DELETE from customers
                WHERE customers.id = ?
            "#,
            &customer_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CustomerRepositoryError::FetchFailed(e.to_string()))?;

        Ok(())
    }

    pub async fn set_contacted(
        &self,
        customer_id: i64,
        value: bool,
    ) -> Result<Customer<Persisted>, CustomerRepositoryError> {
        let patched_customer = sqlx::query_as!(
            Customer::<Persisted>,
            r#"
                UPDATE customers
                SET contacted = ?
                WHERE customers.id = ?
                RETURNING *
            "#,
            value,
            customer_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| CustomerRepositoryError::PatchError(e.to_string()))?;

        Ok(patched_customer)
    }
    pub async fn set_is_client(
        &self,
        customer_id: i64,
        value: bool,
    ) -> Result<Customer<Persisted>, CustomerRepositoryError> {
        let patched_customer = sqlx::query_as!(
            Customer::<Persisted>,
            r#"
                UPDATE customers
                SET is_client = ?
                WHERE customers.id = ?
                RETURNING *
            "#,
            value,
            customer_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| CustomerRepositoryError::PatchError(e.to_string()))?;

        Ok(patched_customer)
    }
}
