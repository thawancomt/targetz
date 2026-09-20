use std::collections::HashMap;

use gpui_kit::{
    Entity,
    component::{date_picker::DatePickerState, input::InputState, select::SelectState},
};

pub struct TextFieldState {
    pub valeu: String,
    pub input: Entity<InputState>,
}

pub struct DateFieldState {
    pub valeu: String,
    pub input: Entity<DatePickerState>,
}

pub struct SelectFieldState {
    pub value: String,
    pub input: Entity<SelectState<Vec<&'static str>>>,
    pub options: &'static [&'static str],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FormFieldItem<T> {
    pub id: T,
    pub label: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FormFieldSelectItem<T> {
    pub id: T,
    pub label: &'static str,
    pub options: &'static [&'static str],
}

pub struct AppForm<T, F>(pub HashMap<FormFieldItem<T>, F>);
