use imgui::ImColor32;
use imnodes::{InputPinId, NodeId};
use odeir::models::CompositionStyle;
use strum::VariantArray;

use crate::{
    core::App, exprtree::{ExpressionNode, ExpressionTree, Operation, Sign}, locale::Locale, message::Message, pins::{InputPin, OutputPin, Pin}, utils::ModelFragment
};

use super::{
    composition_utils::{build_composition, build_from_composition},
    ExprWrapper, LinkEvent, NodeImpl, PendingOperation, PendingOperations, SimpleNodeBuilder,
};

const MINIMUM_PIN_COUNT: usize = 2;

#[derive(Debug)]
pub struct Expression {
    pub id: NodeId,
    pub name: String,
    pub expr_wrapper: ExprWrapper<ExpressionTree<InputPinId>>,
    pub inputs: Vec<InputPin>,
    pub output: OutputPin,
}

impl Expression {
    fn new(node_id: NodeId, name: String, input_count: usize) -> Self {
        Self {
            id: node_id,
            name,
            expr_wrapper: Default::default(),
            inputs: std::iter::repeat_with(|| InputPin::new(node_id))
                .take(input_count)
                .collect(),
            output: Pin::new(node_id),
        }
    }

    fn with_expr_wapper(mut self, expr_wrapper: ExprWrapper<ExpressionTree<InputPinId>>) -> Self {
        self.expr_wrapper = expr_wrapper;
        self
    }

    pub fn all_pins_linked(&self, notifying_pin_id: InputPinId) -> bool {
        self.inputs
            .iter()
            .all(|pin| (pin.id == notifying_pin_id) || pin.has_links())
    }
}

impl SimpleNodeBuilder for Expression {
    fn new(node_id: NodeId, name: String) -> Self
    where
        Self: Sized,
    {
        Self::new(node_id, name, MINIMUM_PIN_COUNT)
    }
}

impl NodeImpl for Expression {
    fn id(&self) -> NodeId {
        self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn name_mut(&mut self) -> &mut String {
        &mut self.name
    }

    fn send_data(&self) -> ExpressionNode<InputPinId> {
        ExpressionNode::SubExpr(self.expr_wrapper.clone())
    }

    fn color(&self) -> ImColor32 {
        ImColor32::from_rgb(0, 160, 110)
    }

    fn selected_color(&self) -> ImColor32 {
        ImColor32::from_rgb(6, 214, 140)
    }

    fn notify(&mut self, link_event: LinkEvent) -> Option<Vec<Message>> {
        let mut messages = vec![];
        match link_event {
            LinkEvent::Push {
                from_pin_id,
                payload,
            } => {
                if self.all_pins_linked(from_pin_id) {
                    let input_pin = InputPin::new_signed(self.id, Sign::Positive);
                    messages.push(Message::RegisterPin(self.id, input_pin.id));
                    self.inputs.push(input_pin);
                }

                let pin = self
                    .inputs
                    .iter()
                    .find(|pin| pin.id() == &from_pin_id)
                    .expect("The pin must exist if we received data through it");

                self.expr_wrapper
                    .members
                    .insert(from_pin_id, pin.map_data(payload))
            }
            LinkEvent::Pop(from_pin_id) => {
                if self.inputs.len() > MINIMUM_PIN_COUNT {
                    let idx = self.inputs.iter().position(|pin| pin.id == from_pin_id);
                    if let Some(idx) = idx {
                        let removed_pin = self.inputs.remove(idx);
                        messages.push(Message::UnregisterPin(removed_pin.id));
                    }
                }
                self.expr_wrapper.members.remove(&from_pin_id)
            }
        };

        self.expr_wrapper.resolution.reset();
        Some(messages)
    }

    fn state_changed(&mut self) -> bool {
        for input in &self.inputs {
            if let Some(input_tree) = self.expr_wrapper.members.get_mut(input.id()) {
                input_tree.set_unary(input.sign);
            };
        }

        self.expr_wrapper.resolution.reset();
        true
    }

    fn draw(&mut self, ui: &imgui::Ui, locale: &Locale) -> bool {
        let mut selected = self.expr_wrapper.join_op() as usize;
        let mut changed = false;

        // Needs to be assigned to a variable other than `_`. Otherwise, the
        // style isn't applied. That's probably the case because it needs to be
        // dropped *after* the combo below has been executed.
        let _smth = ui.push_item_width(50.);

        if ui.combo(
            "##Expression operation select",
            &mut selected,
            Operation::VARIANTS,
            |op| format!("{op}").into(),
        ) {
            self.expr_wrapper.set_join_op(
                Operation::from_repr(selected as u8)
                    .expect("ImGui returned an out-of-range value in combobox"),
            );

            changed = true
        }

        match self.expr_wrapper.get_expr_repr() {
            Some(expr) => ui.text(expr),
            None => ui.text(locale.get("nothing-yet")),
        };

        changed
    }

    fn inputs(&self) -> Option<&[InputPin]> {
        Some(&self.inputs)
    }

    fn outputs(&self) -> Option<&[OutputPin]> {
        Some(std::array::from_ref(&self.output))
    }

    fn inputs_mut(&mut self) -> Option<&mut [InputPin]> {
        Some(&mut self.inputs)
    }

    fn outputs_mut(&mut self) -> Option<&mut [OutputPin]> {
        Some(std::array::from_mut(&mut self.output))
    }

    fn to_model_fragment(&self, app: &App) -> Option<ModelFragment> {
        build_composition(
            &self.name,
            &self.inputs,
            Into::<char>::into(self.expr_wrapper.join_op()).into(),
            CompositionStyle::Infixed,
            app,
        )
    }

    fn try_from_model_fragment(
        node_id: NodeId,
        frag: &ModelFragment,
        app: &App,
    ) -> Option<(Self, Option<PendingOperations>)> {
        if !matches!(
            frag,
            ModelFragment::Argument(odeir::Argument::Composite {
                style: CompositionStyle::Infixed,
                ..
            })
        ) {
            return None;
        };

        build_from_composition(node_id, frag, |name, composition, expr_wrapper| {
            Self::new(
                node_id,
                name.to_owned(),
                composition.len().max(MINIMUM_PIN_COUNT),
            )
            .with_expr_wapper(expr_wrapper)
        })
    }
}
