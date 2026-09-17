use crate::customer::Customer;

pub enum AppEvent {
    NewCustomer(Customer),
    LoadedCustomers(Vec<Customer>),
}
