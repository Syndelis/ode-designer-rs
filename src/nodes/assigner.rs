use std::str::FromStr;

use imgui::ImColor32;
use imnodes::{InputPinId, NodeId};

use crate::{
    core::{app::AppState, App}, exprtree::{ExpressionNode, Sign}, locale::Locale, message::Message, pins::{InputPin, Pin}, utils::ModelFragment
};

use super::{
    ExprWrapper, LinkEvent, NodeImpl, PendingOperation, PendingOperations, SimpleNodeBuilder,
};

#[derive(Debug)]
pub struct Assigner {
    pub id: NodeId,
    pub name: String,
    pub input: InputPin,
    pub expr_node: ExprWrapper<Option<ExpressionNode<InputPinId>>>,
    pub operates_on: Option<(NodeId, String)>,
}

impl SimpleNodeBuilder for Assigner {
    fn new(node_id: NodeId, name: String) -> Self {
        Self {
            id: node_id,
            name,
            input: InputPin::new_signed(node_id, Sign::Positive),
            expr_node: Default::default(),
            operates_on: None,
        }
    }
}

impl NodeImpl for Assigner {
    fn id(&self) -> imnodes::NodeId {
        self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn name_mut(&mut self) -> &mut String {
        &mut self.name
    }

    fn color(&self) -> ImColor32 {
        ImColor32::from_rgb(209, 73, 91)
    }

    fn selected_color(&self) -> ImColor32 {
        ImColor32::from_rgb(239, 71, 111)
    }

    fn send_data(&self) -> ExpressionNode<InputPinId> {
        unreachable!("This node doesn't feature an output pin")
    }

    fn notify(&mut self, link_event: LinkEvent) -> Option<Vec<Message>> {
        match link_event {
            LinkEvent::Push { payload, .. } => {
                let payload = self.input.map_data(payload);
                self.expr_node.set_expr(Some(payload))
            }
            LinkEvent::Pop(_) => self.expr_node.set_expr(None),
        };

        None
    }

    fn state_changed(&mut self) -> bool {
        if let Some(ref mut expr_node) = *self.expr_node {
            expr_node.set_unary(self.input.sign);
            self.expr_node.resolution.reset();
        }

        false
    }

    fn draw(&mut self, ui: &imgui::Ui, locale: &Locale) -> bool {
        match self.expr_node.get_expr_repr() {
            Some(expr) => ui.text(expr),
            None => ui.text(locale.get("nothing-yet")),
        }

        ui.text(locale.get("assigner-variable"));
        ui.same_line();

        match &self.operates_on {
            Some((_, node_name)) => {
                ui.text(node_name);
                ui.button(locale.get("assigner-change"))
            }
            None => ui.button(locale.get("assigner-choose")),
        }
    }

    fn trigger_app_state_change(&self) -> Option<AppState> {
        Some(AppState::AttributingAssignerOperatesOn {
            attribute_to: self.id,
            search_query: "".into(),
        })
    }

    fn inputs(&self) -> Option<&[InputPin]> {
        Some(std::array::from_ref(&self.input))
    }

    fn inputs_mut(&mut self) -> Option<&mut [InputPin]> {
        Some(std::array::from_mut(&mut self.input))
    }

    fn to_model_fragment(&self, app: &crate::core::App) -> Option<ModelFragment> {
        let Some(linked_pin_id) = self.input.linked_to else {
            return None;
        };

        let node_id = app
            .output_pins
            .get(&linked_pin_id)
            .expect("The node must exist, otherwise this should have been unlinked");

        let node = app
            .get_node(*node_id)
            .expect("The node must exist, otherwise this should have been unlinked");

        let argument = node.name().to_owned();
        let contribution = self.input.sign.into();

        Some(
            odeir::Equation {
                name: self.name().to_owned(),
                operates_on: self.operates_on.clone().map(|(_, name)| name),
                argument,
                contribution,
            }
            .into(),
        )
    }

    fn try_from_model_fragment(
        node_id: NodeId,
        frag: &ModelFragment,
        app: &App,
    ) -> Option<(Self, Option<PendingOperations>)> {
        let ModelFragment::Equation(eq) = frag else {
            return None;
        };

        let node = Self {
            id: node_id,
            name: eq.name.clone(),
            input: InputPin::new_signed(
                node_id,
                Sign::try_from(eq.contribution).expect("Should be a valid sign"),
            ),
            expr_node: Default::default(),
            operates_on: None,
        };

        let mut pending_ops = PendingOperations {
            node_id,
            operations: vec![PendingOperation::LinkWith {
                node_name: eq.argument.clone(),
                via_pin_id: *node.input.id(),
                sign: node.input.sign,
            }],
        };

        if let Some(target_node_name) = &eq.operates_on {
            pending_ops
                .operations
                .push(PendingOperation::SetAssignerOperatesOn {
                    target_node_name: target_node_name.to_owned(),
                })
        }

        Some((node, Some(pending_ops)))
    }
}
