use imgui::{ Ui, StyleVar };

pub fn rgb(r: u8, g: u8, b: u8) -> [f32; 4] {
    [r as f32, b as f32, g as f32, 255.0].map(|x| x / 255.0)
}

pub fn input_num(ui: &Ui, label: &str, value: &mut f64) -> bool {
    let _width = ui.push_item_width(72.0);
    ui.input_scalar(label, value)
        .display_format("%.8lf")
        .build()
}

pub fn search_bar(ui: &Ui, mut buf: &mut String) {
    let _k = ui.push_style_var(StyleVar::ItemSpacing([0.2, 0.0]));
    // Magnifying glass icon
    ui.text("\u{f002}");
    ui.same_line();
    let _k = ui.push_item_width(-1.0);
    ui.input_text("##search query", buf).build();
}
