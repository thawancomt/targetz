use customers::{
    create_customer_view::{CreateCustomerEvent, CreateCustomerView},
    customers_list_view::CustomerListView,
    tab_customers_view::{self, TabCustomerView},
};
use gpui_kit::{
    App, AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Window,
    base::StyledExt,
    component::{Root, TitleBar},
    div,
};
use settings::settings_view::SettingsView;
use shared::AppTab;
use sidebar::sidebar_view::{SidebarEvent, SidebarView};
use sqlx::{Pool, Sqlite};

use crate::DbPool;

struct State {
    pool: Pool<Sqlite>,
}

pub struct AppShell {
    pub current_tab: AppTab,
    pub sidebar: Entity<SidebarView>,
    pub settings_view: Entity<SettingsView>,
    pub create_customer_view: Entity<CreateCustomerView>,
    pub customer_tab_view: Entity<TabCustomerView>,
    pub state: State,
}

impl AppShell {
    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }

    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let pool = cx.global::<DbPool>().0.clone();

        let initial_current_tab = AppTab::Home;
        let sidebar = SidebarView::view(window, cx);
        let settings_view = SettingsView::view(window, cx);

        // Customers
        let create_customer_view = CreateCustomerView::view(window, cx);

        let tab_customers_view = TabCustomerView::view(window, cx);

        cx.subscribe(&sidebar, |this, _entity, event, cx| match event {
            SidebarEvent::TabClick(tab) => {
                this.current_tab = tab.to_owned();
                cx.notify();
            }
        })
        .detach();

        Self {
            sidebar,
            settings_view,
            create_customer_view,
            state: State { pool: pool },
            current_tab: initial_current_tab,
            customer_tab_view: tab_customers_view,
        }
    }
}

impl Render for AppShell {
    fn render(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl gpui_kit::prelude::IntoElement {
        let tab_to_show = match self.current_tab {
            AppTab::Settings => self.settings_view.clone().into_any_element(),
            AppTab::CreateCustomer => self.create_customer_view.clone().into_any_element(),
            AppTab::Targetz => self.customer_tab_view.clone().into_any_element(),
            _ => self.settings_view.clone().into_any_element(),
        };

        let dialog_layer = Root::render_dialog_layer(window, cx);
        let notification_layer = Root::render_notification_layer(window, cx);

        div().size_full().v_flex().child(TitleBar::new()).child(
            div()
                .flex()
                .flex_1()
                .child(self.sidebar.clone())
                .child(div().h_flex().flex_1().child(tab_to_show))
                .children(dialog_layer)
                .children(notification_layer),
        )
    }
}
