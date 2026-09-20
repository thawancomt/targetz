use std::collections::HashMap;

use gpui_kit::{
    App, AppContext, Context, Entity, EventEmitter, InteractiveElement, IntoElement, ParentElement,
    Render, Styled, Window,
    base::{IndexPath, input::InputEvent},
    component::{
        ActiveTheme, Sizable, WindowExt,
        button::{Button, ButtonVariants},
        date_picker::{DatePicker, DatePickerState},
        form::Field,
        input::{Input, InputState},
        select::{Select, SelectItem, SelectState},
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FormFieldSelectDescriptor {
    pub id: InteractionFormFieldId,
    pub label: &'static str,
    pub options: fn() -> Vec<String>,
}

pub const TEXT_FIELDS: &[FormFieldDescriptor] = &[FormFieldDescriptor {
    id: InteractionFormFieldId::Note,
    label: "Notes about interaction",
}];

pub const DATE_FIELDS: &[FormFieldDescriptor] = &[FormFieldDescriptor {
    id: InteractionFormFieldId::InteractionDate,
    label: "Interaction date",
}];

pub const SELECT_FIELDS: &[FormFieldSelectDescriptor] = &[FormFieldSelectDescriptor {
    id: InteractionFormFieldId::Status,
    label: "Interaction date",
    options: || {
        vec![
            InteractionStatus::Contacted.as_str().to_string(),
            InteractionStatus::NewClient.as_str().to_string(),
            InteractionStatus::NoResponse.as_str().to_string(),
            InteractionStatus::Refused.as_str().to_string(),
        ]
    },
}];

pub struct FormFieldState<T> {
    pub value: String,
    pub input: Entity<T>,
}

impl EventEmitter<CreateInteractionViewEvent> for CreateInteractionView {}

pub struct CreateInteractionView {
    pub interaction_repository: InteractionRepository,
    pub text_fields: HashMap<InteractionFormFieldId, FormFieldState<InputState>>,

    pub date_fields: HashMap<InteractionFormFieldId, FormFieldState<DatePickerState>>,
    pub select_fields: HashMap<InteractionFormFieldId, FormFieldState<SelectState<Vec<String>>>>,

    pub customer: Option<Customer>,
}
fn field(label: String, input: &Entity<InputState>) -> impl IntoElement {
    Field::new().label(label.clone()).child(Input::new(input))
}
fn date_field(label: String, input: &Entity<DatePickerState>) -> impl IntoElement {
    Field::new()
        .label(label.clone())
        .child(DatePicker::new(input).large())
}

fn select_field(label: String, input: &Entity<SelectState<Vec<String>>>) -> impl IntoElement {
    Field::new().label(label.clone()).child(Select::new(input))
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
                        "Register interaction with {}",
                        if self.customer.is_some() {
                            self.customer.clone().unwrap().name
                        } else {
                            "".to_string()
                        }
                    ))
                    .text_lg()
                    .text_color(theme.primary),
            )
            .children(TEXT_FIELDS.iter().map(|f| {
                let form_state = self.text_fields.get(&f.id).unwrap();

                let input = &form_state.input;
                let label = f.label;

                field(label.to_string(), input)
            }))
            .children(DATE_FIELDS.iter().map(|f| {
                let form_state = self.date_fields.get(&f.id).unwrap();

                let input = &form_state.input;
                let label = f.label;

                date_field(label.to_string(), input)
            }))
            .children(SELECT_FIELDS.iter().map(|f| {
                let form_state = self.select_fields.get(&f.id).unwrap();

                let input = &form_state.input;
                let label = f.label;

                select_field(label.to_string(), input)
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
        let mut form: HashMap<InteractionFormFieldId, FormFieldState<InputState>> = HashMap::new();
        let mut date_fields: HashMap<InteractionFormFieldId, FormFieldState<DatePickerState>> =
            HashMap::new();

        let mut select_fields: HashMap<
            InteractionFormFieldId,
            FormFieldState<SelectState<Vec<String>>>,
        > = HashMap::new();

        for desc in DATE_FIELDS {
            let id = desc.id;
            let date_picker = cx.new(|cx| DatePickerState::new(window, cx));

            cx.observe(&date_picker, move |this, picker, cx| {
                let selected_date = picker.read(cx).date();
                if let Some(state) = this.date_fields.get_mut(&id) {
                    state.value = selected_date.to_string();
                }
                cx.notify();
            })
            .detach();

            date_fields.insert(
                id,
                FormFieldState {
                    value: String::new(),
                    input: date_picker,
                },
            );
        }
        for desc in TEXT_FIELDS {
            let input = cx.new(|cx| InputState::new(window, cx));
            let id = desc.id;

            cx.subscribe(&input, move |this, input, _event: &InputEvent, cx| {
                let text = this.text_fields.get_mut(&id);

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
        for desc in SELECT_FIELDS {
            let select_state = cx.new(|cx| {
                SelectState::new((desc.options)(), Some(IndexPath::default()), window, cx)
            });

            cx.observe(&select_state, move |this, picker, cx| {
                let selected = picker.read(cx).selected_value();
                if let Some(state) = this.select_fields.get_mut(&desc.id) {
                    state.value = selected.unwrap_or(&String::new()).to_string()
                }
                cx.notify();
            })
            .detach();

            select_fields.insert(
                desc.id,
                FormFieldState {
                    value: String::new(),
                    input: select_state,
                },
            );
        }
        Self {
            interaction_repository,
            text_fields: form,
            date_fields: date_fields,
            select_fields,
            customer,
        }
    }

    pub fn save_interaction(&mut self, cx: &mut Context<Self>) {
        let repository_handle = self.interaction_repository.clone();

        let note = self
            .text_fields
            .get(&InteractionFormFieldId::Note)
            .unwrap()
            .value
            .clone();

        let status = self
            .select_fields
            .get(&InteractionFormFieldId::Status)
            .unwrap()
            .value
            .clone();

        let date = self
            .date_fields
            .get(&InteractionFormFieldId::InteractionDate)
            .unwrap()
            .value
            .clone();

        println!("Target date: {date}");

        let interaction = Interaction {
            id: Draft,
            interaction_date: date.to_string(),
            note: if !note.is_empty() { Some(note) } else { None },
            status: InteractionStatus::from_str(&status),
        };

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
        for field in &mut self.text_fields {
            field.1.input.update(cx, |input, context| {
                input.clean(window, context);
            });
            field.1.value = String::new();

            cx.notify();
        }
    }
}
