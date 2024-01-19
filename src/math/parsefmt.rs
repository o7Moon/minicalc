use std::str::FromStr;

use super::{
    Num,
    NumComponent,
    base::NumberBase,
};

macro_rules! num {
    ($numer:expr, $denom:expr) => {
        Num::new(NumComponent::from($numer), NumComponent::from($denom))
    };
}

// only parse b10 numbers at the moment
pub fn parse(s: String) -> Option<Num> {
    let parts = s.split(".").collect::<Vec<&str>>();
    if parts.len() == 1 {
        let int = NumComponent::from_str(parts[0]);
        let int = match int {
            Ok(int) => {int},
            Err(_) => {return None},
        };
        Some(num!(int, 1))
    } else if parts.len() == 2 {
        let int = NumComponent::from_str(parts[0]);
        let int = match int {
            Ok(int) => {int},
            Err(_) => {return None},
        };
        let fract = parse_fract(parts[1]);
        let fract = match fract {
            Some(fract) => {fract},
            None => {return None},
        };
        Some(num!(int, 1) + fract)
    } else {
        None
    }
}

fn parse_fract(s: &str) -> Option<Num> {
    let comp = NumComponent::from_str(s);
    let comp = match comp {
        Ok(comp) => {comp},
        Err(_) => {return None},
    };
    let mut bound = NumComponent::from(10);
    while comp >= bound {
        bound *= 10;
    }
    Some(Num::new(comp, bound))
}

pub fn fmt(n: Num, base: NumberBase) -> String {
    if n.is_integer() {
        fmt_int(n.to_integer(), base)
    } else {
        fmt_int(n.to_integer(), base.clone()) + "." + fmt_fract(n.fract(), base).as_str()
    }
}

fn fmt_int(n: NumComponent, base: NumberBase) -> String {
    match base {
        NumberBase::Binary => {
            format!("{n:#b}")
        },
        NumberBase::Decimal => {
            format!("{n}")
        },
        NumberBase::Hexadecimal => {
            format!("{n:#X}")
        }
    }
}

fn fmt_fract(n: Num, base: NumberBase) -> String {
    let mut out = "".to_owned();

        let mut n = n;
        let placevalue = num!(base.place_value(), 1); 
        
        for _ in 0..128 {// arbitrary max iteration
            n = n * placevalue.clone();// multiply by base to get a single digit in the integer part
            let int = n.trunc().to_integer();
            out += match base {// format the digit and add it to output
                NumberBase::Binary => {format!("{:b}", int)},
                NumberBase::Decimal => {format!("{}", int)},
                NumberBase::Hexadecimal => {format!("{:X}", int)},
            }.as_str();
            n = n.fract();// cut off the integer part and repeat
            if n.numer() == &NumComponent::from(0) {break}// no trailing zeros
        }

        out
}

#[test]
fn fmt_test() {
    assert_eq!(fmt(num!(1, 3),NumberBase::Decimal),
    // 128 decimal places of 3 !!
    "0.33333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333".to_owned()
    );

    assert_eq!(fmt(num!(1, 2), NumberBase::Binary),
    "0b0.1".to_owned()
    );

    assert_eq!(fmt(num!(7, 1), NumberBase::Hexadecimal),
    "0x7".to_owned()
    );

    assert_eq!(fmt(num!(1, 4), NumberBase::Binary),
    "0b0.01".to_owned()
    );
}

#[test]
fn parse_test() {
    assert_eq!(
        parse("3.14159".to_owned()),
        Some(num!(314159,100000))
    );
}