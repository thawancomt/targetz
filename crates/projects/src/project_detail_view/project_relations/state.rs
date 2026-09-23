use customers::customer_repository::CustomerRepository;
use gpui_kit::{App, AppContext, Context, Entity, Window, base::input::InputState};
use shared::{
    customer::{Customer, Stakeholder},
    db::DbPool,
    project::{self, Project},
};

use crate::{
    project_detail_view::components::stakeholder_item::{
        StakeholderItemEvent, StakeholderItemView,
    },
    project_repository::ProjectRepository,
};

pub struct ProjectRelationView {
    pub customers: Option<Vec<Customer>>,
    pub initial_stakeholders: Option<Vec<Stakeholder>>,
    pub stakeholders: Option<Vec<Stakeholder>>,
    pub stakeholders_views: Option<Vec<Entity<StakeholderItemView>>>,
    pub query: String,
    pub query_input: Entity<InputState>,
    pub saving: bool,
    pub filtered_customers: Vec<Customer>,
    pub project: Option<Project>,
}

impl ProjectRelationView {
    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }

    pub fn add_stakeholder(
        &mut self,
        customer: Customer,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let new_stakeholder = Stakeholder::new(customer, None);

        if let Some(stakeholders) = self.stakeholders.as_mut() {
            if stakeholders.contains(&new_stakeholder) {
                return;
            }
            stakeholders.push(new_stakeholder);
        }

        self.sync_stakeholders_views(self.stakeholders.clone().unwrap_or_default(), window, cx);

        cx.notify();
    }

    pub fn set_project(&mut self, project: Project, window: &mut Window, cx: &mut Context<Self>) {
        self.project = Some(project.clone());

        self.hydrate_customers(project.id, cx);
        self.hydrate_initial_stakeholders(project.id, cx);

        cx.notify();
    }

    pub fn hydrate_customers(&mut self, project_id: i64, cx: &mut Context<Self>) {
        let pool = cx.global::<DbPool>().0.clone();

        let repo = CustomerRepository::new(pool);
        cx.spawn(async move |view, cx| {
            let result = cx
                .background_spawn(async move { repo.get_customers().await })
                .await;

            match result {
                Ok(customers) => {
                    let _ = view.update(cx, |this, _cx| this.customers = Some(customers));
                }
                Err(e) => {
                    eprintln!("{e}")
                }
            }
        })
        .detach();
    }

    pub fn hydrate_initial_stakeholders(&mut self, project_id: i64, cx: &mut Context<Self>) {
        let pool = cx.global::<DbPool>().0.clone();

        let repo = ProjectRepository::new(pool);

        cx.spawn(async move |view, cx| {
            let result = cx
                .background_spawn(async move { repo.get_stakeholders(project_id).await })
                .await;

            match result {
                Ok(stakeholders) => {
                    let _ = view.update_in(cx, |this, window, cx| {
                        this.stakeholders = Some(stakeholders.clone());
                        this.initial_stakeholders = Some(stakeholders.clone());
                        this.sync_stakeholders_views(stakeholders, window, cx);
                    });
                }
                Err(e) => {
                    eprintln!("{e}")
                }
            }
        })
        .detach();
    }

    pub fn sync_stakeholders_views(
        &mut self,
        stakeholders: Vec<Stakeholder>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let mut stakeholders_views: Vec<Entity<StakeholderItemView>> = Vec::new();

        for stakeholder in stakeholders {
            let view = StakeholderItemView::new(stakeholder, window, cx);

            cx.subscribe(&view, move |this, _e, event, cx| match event {
                StakeholderItemEvent::RemoveStakeholder(sk) => {
                    this.remove_stakeholder(sk.clone(), cx);
                    cx.notify();
                }
                _ => {}
            })
            .detach();

            stakeholders_views.push(view);
        }

        self.stakeholders_views = Some(stakeholders_views);
    }

    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            customers: None,
            filtered_customers: Vec::new(),
            initial_stakeholders: None,
            query: String::new(),
            query_input: cx.new(|cx| InputState::new(window, cx)),
            saving: false,
            stakeholders: None,
            project: None,
            stakeholders_views: None,
        }
    }

    pub fn remove_stakeholder(&mut self, stakeholder: Stakeholder, cx: &mut Context<Self>) {
        if let Some(stakeholders) = self.stakeholders.as_mut() {
            stakeholders.retain(|s| s.id != stakeholder.id);
        };

        if let Some(views) = self.stakeholders_views.as_mut() {
            views.retain(|v| v.read(cx).stakeholder != stakeholder);
        }

        cx.notify();
    }
}
