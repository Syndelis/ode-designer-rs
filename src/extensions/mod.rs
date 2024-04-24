use std::{
    borrow::Cow,
    fs::File,
    io::{BufReader, Read},
    path::PathBuf,
    rc::Rc,
    str::FromStr,
};

use rfd::FileDialog;

use crate::{
    core::App,
    nodes::{NodeTypeRepresentation, NodeVariant},
};

use self::{format::Format, loader::NodeFunction};

pub mod format;
mod loader;

pub struct Extension {
    pub filename: String,
    pub file_path: PathBuf,
    pub file_hash: u32,
    pub nodes: Vec<Rc<CustomNodeSpecification>>,
}

#[derive(Debug)]
pub struct CustomNodeSpecification {
    pub function: NodeFunction,
    pub format: Format,
}

impl CustomNodeSpecification {
    pub fn input_count(&self) -> usize {
        self.function.required_arguments.len()
    }
}

impl From<NodeFunction> for CustomNodeSpecification {
    fn from(mut function: NodeFunction) -> Self {
        let format = function
            .format
            .take()
            .and_then(|format| Format::from_str(&format).ok())
            .unwrap_or_else(|| Format::default_with_name(&function.name));

        Self { function, format }
    }
}

impl App {
    pub fn pick_extension_file(&mut self) -> color_eyre::Result<()> {
        let file_path = FileDialog::new()
            .add_filter("Python", &["py"])
            .pick_file()
            .ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::NotFound, "Could not open file")
            })?;

        self.load_extension_from_path(file_path)
    }

    pub fn load_extension_from_path(&mut self, origin: PathBuf) -> color_eyre::Result<()> {
        let mut file = File::open(&origin)?;
        let mut user_code = String::new();
        file.read_to_string(&mut user_code)?;

        let file_hash = crc32fast::hash(user_code.as_bytes());
        let filename = origin
            .file_name()
            .map(std::ffi::OsStr::to_string_lossy)
            .map(Into::into)
            .unwrap_or_else(|| String::from("Unknown"));

        if let Some((idx, ext)) = self
            .extensions
            .iter()
            .enumerate()
            .find(|(_idx, ext)| ext.filename == filename)
        {
            if ext.file_hash == file_hash {
                return Ok(());
            }

            self.extensions.remove(idx);
        }

        let node_specs: Vec<_> = loader::inspect_user_code(&user_code)?
            .into_iter()
            .map(CustomNodeSpecification::from)
            .map(Rc::from)
            .inspect(|node_spec| {
                self.node_types.push(
                    NodeTypeRepresentation::new(
                        format!("󰯂 {}", node_spec.function.name),
                        NodeVariant::Custom,
                        Some(Rc::clone(node_spec)),
                    )
                );
            })
            .collect();

        self.extensions.push(Extension {
            filename,
            file_hash,
            file_path: origin,
            nodes: node_specs,
        });

        Ok(())
    }
}
