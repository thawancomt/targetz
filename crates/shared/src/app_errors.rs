#[derive(Debug, thiserror::Error)]
pub enum AppRepositoryError {
    #[error("Failed to create entity {0}")]
    FailedToCreate(String),
    #[error("Failed to create the record on the pivot table {0}")]
    FailedToCreatePivot(String),

    #[error("Failed to delete entity {0} : {1}")]
    FailedToDelete(String, String),

    #[error("Failed to fetch data entity:  {0}")]
    FailedToFetch(String),
}
