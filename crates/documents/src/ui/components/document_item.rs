use std::path::PathBuf;

use crate::models::DocumentWithCustomers;
use gpui_kit::{
    App, ClickEvent, IntoElement, ParentElement, RenderOnce, Styled, Window,
    base::{Disableable, StyledExt},
    component::{
        ActiveTheme, IconName,
        button::{Button, ButtonVariants},
    },
    div,
};
use shared::ui::rail_card;

/// Card showing a document's name, extension and path, with buttons to open
/// the file in the OS default application or to ask the parent to delete it.
#[derive(IntoElement)]
pub struct DocumentItem {
    document: DocumentWithCustomers,
    data_dir: PathBuf,
    customer_count: usize,
    delete_disabled: bool,
    on_delete: Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>,
}

impl DocumentItem {
    /// `data_dir` is the base directory the document's relative `path` lives in;
    /// `customer_count` is how many distinct customers the document mentions.
    /// `delete_disabled` locks Delete while the parent is already deleting.
    /// `on_delete` is invoked on Delete; the parent owns persistence.
    pub fn new(
        document: DocumentWithCustomers,
        data_dir: PathBuf,
        customer_count: usize,
        delete_disabled: bool,
        on_delete: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        Self {
            document,
            data_dir,
            customer_count,
            delete_disabled,
            on_delete: Box::new(on_delete),
        }
    }
}

impl RenderOnce for DocumentItem {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let delete_disabled = self.delete_disabled;
        let doc = self.document;
        let full_path = self.data_dir.join(&doc.path);
        let on_delete = self.on_delete;

        rail_card(
            false,
            div()
                .h_flex()
                .flex_1()
                .min_w_0()
                .p_3()
                .gap_3()
                .justify_between()
                .items_center()
                .child(
                    div()
                        .v_flex()
                        .gap_1()
                        .child(
                            div()
                                .child(doc.original_name.clone())
                                .text_color(theme.foreground),
                        )
                        .child(
                            div()
                                .v_flex()
                                .font_family("Geist Mono")
                                .text_xs()
                                .text_color(theme.muted_foreground)
                                .child(format!(".{} · {}", doc.extension, doc.path))
                                .child(format!("{} customers mentioned", self.customer_count)),
                        ),
                )
                .child(
                    div()
                        .h_flex()
                        .gap_2()
                        .child(
                            Button::new(format!("open-doc-{}", doc.id))
                                .primary()
                                .label("Open")
                                .child(IconName::ExternalLink)
                                .on_click(move |_, _, _| {
                                    // Hands the file to the OS, which picks the default app.
                                    if let Err(e) = open::that_detached(&full_path) {
                                        eprintln!("Failed to open {}: {e}", full_path.display());
                                    }
                                }),
                        )
                        .child(
                            Button::new(format!("delete-doc-{}", doc.id))
                                .disabled(delete_disabled)
                                .secondary()
                                .label("Delete")
                                .child(IconName::SquareTerminal)
                                .on_click(move |event, window, cx| {
                                    (on_delete)(event, window, cx);
                                }),
                        ),
                ),
            &theme,
        )
    }
}
