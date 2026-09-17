use std::fmt;
pub mod customer;
pub mod db;
pub mod events;
pub mod theme;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AppTab {
    Targetz,
    Home,
    Settings,
    CreateCustomer,
}

impl fmt::Display for AppTab {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppTab::Home => writeln!(f, "Homepage"),
            AppTab::Settings => writeln!(f, "Settings"),
            AppTab::Targetz => writeln!(f, "Targetz"),
            AppTab::CreateCustomer => writeln!(f, "Create Customer"),
        }
    }
}

impl AppTab {
    pub fn as_str(&self) -> &str {
        match self {
            AppTab::CreateCustomer => "Create customer",
            AppTab::Home => "Homepage",
            AppTab::Settings => "Settings",
            AppTab::Targetz => "Targetz",
        }
    }
}
