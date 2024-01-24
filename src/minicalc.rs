use crate::math::{Num, equation::Equation, base::NumberBase, parsefmt};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use crate::config::Config;

// contains state that is shared across frontends
pub struct State {
    pub equation: Equation,
    pub command: Option<String>,
    pub base: NumberBase,
    pub variables: HashMap<String, Num>,
    pub vars_path: String,
    pub cached_equation_display: Option<String>,
    pub config: Config,
    pub exiting: bool,
}

impl Default for State {
    fn default() -> Self {
        let conf = Config::load();
        Self { 
            equation: Equation::default(), 
            command: None, 
            base: conf.base.clone(),
            variables: HashMap::new(), 
            vars_path: "minicalc-vars".to_owned(),
            cached_equation_display: None,
            config: conf,
            exiting: false,
        }
    }
}

impl State {
    pub fn display(&mut self) -> String {
        if self.command.is_some() {
            format!(":{}", self.command.as_ref().unwrap())
        } else {
            if self.cached_equation_display.is_some() {
                self.cached_equation_display.clone().unwrap()
            } else {
                let display = self.equation.display(self.base.clone(), self.config.max_fractional_places);
                self.cached_equation_display = Some(display.clone());
                display
            }
        }
    } 
    pub fn try_type_single(&mut self, char: char) {
        self.equation.try_type_single(char.to_uppercase().next().unwrap().to_string().as_str(), self.base.clone(), self.config.max_fractional_places) 
    }
    pub fn enter_command_entry(&mut self, command: String) {
        self.command = Some(command);
    }
    pub fn enter_equation_entry(&mut self) {
        self.command = None;
    }
    pub fn write_vars(&mut self) {
        let mut out = String::new();
        for (name, n) in self.variables.iter() {
            out += format!("{}\n{}\n",name,n).as_str();
        }
        _ = fs::write(self.vars_path.clone(), out);
    } 
    pub fn read_vars(&mut self) {
        let data = fs::read_to_string(self.vars_path.clone());
        if data.is_err() {
            return
        }
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
    pub fn delete_one(&mut self) {
        if self.command.is_some() {
            let c = self.command.as_mut().unwrap();
            if c.len() > 0 {
                *c = c[..c.len()-1].to_owned();
            } else {
                self.command = None;
            }
        } else {
            self.equation.delete_one_mut(self.base.clone(), self.config.max_fractional_places);
        }
        self.cached_equation_display = None;
    }
    pub fn type_string(&mut self, text: String) {
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
        self.cached_equation_display = None;// invalidate the cached display
    }
    pub fn execute_command(&mut self) {
        let command = self.command.clone().unwrap_or("".to_owned());
        match command.trim() {
            // single string commands with no arguments
            "b" | "binary" | "b2"  => {
                self.base = NumberBase::Binary; 
                self.cached_equation_display = None;
            },
            "x" | "hex" | "hexadecimal" | "b16" => {
                self.base = NumberBase::Hexadecimal; 
                self.cached_equation_display = None;
            },
            "d" | "decimal" | "b10" => {
                self.base = NumberBase::Decimal; 
                self.cached_equation_display = None;
            },
            "q" | "quit" | "exit" => {self.exiting = true},
            "w" | "write" => {self.write_vars()},
            "r" | "read" => {self.read_vars()},
            "c" | "clear" => {self.variables = HashMap::new()},
            "wq" => {self.write_vars(); self.exiting = true},
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
                                if self.equation.editing_left() {
                                    break 's_case
                                };
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

                        if !self.variables.contains_key(name) {
                            break 'l_case
                        };

                        match side {
                            "l" | "left" => {
                                self.equation.left = self.variables.get(name).unwrap().clone();
                                self.cached_equation_display = None;
                            },
                            "r" | "right" => {
                                if self.equation.editing_left() {
                                    break 'l_case
                                };
                                self.equation.right = Some(self.variables.get(name).unwrap().clone());
                                self.cached_equation_display = None;
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
                    },
                    _ => {},
                }
            },
        }
        self.enter_equation_entry();
    } 
}
