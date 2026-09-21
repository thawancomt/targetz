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
    project::{ProjectDraft, ProjectStatus},
};

use crate::{
    project_form_view::events::CreateProjectEvents, project_repository::ProjectRepository,
};

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

pub(super) fn text(input: &Entity<InputState>, cx: &App) -> String {
    input.read(cx).text().to_string()
}

pub(super) fn date(input: &Entity<DatePickerState>, cx: &App) -> String {
    input.read(cx).date().to_string()
}

pub(super) fn select(input: &Entity<SelectState<Vec<&'static str>>>, cx: &App) -> String {
    match input.read(cx).selected_value() {
        Some(value) => value.to_string(),
        None => String::new(),
    }
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
        let project_draft: ProjectDraft = ProjectDraft {
            name: text(&self.name, cx),
            current_version: text(&self.version, cx),
            status: ProjectStatus::from(select(&self.status, cx)),
            description: Some(text(&self.description, cx)),
            start_date: Some(date(&self.start_date, cx)),
            site_url: Some(text(&self.name, cx)),
            codename: Some(text(&self.codename, cx)),
            target_deadline: Some(date(&self.target_deadline, cx)),
            budget: Some(parse_budget(text(&self.budget, cx))),
        };

        let repository = self.repository.clone();

        cx.spawn(async move |this, cx| {
            match repository.create_project(project_draft).await {
                Ok(new_project) => {
                    println!("new project created {}", new_project.name);

                    let _ = this.update(cx, |_, cx| {
                        cx.emit(CreateProjectEvents::CreatedProject(new_project));
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

impl EventEmitter<CreateProjectEvents> for CreateProjectView {}
