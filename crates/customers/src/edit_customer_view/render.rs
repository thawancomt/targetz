use gpui_kit::{
    Context, IntoElement, ParentElement, Render, Styled, Window,
    component::{
        button::{Button, ButtonVariants},
        form::Field,
        scroll::ScrollableElement,
        switch::Switch,
    },
    div, px,
};
use shared::{form_utils::app_field, theme::AppColors};

use crate::edit_customer_view::state::CustomerUpdateView;

fn switch_field(label: String, child: impl IntoElement) -> impl IntoElement {
    Field::new()
        .label(label)
        .bg(AppColors::Border.hsla())
        .p_2()
        .w_auto()
        .rounded_md()
        .child(child)
}

impl Render for CustomerUpdateView {
    fn render(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let wide = window.viewport_size().width >= px(768. + 300.);

        div()
            .p_2()
            .flex()
            .flex_col()
            .gap_2()
            .overflow_y_scrollbar()
            .child(
                div()
                    .grid()
                    .grid_cols(if wide { 2 } else { 1 })
                    .gap_2()
                    .child(app_field("Name", &self.name))
                    .child(app_field("Email", &self.email))
                    .child(app_field("Phone number", &self.phone_number))
                    .child(app_field("Address", &self.address))
                    .child(app_field("Instagram", &self.instagram_url))
                    .child(app_field("Site url", &self.site_url))
                    .child(
                        div()
                            .col_span_full()
                            .flex()
                            .w_full()
                            .gap_2()
                            .mt_2()
                            .child(switch_field(
                                "Have been contacted?".to_string(),
                                Switch::new("edit-contacted").checked(self.contacted).on_change(
                                    cx.listener(|this, value, _window, cx| {
                                        this.set_contacted(*value);
                                        cx.notify();
                                    }),
                                ),
                            ))
                            .child(switch_field(
                                "Is already client?".to_string(),
                                Switch::new("edit-is-client").checked(self.is_client).on_change(
                                    cx.listener(|this, value, _, cx| {
                                        this.set_is_client(*value);
                                        cx.notify();
                                    }),
                                ),
                            )),
                    ),
            )
            .child(
                div()
                    .flex()
                    .justify_end()
                    .gap_2()
                    .mt_3()
                    .child(
                        Button::new("reset-edit-form")
                            .ghost()
                            .label("Reset")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.reset_form(window, cx);
                            })),
                    )
                    .child(
                        Button::new("save-customer-changes")
                            .primary()
                            .label("Save Changes")
                            .on_click(cx.listener(|this, _, _window, cx| {
                                this.update_customer(cx);
                            })),
                    ),
            )
    }
}
