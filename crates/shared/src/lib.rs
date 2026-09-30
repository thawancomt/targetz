use std::fmt;
pub mod app_errors;
pub mod app_form;
pub mod customer;
pub mod customer_interaction;
pub mod db;
pub mod events;
pub mod form_utils;
pub mod project;
pub mod theme;
pub mod ui;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AppTab {
    Targetz,
    Home,
    Settings,
    CreateCustomer,
    Projects,
    Documents,
}

impl fmt::Display for AppTab {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppTab::Home => writeln!(f, "Homepage"),
            AppTab::Settings => writeln!(f, "Settings"),
            AppTab::Targetz => writeln!(f, "Targetz"),
            AppTab::CreateCustomer => writeln!(f, "Create Customer"),
            AppTab::Projects => writeln!(f, "Projects"),
            AppTab::Documents => writeln!(f, "Documents"),
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
            AppTab::Projects => "Projects",
            AppTab::Documents => "Documents",
        }
    }
}
