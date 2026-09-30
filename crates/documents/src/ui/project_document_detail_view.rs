use std::path::PathBuf;

use crate::ui::components::document_item::DocumentItem;
use crate::{
    DocumentExtractorService, DocumentManager, RelationManager,
    models::{DocumentWithCustomers, ProjectDocumentStats},
};
use customers::customer_repository::CustomerRepository;
use gpui_kit::base::Disableable;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::{
    App, AppContext, Context, Entity, EventEmitter, ParentElement, Render, Styled, Window,
    base::StyledExt,
    component::{
        ActiveTheme, IconName, WindowExt,
        button::{Button, ButtonVariants},
        scroll::ScrollableElement,
    },
    div,
};
use shared::db::DbPool;
use sqlx::{Pool, Sqlite};

pub mod events {
    use super::*;
    pub enum DocumentDetailEvents {
        /// Emitted after an upload finishes, carrying the project's full, refreshed document list.
        DocumentsUploaded {
            project_id: i64,
            documents: Vec<DocumentWithCustomers>,
        },
    }
}

pub struct ProjectDocumentDetailView {
    pub project_stats: Option<ProjectDocumentStats>,
    pub documents: Vec<DocumentWithCustomers>,
    data_dir: Option<PathBuf>,
    is_uploading: bool,
    is_getting_customers: bool,
}

impl EventEmitter<events::DocumentDetailEvents> for ProjectDocumentDetailView {}

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
            data_dir: DocumentManager::default_data_dir()
                .map_err(|e| eprintln!("{e}"))
                .ok(),
            is_uploading: false,
            is_getting_customers: false,
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

    /// Re-scans every document of the current project for mentioned customers.
    ///
    /// Same discovery as after an upload, but over the documents already in the
    /// project. Existing (e.g. confirmed) relations are kept; only new ones are
    /// added. Refreshes the view and emits
    /// [`events::DocumentDetailEvents::DocumentsUploaded`] so counts update.
    pub fn get_customers(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.is_getting_customers {
            return;
        }

        let Some(project) = self.project_stats.clone() else {
            return;
        };
        let doc_ids: Vec<i64> = self.documents.iter().map(|d| d.id).collect();
        if doc_ids.is_empty() {
            window.push_notification("No documents to scan", cx);
            return;
        }

        let pool = cx.global::<DbPool>().0.clone();
        let service = match DocumentManager::new(pool.clone()) {
            Ok(service) => service,
            Err(e) => {
                eprintln!("{e}");
                return;
            }
        };

        self.is_getting_customers = true;
        cx.notify();

        cx.spawn_in(window, async move |this, cx| {
            discover_customers(&service, pool, project.project_id, doc_ids).await;

            let result = service.get_project_documents(project.project_id).await;
            let _ = this.update_in(cx, |this, window, cx| {
                this.is_getting_customers = false;
                this.apply_documents(project.project_id, result, "Customers updated", window, cx);
            });
        })
        .detach();
    }

    /// Stores freshly fetched documents, notifies the user and emits
    /// [`events::DocumentDetailEvents::DocumentsUploaded`].
    fn apply_documents(
        &mut self,
        project_id: i64,
        result: Result<Vec<DocumentWithCustomers>, sqlx::Error>,
        message: &'static str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match result {
            Ok(documents) => {
                self.documents = documents.clone();
                window.push_notification(message, cx);
                cx.emit(events::DocumentDetailEvents::DocumentsUploaded {
                    project_id,
                    documents,
                });
            }
            Err(e) => eprintln!("{e}"),
        }
        cx.notify();
    }

    /// Uploads `doc_paths` to the currently selected project.
    ///
    /// Does nothing (besides logging) if no project is selected or the
    /// [`DocumentManager`] cannot be created. The upload runs in the background;
    /// if at least one file was uploaded, customers mentioned in the new files are
    /// discovered and saved (see [`discover_customers`]), then the project's
    /// documents are re-fetched, the view is refreshed, a notification is shown and
    /// [`events::DocumentDetailEvents::DocumentsUploaded`] is emitted with the
    /// refreshed list so listeners (e.g. the projects list) can update their counts.
    /// If no file was uploaded, nothing is refreshed or emitted.
    pub fn upload_documents(
        &mut self,
        doc_paths: Vec<PathBuf>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.is_uploading {
            return;
        }

        let pool = cx.global::<DbPool>().0.clone();

        let document_manager = DocumentManager::new(pool.clone());

        let Some(project) = self.project_stats.clone() else {
            println!(
                "Tried to upload documentation but no  project is settled thougt the self.project_stats (it should)"
            );
            return;
        };

        match document_manager {
            Ok(service) => {
                self.is_uploading = true;
                cx.notify();

                cx.spawn_in(window, async move |this, cx| {
                    let upload_result = service
                        .upload_documents(&doc_paths, project.project_id)
                        .await;

                    if upload_result.successes.is_empty() {
                        let _ = this.update_in(cx, |this, _window, cx| {
                            this.is_uploading = false;
                            cx.notify();
                        });
                        return;
                    }

                    let doc_ids = upload_result.successes.iter().map(|d| d.id).collect();
                    discover_customers(&service, pool, project.project_id, doc_ids).await;

                    let result = service.get_project_documents(project.project_id).await;
                    let _ = this.update_in(cx, |this, window, cx| {
                        this.is_uploading = false;
                        this.apply_documents(
                            project.project_id,
                            result,
                            "Document uploaded",
                            window,
                            cx,
                        );
                    });
                })
                .detach();
            }
            Err(e) => {
                eprintln!("{e}")
            }
        }
    }
}

/// Extracts text from `doc_ids`, matches it against all customers and saves the
/// resulting relations for `project_id`. Failures are logged, never fatal.
async fn discover_customers(
    service: &DocumentManager,
    pool: Pool<Sqlite>,
    project_id: i64,
    doc_ids: Vec<i64>,
) {
    let customers = match CustomerRepository::new(pool).get_customers().await {
        Ok(customers) => customers,
        Err(e) => return eprintln!("Failed to fetch customers: {e}"),
    };

    let relation_manager = RelationManager::new(service.clone(), DocumentExtractorService::new());
    let relations = match relation_manager.get_relations(doc_ids, customers).await {
        Ok(relations) => relations,
        Err(e) => return eprintln!("Failed to get relations: {e}"),
    };

    for (id, err) in &relations.failures {
        eprintln!("Could not extract document {id}: {err}");
    }
    if let Err(e) = service.save_relations(project_id, &relations).await {
        eprintln!("Failed to save relations: {e}");
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
            .overflow_y_scrollbar()
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
                        div()
                            .h_flex()
                            .gap_2()
                            .child(
                                Button::new("get-customers-btn")
                                    .disabled(self.is_getting_customers)
                                    .when_else(
                                        self.is_getting_customers,
                                        |this| this.label("Loading..."),
                                        |this| this.label("Get customers").child(IconName::User),
                                    )
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.get_customers(window, cx);
                                    })),
                            )
                            .child(
                                Button::new("upload-doc-btn")
                                    .primary()
                                    .disabled(self.is_uploading)
                                    .when_else(
                                        self.is_uploading,
                                        |this| this.label("Uploading..."),
                                        |this| this.label("Upload Document").child(IconName::Plus),
                                    )
                                    .on_click(cx.listener(|_this, _, window, cx| {
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
                    ),
            )
            .children(self.data_dir.clone().into_iter().flat_map(|dir| {
                self.documents.iter().map(move |doc| {
                    DocumentItem::new(doc.clone(), dir.clone(), doc.customer_count())
                })
            }))
    }
}
