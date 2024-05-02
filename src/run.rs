use crate::{parser::Statement, Config};
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
    Statement::DeclarePlus(v0, v1, v2) => match (state.get(v1), state.get(v2)) {
      (Some(v1), Some(v2)) => {
        match v1.checked_add(v2.to_owned()) {
          Some(val) => state.insert(v0.to_owned(), val),
          None => return Err(RuntimeError::VariableOverflow(v0.to_owned())),
        };
      }
      (None, _) => return Err(RuntimeError::UnassignedVariable(v1.to_owned())),
      (_, None) => return Err(RuntimeError::UnassignedVariable(v2.to_owned())),
    },
    Statement::DeclareMin(v0, v1, v2) => match (state.get(v1), state.get(v2)) {
      (Some(v1), Some(v2)) => {
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
      (None, _) => return Err(RuntimeError::UnassignedVariable(v1.to_owned())),
      (_, None) => return Err(RuntimeError::UnassignedVariable(v2.to_owned())),
    },
    Statement::DeclareConst(v0, c) => {
      state.insert(v0.to_owned(), *c);
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
        "VariableUnderflow {v} (try running with '--allow_underflow'?)"
      ),
      Self::MaxLoopsReached => write!(f, "MaxLoopsReached"),
    }
  }
}
