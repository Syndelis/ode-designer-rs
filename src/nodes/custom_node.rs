use imgui::ImColor32;
use imnodes::{InputPinId, NodeId};

use crate::{
    core::App, exprtree::ExpressionNode, message::Message, pins::{InputPin, OutputPin, Pin}, utils::ModelFragment
};

use super::{LinkEvent, NodeImpl, PendingOperations};

#[derive(Debug)]
pub struct CustomNode {
    pub id: NodeId,
    pub name: String,
    pub inputs: Vec<InputPin>,
    pub outputs: Vec<OutputPin>,
    pub selected_function: usize,
}

impl NodeImpl for CustomNode {
    fn id(&self) -> imnodes::NodeId {
        self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn name_mut(&mut self) -> &mut String {
        &mut self.name
    }

    #[inline]
    fn color(&self) -> ImColor32 {
        ImColor32::from_rgb(209, 73, 209)
    }

    #[inline]
    fn selected_color(&self) -> ImColor32 {
        ImColor32::from_rgb(239, 71, 239)
    }

    fn send_data(&self) -> ExpressionNode<InputPinId> {
        unreachable!("This node doesn't feature an output pin")
    }

    fn notify(&mut self, link_event: LinkEvent) -> Option<Vec<Message>> {
        None
    }

    fn state_changed(&mut self) -> bool {
        todo!()
    }

    fn draw(&mut self, ui: &imgui::Ui, app: &App) -> bool {
        todo!()
    }

    fn inputs(&self) -> Option<&[InputPin]> {
        Some(&self.inputs)
    }

    fn inputs_mut(&mut self) -> Option<&mut [InputPin]> {
        Some(&mut self.inputs)
    }

    fn to_model_fragment(&self, app: &crate::core::App) -> Option<ModelFragment> {
        unimplemented!("Missing odeir repr")
    }
    fn new(node_id: NodeId, name: String) -> Self {
        Self {
            id: node_id,
            selected_function: 0,
            name,
            inputs: Default::default(),
            outputs: Default::default(),
        }
    }

    fn try_from_model_fragment(
        node_id: NodeId,
        frag: &ModelFragment,
    ) -> Option<(Self, Option<PendingOperations>)> {
        unimplemented!("Missing odeir repr")
    }
}
