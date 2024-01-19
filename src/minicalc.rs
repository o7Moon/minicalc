use crate::math::{Num, equation::Equation, base::NumberBase};
use std::collections::HashMap;

// contains state that is shared across frontends
pub struct State {
    pub equation: Equation,
    pub command: Option<String>,
    pub base: NumberBase,
    pub variables: HashMap<String, Num>,
    pub vars_path: String,
    pub cached_equation_display: Option<String>,
}

impl Default for State {
    fn default() -> Self {
        Self { 
            equation: Equation::default(), 
            command: None, 
            base: NumberBase::Decimal,
            variables: HashMap::new(), 
            vars_path: "minicalc-vars".to_owned(),
            cached_equation_display: None,
        }
    }
}