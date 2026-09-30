use crate::models::{CustomerDocumentStatus, DocumentWithCustomers, MentionedCustomer};
use crate::ui::components::document_item::DocumentItem;
use crate::{DocumentExtractorService, DocumentManager, RelationManager};
use customers::customer_repository::CustomerRepository;
use gpui_kit::{
    App, AppContext, Context, Entity, InteractiveElement, IntoElement, ParentElement, Render,
    Styled, Window,
    base::{Disableable, StyledExt},
    component::{
        ActiveTheme, IconName, WindowExt,
        button::{Button, ButtonVariants},
        switch::Switch,
    },
    div,
    prelude::FluentBuilder,
};
use shared::{
    db::DbPool,
    ui::{rail_card, section_frame},
};
use sqlx::{Pool, Sqlite};
use std::path::PathBuf;

/// Documents and detected mentions for one project.
///
/// Hosted inside project detail. Stakeholders are a separate relation and are
/// not listed here. Confirmation is per document mention.
pub struct ProjectDocumentsSection {
    project_id: Option<i64>,
    documents: Vec<DocumentWithCustomers>,
    mentions: Vec<MentionedCustomer>,
    data_dir: Option<PathBuf>,
    documents_expanded: bool,
    mentions_expanded: bool,
    is_uploading: bool,
    is_deleting: bool,
    is_scanning: bool,
    /// Bumped on each load so a slow fetch cannot overwrite a newer project.
    load_generation: u64,
}

impl ProjectDocumentsSection {
    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }

    pub fn new(_window: &mut Window, _cx: &mut Context<Self>) -> Self {
        Self {
            project_id: None,
            documents: Vec::new(),
            mentions: Vec::new(),
            data_dir: DocumentManager::default_data_dir()
                .map_err(|e| eprintln!("{e}"))
                .ok(),
            documents_expanded: true,
            mentions_expanded: true,
            is_uploading: false,
            is_deleting: false,
            is_scanning: false,
            load_generation: 0,
        }
    }

    pub fn set_project(&mut self, project_id: i64, cx: &mut Context<Self>) {
        self.project_id = Some(project_id);
        self.documents.clear();
        self.mentions.clear();
        self.load(cx);
    }

    fn load(&mut self, cx: &mut Context<Self>) {
        let Some(project_id) = self.project_id else {
            return;
        };
        self.load_generation = self.load_generation.wrapping_add(1);
        let generation = self.load_generation;
        let pool = cx.global::<DbPool>().0.clone();

        cx.spawn(async move |this, cx| {
            let Ok(manager) = DocumentManager::new(pool) else {
                return;
            };
            let documents = manager.get_project_documents(project_id).await;
            let mentions = manager.get_mentioned_customers(project_id).await;
            let _ = this.update(cx, |section, cx| {
                if section.load_generation != generation || section.project_id != Some(project_id) {
                    return;
                }
                if let Ok(documents) = documents {
                    section.documents = documents;
                }
                if let Ok(mentions) = mentions {
                    section.mentions = mentions;
                }
                cx.notify();
            });
        })
        .detach();

        cx.notify();
    }

    pub fn toggle_documents(&mut self, cx: &mut Context<Self>) {
        self.documents_expanded = !self.documents_expanded;
        cx.notify();
    }

    pub fn toggle_mentions(&mut self, cx: &mut Context<Self>) {
        self.mentions_expanded = !self.mentions_expanded;
        cx.notify();
    }

    pub fn upload_documents(
        &mut self,
        doc_paths: Vec<PathBuf>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.is_uploading {
            return;
        }
        let Some(project_id) = self.project_id else {
            return;
        };
        let pool = cx.global::<DbPool>().0.clone();
        let Ok(service) = DocumentManager::new(pool.clone()) else {
            return;
        };

        self.is_uploading = true;
        cx.notify();

        cx.spawn_in(window, async move |this, cx| {
            let upload_result = service.upload_documents(&doc_paths, project_id).await;
            if !upload_result.successes.is_empty() {
                let doc_ids = upload_result.successes.iter().map(|d| d.id).collect();
                discover_customers(&service, pool, project_id, doc_ids).await;
            }
            let _ = this.update_in(cx, |this, window, cx| {
                this.is_uploading = false;
                if upload_result.successes.is_empty() {
                    window.push_notification("No document uploaded", cx);
                } else {
                    window.push_notification("Document uploaded", cx);
                    this.load(cx);
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub fn delete_document(
        &mut self,
        document_id: i64,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.is_deleting {
            return;
        }
        let pool = cx.global::<DbPool>().0.clone();
        let Ok(service) = DocumentManager::new(pool) else {
            return;
        };

        self.is_deleting = true;
        cx.notify();

        cx.spawn_in(window, async move |this, cx| {
            let deleted = service.delete_document(document_id).await;
            let _ = this.update_in(cx, |this, window, cx| {
                this.is_deleting = false;
                match deleted {
                    Ok(()) => {
                        window.push_notification("Document deleted", cx);
                        this.load(cx);
                    }
                    Err(e) => {
                        eprintln!("{e}");
                        window.push_notification(format!("Failed to delete document: {e}"), cx);
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub fn scan_customers(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.is_scanning {
            return;
        }
        let Some(project_id) = self.project_id else {
            return;
        };
        let doc_ids: Vec<i64> = self.documents.iter().map(|d| d.id).collect();
        if doc_ids.is_empty() {
            window.push_notification("No documents to scan", cx);
            return;
        }
        let pool = cx.global::<DbPool>().0.clone();
        let Ok(service) = DocumentManager::new(pool.clone()) else {
            return;
        };

        self.is_scanning = true;
        cx.notify();

        cx.spawn_in(window, async move |this, cx| {
            discover_customers(&service, pool, project_id, doc_ids).await;
            let _ = this.update_in(cx, |this, window, cx| {
                this.is_scanning = false;
                window.push_notification("Customers updated", cx);
                this.load(cx);
            });
        })
        .detach();
    }

    pub fn set_confirmed(
        &mut self,
        customer_id: i64,
        document_id: i64,
        confirmed: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(project_id) = self.project_id else {
            return;
        };
        let status = if confirmed {
            CustomerDocumentStatus::Confirmed
        } else {
            CustomerDocumentStatus::NotConfirmed
        };

        if let Some(mention) = self
            .mentions
            .iter_mut()
            .find(|m| m.customer_id == customer_id && m.document_id == document_id)
        {
            mention.status = status.as_str().to_string();
        }
        cx.notify();

        let pool = cx.global::<DbPool>().0.clone();
        cx.spawn_in(window, async move |this, cx| {
            let updated = match DocumentManager::new(pool) {
                Ok(manager) => {
                    manager
                        .set_customer_document_status(project_id, customer_id, document_id, status)
                        .await
                }
                Err(e) => Err(sqlx::Error::Protocol(e.to_string())),
            };
            let _ = this.update_in(cx, |this, window, cx| {
                match updated {
                    Ok(true) => {}
                    Ok(false) => {
                        window.push_notification("Mention was not found", cx);
                        this.load(cx);
                    }
                    Err(e) => {
                        eprintln!("{e}");
                        window.push_notification("Could not update confirmation", cx);
                        this.load(cx);
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }
}

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

impl Render for ProjectDocumentsSection {
    fn render(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl gpui_kit::IntoElement {
        let theme = cx.theme().clone();
        let documents_expanded = self.documents_expanded;
        let mentions_expanded = self.mentions_expanded;
        let document_count = self.documents.len();
        let mention_count = self.mentions.len();
        let is_uploading = self.is_uploading;
        let is_deleting = self.is_deleting;
        let is_scanning = self.is_scanning;
        let documents_empty = self.documents.is_empty();
        let mentions_empty = self.mentions.is_empty();
        let empty_documents = empty_row("No documents yet.", cx);
        let empty_mentions = empty_row("No customers detected. Upload a document, then scan.", cx);
        let toggle_documents = cx.listener(|this, _, _, cx| this.toggle_documents(cx));
        let toggle_mentions = cx.listener(|this, _, _, cx| this.toggle_mentions(cx));
        let on_upload = cx.listener(|_this, _, window, cx| {
            cx.spawn_in(window, async move |this, cx| {
                let files = rfd::AsyncFileDialog::new()
                    .add_filter("Documents", &["pdf", "txt", "html"])
                    .pick_files()
                    .await;
                let Some(entries) = files else {
                    return;
                };
                if entries.is_empty() {
                    return;
                }
                let paths: Vec<_> = entries
                    .into_iter()
                    .map(|f| f.path().to_path_buf())
                    .collect();
                let _ = this.update_in(cx, |this, window, cx| {
                    this.upload_documents(paths, window, cx);
                });
            })
            .detach();
        });
        let on_scan = cx.listener(|this, _, window, cx| {
            this.scan_customers(window, cx);
        });
        let document_items: Vec<_> = self
            .data_dir
            .clone()
            .into_iter()
            .flat_map(|dir| {
                self.documents
                    .iter()
                    .map(|doc| {
                        let document_id = doc.id;
                        DocumentItem::new(
                            doc.clone(),
                            dir.clone(),
                            doc.customer_count(),
                            is_deleting,
                            cx.listener(move |this, _, window, cx| {
                                this.delete_document(document_id, window, cx);
                            }),
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .collect();
        let mention_rows: Vec<_> = self
            .mentions
            .clone()
            .into_iter()
            .map(|mention| mention_row(mention, cx))
            .collect();

        let documents_body = div()
            .v_flex()
            .w_full()
            .gap_2()
            .child(
                div().h_flex().justify_end().gap_2().child(
                    Button::new("section-upload-doc")
                        .primary()
                        .disabled(is_uploading)
                        .when_else(
                            is_uploading,
                            |this| this.label("Uploading..."),
                            |this| this.label("Upload").child(IconName::Plus),
                        )
                        .on_click(on_upload),
                ),
            )
            .when(documents_empty, |parent| parent.child(empty_documents))
            .children(document_items);

        let mentions_body = div()
            .v_flex()
            .w_full()
            .gap_2()
            .child(
                div()
                    .h_flex()
                    .justify_between()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .text_sm()
                            .text_color(theme.muted_foreground)
                            .child("Detected in documents. Not the stakeholder list."),
                    )
                    .child(
                        Button::new("section-scan-customers")
                            .disabled(is_scanning)
                            .when_else(
                                is_scanning,
                                |this| this.label("Scanning..."),
                                |this| this.label("Scan documents").child(IconName::User),
                            )
                            .on_click(on_scan),
                    ),
            )
            .when(mentions_empty, |parent| parent.child(empty_mentions))
            .children(mention_rows);

        div()
            .v_flex()
            .w_full()
            .gap_6()
            .child(section_frame(
                "Documents",
                Some(document_count),
                Some(toggle_button(
                    "documents-section",
                    documents_expanded,
                    toggle_documents,
                )),
                div().when(documents_expanded, |d| d.child(documents_body)),
                &theme,
            ))
            .child(section_frame(
                "Mentioned customers",
                Some(mention_count),
                Some(toggle_button(
                    "mentions-section",
                    mentions_expanded,
                    toggle_mentions,
                )),
                div().when(mentions_expanded, |d| d.child(mentions_body)),
                &theme,
            ))
    }
}

fn toggle_button(
    id: &'static str,
    expanded: bool,
    on_click: impl Fn(&gpui_kit::ClickEvent, &mut Window, &mut App) + 'static,
) -> gpui_kit::AnyElement {
    Button::new(id)
        .secondary()
        .label(if expanded { "Hide" } else { "Show" })
        .on_click(on_click)
        .into_any_element()
}

fn empty_row(
    message: &'static str,
    cx: &mut Context<ProjectDocumentsSection>,
) -> gpui_kit::AnyElement {
    let theme = cx.theme().clone();
    div()
        .w_full()
        .p_4()
        .flex()
        .items_center()
        .justify_center()
        .text_sm()
        .text_color(theme.muted_foreground)
        .child(message)
        .into_any_element()
}

fn mention_row(
    mention: MentionedCustomer,
    cx: &mut Context<ProjectDocumentsSection>,
) -> gpui_kit::AnyElement {
    let theme = cx.theme().clone();
    let customer_id = mention.customer_id;
    let document_id = mention.document_id;
    let confirmed = mention.is_confirmed();

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
                .gap_1()
                .child(div().text_sm().child(mention.customer_name.clone()))
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.muted_foreground)
                        .child(mention.document_name.clone()),
                ),
        )
        .child(
            Switch::new(format!("confirm-mention-{customer_id}-{document_id}"))
                .label(if confirmed {
                    "Confirmed"
                } else {
                    "Not confirmed"
                })
                .checked(confirmed)
                .on_change(cx.listener(move |this, checked, window, cx| {
                    this.set_confirmed(customer_id, document_id, *checked, window, cx);
                })),
        ),
        &theme,
    )
    .id(format!("mention-{customer_id}-{document_id}"))
    .into_any_element()
}
