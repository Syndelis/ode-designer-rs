use imgui::Ui;

use crate::App;
use rfd::FileDialog;

use std::process::{Command, Stdio};

use super::app::SimulationState;

impl App {
    fn draw_menu_load_csv(&mut self, ui: &Ui) {
        if ui.menu_item("Plot CSV file") {
            let file = FileDialog::new()
                .add_filter("csv", &["csv"])
                .set_directory(".")
                .pick_file();

            if let Some(file_path) = file {
                let fp = std::fs::File::open(file_path).unwrap();
                self.simulation_state = Some(SimulationState::from_csv(fp));
            }
        }
    }

    pub fn draw_menu(&mut self, ui: &Ui) {
        ui.menu_bar(|| {
            ui.menu("File", || {
                if ui.menu_item("New") {
                    self.clear_state();
                }

                if ui.menu_item("Load") {
                    self.clear_state();
                    self.load_state();
                }

                if ui.menu_item("Save") {
                    self.save_state();
                }

                self.draw_menu_load_csv(ui);
            });

            ui.menu("Export", || {
                if ui.menu_item("Generate Code") {
                    let py_code = self.generate_code();
                    self.save_to_file(py_code, "py");
                }

                if ui.menu_item("Plot to PDF") {
                    if let Some(file_path) =
                        FileDialog::new().add_filter("pdf", &["pdf"]).save_file()
                    {
                        let py_code = self.generate_code();
                        Command::new("python3")
                            .arg("-c")
                            .arg(py_code)
                            .arg("--output")
                            .arg(file_path)
                            .spawn()
                            .unwrap();
                    }
                }
            });

            if ui.menu_item("Run") {
                let py_code = self.generate_code();
                let python_out = Command::new("python3")
                    .arg("-c")
                    .arg(py_code)
                    .arg("--csv")
                    .stdout(Stdio::piped())
                    .spawn()
                    .unwrap();

                self.simulation_state = Some(SimulationState::from_csv(python_out.stdout.unwrap()));
            }
        });
    }
}
