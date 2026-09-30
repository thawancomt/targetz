use crate::home::{HomeEvent, HomeView, QuickAction};
use customers::{
    customer_repository::CustomerRepository, customers_list_view::CustomerListView,
    tab_customers_view::TabCustomerView,
};
use gpui_fps::fps_monitor;
use gpui_kit::{
    base::StyledExt,
    component::{Root, TitleBar},
    div,
    prelude::FluentBuilder,
    App, AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Window,
};
use projects::{
    customer_projects_section::CustomerProjectsSection,
    project_form_view::CreateProjectView,
    project_repository::ProjectRepository,
    projects_list_view::{events::ProjectListEvents, state::ProjectsListView},
    tab_projects_view::TabProjectsView,
};
use documents::{
    document_manager::DocumentManager,
    ui::{
        customer_documents_section::CustomerDocumentsSection,
        project_documents_list_view::ProjectDocumentsListView,
        project_documents_section::ProjectDocumentsSection,
        tab_documents_view::TabDocumentsView,
    },
};
use settings::settings_view::SettingsView;
use shared::{db::DbPool, events::AppEvent, AppTab};
use sidebar::sidebar_view::{SidebarEvent, SidebarView};

pub struct AppShell {
    pub current_tab: AppTab,
    pub sidebar: Entity<SidebarView>,
    pub settings_view: Entity<SettingsView>,
    pub customer_tab_view: Entity<TabCustomerView>,
    pub create_project_view: Entity<CreateProjectView>,
    pub project_list_view: Entity<ProjectsListView>,
    pub project_tab_view: Entity<TabProjectsView>,
    pub home_view: Entity<HomeView>,
    pub documents_list_view: Entity<ProjectDocumentsListView>,
    pub documents_tab_view: Entity<TabDocumentsView>,
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
        let project_tab_view = TabProjectsView::view(window, cx, project_list_view.clone());
        let documents_section = ProjectDocumentsSection::view(window, cx);
        let documents_section_for_detail = documents_section.clone();
        project_tab_view.update(cx, |tab, cx| {
            tab.project_detail_view.update(cx, |detail, cx| {
                detail.set_documents_section(
                    documents_section_for_detail.clone().into(),
                    {
                        let section = documents_section_for_detail.clone();
                        move |project_id, _window, cx| {
                            section.update(cx, |section, cx| {
                                section.set_project(project_id, cx);
                            });
                        }
                    },
                    cx,
                );
            });
        });
        let customer_projects_section = CustomerProjectsSection::view(window, cx);
        let customer_documents_section = CustomerDocumentsSection::view(window, cx);
        tab_customers_view.update(cx, |tab, cx| {
            tab.customer_detail_view.update(cx, |detail, cx| {
                detail.set_extra_sections(
                    vec![
                        customer_projects_section.clone().into(),
                        customer_documents_section.clone().into(),
                    ],
                    {
                        let projects = customer_projects_section.clone();
                        let documents = customer_documents_section.clone();
                        move |customer_id, cx| {
                            projects.update(cx, |s, cx| s.set_customer(customer_id, cx));
                            documents.update(cx, |s, cx| s.set_customer(customer_id, cx));
                        }
                    },
                    cx,
                );
            });
        });
        let documents_list_view = ProjectDocumentsListView::view(window, cx);
        let documents_tab_view = TabDocumentsView::view(window, cx, documents_list_view.clone());
        let home_view = HomeView::view(window, cx);

        let customer_tab_view_for_home = tab_customers_view.clone();
        let project_tab_view_for_home = project_tab_view.clone();
        let sidebar_for_home = sidebar.clone();

        cx.subscribe_in(
            &home_view,
            window,
            move |this, _emitter, event: &HomeEvent, window, cx| match event {
                HomeEvent::OpenCustomer(customer) => {
                    this.current_tab = AppTab::Targetz;
                    sidebar_for_home.update(cx, |sidebar, cx| {
                        sidebar.active_tab = AppTab::Targetz;
                        cx.notify();
                    });
                    customer_tab_view_for_home.update(cx, |tab, cx| {
                        tab.state.open(customer.clone());
                        tab.customer_detail_view.update(cx, |detail, cx| {
                            detail.set_customer(customer.clone(), cx);
                            detail.hydrate_interactions(cx);
                            cx.notify();
                        });
                        cx.notify();
                    });
                    cx.notify();
                }
                HomeEvent::OpenProject(project) => {
                    this.current_tab = AppTab::Projects;
                    sidebar_for_home.update(cx, |sidebar, cx| {
                        sidebar.active_tab = AppTab::Projects;
                        cx.notify();
                    });
                    project_tab_view_for_home.update(cx, |tab, cx| {
                        tab.state.open(project.clone());
                        tab.project_detail_view.update(cx, |detail, cx| {
                            detail.set_project(project.clone(), window, cx);
                            cx.notify();
                        });
                        cx.notify();
                    });
                    cx.notify();
                }
                HomeEvent::TriggerAction(action) => match action {
                    QuickAction::CreateCustomer => {
                        this.current_tab = AppTab::Targetz;
                        sidebar_for_home.update(cx, |sidebar, cx| {
                            sidebar.active_tab = AppTab::Targetz;
                            cx.notify();
                        });
                        customer_tab_view_for_home.update(cx, |tab, cx| {
                            tab.customer_list_view.update(cx, |list, cx| {
                                list.open_create_customer_dialog(window, cx);
                            });
                        });
                        cx.notify();
                    }
                    QuickAction::CreateProject => {
                        this.current_tab = AppTab::Projects;
                        sidebar_for_home.update(cx, |sidebar, cx| {
                            sidebar.active_tab = AppTab::Projects;
                            cx.notify();
                        });
                        project_tab_view_for_home.update(cx, |tab, cx| {
                            tab.projects_list_view.update(cx, |list, cx| {
                                list.open_create_project_dilaog(window, cx);
                            });
                        });
                        cx.notify();
                    }
                    QuickAction::ToggleTheme => {
                        sidebar_for_home.update(cx, |sidebar, cx| {
                            sidebar.toggle_theme(window, cx);
                        });
                    }
                    QuickAction::OpenSettings => {
                        this.current_tab = AppTab::Settings;
                        sidebar_for_home.update(cx, |sidebar, cx| {
                            sidebar.active_tab = AppTab::Settings;
                            cx.notify();
                        });
                        cx.notify();
                    }
                },
            },
        )
        .detach();

        Self::observe_tab_events(&sidebar, cx);

        // we need window here due the select on the form (Yeah, I dont like it too)
        Self::observer_project_list(&project_list_view.clone(), window, cx);

        Self {
            sidebar,
            settings_view,
            current_tab: initial_current_tab,
            customer_tab_view: tab_customers_view,
            create_project_view,
            project_list_view,
            project_tab_view,
            home_view,
            documents_list_view,
            documents_tab_view,
        }
    }

    pub fn observe_tab_events(tab_view: &Entity<SidebarView>, cx: &mut Context<Self>) {
        cx.subscribe(&tab_view, move |this, _entity, event, cx| {
            match event {
                SidebarEvent::TabClick(AppTab::Home) => {
                    this.home_view.update(cx, |home, cx| {
                        home.hydrate_data(cx);
                    });
                }
                SidebarEvent::TabClick(AppTab::Settings) => {
                    this.settings_view.update(cx, |_settings, cx| {
                        // Notify settings to re-sync active theme
                        cx.notify();
                    });
                }
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
                SidebarEvent::TabClick(AppTab::Documents) => {
                    let doc_list_handle = this.documents_list_view.clone();
                    let pool = cx.global::<DbPool>().0.clone();

                    cx.spawn(async move |_this, cx| {
                        if let Ok(manager) = DocumentManager::new(pool) {
                            if let Ok(stats) = manager.get_project_document_stats().await {
                                doc_list_handle.update(cx, |view, cx| {
                                    view.with_projects(stats, cx);
                                    cx.notify();
                                });
                            }
                        }
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
            AppTab::Home => self.home_view.clone().into_any_element(),
            AppTab::Settings => self.settings_view.clone().into_any_element(),
            AppTab::Targetz => self.customer_tab_view.clone().into_any_element(),
            AppTab::Projects => self.project_tab_view.clone().into_any_element(),
            AppTab::Documents => self.documents_tab_view.clone().into_any_element(),
            _ => self.home_view.clone().into_any_element(),
        };

        let dialog_layer = Root::render_dialog_layer(window, cx);
        let notification_layer = Root::render_notification_layer(window, cx);

        div().size_full().v_flex().child(TitleBar::new()).child(
            div()
                .flex()
                .flex_1()
                .min_h_0()
                .child(self.sidebar.clone())
                .child(
                    div()
                        .h_flex()
                        .flex_1()
                        .min_w_0()
                        .min_h_0()
                        .h_full()
                        .child(tab_to_show),
                )
                .children(dialog_layer)
                .children(notification_layer)
                .relative(), /* .when(true, |this| this.child(fps_monitor(window, cx))), */
        )
    }
}
