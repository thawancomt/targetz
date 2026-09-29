use gpui_kit::{
    App, AppContext, Context, Entity, EventEmitter, SharedString, Window,
    component::{
        date_picker::DatePickerState,
        input::InputState,
        select::{SelectItem, SelectState},
    },
};
use shared::{
    customer::Customer,
    db::DbPool,
    form_utils::{date_input, select, text_input},
    project::{ProjectDraft, ProjectStatus},
};

use crate::{project_form_view::events::CreateProjectEvent, project_repository::ProjectRepository};

pub struct CreateProjectView {
    pub(super) name: Entity<InputState>,
    pub(super) description: Entity<InputState>,
    pub(super) site_url: Entity<InputState>,
    pub(super) codename: Entity<InputState>,
    pub(super) version: Entity<InputState>,
    pub(super) budget: Entity<InputState>,
    pub(super) status: Entity<SelectState<Vec<&'static str>>>,
    pub(super) owner: Entity<SelectState<Vec<OwnerOptions>>>,

    pub(super) start_date: Entity<DatePickerState>,
    pub(super) target_deadline: Entity<DatePickerState>,

    pub repository: ProjectRepository,
    pub(super) customers: Option<Vec<Customer>>,
}

pub fn parse_budget(value: String) -> f64 {
    match value.parse::<f64>() {
        Ok(budget_float) => budget_float,
        Err(_) => 0.,
    }
}

#[derive(Clone)]
pub struct OwnerOptions {
    title: SharedString,
    value: i64,
}

impl SelectItem for OwnerOptions {
    fn title(&self) -> SharedString {
        self.title.clone()
    }
    fn value(&self) -> &Self::Value {
        &self.value
    }
    type Value = i64;
}

impl CreateProjectView {
    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }

    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let pool = cx.global::<DbPool>().0.clone();

        let name = cx.new(|cx| InputState::new(window, cx));
        cx.observe(&name, |_, _, cx| cx.notify()).detach();

        let status_options = vec![
            ProjectStatus::Started.as_str(),
            ProjectStatus::Prospecting.as_str(),
        ];

        let owner_options = Vec::new();

        let repository = ProjectRepository::new(pool);

        Self {
            name,
            description: cx.new(|cx| InputState::new(window, cx)),
            site_url: cx.new(|cx| InputState::new(window, cx)),
            codename: cx.new(|cx| InputState::new(window, cx)),
            version: cx.new(|cx| InputState::new(window, cx)),
            budget: cx.new(|cx| InputState::new(window, cx)),
            start_date: cx.new(|cx| DatePickerState::new(window, cx)),
            target_deadline: cx.new(|cx| DatePickerState::new(window, cx)),
            status: cx.new(|cx| SelectState::new(status_options, None, window, cx)),
            owner: cx.new(|cx| SelectState::new(owner_options, None, window, cx)),
            customers: None,
            repository,
        }
    }

    pub fn with_customers(
        &mut self,
        customers: Vec<Customer>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.customers = Some(customers.clone());

        let customers_options: Vec<OwnerOptions> = customers
            .into_iter()
            .map(|c| OwnerOptions {
                title: c.name.clone().into(),
                value: c.id,
            })
            .collect();

        self.owner.update(cx, |this, cx| {
            this.set_items(customers_options, window, cx);
        });
    }

    pub fn create_project(&self, cx: &mut Context<Self>) {
        let status = match select(&self.status, cx) {
            Some(s) => ProjectStatus::from(s),
            _ => ProjectStatus::Started,
        };

        let project_draft: ProjectDraft = ProjectDraft {
            name: text_input(&self.name, cx).unwrap_or("default".to_string()),
            current_version: text_input(&self.version, cx).unwrap_or("0.1".to_string()),
            status,
            description: text_input(&self.description, cx),
            start_date: date_input(&self.start_date, cx),
            site_url: text_input(&self.name, cx),
            codename: text_input(&self.codename, cx),
            target_deadline: date_input(&self.target_deadline, cx),
            budget: Some(parse_budget(
                text_input(&self.budget, cx).unwrap_or("0.".to_string()),
            )),
        };

        let repository = self.repository.clone();

        cx.spawn(async move |this, cx| {
            match repository.create_project(project_draft).await {
                Ok(new_project) => {
                    println!("new project created {}", new_project.name);

                    let _ = this.update(cx, |_, cx| {
                        cx.emit(CreateProjectEvent::CreatedProject(new_project));
                        cx.notify();
                    });
                }
                Err(e) => {
                    eprintln!("{e}")
                }
            };
        })
        .detach();
    }
}

impl EventEmitter<CreateProjectEvent> for CreateProjectView {}
