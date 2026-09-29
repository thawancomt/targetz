use customers::customer_repository::CustomerRepository;
use gpui_kit::{
    component::input::{InputEvent, InputState},
    App, AppContext, Context, Entity, EventEmitter, Window,
};
use projects::project_repository::ProjectRepository;
use shared::db::DbPool;

use super::{
    events::HomeEvent,
    item::{ItemCategory, QuickAction, SearchItem},
};

pub struct HomeView {
    pub query: String,
    pub query_input: Entity<InputState>,
    pub items: Vec<SearchItem>,
    pub customer_repo: CustomerRepository,
    pub project_repo: ProjectRepository,
}

impl EventEmitter<HomeEvent> for HomeView {}

impl HomeView {
    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }

    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let pool = cx.global::<DbPool>().0.clone();
        let customer_repo = CustomerRepository::new(pool.clone());
        let project_repo = ProjectRepository::new(pool);

        let query_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Search customers, projects, actions by name, email, phone, codename...")
        });

        cx.subscribe(&query_input, |this, _input, event: &InputEvent, cx| match event {
            InputEvent::Change => {
                let val = this.query_input.read(cx).value();
                this.query = val.to_string();
                cx.notify();
            }
            _ => {}
        })
        .detach();

        let mut view = Self {
            query: String::new(),
            query_input,
            items: Vec::new(),
            customer_repo,
            project_repo,
        };

        view.hydrate_data(cx);
        view
    }

    pub fn hydrate_data(&mut self, cx: &mut Context<Self>) {
        let cust_repo = self.customer_repo.clone();
        let proj_repo = self.project_repo.clone();

        cx.spawn(async move |this, cx| {
            let customers = cust_repo.get_customers().await.unwrap_or_default();
            let projects = proj_repo.get_projects().await.unwrap_or_default();

            let _ = this.update(cx, |this, cx| {
                let mut all_items: Vec<SearchItem> = Vec::new();

                for c in customers {
                    all_items.push(SearchItem::from_customer(c));
                }

                for p in projects {
                    all_items.push(SearchItem::from_project(p));
                }

                for a in QuickAction::all() {
                    all_items.push(SearchItem::from_action(a));
                }

                this.items = all_items;
                cx.notify();
            });
        })
        .detach();
    }

    pub fn filtered_items(&self) -> Vec<&SearchItem> {
        self.items
            .iter()
            .filter(|item| item.matches(&self.query))
            .collect()
    }

    pub fn grouped_items(&self) -> Vec<(ItemCategory, Vec<&SearchItem>)> {
        let filtered = self.filtered_items();
        let categories = [
            ItemCategory::Customers,
            ItemCategory::Projects,
            ItemCategory::Actions,
        ];

        let mut groups = Vec::new();
        for cat in categories {
            let cat_items: Vec<&SearchItem> = filtered
                .iter()
                .filter(|item| item.category == cat)
                .copied()
                .collect();

            if !cat_items.is_empty() {
                groups.push((cat, cat_items));
            }
        }
        groups
    }
}
