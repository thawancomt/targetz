use gpui_kit::{
    App, AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Window,
    base::StyledExt,
    component::{ActiveTheme, tag::Tag},
    div,
    prelude::FluentBuilder,
};
use shared::{
    db::DbPool,
    ui::{rail_card, section_frame},
};

use crate::project_repository::{CustomerProject, ProjectRepository};

/// Projects one customer is a stakeholder of.
///
/// Hosted inside customer detail; the app shell installs it so `customers`
/// does not depend on `projects`.
pub struct CustomerProjectsSection {
    customer_id: Option<i64>,
    projects: Vec<CustomerProject>,
    /// Bumped on each load so a slow fetch cannot overwrite a newer customer.
    load_generation: u64,
}

impl CustomerProjectsSection {
    pub fn view(_window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|_cx| Self {
            customer_id: None,
            projects: Vec::new(),
            load_generation: 0,
        })
    }

    pub fn set_customer(&mut self, customer_id: i64, cx: &mut Context<Self>) {
        self.customer_id = Some(customer_id);
        self.projects.clear();
        self.load_generation = self.load_generation.wrapping_add(1);
        let generation = self.load_generation;
        let pool = cx.global::<DbPool>().0.clone();

        cx.spawn(async move |this, cx| {
            let repository = ProjectRepository::new(pool);
            let result = cx
                .background_spawn(
                    async move { repository.get_projects_for_customer(customer_id).await },
                )
                .await;
            let _ = this.update(cx, |section, cx| {
                if section.load_generation != generation {
                    return;
                }
                match result {
                    Ok(projects) => section.projects = projects,
                    Err(e) => eprintln!("Failed to load customer projects: {e}"),
                }
                cx.notify();
            });
        })
        .detach();

        cx.notify();
    }
}

impl Render for CustomerProjectsSection {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();

        let body = div()
            .v_flex()
            .w_full()
            .gap_2()
            .when(self.projects.is_empty(), |parent| {
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
                        .child("Not a stakeholder of any project."),
                )
            })
            .children(self.projects.iter().map(|project| {
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
                                .child(div().text_sm().child(project.project_name.clone()))
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(theme.muted_foreground)
                                        .child(format!("Role: {}", project.role)),
                                ),
                        )
                        .child(Tag::secondary().child(project.status.clone())),
                    &theme,
                )
            }));

        section_frame("Projects", Some(self.projects.len()), None, body, &theme)
    }
}
