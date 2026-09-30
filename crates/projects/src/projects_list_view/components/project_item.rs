use gpui_kit::{
    App, AppContext, Context, Entity, EventEmitter, IntoElement, ParentElement, Render, Styled,
    base::{Disableable, v_flex},
    component::{
        ActiveTheme,
        button::{Button, ButtonVariants},
        tag::Tag,
    },
    div, px,
};
use shared::{
    project::{Project, ProjectStatus},
    ui::{caption, rail_card},
};

use crate::projects_list_view::components::events::ProjectItemEvents;

#[derive(Debug)]
pub struct ProjectItem {
    project: Project,
}

impl EventEmitter<ProjectItemEvents> for ProjectItem {}

impl Render for ProjectItem {
    fn render(
        &mut self,
        _window: &mut gpui_kit::Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let theme = cx.theme();
        let muted = theme.muted_foreground;

        let Project {
            id,
            name,
            current_version,
            status,
            description,
            start_date,
            site_url,
            codename,
            target_deadline,
            budget,
            ..
        } = self.project.clone();

        let field = |label: &'static str, value: String| {
            v_flex()
                .gap_1()
                .child(caption(label, theme))
                .child(div().text_sm().child(value))
        };

        let period = format!(
            "{} → {}",
            start_date.unwrap_or_else(|| "—".into()),
            target_deadline.unwrap_or_else(|| "—".into()),
        );
        let budget = budget
            .map(|b| format!("$ {b:.2}"))
            .unwrap_or_else(|| "—".into());

        let status_tag = match status {
            ProjectStatus::Finished => Tag::success(),
            ProjectStatus::Started => Tag::info(),
            ProjectStatus::Prospecting => Tag::secondary(),
            ProjectStatus::Propousing => Tag::warning(),
            ProjectStatus::Refactoring => Tag::primary(),
        }
        .child(status.as_str());

        let mut caption_text = format!("Project #{id} · v{current_version}");
        if let Some(c) = codename.filter(|c| !c.is_empty()) {
            caption_text.push_str(&format!(" · {c}"));
        }

        rail_card(
            false,
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .p_3()
                    .gap_2()
                    .child(
                        div()
                            .flex()
                            .justify_between()
                            .items_center()
                            .gap_2()
                            .child(caption(caption_text, theme))
                            .child(status_tag),
                    )
                    .child(
                        v_flex()
                            .gap_1()
                            .child(div().child(name).text_xl().text_color(theme.foreground))
                            .children(site_url.filter(|u| !u.is_empty()).map(|u| {
                                div()
                                    .child(u)
                                    .font_family("Geist Mono")
                                    .text_xs()
                                    .text_color(muted)
                            })),
                    )
                    .children(description.filter(|d| !d.trim().is_empty()).map(|d| {
                        div()
                            .p_2()
                            .border_1()
                            .border_color(theme.border)
                            .bg(theme.background)
                            .text_sm()
                            .text_color(theme.foreground)
                            .child(d)
                    }))
                    .child(
                        div()
                            .flex()
                            .items_end()
                            .justify_between()
                            .pt_2()
                            .border_t_1()
                            .border_color(theme.border)
                            .child(
                                div()
                                    .flex()
                                    .gap_6()
                                    .child(field("Period", period))
                                    .child(field("Budget", budget)),
                            )
                            .child(
                                Button::new(format!("open-project-{id}"))
                                    .primary()
                                    .label("Open")
                                    .on_click(cx.listener(|this, _, _window, cx| {
                                        this.open(cx);
                                    })),
                            ),
                    ),
            &theme,
        )
    }
}

impl ProjectItem {
    pub fn new(project: Project, cx: &mut App) -> Entity<Self> {
        cx.new(|_cx| Self { project })
    }

    pub fn open(&self, cx: &mut Context<Self>) {
        cx.emit(ProjectItemEvents::OpenProject(self.project.clone()));
    }
}
