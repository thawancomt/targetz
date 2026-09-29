use gpui_kit::{
    App, Entity, IntoElement, ParentElement,
    base::input::InputState,
    component::{
        date_picker::{DatePicker, DatePickerState},
        form::Field,
        input::Input,
        select::SelectState,
    },
};

/// Given an InputState extract its text value
pub fn text_input(input: &Entity<InputState>, cx: &App) -> Option<String> {
    let value = input.read(cx).text().to_string();

    match value.is_empty() {
        true => None,
        false => Some(value),
    }
}

/// Given an DatePickerState extract its selected date
pub fn date_input(input: &Entity<DatePickerState>, cx: &App) -> Option<String> {
    let date = input.read(cx).date();

    Some(date.to_string())
}

/// Given and SelectState return its selected value
/// Note: only single select mode is supported
pub fn select(input: &Entity<SelectState<Vec<&'static str>>>, cx: &App) -> Option<String> {
    if let Some(value) = input.read(cx).selected_value() {
        if value.is_empty() {
            return None;
        }
        Some(value.to_string())
    } else {
        None
    }
}

/// Return a app stylized field, its just a wrapper but, this implement the app style so use this for consistence
pub fn app_field(label: &'static str, input: &Entity<InputState>) -> Field {
    Field::new().child(Input::new(input)).label(label)
}

/// Return a app stylized date field, its just a wrapper but, this implement the app style so use this for consistence
pub fn app_date_field(label: &'static str, input: &Entity<DatePickerState>) -> Field {
    Field::new().child(DatePicker::new(input)).label(label)
}
