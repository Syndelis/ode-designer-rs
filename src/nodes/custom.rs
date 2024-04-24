use std::{collections::HashMap, rc::Rc};

use imgui::ImColor32;
use imnodes::{InputPinId, NodeId};
use odeir::models::CompositionStyle;

use crate::{
    core::App, exprtree::{ExpressionNode, ExpressionTree}, extensions::CustomNodeSpecification, locale::Locale, message::Message, pins::{InputPin, OutputPin, Pin}, utils::ModelFragment
};

use super::{
    composition_utils::{build_composition, build_from_composition},
    ExprWrapper, LinkEvent, NodeImpl,
};

#[derive(Debug)]
pub struct CustomFunctionNode {
    pub id: NodeId,
    pub name: String,
    pub inputs: Vec<InputPin>,
    pub output: OutputPin,
    pub expr_wrapper: ExprWrapper<ExpressionTree<InputPinId>>,
    spec: Rc<CustomNodeSpecification>,
}

impl CustomFunctionNode {
    pub fn from_spec(node_id: NodeId, name: String, spec: Rc<CustomNodeSpecification>) -> Self {
        let mut expr_wrapper = ExprWrapper::<ExpressionTree<InputPinId>>::default();
        #[allow(clippy::field_reassign_with_default)]
        {
            expr_wrapper.format = spec.format.clone();
        }

        Self {
            id: node_id,
            name,
            inputs: std::iter::repeat_with(|| InputPin::new(node_id))
                .take(spec.input_count())
                .collect(),
            output: Pin::new(node_id),
            expr_wrapper,
            spec,
        }
    }
}

impl NodeImpl for CustomFunctionNode {
    fn id(&self) -> NodeId {
        self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn name_mut(&mut self) -> &mut String {
        &mut self.name
    }

    fn color(&self) -> imgui::ImColor32 {
        ImColor32::from_rgb(120, 0, 120)
    }

    fn selected_color(&self) -> imgui::ImColor32 {
        ImColor32::from_rgb(112, 45, 151)
    }

    fn send_data(&self) -> ExpressionNode<InputPinId> {
        ExpressionNode::SubExpr(self.expr_wrapper.clone())
    }

    fn inputs(&self) -> Option<&[InputPin]> {
        Some(&self.inputs)
    }

    fn inputs_mut(&mut self) -> Option<&mut [InputPin]> {
        Some(&mut self.inputs)
    }

    fn outputs(&self) -> Option<&[OutputPin]> {
        Some(std::array::from_ref(&self.output))
    }

    fn outputs_mut(&mut self) -> Option<&mut [OutputPin]> {
        Some(std::array::from_mut(&mut self.output))
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

    fn notify(&mut self, link_event: LinkEvent) -> Option<Vec<Message>> {
        match link_event {
            LinkEvent::Push {
                from_pin_id,
                payload,
            } => {
                let pin = self
                    .inputs
                    .iter()
                    .find(|pin| pin.id() == &from_pin_id)
                    .expect("The pin must exist if we received data through it");

                self.expr_wrapper
                    .members
                    .insert(from_pin_id, pin.map_data(payload));
            }
            LinkEvent::Pop(from_pin_id) => {
                self.expr_wrapper.members.remove(&from_pin_id);
            }
        }

        self.expr_wrapper.resolution.reset();
        None
    }

    fn draw(&mut self, ui: &imgui::Ui, locale: &Locale) -> bool {
        match self.expr_wrapper.get_expr_repr() {
            Some(expr) => ui.text(expr),
            None => ui.text(locale.get("nothing-yet")),
        };
        false
    }

    fn try_from_model_fragment(
        node_id: imnodes::NodeId,
        frag: &ModelFragment,
        app: &App,
    ) -> Option<(Self, Option<super::PendingOperations>)>
    where
        Self: Sized,
    {
        let ModelFragment::Argument(odeir::Argument::Composite {
            operation,
            style: CompositionStyle::Prefixed,
            ..
        }) = &frag
        else {
            return None;
        };

        let Some(spec) = app
            .extensions
            .iter()
            .flat_map(|ext| ext.nodes.iter())
            .find(|node_spec| &node_spec.function.name == operation)
        else {
            return None;
        };

        build_from_composition(node_id, frag, |name, composition, expr_wrapper| {
            Self::from_spec(node_id, name.to_owned(), Rc::clone(spec))
        })
    }

    fn to_model_fragment(&self, app: &crate::core::App) -> Option<crate::utils::ModelFragment> {
        build_composition(
            &self.name,
            &self.inputs,
            self.spec.function.name.clone(),
            CompositionStyle::Prefixed,
            app,
        )
    }
}
