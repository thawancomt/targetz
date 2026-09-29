use shared::{customer::Customer, project::Project};

use super::item::QuickAction;

#[derive(Clone, Debug)]
pub enum HomeEvent {
    OpenCustomer(Customer),
    OpenProject(Project),
    TriggerAction(QuickAction),
}
