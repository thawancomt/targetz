use std::collections::HashMap;

use gpui_kit::{
    Entity,
    component::{date_picker::DatePickerState, input::InputState, select::SelectState},
};

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
