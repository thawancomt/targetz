use gpui_kit::{
    App, AppContext, Context, Entity, EventEmitter, IntoElement, ParentElement, Render, Styled,
    Window,
    base::IndexPath,
    component::{
        ActiveTheme,
        button::{Button, ButtonVariants},
        select::{
            Select,
            SelectEvent::{self, Confirm},
            SelectItem, SelectState,
        },
    },
    div,
    prelude::FluentBuilder,
};
use shared::customer::{Stakeholder, StakeholderRole};

pub struct StakeholderItemView {
    pub stakeholder: Stakeholder,
    pub select_state: Entity<SelectState<Vec<String>>>,
    pub options: Vec<String>,
}

impl StakeholderItemView {
    pub fn new(stakeholder: Stakeholder, window: &mut Window, cx: &mut App) -> Entity<Self> {
        let options: Vec<String> = vec![
            StakeholderRole::Owner.as_str().to_string(),
            StakeholderRole::Investor.as_str().to_string(),
            StakeholderRole::Partner.as_str().to_string(),
        ];

        let initial_index = options
            .iter()
            .position(|opt| opt == stakeholder.role.as_str())
            .map(|idx| IndexPath::new(idx));

        let select_state =
            cx.new(|cx| SelectState::new(options.clone(), initial_index, window, cx));

        cx.new(|cx| {
            cx.subscribe(&select_state, |this: &mut Self, _emitter, event, cx| {
                match event {
                    SelectEvent::Confirm(Some(role_str)) => {
                        this.stakeholder.role = StakeholderRole::from(role_str.clone());
                        cx.emit(StakeholderItemEvent::UpdateStakeholder(
                            this.stakeholder.clone(),
                        ));
                        cx.notify(); // Redesenha a view com o novo valor
                    }
                    SelectEvent::Confirm(None) => {
                        // Se o select foi limpo (caso cleanable seja true)
                    }
                }
            })
            .detach();

            Self {
                stakeholder,
                select_state,
                options,
            }
        })
    }

    pub fn update_stakeholder_role(&mut self, role: StakeholderRole) {
        self.stakeholder.role = role;
    }
}

#[derive(Debug, Clone)]
pub enum StakeholderItemEvent {
    AddedStakeholder(Stakeholder),
    RemoveStakeholder(Stakeholder),
    UpdateStakeholder(Stakeholder),
}

impl EventEmitter<StakeholderItemEvent> for StakeholderItemView {}

impl Render for StakeholderItemView {
    fn render(
        &mut self,
        window: &mut Window,
        cx: &mut gpui_kit::prelude::Context<Self>,
    ) -> impl IntoElement {
        let sh = self.stakeholder.clone();

        let select_role = self.select_state.read(cx).selected_value();
        let theme = cx.theme();

        div()
            .flex()
            .justify_between()
            .bg(theme.secondary)
            .p_2()
            .border_1()
            .border_color(theme.border)
            .child(
                div()
                    .child(format!(
                        "{} [{}]",
                        self.stakeholder.name,
                        if select_role.is_some() {
                            select_role.unwrap()
                        } else {
                            self.stakeholder.role.as_str()
                        }
                    ))
                    .child(
                        div()
                            .child(format!("Created at: {}", self.stakeholder.created_at))
                            .text_xs(),
                    )
                    .when(!self.stakeholder.email.is_empty(), |d| {
                        d.child(div().child(format!("email: {}", self.stakeholder.email)))
                            .text_xs()
                    }),
            )
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(Select::new(&self.select_state))
                    .child(
                        Button::new(format!("remove-{}", self.stakeholder.id))
                            .danger()
                            .label("Remove")
                            .on_click(cx.listener(move |_, _, _, cx| {
                                cx.emit(StakeholderItemEvent::RemoveStakeholder(sh.clone()))
                            })),
                    ),
            )
    }
}
