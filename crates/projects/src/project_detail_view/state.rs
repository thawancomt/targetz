use gpui_kit::{
    App, AppContext, Context, Entity, IntoElement, ParentElement, Styled, Window,
    component::{
        ActiveTheme, IndexPath,
        list::{ListDelegate, ListItem, ListState},
    },
    div,
};
use shared::{
    customer::{Customer, Stakeholder},
    db::DbPool,
    project::{Project, ProjectHistoryRow, ProjectStatus},
};

use crate::{
    project_detail_view::project_relations::state::ProjectRelationView,
    project_repository::ProjectRepository,
};

#[derive(Clone)]
pub struct ProjectHistoryDelegate {
    pub history: Vec<ProjectHistoryRow>,
    pub selected_index: Option<IndexPath>,
}

impl ProjectHistoryDelegate {
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
            selected_index: None,
        }
    }

    pub fn set_history(&mut self, history: Vec<ProjectHistoryRow>) {
        self.history = history;
    }
}

impl ListDelegate for ProjectHistoryDelegate {
    type Item = ListItem;

    fn items_count(&self, _section: usize, _cx: &App) -> usize {
        self.history.len()
    }

    fn render_item(
        &mut self,
        ix: IndexPath,
        _window: &mut Window,
        cx: &mut Context<ListState<Self>>,
    ) -> Option<Self::Item> {
        let item = self.history.get(ix.row)?;
        let theme = cx.theme();
        let id = item
            .id
            .map(|id| format!("history-{id}"))
            .unwrap_or_else(|| format!("history-row-{}", ix.row));

        Some(
            ListItem::new(id)
                .w_full()
                .p_2()
                .border_b_1()
                .border_color(theme.border)
                .child(
                    div()
                        .w_full()
                        .p_3()
                        .border_1()
                        .border_color(theme.border)
                        .bg(theme.muted)
                        .rounded_sm()
                        .flex()
                        .flex_col()
                        .gap_2()
                        // created_at
                        .child(
                            div()
                                .text_sm()
                                .child(item.created_at.clone())
                                .text_color(theme.primary)
                                .text_lg(),
                        )
                        // from -> to  |  impact_on_target_deadline
                        .child(
                            div()
                                .flex()
                                .justify_between()
                                .items_center()
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_2()
                                        .text_sm()
                                        .child(item.from_status.clone())
                                        .child(div().child("->"))
                                        .child(item.to_status.clone()),
                                )
                                .child(
                                    div().text_sm().child(
                                        item.impact_on_target_deadline
                                            .clone()
                                            .unwrap_or_else(|| "N/A".to_string()),
                                    ),
                                ),
                        )
                        // caixa de note
                        .child(
                            div()
                                .p_2()
                                .border_1()
                                .border_color(theme.border)
                                .rounded_sm()
                                .text_sm()
                                .child(item.note.clone().unwrap_or_else(|| "N/A".to_string())),
                        )
                        // rodapé: change_ask_by | reason
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .min_w_64()
                                .w_64()
                                .max_w_auto()
                                .justify_between()
                                .child(
                                    div()
                                        .flex()
                                        .justify_between()
                                        .gap_2()
                                        .child(div().child("change_ask_by"))
                                        .child(
                                            item.change_ask_by
                                                .clone()
                                                .unwrap_or_else(|| "N/A".to_string()),
                                        ),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .justify_between()
                                        .gap_2()
                                        .child(div().child("reason"))
                                        .child(
                                            item.reason
                                                .clone()
                                                .unwrap_or_else(|| "N/A".to_string()),
                                        ),
                                ),
                        ),
                ),
        )
    }

    fn render_empty(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<ListState<Self>>,
    ) -> impl IntoElement {
        div()
            .p_4()
            .w_full()
            .flex()
            .items_center()
            .justify_center()
            .text_sm()
            .text_color(cx.theme().muted_foreground)
            .child("No history records yet.")
    }

    fn set_selected_index(
        &mut self,
        ix: Option<IndexPath>,
        _window: &mut Window,
        cx: &mut Context<ListState<Self>>,
    ) {
        self.selected_index = ix;
        cx.notify();
    }
}

pub struct ProjectDetailView {
    pub project: Option<Project>,
    pub history: Option<Vec<ProjectHistoryRow>>,
    pub history_list: Entity<ListState<ProjectHistoryDelegate>>,
    pub relations_view: Entity<ProjectRelationView>,
    pub show_history: bool,
    pub stakeholders: Option<Vec<Stakeholder>>,
}

impl ProjectDetailView {
    pub fn view(window: &mut Window, cx: &mut App, project: Option<Project>) -> Entity<Self> {
        let history_list = cx.new(|cx| ListState::new(ProjectHistoryDelegate::new(), window, cx));
        let relations_view = ProjectRelationView::view(window, cx);

        cx.new(|_cx| Self {
            project,
            history: None,
            history_list,
            show_history: false,
            stakeholders: Some(Vec::new()),
            relations_view,
        })
    }

    pub fn toggle_history(&mut self, cx: &mut Context<Self>) {
        self.show_history = !self.show_history;
        cx.notify();
    }

    pub fn set_project(&mut self, project: Project, window: &mut Window, cx: &mut Context<Self>) {
        self.project = Some(project.clone());
        self.hydrate_history(cx);
        self.hydrate_stakeholders(project.id, cx);

        self.relations_view.update(cx, |t, cx| {
            t.set_project(project, window, cx);
            cx.notify();
        });

        cx.notify();
    }

    pub fn hydrate_stakeholders(&mut self, project_id: i64, cx: &mut Context<Self>) {
        let pool = cx.global::<DbPool>().0.clone();

        cx.spawn(async move |view, cx| {
            let repo = ProjectRepository::new(pool);
            let stakeholders_result = cx
                .background_spawn(async move { repo.get_stakeholders(project_id.clone()).await })
                .await;

            match stakeholders_result {
                Ok(stakeholders) => {
                    let _ = view.update(cx, |this, _cx| {
                        this.stakeholders = Some(stakeholders);
                    });
                }
                Err(e) => {
                    eprintln!("{e}")
                }
            };
        })
        .detach();

        cx.notify();
    }

    pub fn hydrate_history(&mut self, cx: &mut Context<Self>) {
        let pool = cx.global::<DbPool>().0.clone();

        let Some(project) = self.project.clone() else {
            return;
        };

        let history_list = self.history_list.clone();

        cx.spawn(async move |this, cx| {
            let repository = ProjectRepository::new(pool);

            let history_result = cx
                .background_spawn(async move { repository.get_project_history(project.id).await })
                .await;

            match history_result {
                Ok(history) => {
                    let _ = history_list.update(cx, |list_state, cx| {
                        list_state.delegate_mut().set_history(history.clone());
                        cx.notify();
                    });
                    let _ = this.update(cx, |detail, context| {
                        detail.history = Some(history);
                        context.notify();
                    });
                }
                Err(e) => {
                    eprintln!("Failed to get project history: {e}");
                }
            };
        })
        .detach();
    }

    pub fn toggle_status(&mut self, status: ProjectStatus, cx: &mut Context<Self>) {
        let pool = cx.global::<DbPool>().0.clone();

        let Some(project) = self.project.clone() else {
            eprintln!("No project to toggle");
            return;
        };

        // no change
        if project.status == status {
            return;
        }

        let repository = ProjectRepository::new(pool);

        let mut current_history = self.history.clone().unwrap_or_default();

        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move { repository.update_status(project, status).await })
                .await;

            let updated = match &result {
                Ok(data) => Some(data),
                Err(e) => {
                    eprintln!("{e}");
                    None
                }
            };

            let _ = this.update(cx, |this, cx| {
                if let Some(update_data) = updated {
                    let updated_project = update_data.0.clone();
                    let new_history_record = update_data.1.clone();

                    this.project = Some(updated_project);

                    current_history.push(new_history_record);

                    this.history = Some(current_history.clone());

                    this.history_list.update(cx, move |list_state, cx| {
                        list_state
                            .delegate_mut()
                            .set_history(current_history.clone());
                        cx.notify();
                    });
                }

                cx.notify();
            });
        })
        .detach();

        cx.notify();
    }
}
