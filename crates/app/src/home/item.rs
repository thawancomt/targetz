use gpui_kit::{component::IconName, SharedString};
use shared::{customer::Customer, project::Project};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub enum ItemCategory {
    Customers,
    Projects,
    Actions,
}

impl ItemCategory {
    pub fn title(&self) -> &'static str {
        match self {
            ItemCategory::Customers => "Customers",
            ItemCategory::Projects => "Projects",
            ItemCategory::Actions => "Actions",
        }
    }

    pub fn icon(&self) -> IconName {
        match self {
            ItemCategory::Customers => IconName::User,
            ItemCategory::Projects => IconName::Folder,
            ItemCategory::Actions => IconName::SquareTerminal,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum QuickAction {
    CreateCustomer,
    CreateProject,
    ToggleTheme,
    OpenSettings,
}

impl QuickAction {
    pub fn all() -> Vec<Self> {
        vec![
            QuickAction::CreateCustomer,
            QuickAction::CreateProject,
            QuickAction::ToggleTheme,
            QuickAction::OpenSettings,
        ]
    }

    pub fn title(&self) -> &'static str {
        match self {
            QuickAction::CreateCustomer => "Create New Customer",
            QuickAction::CreateProject => "Create New Project",
            QuickAction::ToggleTheme => "Toggle Color Theme",
            QuickAction::OpenSettings => "Open Settings",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            QuickAction::CreateCustomer => "Add a client or lead to Targetz",
            QuickAction::CreateProject => "Initialize a new project and roadmap",
            QuickAction::ToggleTheme => "Switch between Dark and Light mode",
            QuickAction::OpenSettings => "Configure application preferences",
        }
    }

    pub fn icon(&self) -> IconName {
        match self {
            QuickAction::CreateCustomer => IconName::Plus,
            QuickAction::CreateProject => IconName::Folder,
            QuickAction::ToggleTheme => IconName::Sun,
            QuickAction::OpenSettings => IconName::Settings,
        }
    }
}

#[derive(Clone)]
pub enum ItemKind {
    Customer(Customer),
    Project(Project),
    Action(QuickAction),
}

#[derive(Clone)]
pub struct SearchItem {
    pub id: String,
    pub title: SharedString,
    pub subtitle: Option<SharedString>,
    /// Searchable aggregate string containing all text data
    /// (e.g., email, phone number, address, website, codename, description, etc.)
    pub content: String,
    pub icon: IconName,
    pub kind: ItemKind,
    pub category: ItemCategory,
}

impl SearchItem {
    pub fn from_customer(customer: Customer) -> Self {
        let mut content_parts = vec![
            customer.name.clone(),
            customer.email.clone(),
            customer.phone_number.clone(),
        ];
        if let Some(addr) = &customer.address {
            content_parts.push(addr.clone());
        }
        if let Some(site) = &customer.site_url {
            content_parts.push(site.clone());
        }
        if let Some(ig) = &customer.instagram_url {
            content_parts.push(ig.clone());
        }

        let subtitle = format!("{} • {}", customer.email, customer.phone_number);

        Self {
            id: format!("customer-{}", customer.id),
            title: customer.name.clone().into(),
            subtitle: Some(subtitle.into()),
            content: content_parts.join(" "),
            icon: IconName::User,
            kind: ItemKind::Customer(customer),
            category: ItemCategory::Customers,
        }
    }

    pub fn from_project(project: Project) -> Self {
        let mut content_parts = vec![
            project.name.clone(),
            project.status.as_str().to_string(),
            format!("v{}", project.current_version),
        ];
        if let Some(code) = &project.codename {
            content_parts.push(code.clone());
        }
        if let Some(desc) = &project.description {
            content_parts.push(desc.clone());
        }
        if let Some(site) = &project.site_url {
            content_parts.push(site.clone());
        }

        let subtitle = format!(
            "v{} • Status: {}",
            project.current_version,
            project.status.as_str()
        );

        Self {
            id: format!("project-{}", project.id),
            title: project.name.clone().into(),
            subtitle: Some(subtitle.into()),
            content: content_parts.join(" "),
            icon: IconName::Folder,
            kind: ItemKind::Project(project),
            category: ItemCategory::Projects,
        }
    }

    pub fn from_action(action: QuickAction) -> Self {
        let content = format!(
            "{} {} action command shortcut",
            action.title(),
            action.description()
        );

        Self {
            id: format!("action-{}", action.title().to_lowercase().replace(' ', "-")),
            title: action.title().into(),
            subtitle: Some(action.description().into()),
            content,
            icon: action.icon(),
            kind: ItemKind::Action(action),
            category: ItemCategory::Actions,
        }
    }

    pub fn matches(&self, query: &str) -> bool {
        let q = query.trim().to_lowercase();
        if q.is_empty() {
            return true;
        }
        self.title.to_lowercase().contains(&q)
            || self.content.to_lowercase().contains(&q)
            || self
                .subtitle
                .as_ref()
                .map_or(false, |s| s.to_lowercase().contains(&q))
    }
}
