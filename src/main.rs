mod app;
mod config;
mod db;
mod export;
mod llm;
mod ui;

use eframe::egui;

fn main() -> eframe::Result<()> {
    tracing_subscriber::fmt::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "FBench",
        options,
        Box::new(|cc| Ok(Box::new(app::App::new(cc)))),
    )
}
