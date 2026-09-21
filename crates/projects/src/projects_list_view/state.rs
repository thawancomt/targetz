use gpui_kit::{
    App, AppContext, Context, Entity, EventEmitter, ParentElement, Styled, Window,
    component::WindowExt, px,
};
use shared::project::Project;

use crate::{
    project_form_view::{CreateProjectView, events::CreateProjectEvents},
    projects_list_view::{
        components::{events::ProjectItemEvents, project_item::ProjectItem},
        events::ProjectListEvents,
    },
};

pub struct ProjectsListView {
    pub projects: Vec<Project>,
    pub project_views: Vec<Entity<ProjectItem>>,
    pub create_project_view: Entity<CreateProjectView>,
    pub open_projects: Vec<Project>,

    // tab
    pub active_project: Option<Project>,
}

impl EventEmitter<ProjectListEvents> for ProjectsListView {}

impl ProjectsListView {
    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }

    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let create_project_view = CreateProjectView::view(window, cx);

        //Observe the creation of the form
        Self::observe_form_events(&create_project_view.clone(), cx);

        Self {
            projects: Vec::new(),
            create_project_view,
            open_projects: Vec::new(),
            project_views: Vec::new(),
            active_project: None,
        }
    }

    pub fn observe_form_events(view: &Entity<CreateProjectView>, cx: &mut Context<Self>) {
        cx.subscribe(&view, |this, _, event, cx| {
            match event {
                CreateProjectEvents::CreatedProject(project) => {
                    this.projects.push(project.clone());
                    cx.notify();
                }
            };
        })
        .detach();
    }

    pub fn observe_items_events(view: &Entity<Self>, cx: &mut Context<Self>) {
        cx.subscribe(&view, |_this, _, _event, _cx| {}).detach();
    }

    pub fn with_projects(&mut self, projects: Vec<Project>, cx: &mut Context<Self>) {
        let project_item = |project: Project, cx: &mut gpui_kit::prelude::Context<Self>| {
            ProjectItem::new(project, cx)
        };

        let views: Vec<Entity<ProjectItem>> = projects
            .iter()
            .map(|p| project_item(p.clone(), cx))
            .collect();

        for view in views.clone() {
            let view_handle = view.clone();
            cx.subscribe(&view, move |list_view, _emitter, event, list_context| {
                match event {
                    ProjectItemEvents::OpenProject(project) => {
                        view_handle.update(list_context, |_this, _cx| {
                            list_view.open_projects.push(project.clone());
                            list_view.active_project = Some(project.clone());
                        });
                    }
                    _ => {}
                };
                list_context.notify();
            })
            .detach();
        }

        self.projects = projects.clone();
        self.project_views = views;
    }

    pub fn open_create_project_dilaog(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let view = self.create_project_view.clone();
        let h = window.viewport_size().height.clone();

        // emit the opening
        cx.emit(ProjectListEvents::OpenFormView);

        window.open_dialog(cx, move |dialog, _window, _cx| {
            dialog
                .title("Create a new projet")
                .child(view.clone())
                .w(px(720.))
                .h(h)
        });

        cx.notify();
    }
}
