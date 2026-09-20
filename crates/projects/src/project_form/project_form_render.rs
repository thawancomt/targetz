use gpui_kit::{
    AppContext, ParentElement, Render, Styled,
    component::{
        ActiveTheme, form::Field, input::Input, scroll::ScrollableElement, select::Select,
    },
    div,
};

use crate::project_form::project_form::{
    CreateProjectView, CreateProjectViewField, SELECT_FIELDS, TEXT_FIELDS,
};

impl Render for CreateProjectView {
    fn render(
        &mut self,
        _window: &mut gpui_kit::Window,
        cx: &mut gpui_kit::prelude::Context<Self>,
    ) -> impl gpui_kit::prelude::IntoElement {
        let theme = cx.theme();

        let project_name = match self.text_fields.get(&CreateProjectViewField::Name).as_ref() {
            Some(state) => state.valeu.clone(),
            None => "".to_string(),
        };

        div()
            .p_2()
            .flex()
            .flex_col()
            .min_h_0()
            .overflow_y_scrollbar()
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .child("Create a new project ")
                    .child(
                        div()
                            .child(format!(" [{}]", project_name))
                            .text_color(theme.accent_foreground),
                    )
                    .text_2xl()
                    .text_color(theme.primary),
            )
            .child(div().children({
                TEXT_FIELDS.iter().map(|f| {
                    let label = f.label;
                    let input = self.text_fields.get(&f.id).unwrap().input.clone();

                    Field::new().child(Input::new(&input)).label(label)
                })
            }))
            .child(div().children({
                SELECT_FIELDS.iter().map(|f| {
                    let label = f.label;
                    let input = self.select_fields.get(&f.id).unwrap().input.clone();

                    Field::new().child(Select::new(&input)).label(label)
                })
            }))
    }
}
