use std::collections::HashSet;

use crate::models::{DocumentWithCustomers, ProjectDocumentStats};
use gpui_kit::{
    App, AppContext, Context, Entity, EventEmitter, ParentElement, Render, Styled, Window,
    base::StyledExt,
    component::{ActiveTheme, IconName, button::Button},
    div,
};

pub mod events {
    use super::*;
    pub enum DocumentListEvents {
        OpenProject(ProjectDocumentStats),
    }
}

pub struct ProjectDocumentsListView {
    pub projects: Vec<ProjectDocumentStats>,
}

impl EventEmitter<events::DocumentListEvents> for ProjectDocumentsListView {}

impl ProjectDocumentsListView {
    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }

    pub fn new(_window: &mut Window, _cx: &mut Context<Self>) -> Self {
        Self {
            projects: Vec::new(),
        }
    }

    pub fn with_projects(&mut self, projects: Vec<ProjectDocumentStats>, cx: &mut Context<Self>) {
        self.projects = projects;
        cx.notify();
    }

    /// Refreshes the document and (distinct) customer counts of `project_id`
    /// from its up-to-date `documents`.
    pub fn update_project_documents(
        &mut self,
        project_id: i64,
        documents: &[DocumentWithCustomers],
        cx: &mut Context<Self>,
    ) {
        let Some(project) = self
            .projects
            .iter_mut()
            .find(|p| p.project_id == project_id)
        else {
            return;
        };

        let customers: HashSet<&str> = documents.iter().flat_map(|d| d.customer_id_set()).collect();

        project.document_count = documents.len() as i64;
        project.customer_count = customers.len() as i64;
        cx.notify();
    }
}

impl Render for ProjectDocumentsListView {
    fn render(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl gpui_kit::prelude::IntoElement {
        let projects = self.projects.clone();
        let _theme = cx.theme().clone();

        div()
            .v_flex()
            .w_full()
            .h_full()
            .p_4()
            .gap_4()
            .child(div().child("Projects Documents").text_xl())
            .children(projects.into_iter().map(|stats| {
                let p = stats.clone();
                Button::new(format!("doc-card-{}", stats.project_id))
                    .w_full()
                    .p_4()
                    .gap_2()
                    .min_h_20()
                    .on_click(cx.listener(move |_this, _, _window, cx| {
                        cx.emit(events::DocumentListEvents::OpenProject(p.clone()));
                    }))
                    .child(
                        div()
                            .h_flex()
                            .justify_between()
                            .w_full()
                            .child(div().child(stats.project_name.clone()))
                            .child(
                                div()
                                    .h_flex()
                                    .gap_4()
                                    .child(
                                        div()
                                            .h_flex()
                                            .gap_2()
                                            .child(IconName::FileText)
                                            .child(format!("{} docs", stats.document_count)),
                                    )
                                    .child(
                                        div()
                                            .h_flex()
                                            .gap_2()
                                            .child(IconName::User)
                                            .child(format!("{} customers", stats.customer_count)),
                                    ),
                            ),
                    )
            }))
    }
}
