use imgui::Ui;

use crate::App;
use rfd::FileDialog;

use std::{
    fs::read_to_string,
    process::{Command, Stdio},
};

use super::{
    app::{AppState, SimulationState},
    python::{execute_python_code, PythonError},
};

impl<'n> App<'n> {
    fn draw_menu_load_csv(&mut self, ui: &Ui) {
        if ui.menu_item("Plot CSV file") {
            let file = FileDialog::new()
                .add_filter("csv", &["csv"])
                .set_directory(".")
                .pick_file();

            if let Some(file_path) = file {
                if let Ok(file_content) = read_to_string(&file_path) {
                    self.simulation_state = Some(SimulationState::from_csv(file_content));
                } else {
                    eprintln!("Error: Failed to read file content.");
                }
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
                    if let Err(err) = self.load_state() {
                        eprintln!("Couldn't load model from file: {err}");
                    }
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

                        let mut command = Command::new("python3");
                        command
                            .arg("-c")
                            .arg(&py_code)
                            .arg("--output")
                            .arg(file_path)
                            .args(self.sidebar_state.time_flags());

                        match execute_python_code(&mut command) {
                            Ok(_) => {}
                            Err(err) => eprintln!("{err}"),
                        }
                    }
                }
            });

            if ui.menu_item("Run") {
                let py_code = self.generate_code();

                let mut command = Command::new("python3");
                command
                    .arg("-c")
                    .arg(&py_code)
                    .arg("--csv")
                    .args(self.sidebar_state.time_flags());

                match execute_python_code(&mut command) {
                    Ok(output) => {
                        self.simulation_state = Some(SimulationState::from_csv(output));
                    }
                    Err(err) => eprintln!("{err}"),
                }
            }

            if ui.menu_item("Manage Extensions") {
                self.state = if let Some(AppState::ManagingExtensions) = self.state {
                    None
                } else {
                    Some(AppState::ManagingExtensions)
                }
            }

            ui.menu("Help", || {
                if ui.menu_item("About") {
                    self.state = if let Some(AppState::MenuHelp) = self.state {
                        None
                    } else {
                        Some(AppState::MenuHelp)
                    }
                }
            });
        });
    }
}
