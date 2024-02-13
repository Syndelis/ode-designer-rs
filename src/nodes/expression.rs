use std::str::FromStr;

use imgui::ImColor32;
use imnodes::{InputPinId, NodeId};
use strum::StaticVariantsArray;

use crate::{
    core::App,
    exprtree::{ExpressionNode, ExpressionTree, Operation, Sign},
    message::Message,
    pins::{InputPin, OutputPin, Pin},
    utils::ModelFragment,
};

use super::{ExprWrapper, LinkEvent, NodeImpl, PendingOperation, PendingOperations};

const MINIMUM_PIN_COUNT: usize = 2;

#[derive(Debug)]
pub struct Expression {
    pub id: NodeId,
    pub name: String,
    pub expr_wrapper: ExprWrapper<ExpressionTree<InputPinId>>,
    pub inputs: Vec<InputPin>,
    pub output: OutputPin,

    message_buffer: Vec<Message>,
}

impl Expression {
    fn new(node_id: NodeId, name: String, input_count: usize) -> Self {
        Self {
            id: node_id,
            name,
            expr_wrapper: Default::default(),
            message_buffer: vec![],
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

    fn notify(&mut self, link_event: LinkEvent) -> bool {
        match link_event {
            LinkEvent::Push {
                from_pin_id,
                payload,
            } => {
                if self.all_pins_linked(from_pin_id) {
                    let input_pin = InputPin::new_signed(self.id, Sign::Positive);
                    self.message_buffer
                        .push(Message::RegisterPin(self.id, input_pin.id));
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
                        self.message_buffer
                            .push(Message::UnregisterPin(removed_pin.id))
                    }
                }
                self.expr_wrapper.members.remove(&from_pin_id)
            }
        };

        self.expr_wrapper.resolution.reset();
        true
    }

    fn notify(&mut self, link_event: LinkEvent) -> Option<Vec<Message>> {
        self.notify(link_event)
            .then(|| self.send_data())
            .map(|data| [std::mem::take(&mut self.message_buffer), data].concat())
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

    fn draw(&mut self, ui: &imgui::Ui) -> bool {
        let mut selected = self.expr_wrapper.join_op as usize;
        let mut changed = false;

        // Needs to be assigned to a variable other than `_`. Otherwise, the
        // style isn't applied. That's probably the case because it needs to be
        // dropped *after* the combo below has been executed.
        let _smth = ui.push_item_width(50.);

        if ui.combo(
            "##Expression operation select",
            &mut selected,
            Operation::ALL_VARIANTS,
            |op| format!("{op}").into(),
        ) {
            self.expr_wrapper.join_op = Operation::from_repr(selected as u8)
                .expect("ImGui returned an out-of-range value in combobox");

            changed = true
        }

        match self.expr_wrapper.get_expr_repr() {
            Some(expr) => ui.text(expr),
            None => ui.text("Nothing yet!"),
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
        let mut composition = Vec::with_capacity(self.inputs.len());

        for input_pin in &self.inputs {
            let Some(linked_pin_id) = input_pin.linked_to else {
                continue;
            };

            let node_id = app
                .output_pins
                .get(&linked_pin_id)
                .expect("The node must exist, otherwise this should have been unlinked");

            let node = app
                .get_node(*node_id)
                .expect("The node must exist, otherwise this should have been unlinked");

            composition.push(odeir::models::Component {
                name: node.name().to_owned(),
                contribution: input_pin.sign.into(),
            });
        }

        Some(
            odeir::Argument::Composite {
                name: self.name().to_owned(),
                operation: Into::<char>::into(self.expr_wrapper.join_op).into(),
                composition,
            }
            .into(),
        )
    }

    fn new(node_id: NodeId, name: String) -> Self
    where
        Self: Sized,
    {
        Self::new(node_id, name, MINIMUM_PIN_COUNT)
    }

    fn try_from_model_fragment(
        node_id: NodeId,
        frag: &ModelFragment,
    ) -> Option<(Self, Option<PendingOperations>)> {
        let ModelFragment::Argument(odeir::Argument::Composite {
            name,
            operation,
            composition,
        }) = frag
        else {
            return None;
        };

        let mut expr_wrapper: ExprWrapper<ExpressionTree<InputPinId>> = Default::default();
        expr_wrapper
            .set_join_op(Operation::from_str(operation).expect("Should be a valid operation"));

        let node = Self::new(
            node_id,
            name.clone(),
            composition.len().max(MINIMUM_PIN_COUNT),
        ).with_expr_wapper(expr_wrapper);

        let pending_ops = PendingOperations {
            node_id,
            operations: composition
                .iter()
                .cloned()
                .zip(node.inputs.iter())
                .map(|(comp, input_pin)| {
                    Some(PendingOperation::LinkWith {
                        node_name: comp.name,
                        via_pin_id: *input_pin.id(),
                        sign: comp.contribution.try_into().ok()?,
                    })
                })
                .collect::<Option<Vec<PendingOperation>>>()?,
        };

        Some((node, Some(pending_ops)))
    }
}
