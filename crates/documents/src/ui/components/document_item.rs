use std::path::PathBuf;

use crate::models::DocumentWithCustomers;
use gpui_kit::{
    App, InteractiveElement, IntoElement, ParentElement, RenderOnce, Styled, Window,
    base::StyledExt,
    component::{
        ActiveTheme, IconName,
        button::{Button, ButtonVariants},
    },
    div,
};

/// Card showing a document's name, extension and path, with a button that
/// opens the file in the OS default application.
#[derive(IntoElement)]
pub struct DocumentItem {
    document: DocumentWithCustomers,
    data_dir: PathBuf,
    customer_count: usize,
}

impl DocumentItem {
    /// `data_dir` is the base directory the document's relative `path` lives in;
    /// `customer_count` is how many distinct customers the document mentions.
    pub fn new(document: DocumentWithCustomers, data_dir: PathBuf, customer_count: usize) -> Self {
        Self {
            document,
            data_dir,
            customer_count,
        }
    }
}

impl RenderOnce for DocumentItem {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let doc = self.document;
        let full_path = self.data_dir.join(&doc.path);

        div()
            .h_flex()
            .w_full()
            .p_4()
            .gap_4()
            .justify_between()
            .items_end()
            .border_1()
            .border_color(theme.border)
            .rounded_md()
            .child(
                div()
                    .v_flex()
                    .gap_1()
                    .child(div().child(doc.original_name.clone()).text_xl())
                    .child(
                        div()
                            .v_flex()
                            .text_sm()
                            .text_color(theme.muted_foreground)
                            .child(format!("Extension: {}", doc.extension))
                            .child(format!("Path: {}", doc.path))
                            .child(format!("Customers mentioned: {}", self.customer_count)),
                    ),
            )
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
    }
}
