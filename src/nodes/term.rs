use imgui::Ui;
use imnodes::{InputPinId, NodeId};

use crate::{
    core::widgets::input_num,
    core::App,
    exprtree::{ExpressionNode, Leaf, Sign},
    pins::{OutputPin, Pin},
    utils::ModelFragment,
};

use super::{NodeImpl, PendingOperations};

#[derive(Debug)]
pub struct Term {
    pub id: NodeId,
    pub leaf: Leaf,
    pub initial_value: f64,
    pub output: OutputPin,
}

impl NodeImpl for Term {
    fn id(&self) -> NodeId {
        self.id
    }

    fn name(&self) -> &str {
        &self.leaf.symbol
    }

    #[inline]
    fn is_assignable(&self) -> bool {
        true
    }

    fn name_mut(&mut self) -> &mut String {
        &mut self.leaf.symbol
    }

    fn send_data(&self) -> ExpressionNode<InputPinId> {
        ExpressionNode::Leaf(self.leaf.clone())
    }

    fn draw(&mut self, ui: &Ui) -> bool {
        ui.text("Initial Value: ");
        ui.same_line();
        input_num(ui, "##population initial value", &mut self.initial_value)
    }

    fn outputs(&self) -> Option<&[OutputPin]> {
        Some(std::array::from_ref(&self.output))
    }

    fn outputs_mut(&mut self) -> Option<&mut [OutputPin]> {
        Some(std::array::from_mut(&mut self.output))
    }

    fn to_model_fragment(&self, _app: &App) -> Option<ModelFragment> {
        Some(
            odeir::Argument::Value {
                name: self.name().to_owned(),
                value: self.initial_value,
            }
            .into(),
        )
    }

    fn new(node_id: NodeId, name: String) -> Self {
        Self {
            id: node_id,
            leaf: Leaf {
                symbol: name,
                unary_op: Sign::Positive,
            },
            initial_value: 0.00,
            output: Pin::new(node_id),
        }
    }

    fn try_from_model_fragment(
        node_id: NodeId,
        frag: &ModelFragment,
    ) -> Option<(Self, Option<PendingOperations>)> {
        let ModelFragment::Argument(odeir::Argument::Value { name, value }) = frag else {
            return None;
        };

        let node = Self {
            id: node_id,
            leaf: Leaf {
                symbol: name.clone(),
                unary_op: Sign::Positive,
            },
            initial_value: *value,
            output: Pin::new(node_id),
        };

        Some((node, None))
    }
}
