#![feature(map_many_mut)]
#![feature(try_blocks)]
#![feature(lazy_cell)]
#![feature(int_roundings)]
#![feature(let_chains)]
#![feature(iter_collect_into)]

use core::{initialize_id_generator, style, System};

use core::App;
use imnodes::AttributeFlag;
use locale::Locale;

mod message;

mod core;

pub mod errors;
pub mod exprtree;
pub mod extensions;
pub mod nodes;
pub mod pins;
pub mod utils;
pub mod locale;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    color_eyre::install().unwrap();

    let mut system = System::make_window("ODE Editor", (1024.0, 768.0));

    style::set_eel_style(system.imgui.style_mut());

    let nodesctx = imnodes::Context::new();
    let mut nodeseditor = nodesctx.create_editor();

    style::set_imnodes_style(nodeseditor.get_style());

    // SAFETY: The initialization of this value at the startup code is always
    // safe to do. However, using this value will only ever be safe while the
    // application isn't multithreaded or at least not the GUI part, which
    // honestly doesn't need to be.
    unsafe { initialize_id_generator(nodeseditor.new_identifier_generator()) };

    let _link_detach = nodeseditor.push(AttributeFlag::EnableLinkDetachWithDragClick);
    // let _link_creation = nodeseditor.push(AttributeFlag::EnableLinkCreationOnSnap);

    let mut locale = Locale::default();
    let mut app = App::new(&locale);

    let plot_ctx = implot::Context::create();

    system.main_loop(move |_, ui| {
        app.draw(ui, &mut nodeseditor, &mut plot_ctx.get_plot_ui(), &mut locale);
    });
    Ok(())
}
