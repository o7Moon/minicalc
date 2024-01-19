use crate::math::{
    equation::Equation,
    Num,
    base::NumberBase,
    parsefmt,
};
use eframe::{
    egui::{Label, Visuals, Layout, self},
    emath::Align,
    epaint::Color32,
};
use std::collections::HashMap;
use std::fs;
use std::process;
use std::path::Path;
use arboard::Clipboard;
use std::time::Duration;

pub struct AppState {
    pub equation: Equation,
    pub command: Option<String>,
    pub base: NumberBase,
    pub variables: HashMap<String, Num>,
    pub window_decorated: bool,
    pub vars_path: String,
    pub always_on_top: bool,
}

impl AppState {
    fn display(&self) -> String {
        if self.command.is_some() {
            format!(":{}", self.command.as_ref().unwrap())
        } else {
            self.equation.display(self.base.clone())
        }
    }
    fn try_type_single(&mut self, char: char) {
        self.equation.try_type_single(char.to_uppercase().next().unwrap().to_string().as_str(), self.base.clone()) 
    }
    fn enter_command_entry(&mut self, command: String) {
        self.command = Some(command);
    }
    fn enter_equation_entry(&mut self) {
        self.command = None;
    }
    fn write_vars(&self) {
        let mut out = String::new();
        for (name, n) in self.variables.iter() {
            out += format!("{}\n{}\n",name,n).as_str();
        }
        let _ = fs::write(self.vars_path.clone(), out);
    }
    pub fn read_vars(&mut self) {
        let data = fs::read_to_string(self.vars_path.clone());
        if data.is_err() {return};
        self.variables = HashMap::new();
        let data = data.unwrap();
        let lines: Vec<&str> = data.lines().collect();
        for pair in lines.chunks(2) {
            let n = parsefmt::parse(pair[1].to_owned());
            let n = match n {
                Some(n) => {n},
                None => {continue},
            };
            self.variables.insert(pair[0].to_owned(), n);
        }
    }
    fn delete_one(&mut self) {
        if self.command.is_some() {
            let c = self.command.as_mut().unwrap();
            if c.len() > 0 {
                *c = c[..c.len()-1].to_owned();
            } else {
                self.command = None;
            }
        } else {
            self.equation.delete_one_mut(self.base.clone());
        }
    }
    fn type_string(&mut self, text: String) {
        let text = text.replace("\n", "");
        if self.command.is_some() {
            *self.command.as_mut().unwrap() += text.as_str();
        } else {
            if text.starts_with(":") {
                self.command = Some("".to_owned());
                self.type_string(text[1..].to_owned());
            }
            for char in text.chars() {
                self.try_type_single(char);
            }
        }
    }
    fn execute_command(&mut self) {
        let command = self.command.clone().unwrap_or("".to_owned());
        match command.trim() {
            // single string commands with no arguments
            "b" | "binary" | "b2"  => {self.base = NumberBase::Binary},
            "x" | "hex" | "hexadecimal" | "b16" => {self.base = NumberBase::Hexadecimal},
            "d" | "decimal" | "b10" => {self.base = NumberBase::Decimal},
            "D" | "decorated" | "border" => {self.window_decorated = !self.window_decorated},
            "q" | "quit" | "exit" => {process::exit(0x0)},
            "w" | "write" => {self.write_vars()},
            "r" | "read" => {self.read_vars()},
            "c" | "clear" => {self.variables = HashMap::new()},
            "wq" => {self.write_vars(); process::exit(0x0)},
            "t" | "top" => {self.always_on_top = !self.always_on_top},
            "" => {},// skip this case before we do any other logic
            _ => {
                let mut args = command.split(" ");
                match args.next().unwrap_or("") {
                    "s" | "st" | "store" => 's_case: {
                        let r = args.next();
                        let side = match r {
                            Some(side) => {side},
                            None => {break 's_case}
                        };
                        match side {
                            "l" | "left" | "r" | "right" | "R" | "result" => {},
                            _ => {break 's_case}
                        };
                        let r = args.next();
                        let name = match r {
                            Some(name) => {name},
                            None => {break 's_case}
                        };

                        match side {
                            "l" | "left" => {
                                self.variables.insert(name.to_owned(), self.equation.left.clone());
                            },
                            "r" | "right" => {
                                if self.equation.editing_left() {break 's_case};
                                self.variables.insert(name.to_owned(), self.equation.right.clone().unwrap());
                            },
                            "R" | "result" => {
                                let r = self.equation.eval();
                                if let Some(result) = r {
                                    self.variables.insert(name.to_owned(), result.left);
                                }
                            },
                            _ => {}
                        };
                    },
                    "l" | "ld" | "load" => 'l_case: {
                        let r = args.next();
                        let side = match r {
                            Some(side) => {side},
                            None => {break 'l_case}
                        };
                        match side {
                            "l" | "left" | "r" | "right" => {},
                            _ => {break 'l_case}
                        };
                        let r = args.next();
                        let name = match r {
                            Some(name) => {name},
                            None => {break 'l_case}
                        };

                        if !self.variables.contains_key(name) {break 'l_case};

                        match side {
                            "l" | "left" => {
                                self.equation.left = self.variables.get(name).unwrap().clone();
                            },
                            "r" | "right" => {
                                if self.equation.editing_left() {break 'l_case};
                                self.equation.right = Some(self.variables.get(name).unwrap().clone());
                            },
                            _ => {}
                        }
                    },
                    "p" | "path" => {
                        let remaining: Vec<&str> = args.collect();
                        let path = remaining.join(" ");
                        self.vars_path = path;
                    },
                    "r" | "read" => 'r_case: {
                        let remaining: Vec<&str> = args.collect();
                        let path: String = remaining.join(" ");
                        let path = Path::new(path.as_str());
                        if !Path::exists(&path){break 'r_case};
                        self.vars_path = path.to_string_lossy().into();
                        self.read_vars();
                    },
                    "w" | "write" => {
                        let remaining: Vec<&str> = args.collect();
                        let path = remaining.join(" ");
                        self.vars_path = path;
                        self.write_vars();
                    }
                    _ => {},
                }
            },
        }
        self.enter_equation_entry();
    }
    fn copy_equation(&self) {
        let cbrd = Clipboard::new();
        let mut cbrd = match cbrd {
            Ok(cbrd) => cbrd,
            Err(e) => {
                println!("error while copying: {:?}", e);
                return
            }
        };
        let text = self.equation.display(self.base.clone());
        let _ = cbrd.set_text(text);
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self { 
            equation: Equation::default(), 
            command: None, 
            base: NumberBase::Decimal,
            variables: HashMap::new(), 
            window_decorated: false, 
            vars_path: "minicalc-vars".to_owned(),
            always_on_top: false,
        }
    }
}
impl eframe::App for AppState {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let visuals = Visuals {
            panel_fill: Color32::BLACK,
            override_text_color: Some(Color32::WHITE),
            ..Default::default()
        };
        ctx.set_visuals(visuals);
        for event in ctx.input(|i| i.events.iter().cloned().collect::<Vec<egui::Event>>()) {
            match event {
                egui::Event::Text(t) => {
                    self.type_string(t)
                },
                egui::Event::Paste(t) => {
                    self.type_string(t)
                },
                egui::Event::PointerButton { pos: _, button: _, pressed, modifiers: _ } => {
                    if pressed {
                        ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag)
                    }
                },
                egui::Event::Copy => {
                    self.copy_equation();
                },
                _ => {},
            }
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Backspace)) {
            self.delete_one();
        }
        let rect = ctx.input(|i| i.viewport().outer_rect);
        let size = match rect {
            Some(rect) => {
                rect.height().min(5000.) * 0.55 // min because sometimes unreasonable heights are given when app starts
            },
            None => {12.}
        };
        let display = egui::RichText::new(self.display()).size(size);
        let cursor_blink = ctx.input(|i| (i.time % 1.) > 0.5 );
        let cursor = egui::RichText::new("|").size(size).color(if cursor_blink {Color32::WHITE} else {Color32::TRANSPARENT});
        if self.command.is_some() {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.with_layout(Layout::left_to_right(Align::Center), |ui|{
                    ui.add(Label::new(display).wrap(false));
                    ui.add_space(-8.5);
                    ui.label(cursor);
                });
            });
            if ctx.input(|i| i.key_down(egui::Key::Enter)) {
                self.execute_command();
                ctx.send_viewport_cmd(egui::ViewportCommand::Decorations(self.window_decorated));
                ctx.send_viewport_cmd(if self.always_on_top { 
                    egui::ViewportCommand::WindowLevel(egui::WindowLevel::AlwaysOnTop) 
                } else {
                    egui::ViewportCommand::WindowLevel(egui::WindowLevel::Normal)
                });
            }
        } else {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(cursor);
                    ui.add_space(-8.5);
                    ui.add(Label::new(display).wrap(false));
                });
            });
            if ctx.input(|i| i.key_down(egui::Key::Enter)) {
                self.equation.eval_mut();
            }
        }
        ctx.request_repaint_after(Duration::from_millis(500));
    }
}