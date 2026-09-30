use crate::models::ProjectDocumentStats;
use crate::ui::project_document_detail_view::{
    ProjectDocumentDetailView, events::DocumentDetailEvents,
};
use crate::ui::project_documents_list_view::{
    ProjectDocumentsListView, events::DocumentListEvents,
};
use gpui_kit::{
    App, AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Window,
    base::StyledExt,
    component::{
        ActiveTheme, IconName,
        button::{Button, ButtonVariants},
        tab::{Tab, TabBar},
    },
    div,
};

pub enum ActiveDocumentView {
    AllDocuments,
    Project(ProjectDocumentStats),
}

impl ActiveDocumentView {
    pub fn selected_index(&self) -> usize {
        match self {
            ActiveDocumentView::AllDocuments => 0,
            ActiveDocumentView::Project(_) => 1,
        }
    }
}

pub struct TabDocumentsView {
    pub state: ActiveDocumentView,
    pub list_view: Entity<ProjectDocumentsListView>,
    pub detail_view: Entity<ProjectDocumentDetailView>,
}

impl TabDocumentsView {
    pub fn view(
        window: &mut Window,
        cx: &mut App,
        list_view: Entity<ProjectDocumentsListView>,
    ) -> Entity<Self> {
        let detail_view = ProjectDocumentDetailView::view(window, cx, None);
        cx.new(|cx| Self::new(window, cx, list_view, detail_view))
    }

    pub fn new(
        window: &mut Window,
        cx: &mut Context<Self>,
        list_view: Entity<ProjectDocumentsListView>,
        detail_view: Entity<ProjectDocumentDetailView>,
    ) -> Self {
        cx.subscribe_in(
            &list_view,
            window,
            move |tab_view, _, event, window, cx| match event {
                DocumentListEvents::OpenProject(project_stats) => {
                    tab_view.state = ActiveDocumentView::Project(project_stats.clone());
                    tab_view.detail_view.update(cx, |detail, detail_cx| {
                        detail.set_project(project_stats.clone(), window, detail_cx);
                    });
                    cx.notify();
                }
            },
        )
        .detach();

        cx.subscribe(&detail_view, |tab_view, _, event, cx| {
            let (project_id, documents) = match event {
                DocumentDetailEvents::DocumentsUploaded {
                    project_id,
                    documents,
                }
                | DocumentDetailEvents::DocumentDeleted {
                    project_id,
                    documents,
                } => (project_id, documents),
            };
            tab_view.list_view.update(cx, |list, list_cx| {
                list.update_project_documents(*project_id, documents, list_cx);
            });
        })
        .detach();

        Self {
            state: ActiveDocumentView::AllDocuments,
            list_view,
            detail_view,
        }
    }
}

impl Render for TabDocumentsView {
    fn render(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl gpui_kit::prelude::IntoElement {
        let mut tabs = TabBar::new("documents-tabs")
            .selected_index(self.state.selected_index())
            .child(Tab::new().label("All Projects"));

        if let ActiveDocumentView::Project(ref p) = self.state {
            tabs = tabs.child(
                Tab::new().label(p.project_name.clone()).suffix(
                    Button::new(format!("remove-doc-project-{}", p.project_id))
                        .ghost()
                        .text_color(cx.theme().selection)
                        .on_click(cx.listener(move |this, _, _window, cx| {
                            this.state = ActiveDocumentView::AllDocuments;
                            cx.notify();
                        }))
                        .child(IconName::Close),
                ),
            );
        }

        tabs = tabs.on_click(cx.listener(|this, index: &usize, _, cx| {
            if *index == 0 {
                this.state = ActiveDocumentView::AllDocuments;
                cx.notify();
            }
        }));

        let content = match self.state {
            ActiveDocumentView::AllDocuments => self.list_view.clone().into_any_element(),
            ActiveDocumentView::Project(_) => self.detail_view.clone().into_any_element(),
        };

        div().v_flex().w_full().h_full().child(tabs).child(content)
    }
}
