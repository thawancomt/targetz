use gpui_kit::{
    App, AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Window,
    base::v_flex,
    component::{
        ActiveTheme, IconName,
        button::{Button, ButtonVariants},
        tab::{Tab, TabBar},
    },
};
use shared::customer::{self, Customer};

use crate::{
    customer_detail_view::{self, CustomerDetailView},
    customers_list_view::{CustomerListView, CustomerListViewEvent},
};

/// Dono único do estado "quais tabs existem + qual está ativa".
/// Ninguém fora daqui manipula tabs/seleção diretamente, então o
/// estado nunca fica inconsistente (ex: ativa apontando pra tab que não existe).
pub struct TabState {
    tabs: Vec<Customer>,
    active: Active,
}

#[derive(Clone, Copy, PartialEq)]
enum Active {
    AllUsers,
    Customer(i64), // id
}

impl TabState {
    pub fn new() -> Self {
        Self {
            tabs: Vec::new(),
            active: Active::AllUsers,
        }
    }

    pub fn tabs(&self) -> &[Customer] {
        &self.tabs
    }

    pub fn active_customer(&self) -> Option<&Customer> {
        match self.active {
            Active::AllUsers => None,
            Active::Customer(id) => self.tabs.iter().find(|c| c.id == id),
        }
    }

    pub fn open(&mut self, customer: Customer) {
        if !self.tabs.iter().any(|c| c.id == customer.id) {
            self.tabs.push(customer.clone());
        }
        self.active = Active::Customer(customer.id);
    }

    fn select(&mut self, target: Active) {
        match target {
            Active::AllUsers => self.active = Active::AllUsers,
            Active::Customer(id) => {
                if self.tabs.iter().any(|c| c.id == id) {
                    self.active = Active::Customer(id);
                }
            }
        }
    }

    pub fn close(&mut self, id: i64) {
        let Some(pos) = self.tabs.iter().position(|c| c.id == id) else {
            return;
        };
        let was_active = self.active == Active::Customer(id);

        self.tabs.remove(pos);

        if was_active {
            self.active = self
                .tabs
                .get(pos.saturating_sub(1))
                .map(|c| Active::Customer(c.id))
                .unwrap_or(Active::AllUsers);
        }
    }

    pub fn selected_index(&self) -> usize {
        match self.active {
            Active::AllUsers => 0,
            Active::Customer(id) => self
                .tabs
                .iter()
                .position(|c| c.id == id)
                .map(|p| p + 1)
                .unwrap_or(0),
        }
    }

    pub fn select_by_index(&mut self, index: usize) {
        if index == 0 {
            self.select(Active::AllUsers);
        } else if let Some(c) = self.tabs.get(index - 1) {
            self.select(Active::Customer(c.id));
        }
    }
}

pub struct TabCustomerView {
    state: TabState,
    customer_list_view: Entity<CustomerListView>,
}

impl Render for TabCustomerView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let customer_list_view = self.customer_list_view.clone();
        let tabs_snapshot = self.state.tabs().to_vec();

        v_flex()
            .h_full()
            .w_full()
            .child(
                TabBar::new("customer-tab-view")
                    .selected_index(self.state.selected_index())
                    .child(Tab::new().label("All users"))
                    .children(tabs_snapshot.iter().map(|c| {
                        let id = c.id;
                        Tab::new().label(c.name.to_owned()).suffix(
                            Button::new(format!("remove-{}", c.id))
                                .ghost()
                                .text_color(cx.theme().selection)
                                .on_click(cx.listener(move |this, _, _window, cx| {
                                    this.state.close(id);
                                    cx.notify();
                                }))
                                .child(IconName::Close),
                        )
                    }))
                    .on_click(cx.listener(move |this, index, _, cx| {
                        this.state.select_by_index(*index);
                        cx.notify();
                    })),
            )
            .child(match self.state.active_customer() {
                None => customer_list_view.into_any_element(),
                Some(c) => CustomerDetailView::new(c.clone()).into_any_element(),
            })
    }
}

impl TabCustomerView {
    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }

    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let customer_list_view = CustomerListView::view(window, cx);

        customer_list_view.update(cx, |this, cx| {
            this.hydrate_customers(cx);
        });

        cx.subscribe(&customer_list_view, |this, _view, event, _cx| match event {
            CustomerListViewEvent::OPEN(data) => {
                this.state.open(data.clone());
            }
        })
        .detach();

        Self {
            state: TabState::new(),
            customer_list_view,
        }
    }
}
