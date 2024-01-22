#![allow(dead_code)]
#![windows_subsystem = "windows"]
pub mod egui_frontend;
pub mod math;
pub mod minicalc;
use clap::Parser;

#[derive(Parser)]
#[command()]
pub struct Args {
    #[arg(short, long, default_value = "minicalc-vars")]
    vars: String,
}

fn main() {
    let args = Args::parse();
    let _ = egui_frontend::egui_main(args);
}
