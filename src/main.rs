#![allow(dead_code)]
#![windows_subsystem = "windows"]
pub mod egui_frontend;
pub mod term_frontend;
pub mod math;
pub mod minicalc;
pub mod config;
use clap::{Parser, ValueEnum};

#[derive(Clone, ValueEnum)]
pub enum Frontend {
    Egui,
    Term,
}

impl std::fmt::Display for Frontend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Egui => "Egui".fmt(f),
            Self::Term => "Term".fmt(f),
        }
    }
}

impl std::str::FromStr for Frontend {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Egui" => Ok(Self::Egui),
            "Term" => Ok(Self::Term),
            _ => Err("Invalid frontend".to_owned())
        }
    }
}

#[derive(Parser)]
#[command()]
pub struct Args {
    #[arg(short, long, default_value = "minicalc-vars")]
    vars: String,
    #[arg(short, long, default_value = "egui")]
    frontend: Frontend,
}

fn main() {
    let args = Args::parse();
    match args.frontend {
        Frontend::Egui => {
            _ = egui_frontend::egui_main(args);
        },
        Frontend::Term => {
            term_frontend::main::crossterm_main(args);
        }
    }
    //term_frontend::main::crossterm_main(args);
    //let _ = egui_frontend::egui_main(args);
}
