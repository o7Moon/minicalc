#![allow(dead_code)]
#![windows_subsystem = "windows"]
use eframe::{
    egui::{Label, Visuals, Layout, Key},
    emath::Align,
    epaint::Color32,
};
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use std::{collections::HashMap, time::Duration};
use std::process;
use std::fs;
use std::path::Path;
use clap::Parser;
use eframe::egui;
use arboard::Clipboard;

#[derive(Clone, Debug, PartialEq, Eq, Copy)]
enum Operation {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
}

impl Operation {
    fn char(&self) -> &str {
        match self {
            Operation::Add => "+",
            Operation::Sub => "-",
            Operation::Mul => "*",
            Operation::Div => "/",
            Operation::Mod => "%",
        }
    }
}

impl Operation {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "+" => {Some(Operation::Add)},
            "-" => {Some(Operation::Sub)},
            "*" => {Some(Operation::Mul)},
            "/" => {Some(Operation::Div)},
            "%" => {Some(Operation::Mod)},
            _ => {None}
        }
    }
}

#[derive(Clone)]
enum NumberBase {
    Decimal,
    Binary,
    Hexadecimal,
}

// splits a decimal into integer and fractional parts
fn get_parts(n: Decimal) -> (Decimal, Decimal) {
    let fract = n.fract();
    let int = n.trunc();
    (int, fract)
}

impl NumberBase {
    fn print_decimal_in_base(&self, n: Decimal, trailing_zeros: Option<u8>) -> String {
        match self {
            NumberBase::Decimal => {
                let (int, fract) = get_parts(n);
                let fract =  if fract.is_zero() {
                    if trailing_zeros.is_none() {"".to_owned()} else {
                        let trailing_zeros = trailing_zeros.unwrap() as usize;
                        format!(".{:0<trailing_zeros$}", "")
                    }
                } else {
                    let trailing_zeros = if trailing_zeros.is_none() {"".to_owned()} else {
                        let trailing_zeros = trailing_zeros.unwrap() as usize;
                        format!("{:0<trailing_zeros$}", "")
                    };
                    format!(".{}{}", self.print_fractional_part_in_base(fract), trailing_zeros)
                };
                format!("{}{}", int.mantissa(), fract)
            },
            NumberBase::Binary => {
                let (int, fract) = get_parts(n);
                let fract = if fract.is_zero() {
                    if trailing_zeros.is_none() {
                        "".to_owned()
                    } else {
                        let trailing_zeros = trailing_zeros.unwrap() as usize;
                        format!(".{:0<trailing_zeros$}","")
                    }
                } else {
                    let trailing_zeros = if trailing_zeros.is_none() {"".to_owned()} else {
                        let trailing_zeros = trailing_zeros.unwrap() as usize;
                        format!("{:0<trailing_zeros$}", "")
                    }; 
                    format!(".{}{}", self.print_fractional_part_in_base(fract), trailing_zeros)
                };
                format!("{:#b}{}", int.mantissa(), fract)
            },
            NumberBase::Hexadecimal => {
                let (int, fract) = get_parts(n);
                let fract = if fract.is_zero() {
                    if trailing_zeros.is_none() {"".to_owned()} else {
                        let trailing_zeros = trailing_zeros.unwrap() as usize;
                        format!(".{:0<trailing_zeros$}","")
                    }
                } else {
                    let trailing_zeros = if trailing_zeros.is_none() {"".to_owned()} else {
                        let trailing_zeros = trailing_zeros.unwrap() as usize;
                        format!("{:0<trailing_zeros$}", "")
                    }; 
                    format!(".{}{}", self.print_fractional_part_in_base(fract), trailing_zeros)
                };
                format!("{:#X}{}", int.mantissa(), fract)
            }
        }
    }
    //                                     n is already fractional
    fn print_fractional_part_in_base(&self, n: Decimal) -> String {
        let mut out = "".to_owned();

        let mut n = n;
        let placevalue = Decimal::from_i64(self.place_value() as i64).unwrap(); 
        
        for _ in 0..128 {// arbitrary max iteration
            n = match n.checked_mul(placevalue){// multiply by base to get a single digit in the integer part
                    Some(n) => {n},
                    None => {return "".to_owned()},
            };
            let int = n.trunc().mantissa();
            out += match self {// format the digit and add it to output
                NumberBase::Binary => {format!("{:b}", int)},
                NumberBase::Decimal => {format!("{}", int)},
                NumberBase::Hexadecimal => {format!("{:X}", int)},
            }.as_str();
            n = n.fract();// cut off the integer part and repeat
            if n.is_zero() {break}// no trailing zeros
        }

        out
    }
    
    fn place_value(&self) -> u32 {
        match self {
            NumberBase::Binary => 2,
            NumberBase::Decimal => 10,
            NumberBase::Hexadecimal => 16,
        }
    }
}

#[test]
fn test_print_decimal() {
    assert_eq!(NumberBase::Hexadecimal.print_decimal_in_base(dec!(15.5), None), "0xF.8".to_owned());
    assert_eq!(NumberBase::Decimal.print_decimal_in_base(dec!(3.141592653589), None), "3.141592653589".to_owned());
    assert_eq!(NumberBase::Hexadecimal.print_decimal_in_base(dec!(3.0000), None), "0x3".to_owned());
    assert_eq!(NumberBase::Binary.print_decimal_in_base(dec!(7), None), "0b111".to_owned());
    assert_eq!(NumberBase::Binary.print_decimal_in_base(dec!(3.125), None),"0b11.001".to_owned());
    assert_eq!(NumberBase::Decimal.print_decimal_in_base(dec!(3.000), None),"3".to_owned());
    assert_eq!(NumberBase::Decimal.print_decimal_in_base(dec!(3.03), None),"3.03".to_owned());
    assert_eq!(NumberBase::Decimal.print_decimal_in_base(dec!(1.00007), None), "1.00007".to_owned());
}

#[derive(Clone, Debug, PartialEq, Eq, Copy)]
struct Equation {
    left: Decimal,
    operation: Option<Operation>,
    right: Option<Decimal>,
    editing_trailing_zeros: Option<u8>,// None = no fractional part
}

impl Equation {
    fn eval(&self) -> Option<Equation> {
        if self.operation.is_none() || self.operation.is_none() {return None}

        let result: Decimal = match self.operation.as_ref().unwrap() {
            Operation::Add => {
                let r = self.left.checked_add(self.right.unwrap());
                match r {
                    Some(n) => {n},
                    None => {return None},
                }
            },
            Operation::Sub => {
                let r = self.left.checked_sub(self.right.unwrap());
                match r {
                    Some(n) => {n},
                    None => {return None},
                }
            },
            Operation::Mul => {
                let r = self.left.checked_mul(self.right.unwrap());
                match r {
                    Some(n) => {n},
                    None => {return None},
                }
            },
            Operation::Div => {
                let r = self.left.checked_div(self.right.unwrap());
                match r {
                    Some(n) => {n},
                    None => {return None}
                }
            },
            Operation::Mod => {
                let r = self.left.checked_rem(self.right.unwrap());
                match r {
                    Some(n) => {n},
                    None => {return None},
                }
            },
        };

        Some(Self {left: result, operation: None, right: None, editing_trailing_zeros: None})
    }

    fn eval_mut(&mut self) {
        let result = self.eval();
        if let Some(result) = result {
            let _ = std::mem::replace(self, result); // thanks borrow checker
        }
    }

    fn display(&self, base: NumberBase) -> String {
        let mut out = "".to_owned();
        
        out += base.print_decimal_in_base(self.left, if self.editing_left() {self.editing_trailing_zeros} else {None}).as_str();
        if self.editing_left() {// no operation or right operand
            return out;
        }

        let operation = self.operation.as_ref().unwrap();
        out += format!(" {} ", operation.char()).as_str();

        out += base.print_decimal_in_base(self.right.unwrap(), self.editing_trailing_zeros).as_str();
        
        out
    }

    fn editing_left(&self) -> bool {// false = right
        self.operation.is_none()
    }

    fn add_operation(&mut self, op: Operation) {
        self.editing_trailing_zeros = None;
        self.operation = Some(op);
        self.right = Some(dec!(0));
    }
    fn delete_one(&self, base: NumberBase) -> Self {
        let mut new = self.clone();
        new.delete_one_mut(base);
        new
    } 

    fn delete_one_mut(&mut self, base: NumberBase) {
        if self.editing_trailing_zeros.is_some() {
            let value = self.editing_trailing_zeros.unwrap().clone();
            let n = self.editing_num();
            if value > 0 {
                self.editing_trailing_zeros = Some(value - 1);
            } else {
                self.editing_trailing_zeros = None;
            }
            if !n.is_integer() && value == 1 {self.editing_trailing_zeros = None}
            return;
        }

        let placevalue = Decimal::from_i64(base.place_value() as i64).unwrap();
        let mut n = if self.editing_left() {
            self.left
        } else {
            if self.right.unwrap().is_zero() {
                self.right = None;
                self.operation = None;
                return;
            }
            self.right.unwrap()
        };

        let mut times_shifted = -1;
        while !n.is_integer() {
            times_shifted += 1;
            let r = n.checked_mul(placevalue);
            n = match r {
                Some(n) => {n},
                None => {return},
            }
        }
        let r = n.checked_div(placevalue);
        n = match r {
            Some(n) => {n},
            None => {return},
        };
        n = n.trunc();
        while times_shifted > 0 {
            let r = n.checked_div(placevalue);
            n = match r {
                Some(n) => {n},
                None => {return},
            };
            times_shifted -= 1;
        }

        if self.editing_left() {self.left = n} else {self.right = Some(n)};
    }

    fn editing_num(&self) -> Decimal {
        if self.editing_left() {self.left} else {self.right.unwrap()}
    }

    fn set_editing_num(&mut self, n: Decimal) {
        if self.editing_left() {
            self.left = n;
        } else {
            self.right = Some(n);
        }
    } 

    fn new_single(n: Decimal) -> Self {
        Self {
            left: n,
            ..Default::default()
        }
    }

    fn new_full(left: Decimal, op: Operation, right: Decimal) -> Self {
        Self {
            left,
            operation: Some(op),
            right: Some(right),
            ..Default::default()
        }
    }

    fn with_trailing_zeros(&self, n: u8) -> Self {
        Self {
            left: self.left,
            operation: self.operation,
            right: self.right,
            editing_trailing_zeros: Some(n),
        }
    }

    fn add_trailing_zeros(&mut self, n: u8) {
        let z = self.editing_trailing_zeros;
        if z == None {
            self.editing_trailing_zeros = Some(n);
        } else {
            self.editing_trailing_zeros = Some(z.unwrap().saturating_add(n));
        }
    }

    fn try_type_single(&mut self, input: &str, base: NumberBase) {
        match input {
            "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "A" | "B" | "C" | "D" | "E" | "F" => {
                self.try_add_digit(input, base)
            },
            "+" | "-" | "*" | "/" | "%" => {
                if self.editing_left() {
                    self.add_operation(Operation::from_str(input).unwrap())
                }
            },
            "." => {
                if self.editing_trailing_zeros == None {
                    self.editing_trailing_zeros = Some(0)
                }
            },
            _ => {},
        }
    }

    fn try_add_digit(&mut self, input: &str, base: NumberBase) {
        let places = "0123456789ABCDEF";
        let place = places.find(input);
        if place.is_none() {return;}
        let digit = place.unwrap();
        if digit >= base.place_value() as usize {return;}
        
        let mut n = self.editing_num();
        let placevalue = Decimal::from_i64(base.place_value() as i64).unwrap();

        // adding an integer digit
        if n.is_integer() && self.editing_trailing_zeros == None {
            let r = n.checked_mul(placevalue);
            n = match r {
                Some(n) => {n},
                None => {return},
            };
            let r = n.checked_add(Decimal::from_u8(digit as u8).unwrap());
            n = match r {
                Some(n) => {n},
                None => {return},
            };
            self.set_editing_num(n);
            return;
        }
        // adding a trailing zero
        if digit == 0 {
            self.add_trailing_zeros(1);
            return;
        }
        // adding a nonzero fractional digit
        let mut placesmoved = 0;
        while !n.is_integer() {
            let r = n.checked_mul(placevalue);
            n = match r {
                Some(n) => {n},
                None => {return},
            };
            placesmoved += 1;
        }
        for _ in 0..(self.editing_trailing_zeros.unwrap_or(0) + 1) {
            let r = n.checked_mul(placevalue);
            n = match r {
                Some(n) => {n},
                None => {return},
            };
            placesmoved += 1;
        }
        let r = n.checked_add(Decimal::from_u8(digit as u8).unwrap());
        n = match r {
            Some(n) => {n},
            None => {return},
        };
        for _ in 0..placesmoved {
            let r = n.checked_div(placevalue);
            n = match r {
                Some(n) => {n},
                None => {return},
            };
        }
        self.set_editing_num(n);
        self.editing_trailing_zeros = Some(0);
    }
}

#[test]
fn test_delete_one() {
    assert_eq!(Equation::default(), Equation::default().delete_one(NumberBase::Decimal));
    let eq = Equation::new_single(dec!(3.045));
    assert_eq!(
        eq.delete_one(NumberBase::Decimal).left,
        dec!(3.04)
    );
    let eq = Equation::new_single(dec!(3.125));
    assert_eq!(
        eq.delete_one(NumberBase::Binary).left,
        dec!(3)
    );
    let eq = Equation::new_full(dec!(1), Operation::Add, dec!(0));
    let eq2 = Equation::new_single(dec!(1));
    assert_eq!(eq.delete_one(NumberBase::Hexadecimal), eq2);
}

#[test]
fn test_display() {
    let mut eq = Equation::default();
    assert_eq!(eq.display(NumberBase::Decimal),"0");
    assert_eq!(eq.display(NumberBase::Hexadecimal), "0x0");
    assert_eq!(eq.display(NumberBase::Binary), "0b0");
    eq.left += dec!(3.125);
    eq.add_operation(Operation::Sub);
    eq.right = Some(dec!(2));
    assert_eq!(eq.display(NumberBase::Binary), "0b11.001 - 0b10");
    assert_eq!(eq.display(NumberBase::Hexadecimal), "0x3.2 - 0x2");
}

#[test]
fn test_display_trailing_zero() {
    let mut eq = Equation::new_full(dec!(1), Operation::Add, dec!(8.125));
    assert_eq!(eq.with_trailing_zeros(2).display(NumberBase::Binary), "0b1 + 0b1000.00100");
    eq.delete_one_mut(NumberBase::Binary);
    assert_eq!(eq.display(NumberBase::Binary), "0b1 + 0b1000");
    assert_eq!(eq.with_trailing_zeros(0).display(NumberBase::Hexadecimal), "0x1 + 0x8.");
}

#[test]
fn test_eval() {
    let mut eq = Equation::new_full(dec!(15), Operation::Div, dec!(3));

    eq.eval_mut();

    assert_eq!(eq, Equation::new_single(dec!(5)));
}

#[test]
fn test_add_digit() {
    let mut eq = Equation::new_full(dec!(15), Operation::Add, dec!(3)).with_trailing_zeros(1);
    eq.try_add_digit("5", NumberBase::Decimal);
    assert_eq!(eq.right.unwrap(), dec!(3.05));

    let mut eq = Equation::new_single(dec!(3));
    eq.try_add_digit("1", NumberBase::Binary);
    assert_eq!(eq.left, dec!(7));
    eq.add_trailing_zeros(2);
    eq.try_add_digit("1", NumberBase::Binary);
    assert_eq!(eq.left, dec!(7.125));

    let mut eq = Equation::new_single(dec!(10));
    eq.try_add_digit("A", NumberBase::Decimal);// adding "A" in decimal is nonsense, so it should
    // be unchanged
    assert_eq!(eq.left, dec!(10));
}

impl Default for Equation {
    fn default() -> Self {
        Self {
            left: dec!(0),
            operation: None,
            right: None,
            editing_trailing_zeros: None,
        }
    }
}

struct AppState {
    equation: Equation,
    command: Option<String>,
    base: NumberBase,
    variables: HashMap<String, Decimal>,
    window_decorated: bool,
    vars_path: String,
    always_on_top: bool,
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
    fn read_vars(&mut self) {
        let data = fs::read_to_string(self.vars_path.clone());
        if data.is_err() {return};
        self.variables = HashMap::new();
        let data = data.unwrap();
        let lines: Vec<&str> = data.lines().collect();
        for pair in lines.chunks(2) {
            let n = Decimal::from_str(pair[1]);
            let n = match n {
                Ok(n) => {n},
                Err(_) => {continue},
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
                                self.variables.insert(name.to_owned(), self.equation.left);
                            },
                            "r" | "right" => {
                                if self.equation.editing_left() {break 's_case};
                                self.variables.insert(name.to_owned(), self.equation.right.unwrap());
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

#[derive(Parser)]
#[command()]
struct Args {
    #[arg(short, long, default_value = "minicalc-vars")]
    vars: String,
}

fn main() -> Result<(), eframe::Error> {
    let args = Args::parse();
    let mut state = AppState::default();
    state.vars_path = args.vars;
    state.read_vars();
    
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_decorations(false)
            .with_title("minicalc")
            .with_inner_size([300., 50.])
            .with_resizable(true)
            .with_icon(eframe::icon_data::from_png_bytes(include_bytes!("../icon.png")).expect("failed to load embedded icon")),
        ..Default::default()
    };
    eframe::run_native("minicalc", options, Box::new(|_| Box::new(state)))
}
