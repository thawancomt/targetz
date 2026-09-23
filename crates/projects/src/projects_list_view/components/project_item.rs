use gpui_kit::{
    App, AppContext, Context, Entity, EventEmitter, IntoElement, ParentElement, Render, Styled,
    base::Disableable,
    component::{
        ActiveTheme,
        button::{Button, ButtonVariants},
        tag::Tag,
    },
    div,
};
use shared::project::{Project, ProjectStatus};

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

        // label pequeno em cima, valor embaixo
        let field = |label: &'static str, value: String| {
            div()
                .flex()
                .flex_col()
                .gap_0p5()
                .child(div().text_xs().text_color(muted).child(label))
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

        div()
            .w_full()
            .p_4()
            .gap_3()
            .flex()
            .flex_col()
            .border_1()
            .border_color(theme.border)
            .bg(theme.background)
            // header: nome + meta à esquerda, status à direita
            .child(
                div().flex().gap_2().items_start().child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .text_lg()
                                .text_color(theme.foreground)
                                .child(name)
                                .child(status_tag),
                        )
                        .child(
                            div()
                                .flex()
                                .gap_2()
                                .text_xs()
                                .text_color(muted)
                                .child(format!("v{current_version}"))
                                .children(codename.map(|c| format!("· #{c}")))
                                .children(site_url.map(|u| format!("· {u}"))),
                        ),
                ),
            )
            // descrição só aparece se existir
            .children(description.filter(|d| !d.trim().is_empty()).map(|d| {
                div()
                    .p_3()
                    .bg(theme.muted)
                    .text_sm()
                    .text_color(theme.foreground)
                    .child(d)
            }))
            // rodapé: período + orçamento | botão
            .child(
                div()
                    .flex()
                    .items_end()
                    .justify_between()
                    .child(
                        div()
                            .flex()
                            .gap_6()
                            .child(field("Deadline", period))
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
