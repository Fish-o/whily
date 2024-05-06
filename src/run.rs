use crate::{
  parser::{Statement, Value},
  symbolizer::Operator,
  Config,
};
use std::collections::HashMap;

const MAX_ITERATIONS: usize = 1024 * 128;

pub fn run(config: &Config, prog: &Statement) -> Result<HashMap<String, u64>, RuntimeError> {
  let mut state: HashMap<String, u64> = HashMap::new();
  run_with_state(config, prog, &mut state)?;
  Ok(state)
}

fn run_with_state(
  config: &Config,
  prog: &Statement,
  state: &mut HashMap<String, u64>,
) -> Result<(), RuntimeError> {
  match prog {
    Statement::S(left, right) => {
      run_with_state(config, left, state)?;
      run_with_state(config, right, state)?;
    }
    Statement::DeclareOperation(v0, v1, operator, v2) => {
      let v1 = match v1 {
        Value::Variable(var) if state.contains_key(var) => {
          state.get(var).expect("Variable not accessed?! 1")
        }
        Value::Variable(var) => return Err(RuntimeError::UnassignedVariable(var.to_owned())),
        Value::Constant(c) => c,
      };
      let v2 = match v2 {
        Value::Variable(var) if state.contains_key(var) => {
          state.get(var).expect("Variable not accessed?! 2")
        }
        Value::Variable(var) => return Err(RuntimeError::UnassignedVariable(var.to_owned())),
        Value::Constant(c) => c,
      };
      match operator {
        Operator::Subtract => {
          match v1.checked_sub(v2.to_owned()) {
            Some(val) => state.insert(v0.to_owned(), val),
            None => {
              if !config.allow_underflow {
                return Err(RuntimeError::VariableUnderflow(v0.to_owned()));
              } else {
                state.insert(v0.to_owned(), 0)
              }
            }
          };
        }
        Operator::Add => {
          match v1.checked_add(v2.to_owned()) {
            Some(val) => state.insert(v0.to_owned(), val),
            None => return Err(RuntimeError::VariableOverflow(v0.to_owned())),
          };
        }

        Operator::Multiply => {
          match v1.checked_mul(v2.to_owned()) {
            Some(val) => state.insert(v0.to_owned(), val),
            None => return Err(RuntimeError::VariableOverflow(v0.to_owned())),
          };
        }
      }
    }
    Statement::DeclareConst(v0, v) => {
      let v = match v {
        Value::Variable(var) if state.contains_key(var) => {
          state.get(var).expect("Variable not accessed?! 2")
        }
        Value::Variable(var) => return Err(RuntimeError::UnassignedVariable(var.to_owned())),
        Value::Constant(c) => c,
      };
      state.insert(v0.to_owned(), *v);
    }
    Statement::While(cv, s) => {
      if !state.contains_key(cv) {
        return Err(RuntimeError::UnassignedVariable(cv.to_owned()));
      }
      let mut i = 0;
      while *state.get(cv).unwrap() != 0 {
        i += 1;
        if i > MAX_ITERATIONS {
          println!("CV: {cv}");
          println!("{:?}", state);
          return Err(RuntimeError::MaxLoopsReached);
        }
        run_with_state(config, s, state)?;
      }
    }
  }
  Ok(())
}

pub enum RuntimeError {
  UnassignedVariable(String),
  VariableOverflow(String),
  VariableUnderflow(String),
  // TODO: Detect loops by checking state
  MaxLoopsReached,
}

impl std::fmt::Debug for RuntimeError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::UnassignedVariable(v) => write!(f, "UnassignedVariable {v}"),
      Self::VariableOverflow(v) => write!(f, "VariableOverflow {v}"),
      Self::VariableUnderflow(v) => write!(
        f,
        "VariableUnderflow {v} (you can try running it with 'allow_underflow' enabled)"
      ),
      Self::MaxLoopsReached => write!(f, "MaxLoopsReached"),
    }
  }
}
