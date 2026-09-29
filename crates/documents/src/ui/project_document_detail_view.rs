use std::path::PathBuf;

use crate::{
    DocumentManager, document_manager,
    models::{DocumentWithCustomers, ProjectDocumentStats},
};
use gpui_kit::{
    App, AppContext, Context, Entity, ParentElement, Render, Styled, Window,
    base::StyledExt,
    component::{
        ActiveTheme, IconName, WindowExt,
        button::{Button, ButtonVariants},
    },
    div,
};
use rfd::FileDialog;
use shared::db::DbPool;

pub struct ProjectDocumentDetailView {
    pub project_stats: Option<ProjectDocumentStats>,
    pub documents: Vec<DocumentWithCustomers>,
}

impl ProjectDocumentDetailView {
    pub fn view(
        window: &mut Window,
        cx: &mut App,
        project_stats: Option<ProjectDocumentStats>,
    ) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx, project_stats))
    }

    pub fn new(
        _window: &mut Window,
        _cx: &mut Context<Self>,
        project_stats: Option<ProjectDocumentStats>,
    ) -> Self {
        Self {
            project_stats,
            documents: Vec::new(),
        }
    }

    pub fn set_project(
        &mut self,
        stats: ProjectDocumentStats,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.project_stats = Some(stats.clone());

        let pool = cx.global::<shared::db::DbPool>().0.clone();
        let project_id = stats.project_id;

        cx.spawn(async move |this, cx| {
            if let Ok(manager) = crate::document_manager::DocumentManager::new(pool) {
                if let Ok(docs) = manager.get_project_documents(project_id).await {
                    let _ = this.update(cx, |view, cx| {
                        view.set_documents(docs, cx);
                    });
                }
            }
        })
        .detach();

        cx.notify();
    }

    pub fn set_documents(&mut self, documents: Vec<DocumentWithCustomers>, cx: &mut Context<Self>) {
        self.documents = documents;
        cx.notify();
    }

    pub fn upload_documents(
        &mut self,
        doc_paths: Vec<PathBuf>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let pool = cx.global::<DbPool>().0.clone();

        let document_manager = DocumentManager::new(pool);

        let Some(project) = self.project_stats.clone() else {
            println!(
                "Tried to upload documentation but no  project is settled thougt the self.project_stats (it should)"
            );
            return;
        };

        match document_manager {
            Ok(service) => {
                cx.spawn_in(window, async move |this, cx| {
                    let upload_result = service
                        .upload_documents(&doc_paths, project.project_id)
                        .await;

                    if upload_result.successes.len() == doc_paths.len() {
                        let result = service.get_project_documents(project.project_id).await;
                        let _ = this.update_in(cx, |this, window, cx| {
                            match result {
                                Ok(documents) => {
                                    this.documents = documents;
                                    window.push_notification("Document uploaded", cx);
                                    cx.notify();
                                }
                                Err(e) => {
                                    eprintln!("{e}")
                                }
                            };
                            cx.notify();
                        });
                    }
                })
                .detach();
            }
            Err(e) => {
                eprintln!("{}", e)
            }
        }
    }
}

impl Render for ProjectDocumentDetailView {
    fn render(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl gpui_kit::prelude::IntoElement {
        let stats = self.project_stats.clone();
        let theme = cx.theme().clone();

        div()
            .v_flex()
            .w_full()
            .h_full()
            .p_4()
            .gap_4()
            .child(
                div()
                    .h_flex()
                    .justify_between()
                    .w_full()
                    .child(
                        div()
                            .v_flex()
                            .gap_2()
                            .child(
                                div()
                                    .child(
                                        stats.map_or("Unknown Project".to_string(), |s| {
                                            s.project_name
                                        }),
                                    )
                                    .text_2xl(),
                            )
                            .child(
                                div()
                                    .child("Documents details and associated customers.")
                                    .text_color(theme.muted),
                            ),
                    )
                    .child(
                        Button::new("upload-doc-btn")
                            .primary()
                            .label("Upload Document")
                            .child(IconName::Plus)
                            .on_click(cx.listener(|this, _, window, cx| {
                                cx.spawn_in(window, async move |this, cx| {
                                    let files = rfd::AsyncFileDialog::new()
                                        .add_filter("Documents", &["pdf", "txt", "html"])
                                        .pick_files()
                                        .await;

                                    if let Some(entries) = files {
                                        if entries.len() == 0 {
                                            return;
                                        }

                                        let paths: Vec<_> = entries
                                            .into_iter()
                                            .map(|f| f.path().to_path_buf())
                                            .collect();

                                        let _ = this.update_in(cx, |this, window, cx| {
                                            this.upload_documents(paths, window, cx);
                                        });
                                    }
                                })
                                .detach();
                            })),
                    ),
            )
            .children(self.documents.iter().map(|doc| {
                div()
                    .w_full()
                    .p_4()
                    .border_1()
                    .border_color(theme.border)
                    .rounded_md()
                    .child(div().child(doc.original_name.clone()))
            }))
    }
}
