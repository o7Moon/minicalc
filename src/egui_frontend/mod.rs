use crate::Args;
use self::app::AppState;
use eframe::egui;

pub mod app;
pub mod config;


pub fn egui_main(args: Args) -> Result<(), eframe::Error> {
    let mut app = AppState::default();
    app.state.vars_path = args.vars;
    app.read_vars();

    let mut viewport = egui::ViewportBuilder::default()
        .with_decorations(app.window_decorated)
        .with_title("minicalc")
        .with_inner_size([300., 50.])
        .with_resizable(true)
        .with_icon(eframe::icon_data::from_png_bytes(include_bytes!("../../icon.png")).expect("failed to load embedded icon"));

    if app.config.always_on_top {
        viewport = viewport.with_always_on_top();
    }
    
    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    eframe::run_native("minicalc", options, Box::new(|cc| {
        cc.egui_ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(
            if app.always_on_top {egui::WindowLevel::AlwaysOnTop} else {egui::WindowLevel::Normal}
        ));
        Box::new(app)
    }))
}
