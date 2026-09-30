use crate::DocumentManager;
use crate::models::{CustomerDocumentStatus, CustomerMention};
use gpui_kit::{
    App, AppContext, Context, Entity, InteractiveElement, IntoElement, ParentElement, Render,
    Styled, Window,
    base::StyledExt,
    component::{ActiveTheme, WindowExt, switch::Switch},
    div,
    prelude::FluentBuilder,
};
use shared::db::DbPool;

/// Documents that mention one customer, across all projects.
///
/// Hosted inside customer detail; the app shell installs it so `customers`
/// does not depend on `documents`. Confirmation is per document mention.
pub struct CustomerDocumentsSection {
    customer_id: Option<i64>,
    mentions: Vec<CustomerMention>,
    /// Bumped on each load so a slow fetch cannot overwrite a newer customer.
    load_generation: u64,
}

impl CustomerDocumentsSection {
    pub fn view(_window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|_cx| Self {
            customer_id: None,
            mentions: Vec::new(),
            load_generation: 0,
        })
    }

    pub fn set_customer(&mut self, customer_id: i64, cx: &mut Context<Self>) {
        self.customer_id = Some(customer_id);
        self.mentions.clear();
        self.load(cx);
    }

    fn load(&mut self, cx: &mut Context<Self>) {
        let Some(customer_id) = self.customer_id else {
            return;
        };
        self.load_generation = self.load_generation.wrapping_add(1);
        let generation = self.load_generation;
        let pool = cx.global::<DbPool>().0.clone();

        cx.spawn(async move |this, cx| {
            let Ok(manager) = DocumentManager::new(pool) else {
                return;
            };
            let mentions = manager.get_documents_mentioning_customer(customer_id).await;
            let _ = this.update(cx, |section, cx| {
                if section.load_generation != generation {
                    return;
                }
                match mentions {
                    Ok(mentions) => section.mentions = mentions,
                    Err(e) => eprintln!("Failed to load customer documents: {e}"),
                }
                cx.notify();
            });
        })
        .detach();

        cx.notify();
    }

    fn set_confirmed(
        &mut self,
        project_id: i64,
        document_id: i64,
        confirmed: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(customer_id) = self.customer_id else {
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
            .find(|m| m.project_id == project_id && m.document_id == document_id)
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

impl Render for CustomerDocumentsSection {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let rows: Vec<_> = self
            .mentions
            .clone()
            .into_iter()
            .map(|mention| {
                let project_id = mention.project_id;
                let document_id = mention.document_id;
                let confirmed = mention.is_confirmed();
                div()
                    .id(format!("customer-mention-{project_id}-{document_id}"))
                    .h_flex()
                    .w_full()
                    .p_3()
                    .gap_3()
                    .justify_between()
                    .items_center()
                    .border_1()
                    .border_color(theme.border)
                    .rounded_md()
                    .child(
                        div()
                            .v_flex()
                            .gap_1()
                            .child(div().text_sm().child(mention.document_name.clone()))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme.muted_foreground)
                                    .child(format!("Project: {}", mention.project_name)),
                            ),
                    )
                    .child(
                        Switch::new(format!("confirm-customer-mention-{project_id}-{document_id}"))
                            .label(if confirmed { "Confirmed" } else { "Not confirmed" })
                            .checked(confirmed)
                            .on_change(cx.listener(move |this, checked, window, cx| {
                                this.set_confirmed(project_id, document_id, *checked, window, cx);
                            })),
                    )
            })
            .collect();

        div()
            .v_flex()
            .w_full()
            .gap_2()
            .child(
                div()
                    .text_lg()
                    .text_color(theme.primary)
                    .child(format!("Mentioned in documents [{}]", self.mentions.len())),
            )
            .when(self.mentions.is_empty(), |parent| {
                parent.child(
                    div()
                        .w_full()
                        .p_4()
                        .flex()
                        .items_center()
                        .justify_center()
                        .border_1()
                        .border_color(theme.border)
                        .rounded_md()
                        .text_sm()
                        .text_color(theme.muted_foreground)
                        .child("Not mentioned in any document."),
                )
            })
            .children(rows)
    }
}
