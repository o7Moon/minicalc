use std::ops::RemAssign;

use self::base::NumberBase;
use self::operation::Operation;
use super::parsefmt::fmt;
use super::*;
use num_traits::Zero;
use num_traits::ops::checked::*;
use std::ops::Rem;

macro_rules! num {
    ($numer:expr, $denom:expr) => {
        Num::new(NumComponent::from($numer), NumComponent::from($denom))
    };
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Equation {
    pub left: Num,
    pub operation: Option<Operation>,
    pub right: Option<Num>,
    pub editing_trailing_zeros: Option<u8>,// None = no fractional part
}

impl Default for Equation {
    fn default() -> Self {
        Self {
            left: Num::from(NumComponent::from(0)),
            operation: None,
            right: None,
            editing_trailing_zeros: None,
        }
    }
}

impl Equation {
    pub fn display(&self, base: NumberBase) -> String {
        let mut out = "".to_owned();
        
        out += fmt(self.left.clone(), base.clone()).as_str();

        if self.editing_left() && self.editing_trailing_zeros.is_some() {
            if self.left.is_integer() {out += "."};
            let trailing_zeros = self.editing_trailing_zeros.unwrap() as usize;
            out += format!("{0:<trailing_zeros$}", "").as_str();
        }
        
        if self.editing_left() {// no operation or right operand
            return out;
        }

        let operation = self.operation.as_ref().unwrap();
        out += format!(" {} ", operation.char()).as_str();

        out += fmt(self.right.as_ref().unwrap().clone(), base).as_str();

        if self.editing_left() && self.editing_trailing_zeros.is_some() {
            if self.right.as_ref().unwrap().clone().is_integer() {out += "."};
            let trailing_zeros = self.editing_trailing_zeros.unwrap() as usize;
            out += format!("{0:<trailing_zeros$}", "").as_str();
        }
        
        out
    }
    pub fn editing_left(&self) -> bool {
        self.right.is_none()
    }
    pub fn eval(&self) -> Option<Self> {
        if self.operation.is_none() || self.operation.is_none() {return None}

        let result: Num = match self.operation.as_ref().unwrap() {
            Operation::Add => {
                let r = self.left.checked_add(&self.right.clone().unwrap());
                match r {
                    Some(n) => {n},
                    None => {return None},
                }
            },
            Operation::Sub => {
                let r = self.left.checked_sub(&self.right.clone().unwrap());
                match r {
                    Some(n) => {n},
                    None => {return None},
                }
            },
            Operation::Mul => {
                let r = self.left.checked_mul(&self.right.clone().unwrap());
                match r {
                    Some(n) => {n},
                    None => {return None},
                }
            },
            Operation::Div => {
                let r = self.left.checked_div(&self.right.clone().unwrap());
                match r {
                    Some(n) => {n},
                    None => {return None}
                }
            },
            Operation::Mod => {
                if self.right.clone().unwrap().is_zero() {return None};
                self.left.clone().rem(self.right.clone().unwrap())
            },
        };

        Some(Self {left: result, operation: None, right: None, editing_trailing_zeros: None})
    }

    fn add_operation(&mut self, op: Operation) {
        self.editing_trailing_zeros = None;
        self.operation = Some(op);
        self.right = Some(num!(0,1));
    }

    pub fn try_type_single(&mut self, input: &str, base: NumberBase) {
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

    fn editing_num(&self) -> Num {
        if let Some(n) = self.right.clone() {
            n
        } else {
            self.left.clone()
        }
    }

    fn set_editing_num(&mut self, n: Num) {
        if let Some(_) = self.right {
            self.right = Some(n)
        } else {
            self.left = n
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

    fn try_add_digit(&mut self, input: &str, base: NumberBase) {
        let places = "0123456789ABCDEF";
        let place = places.find(input);
        if place.is_none() {return;}
        let digit = place.unwrap();
        if digit >= base.place_value() as usize {return;}
        
        let mut n = self.editing_num();
        let placevalue = num!(base.place_value(), 1);

        // adding an integer digit
        if n.is_integer() && self.editing_trailing_zeros == None {
            let r = n.checked_mul(&placevalue);
            n = match r {
                Some(n) => {n},
                None => {return},
            };
            let r = n.checked_add(&num!(digit, 1));
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
        while !n.is_integer() && placesmoved < 128 {
            let r = n.checked_mul(&placevalue);
            n = match r {
                Some(n) => {n},
                None => {return},
            };
            placesmoved += 1;
        }

        if placesmoved >= 128 {return};// too many digits, maybe even infinite, like 1/3.

        for _ in 0..(self.editing_trailing_zeros.unwrap_or(0) + 1) {
            let r = n.checked_mul(&placevalue);
            n = match r {
                Some(n) => {n},
                None => {return},
            };
            placesmoved += 1;
        }
        let r = n.checked_add(&num!(digit, 1));
        n = match r {
            Some(n) => {n},
            None => {return},
        };
        for _ in 0..placesmoved {
            let r = n.checked_div(&placevalue);
            n = match r {
                Some(n) => {n},
                None => {return},
            };
        }
        self.set_editing_num(n);
        self.editing_trailing_zeros = Some(0);
    }
    pub fn eval_mut(&mut self) {
        let result = self.eval();
        if let Some(result) = result {
            let _ = std::mem::replace(self, result); // thanks borrow checker
        }
    }
    pub fn delete_one_mut(&mut self, base: NumberBase) {
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

        let placevalue = num!(base.place_value(), 1);
        let mut n = self.editing_num();

        if !self.editing_left() && n.is_zero() {
            self.right = None;
            self.operation = None;
        }

        let mut times_shifted = -1;
        while !n.is_integer() && times_shifted < 128 {
            times_shifted += 1;
            let r = n.checked_mul(&placevalue);
            n = match r {
                Some(n) => {n},
                None => {return},
            }
        }
        n = num!(n.to_integer(), 1);// if theres even more digits (the 128 limit was hit), then chop em off
        let r = n.checked_div(&placevalue);
        n = match r {
            Some(n) => {n},
            None => {return},
        };
        n = n.trunc();
        while times_shifted > 0 {
            let r = n.checked_div(&placevalue);
            n = match r {
                Some(n) => {n},
                None => {return},
            };
            times_shifted -= 1;
        }

        self.set_editing_num(n);
    }
}