use gpui_kit::{
    Context, InteractiveElement, IntoElement, ParentElement, Render, Styled, Window,
    base::{StyledExt, h_flex, v_flex},
    component::{
        ActiveTheme, Icon, IconName,
        button::{Button, ButtonVariants},
        input::Input,
        scroll::ScrollableElement,
    },
    div,
    prelude::FluentBuilder,
    px,
};

use super::{
    events::HomeEvent,
    item::{ItemKind, SearchItem},
    state::HomeView,
};

impl Render for HomeView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let groups = self.grouped_items();
        let query_empty = self.query.trim().is_empty();

        let mut section_elements = Vec::new();
        for (category, items) in groups {
            let count = items.len();
            let mut card_elements = Vec::new();
            for item in items {
                card_elements.push(Self::render_item_card(item, cx));
            }

            section_elements.push(
                v_flex()
                    .w_full()
                    .gap_3()
                    // Category Header with divider line
                    .child(
                        v_flex()
                            .w_full()
                            .gap_1()
                            .child(
                                h_flex()
                                    .w_full()
                                    .items_center()
                                    .justify_between()
                                    .child(
                                        h_flex()
                                            .items_center()
                                            .gap_2()
                                            .text_xs()
                                            .font_semibold()
                                            .text_color(theme.muted_foreground)
                                            .child(Icon::new(category.icon()))
                                            .child(category.title().to_uppercase()),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .px_2()
                                            .py_0p5()
                                            .rounded_full()
                                            .bg(theme.muted)
                                            .text_color(theme.muted_foreground)
                                            .child(format!("{}", count)),
                                    ),
                            )
                            .child(div().h(px(1.)).w_full().bg(theme.border)),
                    )
                    // Items List
                    .child(v_flex().w_full().gap_1p5().children(card_elements)),
            );
        }

        let is_empty = section_elements.is_empty();

        div()
            .id("home-view-scroll")
            .size_full()
            .min_h_0()
            .min_w_0()
            .overflow_y_scrollbar()
            .bg(theme.background)
            .child(
                v_flex()
                    .w_full()
                    .items_center()
                    .py_12()
                    .px_6()
                    .child(
                        v_flex()
                            .w_full()
                            .max_w(px(780.))
                            .gap_6()
                            // Header Section
                            .child(
                                v_flex()
                                    .items_center()
                                    .gap_2()
                                    .pb_2()
                                    .child(
                                        div()
                                            .text_3xl()
                                            .font_bold()
                                            .text_color(theme.foreground)
                                            .child("Targetz"),
                                    )
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(theme.muted_foreground)
                                            .child("Global command center & spotlight search"),
                                    ),
                            )
                            // Search Bar Section
                            .child(
                                div()
                                    .w_full()
                                    .p_1()
                                    .rounded_xl()
                                    .bg(theme.input)
                                    .border_1()
                                    .border_color(theme.border)
                                    .child(
                                        Input::new(&self.query_input)
                                            .prefix(Icon::new(IconName::Search))
                                            .cleanable(true),
                                    ),
                            )
                            // Results or Empty State
                            .when(is_empty, |this| {
                                this.child(
                                    v_flex()
                                        .items_center()
                                        .justify_center()
                                        .py_12()
                                        .gap_3()
                                        .child(
                                            div()
                                                .p_3()
                                                .rounded_full()
                                                .bg(theme.muted)
                                                .child(Icon::new(IconName::Search)),
                                        )
                                        .child(
                                            div()
                                                .text_base()
                                                .font_semibold()
                                                .text_color(theme.foreground)
                                                .when_else(
                                                    query_empty,
                                                    |t| t.child("No items found"),
                                                    |t| t.child(format!("No results for \"{}\"", self.query)),
                                                ),
                                        )
                                        .child(
                                            div()
                                                .text_sm()
                                                .text_color(theme.muted_foreground)
                                                .child("Search by name, email, phone number, codename, or actions."),
                                        ),
                                )
                            })
                            .children(section_elements),
                    ),
            )
    }
}

impl HomeView {
    fn render_item_card(item: &SearchItem, cx: &mut Context<Self>) -> Button {
        let kind = item.kind.clone();
        let title = item.title.clone();
        let subtitle = item.subtitle.clone();
        let icon = item.icon.clone();
        let theme = cx.theme().clone();

        Button::new(item.id.clone())
            .p_2()
            .h_auto()
            .w_full()
            .secondary()
            .child(
                h_flex()
                    .flex_1()
                    .items_center()
                    .justify_between()
                    .gap_3()
                    .p_1()
                    .child(
                        h_flex()
                            .items_center()
                            .gap_3()
                            .min_w_0()
                            .child(
                                div()
                                    .p_2()
                                    .rounded_md()
                                    .bg(theme.muted)
                                    .child(Icon::new(icon)),
                            )
                            .child(
                                v_flex()
                                    .min_w_0()
                                    .gap_0p5()
                                    .child(
                                        div()
                                            .font_medium()
                                            .text_sm()
                                            .text_color(theme.foreground)
                                            .child(title),
                                    )
                                    .when_some(subtitle, |this, sub| {
                                        this.child(
                                            div()
                                                .text_xs()
                                                .text_color(theme.muted_foreground)
                                                .child(sub),
                                        )
                                    }),
                            ),
                    )
                    .child(
                        div()
                            .text_color(theme.muted_foreground)
                            .child(Icon::new(IconName::ArrowRight)),
                    ),
            )
            .on_click(cx.listener(move |_this, _, _window, cx| match &kind {
                ItemKind::Customer(c) => {
                    cx.emit(HomeEvent::OpenCustomer(c.clone()));
                }
                ItemKind::Project(p) => {
                    cx.emit(HomeEvent::OpenProject(p.clone()));
                }
                ItemKind::Action(a) => {
                    cx.emit(HomeEvent::TriggerAction(a.clone()));
                }
            }))
    }
}
