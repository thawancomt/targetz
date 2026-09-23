use gpui_kit::{
    App, AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Window,
    base::v_flex,
    component::{
        ActiveTheme, IconName,
        button::{Button, ButtonVariants},
        tab::{Tab, TabBar},
    },
    div,
};
use shared::project::Project;

use crate::{
    project_detail_view::state::ProjectDetailView,
    projects_list_view::{events::ProjectListEvents, state::ProjectsListView},
};

pub struct TabState {
    tabs: Vec<Project>,
    active: Active,
}

#[derive(Clone, Copy, PartialEq)]
enum Active {
    AllProjects,
    Project(i64), // id
}

impl TabState {
    pub fn new() -> Self {
        Self {
            tabs: Vec::new(),
            active: Active::AllProjects,
        }
    }

    pub fn tabs(&self) -> &[Project] {
        &self.tabs
    }

    pub fn active_project(&self) -> Option<&Project> {
        match self.active {
            Active::AllProjects => None,
            Active::Project(id) => self.tabs.iter().find(|p| p.id == id),
        }
    }

    pub fn open(&mut self, project: Project) {
        if !self.tabs.iter().any(|p| p.id == project.id) {
            self.tabs.push(project.clone());
        }
        self.active = Active::Project(project.id);
    }

    fn select(&mut self, target: Active) {
        match target {
            Active::AllProjects => self.active = Active::AllProjects,
            Active::Project(id) => {
                if self.tabs.iter().any(|p| p.id == id) {
                    self.active = Active::Project(id);
                }
            }
        }
    }

    pub fn close(&mut self, id: i64) {
        let Some(pos) = self.tabs.iter().position(|p| p.id == id) else {
            return;
        };
        let was_active = self.active == Active::Project(id);

        self.tabs.remove(pos);

        if was_active {
            self.active = self
                .tabs
                .get(pos.saturating_sub(1))
                .map(|p| Active::Project(p.id))
                .unwrap_or(Active::AllProjects);
        }
    }

    pub fn selected_index(&self) -> usize {
        match self.active {
            Active::AllProjects => 0,
            Active::Project(id) => self
                .tabs
                .iter()
                .position(|p| p.id == id)
                .map(|p| p + 1)
                .unwrap_or(0),
        }
    }

    pub fn select_by_index(&mut self, index: usize) {
        if index == 0 {
            self.select(Active::AllProjects);
        } else if let Some(p) = self.tabs.get(index - 1) {
            self.select(Active::Project(p.id));
        }
    }
}

pub struct TabProjectsView {
    pub state: TabState,
    pub projects_list_view: Entity<ProjectsListView>,
    pub project_detail_view: Entity<ProjectDetailView>,
}

impl Render for TabProjectsView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let projects_list_view = self.projects_list_view.clone();
        let tabs_snapshot = self.state.tabs().to_vec();

        v_flex()
            .h_full()
            .w_full()
            .child(
                TabBar::new("projects-tab-view")
                    .selected_index(self.state.selected_index())
                    .child(Tab::new().label("All projects"))
                    .children(tabs_snapshot.iter().map(|p| {
                        let id = p.id;
                        Tab::new().label(p.name.to_owned()).suffix(
                            Button::new(format!("remove-project-{}", p.id))
                                .ghost()
                                .text_color(cx.theme().selection)
                                .on_click(cx.listener(move |this, _, window, cx| {
                                    this.state.close(id);
                                    if let Some(active) = this.state.active_project().cloned() {
                                        this.project_detail_view.update(cx, |detail, cx| {
                                            detail.set_project(active, window, cx);
                                        });
                                    }
                                    cx.notify();
                                }))
                                .child(IconName::Close),
                        )
                    }))
                    .on_click(cx.listener(move |this, index, window, cx| {
                        this.state.select_by_index(*index);
                        if let Some(active) = this.state.active_project().cloned() {
                            this.project_detail_view.update(cx, |detail, cx| {
                                detail.set_project(active, window, cx);
                            });
                        }
                        cx.notify();
                    })),
            )
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .min_w_0()
                    .child(match self.state.active_project() {
                        None => projects_list_view.into_any_element(),
                        Some(_) => self.project_detail_view.clone().into_any_element(),
                    }),
            )
    }
}

impl TabProjectsView {
    pub fn view(
        window: &mut Window,
        cx: &mut App,
        projects_list_view: Entity<ProjectsListView>,
    ) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx, projects_list_view))
    }

    pub fn new(
        window: &mut Window,
        cx: &mut Context<Self>,
        projects_list_view: Entity<ProjectsListView>,
    ) -> Self {
        let project_detail_view = ProjectDetailView::view(window, cx, None);

        cx.subscribe_in(
            &projects_list_view,
            window,
            move |tab_view, _, event, window, cx| match event {
                ProjectListEvents::OpenProject(project) => {
                    tab_view.state.open(project.clone());
                    tab_view
                        .project_detail_view
                        .update(cx, |detail, detail_cx| {
                            println!("Opening");
                            detail.set_project(project.clone(), window, detail_cx);
                        });
                }
                _ => {}
            },
        )
        .detach();

        cx.notify();

        Self {
            state: TabState::new(),
            projects_list_view,
            project_detail_view,
        }
    }
}
