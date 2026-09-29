use gpui_kit::{
    App, AppContext, Context, Entity, EventEmitter, Window,
    base::{Date, input::InputState},
    component::{date_picker::DatePickerState, select::SelectState},
};
use shared::{
    db::DbPool,
    form_utils::{date_input, select, text_input},
    project::{Project, ProjectDraft, ProjectStatus},
};
use sqlx::encode::IsNull::No;

use crate::{edit_form_view::events::ProjectUpdateEvent, project_repository::ProjectRepository};

pub struct ProjectUpdateView {
    pub project: Option<Project>,
    pub project_draft: Option<Project>,

    pub(super) name: Entity<InputState>,
    pub(super) description: Entity<InputState>,
    pub(super) site_url: Entity<InputState>,
    pub(super) codename: Entity<InputState>,
    pub(super) version: Entity<InputState>,
    pub(super) budget: Entity<InputState>,

    pub(super) start_date: Entity<DatePickerState>,
    pub(super) target_deadline: Entity<DatePickerState>,
}

pub fn parse_budget(value: String) -> f64 {
    match value.parse::<f64>() {
        Ok(budget_float) => budget_float,
        Err(_) => 0.,
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ProjectUpdateError {
    #[error("Missing field or empty value that is invalid for this field: {0} message : {1}")]
    InvalidField(String, String),
}

impl ProjectUpdateView {
    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }

    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            project: None,
            project_draft: None,
            name: cx.new(|cx| InputState::new(window, cx)),
            description: cx.new(|cx| InputState::new(window, cx)),
            site_url: cx.new(|cx| InputState::new(window, cx)),
            codename: cx.new(|cx| InputState::new(window, cx)),
            version: cx.new(|cx| InputState::new(window, cx)),
            budget: cx.new(|cx| InputState::new(window, cx)),
            start_date: cx.new(|cx| DatePickerState::new(window, cx)),
            target_deadline: cx.new(|cx| DatePickerState::new(window, cx)),
        }
    }

    pub fn set_project(&mut self, project: Project, window: &mut Window, cx: &mut App) {
        self.name.update(cx, |input, cx| {
            input.set_value(&project.name, window, cx);
        });

        self.description.update(cx, |input, cx| {
            input.set_value(
                project.description.as_deref().unwrap_or_default(),
                window,
                cx,
            );
        });

        self.site_url.update(cx, |input, cx| {
            input.set_value(project.site_url.as_deref().unwrap_or_default(), window, cx);
        });

        self.codename.update(cx, |input, cx| {
            input.set_value(project.codename.as_deref().unwrap_or_default(), window, cx);
        });

        self.version.update(cx, |input, cx| {
            input.set_value(&project.current_version, window, cx);
        });

        self.budget.update(cx, |input, cx| {
            let budget_str = project.budget.map(|b| b.to_string()).unwrap_or_default();
            input.set_value(&budget_str, window, cx);
        });

        self.project = Some(project.clone());
        self.project_draft = Some(project);
    }

    pub fn update_project(&mut self, cx: &mut Context<Self>) -> Result<(), ProjectUpdateError> {
        let Some(project) = self.project_draft.clone() else {
            return Err(ProjectUpdateError::InvalidField(
                "project".to_string(),
                "Project not setted".to_string(),
            ));
        };

        let Some(name) = text_input(&self.name, cx) else {
            return Err(ProjectUpdateError::InvalidField(
                "name".to_string(),
                "name must have at least 1 char".to_string(),
            ));
        };
        let Some(current_version) = text_input(&self.name, cx) else {
            return Err(ProjectUpdateError::InvalidField(
                "current_version".to_string(),
                "version must have at least 1 char".to_string(),
            ));
        };

        let project_draft: ProjectDraft = ProjectDraft {
            name,
            current_version,
            status: project.status,
            description: text_input(&self.description, cx),
            start_date: date_input(&self.start_date, cx),
            site_url: text_input(&self.name, cx),
            codename: text_input(&self.codename, cx),
            target_deadline: date_input(&self.target_deadline, cx),
            budget: Some(parse_budget(
                text_input(&self.budget, cx).unwrap_or("0.".to_string()),
            )),
        };

        let pool = cx.global::<DbPool>().0.clone();

        let repository = ProjectRepository::new(pool);
        let project_id = project.id;
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move {
                    repository.update_project(project_id, project_draft).await
                })
                .await;
            match result {
                Ok(new_project) => {
                    println!("new project created {}", new_project.name);

                    let _ = this.update(cx, |_, cx| {
                        cx.emit(ProjectUpdateEvent::UpdatedProject(new_project));
                        cx.notify();
                    });
                }
                Err(e) => {
                    eprintln!("{e}")
                }
            };
        })
        .detach();

        Ok(())
    }
}

impl EventEmitter<ProjectUpdateEvent> for ProjectUpdateView {}
