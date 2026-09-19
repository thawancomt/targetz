use std::collections::HashMap;

use gpui_kit::{
    App, AppContext, Context, Entity, EventEmitter, InteractiveElement, IntoElement, ParentElement,
    Render, Styled, Window,
    base::input::InputEvent,
    component::{
        ActiveTheme, WindowExt,
        button::{Button, ButtonVariants},
        form::Field,
        input::{Input, InputState},
    },
    div,
};
use shared::{
    customer::{Customer, Draft},
    customer_interaction::{Interaction, InteractionStatus},
};

use crate::interactions::interaction_repository::InteractionRepository;

pub enum CreateInteractionViewEvent {
    CreatedNewInteraction(Interaction),
    DeletedInteraction(i64), // interaction id
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionFormFieldId {
    InteractionDate,
    Status,
    Note,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

impl EventEmitter<CreateInteractionViewEvent> for CreateInteractionView {}

pub struct CreateInteractionView {
    pub interaction_repository: InteractionRepository,
    pub form_fields: HashMap<InteractionFormFieldId, FormFieldState>,
    pub customer: Option<Customer>,
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
        let theme = cx.theme();
        div()
            .id("create-interaction-div")
            .p_2()
            .border_1()
            .bg(theme.accent)
            .border_color(cx.theme().selection)
            .hover(|s| s.border_color(theme.primary))
            .child(
                div()
                    .child(format!(
                        "Create new interaction with {}",
                        if self.customer.is_some() {
                            self.customer.clone().unwrap().name
                        } else {
                            "".to_string()
                        }
                    ))
                    .text_lg(),
            )
            .children(TEXT_FIELDS.iter().map(|f| {
                let form_state = self.form_fields.get(&f.id).unwrap();

                let input = &form_state.input;
                let label = f.label;

                field(label.to_string(), input)
            }))
            .child(
                Button::new("Save-interaction")
                    .label("Save")
                    .primary()
                    .mt_2()
                    .on_click(cx.listener(|view, _click, window, context| {
                        view.save_interaction(context);
                    })),
            )
    }
}

impl CreateInteractionView {
    pub fn view(
        window: &mut Window,
        cx: &mut App,
        customer: Option<Customer>,
        interaction_repository: InteractionRepository,
    ) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx, customer, interaction_repository))
    }

    pub fn new(
        window: &mut Window,
        cx: &mut Context<Self>,
        customer: Option<Customer>,
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
            customer,
        }
    }

    pub fn save_interaction(&mut self, cx: &mut Context<Self>) {
        let repository_handle = self.interaction_repository.clone();

        let note = self
            .form_fields
            .get(&InteractionFormFieldId::Note)
            .unwrap()
            .value
            .clone();

        let status = self
            .form_fields
            .get(&InteractionFormFieldId::Status)
            .unwrap()
            .value
            .clone();

        let date = self
            .form_fields
            .get(&InteractionFormFieldId::InteractionDate)
            .unwrap()
            .value
            .clone();

        let interaction = Interaction {
            id: Draft,
            interaction_date: date,
            note: if !note.is_empty() { Some(note) } else { None },
            status: InteractionStatus::from_str(&status),
        };

        println!("{:#?}", interaction);

        cx.spawn(async move |this, cx| {
            let result = repository_handle.create_interaction(interaction).await;

            match result {
                Ok(new_interaction) => {
                    let _ = this.update_in(cx, |this, window, context| {
                        this.reset_form(window, context);

                        // notify listeners that a new interaction have been created
                        context.emit(CreateInteractionViewEvent::CreatedNewInteraction(
                            new_interaction,
                        ));

                        window.push_notification("New interaction registered", context);
                    });
                }
                Err(e) => eprintln!("{}", e.to_string()),
            };
        })
        .detach();
    }

    pub fn reset_form(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        for field in &mut self.form_fields {
            field.1.input.update(cx, |input, context| {
                input.clean(window, context);
            });
            field.1.value = String::new();

            cx.notify();
        }
    }
}
