use shared::customer::{Customer, Persisted};

#[derive(Debug, Clone)]
pub enum CustomerUpdateEvent {
    UpdatedCustomer(Customer<Persisted>),
}
