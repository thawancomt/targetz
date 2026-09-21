use customers::{
    create_customer_view::{CreateCustomerEvent, CreateCustomerView},
    customer_repository::CustomerRepository,
    customers_list_view::CustomerListView,
    tab_customers_view::TabCustomerView,
};
use gpui_fps::fps_monitor;
use gpui_kit::{
    App, AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Window,
    base::StyledExt,
    component::{Root, TitleBar},
    div,
    prelude::FluentBuilder,
};
use projects::{
    project_form_view::CreateProjectView,
    project_repository::ProjectRepository,
    projects_list_view::{events::ProjectListEvents, state::ProjectsListView},
};
use settings::settings_view::SettingsView;
use shared::{AppTab, db::DbPool, events::AppEvent};
use sidebar::sidebar_view::{SidebarEvent, SidebarView};

pub struct AppShell {
    pub current_tab: AppTab,
    pub sidebar: Entity<SidebarView>,
    pub settings_view: Entity<SettingsView>,
    pub create_customer_view: Entity<CreateCustomerView>,
    pub customer_tab_view: Entity<TabCustomerView>,
    pub create_project_view: Entity<CreateProjectView>,
    pub project_list_view: Entity<ProjectsListView>,
}

impl AppShell {
    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        let view = cx.new(|cx| Self::new(window, cx));

        view
    }

    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let initial_current_tab = AppTab::Home;
        let sidebar = SidebarView::view(window, cx);
        let settings_view = SettingsView::view(window, cx);

        // Customers
        let create_customer_view = CreateCustomerView::view(window, cx);
        let customer_list_view = CustomerListView::view(window, cx);
        let tab_customers_view = TabCustomerView::view(window, cx, customer_list_view.clone());
        let create_project_view = CreateProjectView::view(window, cx);
        let project_view_handle = create_project_view.clone();

        let pool = cx.global::<DbPool>().0.clone();
        cx.spawn(async |this, cx| {
            let customer_repo = CustomerRepository::new(pool);

            match customer_repo.get_customers().await {
                Ok(customers) => {
                    let _ = this.update_in(cx, move |_this, window, cx| {
                        project_view_handle.update(cx, |this, app| {
                            this.with_customers(customers.clone(), window, app)
                        })
                    });
                }
                Err(_e) => {}
            };
        })
        .detach();

        let tab_customers_view_clone = tab_customers_view.clone();
        cx.subscribe(&sidebar, |this, _entity, event, cx| match event {
            SidebarEvent::TabClick(tab) => {
                this.current_tab = tab.to_owned();
                cx.notify();
            }
        })
        .detach();

        // Listen the creation of a new customer
        // Move to the targetz page and set the active tab to the ne user
        cx.subscribe(
            &create_customer_view,
            |app_shell, _create_view, event, cx| match event {
                CreateCustomerEvent::Created(new_customer) => {
                    app_shell.customer_tab_view.update(cx, |tab_view, cx| {
                        tab_view
                            .customer_list_view
                            .update(cx, |customer_list_view, cx| {
                                customer_list_view.hydrate_customers(cx)
                            });
                        // open the new customer in tab view -> tabs
                        tab_view.state.open(new_customer.clone());
                    });
                    app_shell
                        .sidebar
                        .update(cx, |sidebar_view, sidebar_context| {
                            sidebar_view.toggle_tab(sidebar_context, AppTab::Targetz);
                            sidebar_context.notify();
                        });

                    app_shell
                        .customer_tab_view
                        .update(cx, |tab_view, tab_context| {
                            tab_view.customer_detail_view.update(
                                tab_context,
                                |detail_view, context| {
                                    detail_view.set_customer(new_customer.clone(), context);
                                    detail_view.hydrate_interactions(context);
                                    context.notify();
                                },
                            );
                            tab_context.notify();
                        });
                }
                _ => {}
            },
        )
        .detach();

        // React to deletion of a customer
        cx.subscribe(
            &customer_list_view,
            move |_app_shell, list_view, event, cx| match event {
                AppEvent::DeletedCustomer(deleted_id) => {
                    list_view.update(cx, |this, cx| this.hydrate_customers(cx));
                    tab_customers_view_clone.update(cx, |tab_view, tab_context| {
                        tab_view.state.close(deleted_id.clone());
                        tab_context.notify();
                    })
                }
                _ => {}
            },
        )
        .detach();

        let project_list_view = ProjectsListView::view(window, cx);

        Self::observe_tab_events(&sidebar, cx);

        // we need window here due the select on the form (Yeah, I dont like it too)
        Self::observer_project_list(&project_list_view.clone(), window, cx);

        Self {
            sidebar,
            settings_view,
            create_customer_view,
            current_tab: initial_current_tab,
            customer_tab_view: tab_customers_view,
            create_project_view,
            project_list_view,
        }
    }

    pub fn observe_tab_events(tab_view: &Entity<SidebarView>, cx: &mut Context<Self>) {
        cx.subscribe(&tab_view, move |this, _entity, event, cx| {
            match event {
                SidebarEvent::TabClick(AppTab::Projects) => {
                    let p_list_handle = this.project_list_view.clone();
                    let pool = cx.global::<DbPool>().0.clone();

                    cx.spawn(async move |_this, cx| {
                        let repository = ProjectRepository::new(pool.clone());
                        let projects_result = repository.get_projects().await;

                        match projects_result {
                            Ok(projects) => p_list_handle.update(cx, |view, cx| {
                                view.with_projects(projects, cx);
                                cx.notify();
                            }),
                            Err(_e) => {}
                        };
                    })
                    .detach();
                }
                _ => {}
            };
        })
        .detach();
    }
    pub fn observe_customer_events(&self, _cx: &mut App) {}

    pub fn observer_project_list(
        view: &Entity<ProjectsListView>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        cx.subscribe_in(
            view,
            window,
            move |this, _emitter, event, window, cx| match event {
                ProjectListEvents::OpenFormView => {
                    let pool = cx.global::<DbPool>().0.clone();
                    let form = this.project_list_view.read(cx).create_project_view.clone();

                    cx.spawn_in(window, async move |_this, cx| {
                        let repository = CustomerRepository::new(pool);
                        if let Ok(customers) = repository.get_customers().await {
                            let _ = form.update_in(cx, |form_view, window, cx| {
                                form_view.with_customers(customers, window, cx);
                            });
                        }
                    })
                    .detach();
                }
                _ => {}
            },
        )
        .detach();
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
            AppTab::Projects => self.project_list_view.clone().into_any_element(),
            _ => self.settings_view.clone().into_any_element(),
        };

        let dialog_layer = Root::render_dialog_layer(window, cx);
        let notification_layer = Root::render_notification_layer(window, cx);

        div().size_full().v_flex().child(TitleBar::new()).child(
            div()
                .flex()
                .flex_1()
                .child(self.sidebar.clone())
                .child(div().h_flex().flex_1().min_w_0().child(tab_to_show))
                .children(dialog_layer)
                .children(notification_layer)
                .relative()
                .when(true, |this| this.child(fps_monitor(window, cx))),
        )
    }
}
