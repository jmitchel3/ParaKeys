//! ParaKeys GUI entry — theme + app shell.

mod app;
mod ds;

use eframe::egui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1020.0, 640.0])
            .with_min_inner_size([860.0, 520.0])
            .with_title("ParaKeys"),
        ..Default::default()
    };
    eframe::run_native(
        "ParaKeys",
        options,
        Box::new(|cc| {
            ds::apply_theme(&cc.egui_ctx);
            Ok(Box::new(app::App::new()))
        }),
    )
}
