use crate::parser::Statement;
use std::collections::HashMap;

const MAX_ITERATIONS: usize = 1024 * 128;

pub fn run(prog: &Statement) -> Result<HashMap<u64, u64>, RuntimeError> {
  let mut state: HashMap<u64, u64> = HashMap::new();
  run_with_state(prog, &mut state)?;
  Ok(state)
}

fn run_with_state(prog: &Statement, state: &mut HashMap<u64, u64>) -> Result<(), RuntimeError> {
  match prog {
    Statement::S(left, right) => {
      run_with_state(left, state)?;
      run_with_state(right, state)?;
    }
    Statement::DeclarePlus(v0, v1, v2) => match (state.get(v1), state.get(v2)) {
      (Some(v1), Some(v2)) => {
        match v1.checked_add(*v2) {
          Some(val) => state.insert(*v0, val),
          None => return Err(RuntimeError::VariableOverflow(*v0)),
        };
      }
      (None, _) => return Err(RuntimeError::UnassignedVariable(*v1)),
      (_, None) => return Err(RuntimeError::UnassignedVariable(*v2)),
    },
    Statement::DeclareMin(v0, v1, v2) => match (state.get(v1), state.get(v2)) {
      (Some(v1), Some(v2)) => {
        match v1.checked_sub(*v2) {
          Some(val) => state.insert(*v0, val),
          None => return Err(RuntimeError::VariableUnderflow(*v0)),
        };
      }
      (None, _) => return Err(RuntimeError::UnassignedVariable(*v1)),
      (_, None) => return Err(RuntimeError::UnassignedVariable(*v2)),
    },
    Statement::DeclareConst(v0, c) => {
      state.insert(*v0, *c);
    }
    Statement::While(cv, s) => {
      if !state.contains_key(cv) {
        return Err(RuntimeError::UnassignedVariable(*cv));
      }
      let mut i = 0;
      while *state.get(cv).unwrap() != 0 {
        i += 1;
        if i > MAX_ITERATIONS {
          println!("CV: {cv}");
          println!("{:?}", state);
          return Err(RuntimeError::MaxLoopsReached);
        }
        run_with_state(s, state)?;
      }
    }
  }
  Ok(())
}

pub enum RuntimeError {
  UnassignedVariable(u64),
  VariableOverflow(u64),
  VariableUnderflow(u64),
  // TODO: Detect loops by checking state
  MaxLoopsReached,
}

impl std::fmt::Debug for RuntimeError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::UnassignedVariable(v) => write!(f, "UnassignedVariable x{v}"),
      Self::VariableOverflow(v) => write!(f, "VariableOverflow x{v}"),
      Self::VariableUnderflow(v) => write!(f, "VariableUnderflow x{v}"),
      Self::MaxLoopsReached => write!(f, "MaxLoopsReached"),
    }
  }
}
