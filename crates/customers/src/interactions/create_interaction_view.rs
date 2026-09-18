use std::collections::HashMap;

use gpui_kit::{
    App, AppContext, Context, Entity, IntoElement, ParentElement, Render, Window,
    base::input::InputEvent,
    component::{
        form::Field,
        input::{Input, InputState},
    },
    div,
};

use crate::interactions::interaction_repository::InteractionRepository;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionFormFieldId {
    InteractionDate,
    Status,
    Note,
}

pub struct FormFieldDescriptor {
    pub id: InteractionFormFieldId,
    pub label: &'static str,
}

pub const TEXT_FIELDS: &[FormFieldDescriptor] = &[
    FormFieldDescriptor {
        id: InteractionFormFieldId::InteractionDate,
        label: "Interaction date",
    },
    FormFieldDescriptor {
        id: InteractionFormFieldId::Note,
        label: "Notes about interaction",
    },
    FormFieldDescriptor {
        id: InteractionFormFieldId::Status,
        label: "The result of interaction",
    },
];

pub struct FormFieldState {
    pub value: String,
    pub input: Entity<InputState>,
}

pub struct CreateInteractionView {
    pub interaction_repository: InteractionRepository,
    pub form_fields: HashMap<InteractionFormFieldId, FormFieldState>,
}
fn field(label: String, input: &Entity<InputState>) -> impl IntoElement {
    Field::new().label(label.clone()).child(Input::new(input))
}

impl Render for CreateInteractionView {
    fn render(
        &mut self,
        window: &mut gpui_kit::Window,
        cx: &mut gpui_kit::prelude::Context<Self>,
    ) -> impl gpui_kit::prelude::IntoElement {
        div()
            .child(format!(
                "Create new interaction with {}",
                if self.interaction_repository.customer.is_some() {
                    self.interaction_repository.clone().customer.unwrap().name
                } else {
                    "".to_string()
                }
            ))
            .children(TEXT_FIELDS.iter().map(|f| {
                let form_state = self.form_fields.get(&f.id).unwrap();

                let input = &form_state.input;
                let label = f.label;

                field(label.to_string(), input)
            }))
    }
}

impl CreateInteractionView {
    pub fn view(
        window: &mut Window,
        cx: &mut App,
        interaction_repository: InteractionRepository,
    ) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx, interaction_repository))
    }

    pub fn new(
        window: &mut Window,
        cx: &mut Context<Self>,
        interaction_repository: InteractionRepository,
    ) -> Self {
        let mut form: HashMap<InteractionFormFieldId, FormFieldState> = HashMap::new();

        for desc in TEXT_FIELDS {
            let input = cx.new(|cx| InputState::new(window, cx));
            let id = desc.id;

            cx.subscribe(&input, move |this, input, _event: &InputEvent, cx| {
                let text = this.form_fields.get_mut(&id);

                if let Some(input_value) = text {
                    input_value.value = input.read(cx).text().to_string();
                };
                cx.notify();
            })
            .detach();

            form.insert(
                id,
                FormFieldState {
                    value: String::new(),
                    input,
                },
            );
        }

        Self {
            interaction_repository,
            form_fields: form,
        }
    }
}
