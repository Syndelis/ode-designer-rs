use imgui::Ui;

use crate::App;
use rfd::FileDialog;

use std::process::{Command, Stdio};

use super::app::{AppState, SimulationState};

impl<'n> App<'n> {
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
                if ui.menu_item_config("New").shortcut("Ctrl + N").build() {
                    self.clear_state();
                }

                if ui.menu_item_config("Load").shortcut("Ctrl + O").build() {
                    self.clear_state();
                    self.load_state().unwrap();
                }

                if ui.menu_item_config("Save").shortcut("Ctrl + S").build() {
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
                println!("{}", py_code);
                let python_out = Command::new("python3")
                    .arg("-c")
                    .arg(py_code)
                    .arg("--csv")
                    .stdout(Stdio::piped())
                    .spawn()
                    .unwrap();

                self.simulation_state = Some(SimulationState::from_csv(python_out.stdout.unwrap()));
            }

            if ui.menu_item("Manage Extensions") {
                self.state = if let Some(AppState::ManagingExtensions) = self.state {
                    None
                } else {
                    Some(AppState::ManagingExtensions)
                }
            }
        });
    }
}
