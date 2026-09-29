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
                    let _ = view.update(cx, |this, cx| {
                        this.customers = Some(customers);
                        cx.notify();
                    });
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
                        let views = stakeholders
                            .iter()
                            .map(|sk| this.create_stakeholder_view(sk.clone(), window, cx))
                            .collect();

                        this.stakeholders = Some(stakeholders.clone());
                        this.initial_stakeholders = Some(stakeholders);
                        this.stakeholders_views = Some(views);
                        cx.notify();
                    });
                }
                Err(e) => {
                    eprintln!("{e}")
                }
            }
        })
        .detach();
    }

    pub fn reset_relations(&mut self, cx: &mut Context<Self>) {
        let Some(stakeholders) = self.stakeholders.clone() else {
            return;
        };

        let Some(initial_stakeholders) = self.initial_stakeholders.clone() else {
            return;
        };

        let Some(mut views) = self.stakeholders_views.take() else {
            return;
        };

        self.stakeholders = Some(initial_stakeholders);

        let stakeholder_ids: Vec<i64> = stakeholders.clone().iter().map(|f| f.id).collect();

        views.retain(|view| {
            let view_data = view.read(cx);
            let view_stakeholder = view_data.stakeholder.clone();

            if stakeholder_ids.contains(&view_stakeholder.id) {
                true
            } else {
                false
            }
        });

        self.stakeholders_views = Some(views);
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
            views.retain(|v| v.read(cx).stakeholder.id != stakeholder.id);
        }

        cx.notify();
    }

    pub fn update_stakeholder(&mut self, stakeholder: Stakeholder) {
        if let Some(stakeholders) = self.stakeholders.as_mut() {
            if let Some(sk) = stakeholders.iter_mut().find(|s| s.id == stakeholder.id) {
                sk.role = stakeholder.role;
            }
        }
    }

    pub fn create_stakeholder_view(
        &mut self,
        stakeholder: Stakeholder,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Entity<StakeholderItemView> {
        let view = StakeholderItemView::new(stakeholder, window, cx);

        cx.subscribe_in(
            &view,
            window,
            move |this, _e, event, window, cx| match event {
                StakeholderItemEvent::RemoveStakeholder(sk) => {
                    this.remove_stakeholder(sk.clone(), cx);
                    cx.notify();
                }
                StakeholderItemEvent::AddedStakeholder(sk) => {
                    let customer = sk.clone().customer();
                    this.add_stakeholder(customer, window, cx);
                    cx.notify();
                }
                StakeholderItemEvent::UpdateStakeholder(updated_stakeholder) => {
                    this.update_stakeholder(updated_stakeholder.to_owned());
                    cx.notify();
                }
            },
        )
        .detach();

        view
    }

    pub fn add_stakeholder(
        &mut self,
        customer: Customer,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let new_stakeholder = Stakeholder::new(customer, None);

        let stakeholders = match self.stakeholders.as_mut() {
            Some(s) => s,
            None => {
                self.stakeholders = Some(Vec::new());
                self.stakeholders.as_mut().unwrap()
            }
        };

        if stakeholders.iter().any(|s| s.id == new_stakeholder.id) {
            return;
        }

        stakeholders.push(new_stakeholder.clone());

        let view = self.create_stakeholder_view(new_stakeholder, window, cx);

        match self.stakeholders_views.as_mut() {
            Some(views) => {
                views.push(view);
            }
            _ => self.stakeholders_views = Some(vec![view]),
        }

        cx.notify();
    }
}
