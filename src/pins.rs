use imgui::{StyleColor, Ui};
use imnodes::{InputPinId, NodeId, OutputPinId};

use crate::{
    core::{widgets::rgb, GeneratesId},
    exprtree::{ExpressionNode, Sign},
};

#[derive(Debug, Clone)]
pub struct InputPin {
    pub id: InputPinId,
    node_id: NodeId,
    pub sign: Sign,
    pub linked_to: Option<OutputPinId>,
    label: Option<String>,
    draw_sign: bool,
}

#[derive(Debug, Clone)]
pub struct OutputPin {
    pub id: OutputPinId,
    node_id: NodeId,
    pub sign: Sign,
    pub linked_to: Vec<InputPinId>,
    label: Option<String>,
}

impl InputPin {
    pub fn map_data(&self, mut data: ExpressionNode<InputPinId>) -> ExpressionNode<InputPinId> {
        data.set_unary(self.sign);
        data
    }
}

pub trait Pin: Sized {
    type SelfIdType: GeneratesId;
    type LinkedToIdType: PartialEq + Copy;

    fn id(&self) -> &Self::SelfIdType;

    fn new_signed(node_id: NodeId, sign: Sign) -> Self;

    fn new(node_id: NodeId) -> Self {
        Self::new_signed(node_id, Sign::Positive)
    }

    fn draw(&mut self, _ui: &Ui) -> bool {
        false
    }

    fn is_linked_to(&self, pin_id: &Self::LinkedToIdType) -> bool;

    fn has_links(&self) -> bool;

    fn link_to(&mut self, linkeable: impl Linkable<Self::LinkedToIdType>);

    fn unlink(&mut self, pin_id: &Self::LinkedToIdType) -> bool;

    fn get_shape(&self) -> imnodes::PinShape {
        if self.has_links() {
            imnodes::PinShape::CircleFilled
        } else {
            imnodes::PinShape::Circle
        }
    }

    fn get_label(&self) -> Option<&str>;

    fn set_label(&mut self, label: impl ToString) -> &mut Self;
}

pub trait Linkable<IdType> {
    fn pin_id(&self) -> IdType;

    fn sign(&self) -> Sign {
        Sign::default()
    }
}

impl<IdType: Copy> Linkable<IdType> for IdType {
    fn pin_id(&self) -> IdType {
        *self
    }
}

impl<IdType: Copy> Linkable<IdType> for (IdType, Sign) {
    fn pin_id(&self) -> IdType {
        self.0
    }

    fn sign(&self) -> Sign {
        self.1
    }
}

pub fn sign_pin_button(ui: &Ui, id: i32, sign: Sign) -> bool {
    let (txt, col) = match sign {
        Sign::Positive => ("+", rgb(40, 200, 40)),
        Sign::Negative => ("-", rgb(200, 50, 50)),
    };
    let hover_col = col.map(|x| x * 1.25);
    let pressed_col = col.map(|x| x.powf(2.2));
    let _c = ui.push_style_color(StyleColor::Button, col);
    let _fc = ui.push_style_color(StyleColor::ButtonHovered, hover_col);
    let _hc = ui.push_style_color(StyleColor::ButtonActive, pressed_col);
    ui.button(format!("  {}  ##{}", txt, id))
}

impl InputPin {
    pub fn remove_sign(&mut self) -> &mut Self {
        self.draw_sign = false;
        self
    }
}

impl Pin for InputPin {
    type SelfIdType = InputPinId;
    type LinkedToIdType = OutputPinId;

    fn id(&self) -> &Self::SelfIdType {
        &self.id
    }

    fn new_signed(node_id: NodeId, _sign: Sign) -> Self {
        Self {
            id: Self::SelfIdType::generate(),
            node_id,
            sign: Sign::Positive,
            linked_to: None,
            draw_sign: true,
            label: None,
        }
    }

    fn draw(&mut self, ui: &Ui) -> bool {
        if !self.draw_sign {
            return false;
        }

        let mut changed = false;
        let id = *self.id();

        if sign_pin_button(ui, id.into(), self.sign) {
            self.sign.toggle();
            changed = true;
        }
        changed
    }

    fn is_linked_to(&self, pin_id: &Self::LinkedToIdType) -> bool {
        self.linked_to.iter().any(|id| id == pin_id)
    }
    fn has_links(&self) -> bool {
        self.linked_to.is_some()
    }

    fn link_to(&mut self, linkeable: impl Linkable<Self::LinkedToIdType>) {
        self.linked_to = Some(linkeable.pin_id());
        self.sign = linkeable.sign();
    }

    fn unlink(&mut self, pin_id: &Self::LinkedToIdType) -> bool {
        match self.linked_to {
            Some(ref linked_to_pin_id) => {
                if pin_id == linked_to_pin_id {
                    self.linked_to = None;
                    true
                } else {
                    false
                }
            }
            None => false,
        }
    }

    fn get_label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    fn set_label(&mut self, label: impl ToString) -> &mut Self {
        self.label = Some(label.to_string());
        self
    }
}

impl Pin for OutputPin {
    type SelfIdType = OutputPinId;
    type LinkedToIdType = InputPinId;

    fn id(&self) -> &Self::SelfIdType {
        &self.id
    }

    fn new_signed(node_id: NodeId, _sign: Sign) -> Self {
        Self {
            id: Self::SelfIdType::generate(),
            node_id,
            sign: Sign::Positive,
            linked_to: Vec::new(),
            label: None,
        }
    }

    fn is_linked_to(&self, pin_id: &Self::LinkedToIdType) -> bool {
        self.linked_to.iter().any(|id| id == pin_id)
    }
    fn has_links(&self) -> bool {
        !self.linked_to.is_empty()
    }

    fn link_to(&mut self, linkeable: impl Linkable<Self::LinkedToIdType>) {
        self.linked_to.push(linkeable.pin_id());
    }

    fn unlink(&mut self, pin_id: &Self::LinkedToIdType) -> bool {
        let o: Option<_> = {
            try {
                let i = self.linked_to.iter().position(|id| id == pin_id)?;
                self.linked_to.swap_remove(i);
            }
        };
        o.is_some()
    }

    fn get_label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    fn set_label(&mut self, label: impl ToString) -> &mut Self {
        self.label = Some(label.to_string());
        self
    }
}
