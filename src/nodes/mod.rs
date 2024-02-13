mod assigner;
pub mod errors;
pub mod expression;
pub mod term;
pub mod custom_node;

use std::ops::{Deref, DerefMut};

pub use assigner::Assigner;
pub use expression::Expression;
use strum::{EnumDeref, EnumDiscriminants, EnumVariantNames, FromRepr};
pub use term::Term;

use imgui::{ImColor32, Ui};
use imnodes::{InputPinId, NodeId, NodeScope, OutputPinId};

use crate::{
    core::{app::AppState, GeneratesId},
    exprtree::{ExpressionNode, ExpressionTree, Sign},
    message::{Message, SendData},
    pins::{InputPin, OutputPin, Pin},
    utils::ModelFragment,
};

use derive_more::From;

use self::errors::NotANode;

#[derive(Debug, Clone, From)]
pub enum LinkPayload {
    Number(f64),
    Text(String),
}

pub enum LinkEvent {
    Push {
        from_pin_id: InputPinId,
        payload: ExpressionNode<InputPinId>,
    },
    Pop(InputPinId),
}

#[derive(Debug, EnumDeref, EnumDiscriminants, EnumVariantNames, From)]
#[strum_deref_target(dyn NodeImpl)]
#[strum_discriminants(name(NodeVariant))]
#[strum_discriminants(derive(FromRepr))]
pub enum Node {
    Term(Term),
    Expression(Expression),
    Assigner(Assigner),
}

impl Node {
    pub fn build_from_ui(name: String, variant: NodeVariant) -> Self {
        let node_id = NodeId::generate();

        match variant {
            NodeVariant::Term => Term::new(node_id, name).into(),
            NodeVariant::Expression => Expression::new(node_id, name).into(),
            NodeVariant::Assigner => Assigner::new(node_id, name).into(),
        }
    }

    pub fn build_from_fragment(
        frag: ModelFragment,
    ) -> Result<(Self, Option<PendingOperations>), NotANode> {
        let node_id = NodeId::generate();

        Term::try_from_model_fragment(node_id, &frag)
            .map(|(node_impl, ops)| (node_impl.into(), ops))
            .or_else(|| {
                Expression::try_from_model_fragment(node_id, &frag)
                    .map(|(node_impl, ops)| (node_impl.into(), ops))
                    .or_else(|| {
                        Assigner::try_from_model_fragment(node_id, &frag)
                            .map(|(node_impl, ops)| (node_impl.into(), ops))
                    })
            })
            .ok_or(NotANode(frag))
    }

    /// Broadcasts messages from a node. Essentially, relays all messages sent by [`NodeImpl::send_data`] to the output pins.
    pub fn broadcast_data(&self) -> Vec<Message> {
        let Some(outputs) = self.outputs() else {
            log::warn!("Tried broadcasting data to node without any output pins");
            return vec![]
        };
        let data = self.send_data();
        outputs
            .iter()
            .flat_map(|output| {
                output.linked_to.iter().copied().map(|to_input| SendData {
                    data: data.clone(),
                    from_output: output.id,
                    to_input,
                })
            })
            .map(Message::SendData)
            .collect()
    }


    /// Displays the node, but also handles the surrounding functionality like: implementing a
    /// "close" button, relays messages and app state, and node renaming.
    pub fn process_node(
        &mut self,
        ui: &Ui,
        ui_node: &mut NodeScope,
    ) -> (Option<Vec<Message>>, Option<AppState>) {
        let mut name_changed = false;
        let mut remove_node = false;

        ui_node.add_titlebar(|| {
            // Close button
            {
                let _col = ui.push_style_color(imgui::StyleColor::Button, crate::style::DIM_RED);
                let _round = ui.push_style_var(imgui::StyleVar::FrameRounding(50.));
                remove_node = ui.button(" × ");
            }

            ui.same_line();

            if self.id().is_selected() {
                let _width = ui.push_item_width((self.name().len() + 1) as f32 * 7.5);
                name_changed = ui
                    .input_text(
                        &format!("##node_{}", Into::<i32>::into(self.id())),
                        self.name_mut(),
                    )
                    .no_horizontal_scroll(true)
                    .build();
            } else {
                ui.text(self.name());
            }
        });

        if remove_node {
            return (Some(vec![Message::RemoveNode(self.id())]), None);
        }

        let mut input_changed = false;

        for input in self.inputs_mut().unwrap_or_default() {
            ui_node.add_input(*input.id(), input.get_shape(), || {
                if let Some(label) = input.get_label() {
                    ui.text(label);
                }

                input_changed |= input.draw(ui);
            })
        }

        for output in self.outputs_mut().unwrap_or_default() {
            let shape = output.get_shape();
            let id = *output.id();
            ui_node.add_output(id, shape, || {});
        }

        let inner_content_changed = self.draw(ui, app);

        let mut messages = ((inner_content_changed || input_changed) && self.state_changed()
            || name_changed)
            .then(|| self.broadcast_data());

        if name_changed {
            let node_rename_msg = Message::RenameNode(self.id(), self.name().to_owned());

            if let Some(ref mut msg) = messages {
                msg.push(node_rename_msg);
            } else {
                messages = Some(vec![node_rename_msg]);
            }
        }

        let app_state_change = inner_content_changed
            .then(|| self.trigger_app_state_change())
            .flatten();

        (messages, app_state_change)
    }

}

pub trait NodeImpl {

    fn new(node_id: NodeId, name: String) -> Self
    where
        Self: Sized;

    fn try_from_model_fragment(
        node_id: NodeId,
        frag: &ModelFragment,
    ) -> Option<(Self, Option<PendingOperations>)>
    where
        Self: Sized;

    fn id(&self) -> NodeId;

    fn name(&self) -> &str;

    fn name_mut(&mut self) -> &mut String;

    fn color(&self) -> ImColor32;

    #[inline]
    fn hovered_color(&self) -> ImColor32 {
        self.selected_color()
    }

    fn selected_color(&self) -> ImColor32;

    fn send_data(&self) -> ExpressionNode<InputPinId>;

    fn trigger_app_state_change(&self) -> Option<AppState> {
        None
    }

    #[inline]
    fn is_assignable(&self) -> bool {
        false
    }

    fn draw(&mut self, ui: &Ui) -> bool;

    fn inputs(&self) -> Option<&[InputPin]> {
        None
    }
    fn outputs(&self) -> Option<&[OutputPin]> {
        None
    }

    fn inputs_mut(&mut self) -> Option<&mut [InputPin]> {
        None
    }
    fn outputs_mut(&mut self) -> Option<&mut [OutputPin]> {
        None
    }

    fn notify(&mut self, link_event: LinkEvent) -> Option<Vec<Message>> {
        None
    }

    fn state_changed(&mut self) -> bool {
        true
    }

    fn get_input(&self, input_pin_id: &InputPinId) -> Option<&InputPin> {
        self.inputs()
            .unwrap_or_default()
            .iter()
            .find(|pin| pin.id() == input_pin_id)
    }

    fn get_output(&self, output_pin_id: &OutputPinId) -> Option<&OutputPin> {
        self.outputs()
            .unwrap_or_default()
            .iter()
            .find(|pin| pin.id() == output_pin_id)
    }

    fn get_input_mut(&mut self, input_pin_id: &InputPinId) -> Option<&mut InputPin> {
        self.inputs_mut()
            .unwrap_or_default()
            .iter_mut()
            .find(|pin| pin.id() == input_pin_id)
    }

    fn get_output_mut(&mut self, output_pin_id: &OutputPinId) -> Option<&mut OutputPin> {
        self.outputs_mut()
            .unwrap_or_default()
            .iter_mut()
            .find(|pin| pin.id() == output_pin_id)
    }

    fn should_link(&self, input_pin_id: &InputPinId) -> bool {
        self.get_input(input_pin_id).is_some()
    }

    fn to_model_fragment(&self) -> Option<ModelFragment>;
}

#[derive(Debug, Default)]
pub struct ExprWrapper<T> {
    pub resolution: ResolutionStatus<Option<String>>,
    pub expr: T,
}

#[derive(Debug, Default)]
pub enum ResolutionStatus<T> {
    Resolved(T),
    #[default]
    Unresolved,
}

impl<T> ResolutionStatus<T> {
    pub fn reset(&mut self) {
        *self = ResolutionStatus::Unresolved;
    }
}

pub trait Resolvable {
    fn resolve(&self) -> String;
}

impl<T: Resolvable> Resolvable for Option<T> {
    fn resolve(&self) -> String {
        match self {
            Some(expr) => expr.resolve(),
            None => "Nothing yet!".to_string(),
        }
    }
}

impl<T: std::hash::Hash> Resolvable for ExpressionNode<T> {
    fn resolve(&self) -> String {
        self.resolve_into_equation_part()
    }
}

impl<T: std::hash::Hash> Resolvable for ExpressionTree<T> {
    fn resolve(&self) -> String {
        self.resolve_into_equation()
    }
}

impl<T: Resolvable> ExprWrapper<T> {
    pub fn get_expr_repr(&mut self) -> Option<&str> {
        if let ResolutionStatus::Unresolved = self.resolution {
            let expr = self.expr.resolve();
            self.resolution = ResolutionStatus::Resolved((!expr.is_empty()).then_some(expr));
        }

        if let ResolutionStatus::Resolved(ref expr) = self.resolution {
            expr.as_deref()
        } else {
            unreachable!("If it was not resolved, the previous `if` made sure to resolve it")
        }
    }

    pub fn set_expr(&mut self, expr: T) {
        self.resolution.reset();
        self.expr = expr;
    }
}

impl<T: Resolvable> Deref for ExprWrapper<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.expr
    }
}

impl<T: Resolvable> DerefMut for ExprWrapper<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.expr
    }
}

pub struct PendingOperations {
    pub node_id: NodeId,
    pub operations: Vec<PendingOperation>,
}

pub enum PendingOperation {
    LinkWith {
        node_name: String,
        via_pin_id: InputPinId,
        sign: Sign,
    },
    SetAssignerOperatesOn {
        target_node_name: String,
    },
}
