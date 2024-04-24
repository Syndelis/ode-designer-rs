use std::{borrow::Cow, default, io::Empty};

use imgui::Ui;

use crate::{
    locale::Locale, message::Message, nodes::{Expression, Node, NodeImpl, NodeTypeRepresentation, NodeVariant}
};

use super::App;

use odeir::models::ode;

#[derive(Debug, Default, Clone)]
pub struct SideBarState {
    node_name: String,
    sim_times: Times,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct Times {
    pub start: f64,
    pub delta: f64,
    pub end: f64,
}

impl SideBarState {
    pub fn draw(&mut self, ui: &Ui, node_types: &[NodeTypeRepresentation], locale: &Locale) -> Option<Node> {
        let table_group = ui.begin_group();
        let mut selected_node_type = None;

        const WIDTH: f32 = 13.0 * 10.0;

        ui.set_next_item_width(WIDTH);

        ui.input_text("##Node name", &mut self.node_name)
            .hint(locale.get("node-name-hint"))
            .build();

        // let _tk = ui.push_style_var(imgui::StyleVar::ButtonTextAlign([0.0; 2]));
        for node_type in node_types {
            if ui.button_with_size(&node_type.name, [WIDTH, 0.0]) {
                selected_node_type = Some(node_type)
            }
        }

        const MAGIC_BUTTON_HEIGHT: f32 = 31.0;
        const MAGIC_SPACING: f32 = 300.0;

        let [_, height] = ui.window_size();
        ui.dummy([0.0, height - MAGIC_SPACING - node_types.len() as f32 * MAGIC_BUTTON_HEIGHT]);

        {
            let _width = ui.push_item_width(WIDTH);
            ui.text(locale.get("start-time"));
            ui.input_scalar("##StartTime", &mut self.sim_times.start)
                .build();
            ui.text(locale.get("delta-time"));
            ui.input_scalar("##DeltaTime", &mut self.sim_times.delta)
                .build();
            ui.text(locale.get("end-time"));
            ui.input_scalar("##EndTime", &mut self.sim_times.end)
                .build();
        }

        table_group.end();

        ui.same_line();
        selected_node_type.map(|nt| {
            let name = std::mem::take(&mut self.node_name);
            Node::build_from_ui(name, nt)
        })
    }

    pub fn get_metadata(&self) -> ode::Metadata {
        self.sim_times.into()
    }

    pub fn set_metadata(&mut self, metadata: ode::Metadata) {
        self.sim_times = metadata.into();
    }

    pub fn time_flags(&self) -> [String; 6] {
        [
            "--st".to_owned(),
            self.sim_times.start.to_string(),
            "--tf".to_owned(),
            self.sim_times.end.to_string(),
            "--dt".to_owned(),
            self.sim_times.delta.to_string(),
        ]
    }

    pub fn clear_state(&mut self) {
        self.sim_times = Times::default();
    }
}

impl From<Times> for ode::Metadata {
    fn from(value: Times) -> Self {
        let Times { start, delta, end } = value;
        Self {
            start_time: start,
            delta_time: delta,
            end_time: end,
        }
    }
}

impl From<ode::Metadata> for Times {
    fn from(value: ode::Metadata) -> Self {
        let ode::Metadata {
            start_time,
            delta_time,
            end_time,
        } = value;

        Self {
            start: start_time,
            delta: delta_time,
            end: end_time,
        }
    }
}
