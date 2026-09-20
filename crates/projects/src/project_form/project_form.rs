use std::collections::HashMap;

use gpui_kit::{
    App, AppContext, Context, Entity, Window,
    component::{input::InputState, select::SelectState},
};
use shared::{
    app_form::{FormFieldItem, FormFieldSelectItem, SelectFieldState, TextFieldState},
    project::ProjectStatus,
};

pub struct CreateProjectView {
    pub text_fields: HashMap<CreateProjectViewField, TextFieldState>,
    pub select_fields: HashMap<CreateProjectViewField, SelectFieldState>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CreateProjectViewField {
    Name,
    CurrentVersion,
    CreatedAt,
    Status,
    Description,
    StartDate,
    SiteUrl,
    Codename,
    TargetDeadline,
    UpdatedAt,
    Budget,
}

pub const TEXT_FIELDS: &[FormFieldItem<CreateProjectViewField>] = &[
    FormFieldItem {
        id: CreateProjectViewField::Name,
        label: "Name",
    },
    FormFieldItem {
        id: CreateProjectViewField::Description,
        label: "Description",
    },
    FormFieldItem {
        id: CreateProjectViewField::SiteUrl,
        label: "Site url",
    },
    FormFieldItem {
        id: CreateProjectViewField::Codename,
        label: "Project Codename",
    },
    FormFieldItem {
        id: CreateProjectViewField::CurrentVersion,
        label: "Version",
    },
    FormFieldItem {
        id: CreateProjectViewField::Budget,
        label: "Budget",
    },
];

pub const SELECT_FIELDS: &[FormFieldSelectItem<CreateProjectViewField>] = &[FormFieldSelectItem {
    id: CreateProjectViewField::Status,
    label: "Status",
    options: &[
        ProjectStatus::Started.as_str(),
        ProjectStatus::Prospecting.as_str(),
    ],
}];

pub const DATE_FIELDS: &[FormFieldItem<CreateProjectViewField>] = &[
    FormFieldItem {
        id: CreateProjectViewField::CreatedAt,
        label: "Created at",
    },
    FormFieldItem {
        id: CreateProjectViewField::StartDate,
        label: "Started at",
    },
    FormFieldItem {
        id: CreateProjectViewField::TargetDeadline,
        label: "Project due",
    },
];

impl CreateProjectView {
    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }

    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let mut text_fields: HashMap<CreateProjectViewField, TextFieldState> = HashMap::new();
        let mut select_fields: HashMap<CreateProjectViewField, SelectFieldState> = HashMap::new();

        for descriptor in TEXT_FIELDS {
            let id = descriptor.id;
            let input = cx.new(|cx| InputState::new(window, cx));

            cx.observe(&input, |this, input, cx| {
                this.text_fields.get_mut(&descriptor.id).unwrap().valeu =
                    input.read(cx).text().to_string()
            })
            .detach();

            text_fields.insert(
                id,
                TextFieldState {
                    valeu: String::new(),
                    input: input,
                },
            );
        }

        for descriptor in SELECT_FIELDS {
            let id = descriptor.id;
            let select_state =
                cx.new(|cx| SelectState::new(descriptor.options.to_vec(), None, window, cx));

            cx.observe(&select_state, move |this, input, cx| {
                this.select_fields.get_mut(&descriptor.id).unwrap().value =
                    input.read(cx).selected_value().unwrap_or(&"").to_string()
            })
            .detach();

            select_fields.insert(
                id,
                SelectFieldState {
                    value: String::new(),
                    input: select_state,
                    options: descriptor.options,
                },
            );
        }

        CreateProjectView {
            text_fields,
            select_fields,
        }
    }
}
