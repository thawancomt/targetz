use std::collections::HashSet;

use crate::models::{DocumentWithCustomers, ProjectDocumentStats};
use gpui_kit::{
    App, AppContext, Context, Entity, EventEmitter, InteractiveElement, ParentElement, Render,
    StatefulInteractiveElement, Styled, Window,
    base::StyledExt,
    component::ActiveTheme,
    div, px,
};
use shared::ui::{caption, rail_card};

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
        let theme = cx.theme().clone();

        let stat = |label: &'static str, value: i64| {
            div()
                .v_flex()
                .gap_1()
                .items_end()
                .child(caption(label, &theme))
                .child(
                    div()
                        .child(format!("{value:02}"))
                        .font_family("Geist Mono")
                        .text_color(theme.foreground),
                )
        };

        div()
            .v_flex()
            .w_full()
            .h_full()
            .p_4()
            .gap_4()
            .child(caption("Projects documents", &theme))
            .children(projects.into_iter().map(|stats| {
                let p = stats.clone();
                rail_card(
                    false,
                        div()
                            .h_flex()
                            .flex_1()
                            .min_w_0()
                            .p_3()
                            .gap_3()
                            .justify_between()
                            .items_center()
                            .child(
                                div()
                                    .v_flex()
                                    .flex_1()
                                    .min_w_0()
                                    .gap_1()
                                    .child(caption(format!("Project #{}", stats.project_id), &theme))
                                    .child(
                                        div()
                                            .child(stats.project_name.clone())
                                            .text_lg()
                                            .text_color(theme.foreground),
                                    ),
                            )
                            .child(
                                div()
                                    .h_flex()
                                    .flex_none()
                                    .gap_6()
                                    .child(stat("Docs", stats.document_count))
                                    .child(stat("Customers", stats.customer_count)),
                            ),
                    &theme,
                )
                .id(format!("doc-card-{}", stats.project_id))
                .cursor_pointer()
                .hover(|f| f.border_color(theme.selection))
                .on_click(cx.listener(move |_this, _, _window, cx| {
                    cx.emit(events::DocumentListEvents::OpenProject(p.clone()));
                }))
            }))
    }
}
