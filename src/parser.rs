use crate::{
  config::Config,
  symbolizer::{Operator, Symbol},
};

#[derive(Debug)]
pub enum Statement {
  S(Box<Statement>, Box<Statement>),
  DeclareOperation(String, Value, Operator, Value),
  DeclareConst(String, Value),
  While(String, Box<Statement>),
}

pub enum Value {
  Variable(String),
  Constant(u64),
}

pub fn parse(
  config: &Config,
  symbols: &Vec<Symbol>,
  mut index: usize,
) -> Result<(usize, Statement), String> {
  let mut left: Option<Statement> = None;

  let mut statement: Option<Statement> = None;
  index = index.wrapping_sub(1);
  loop {
    (left, statement) = match (left, statement) {
      (Some(l), Some(s)) => (None, Some(Statement::S(Box::new(l), Box::new(s)))),
      v => v,
    };
    index = index.wrapping_add(1);
    let first = symbols.get(index);
    if matches!(first, Some(Symbol::EOS)) {
      if statement.is_none() {
        return Err("Did not find left side of ; symbol.".to_owned());
      }
      left = statement;
      statement = None;
      continue;
    }

    match first {
      None => {
        if statement.is_none() {
          return Err("Unexpected end of program, expected a statement first.".to_owned());
        }
        return Ok((index - 1, statement.unwrap()));
      }

      Some(Symbol::Keyword(kw)) if kw == "od" => {
        if statement.is_none() {
          return Err("Unexpected 'od', expected a statement first.".to_owned());
        }
        return Ok((index - 1, statement.unwrap()));
      }
      // 1  2  3  4  5
      // xi := xj + xk
      // xi := xj − xk
      // xi := c
      Some(Symbol::Variable(v0)) => {
        if statement.is_some() {
          return Err(format!("Found two statements in a row, the second starting with '{v0}', did you miss a ; symbol?"));
        }
        index += 1;
        let second = symbols.get(index);
        if !matches!(second, Some(Symbol::Declare)) {
          return Err(format!(
            "Invalid second symbol '{:?}' after variable '{v0}'",
            second
          ));
        }

        index += 1;
        let left = match symbols.get(index) {
          Some(Symbol::Variable(v1)) => Value::Variable(v1.to_owned()),
          Some(Symbol::Constant(c)) => Value::Constant(*c),
          Some(s) => {
            return Err(format!(
              "Unexpected symbol '{s:?}' in  '{v0} := {s:?}', expected either a variable or constant."
            ))
          }
          None => {
            return Err(format!(
              "Unexpected end of program after '{v0} :='."
            ))
          }
        };

        index += 1;
        match symbols.get(index) {
          None | Some(Symbol::EOS) => {
            index -= 1;
            if config.allow_constants_everywhere || matches!(left, Value::Constant(_)) {
              statement = Some(Statement::DeclareConst(v0.to_owned(), left));
            } else {
              return Err(format!(
                "Assigning variables to other variables is not allowed without 'allow_constants_everywhere' enabled."
              ));
            }
          }
          Some(Symbol::Operator(operator)) => {
            if !matches!(operator, Operator::Subtract | Operator::Add) && !config.extra_operators {
              return Err(format!(
                "Using operators other than + or - is not allowed without 'extra_operations' enabled."
              ));
            }
            index += 1;
            let right = symbols.get(index);
            let right = match right {
              Some(Symbol::Variable(v2)) => Value::Variable(v2.to_owned()),
              Some(Symbol::Constant(c)) => Value::Constant(*c),
              Some(s) => {
                return Err(format!(
                  "Invalid symbol '{s:?}' in '{v0} := {s:?}'. Expected variable or constant."
                ))
              }
              None => return Err(format!("Unexpected end of program after '{v0} :='.")),
            };
            match (&left, &right) {
              (Value::Constant(_), _) | (_, Value::Constant(_)) => {
                if !config.allow_constants_everywhere {
                  return Err(format!(
                  "Using constants in + or - operations is not allowed without 'allow_constants_everywhere' enabled."
                ));
                }
              }
              _ => {}
            }
            statement = Some(Statement::DeclareOperation(
              v0.to_owned(),
              left,
              operator.clone(),
              right,
            ))
          }
          Some(s) => {
            return Err(format!(
              "Invalid symbol '{s:?}' in '{v0} := {left:?} {s:?}'. Expected an operator."
            ))
          }
        }
      }

      // }
      // while xi != 0 do P1 od
      Some(Symbol::Keyword(kw)) if kw == "while" => {
        if statement.is_some() {
          return Err("Found two statements in a row, the second starting with 'while'. Did you miss a ; symbol?".to_owned());
        }
        index += 1;
        let cv = match symbols.get(index) {
          Some(Symbol::Variable(cv)) => cv,
          Some(s) => {
            return Err(format!(
              "Invalid symbol '{s:?}' in 'while {s:?}'. 'while' must be followed by 'xi != 0'."
            ))
          }
          None => return Err(format!("Unexpected end of program after 'while'.")),
        };
        // !=
        index += 1;
        match symbols.get(index) {
          Some(Symbol::NotEquals) => {}
          Some(s) => {
            return Err(format!(
              "Invalid symbol '{s:?}' in 'while {cv} {s:?}'. Only != is allowed."
            ))
          }
          None => return Err(format!("Unexpected end of program after 'while x{cv}'.")),
        };
        // 0
        index += 1;
        match symbols.get(index) {
          Some(Symbol::Constant(0)) => {}
          Some(s) => {
            return Err(format!(
              "Invalid symbol '{s:?}' in 'while {cv} != {s:?}'. Only 0 is allowed."
            ))
          }
          None => return Err(format!("Unexpected end of program after 'while {cv} !='.")),
        };
        // do
        index += 1;
        match symbols.get(index) {
          Some(Symbol::Keyword(kw)) if kw == "do" => {}
          Some(s) => {
            return Err(format!(
              "Invalid symbol '{s:?}' in 'while {cv} != 0 {s:?}'. Only 'do' is allowed."
            ))
          }
          None => {
            return Err(format!(
              "Unexpected end of program after 'while {cv} != 0'."
            ))
          }
        };
        // P1
        let p1 = match parse(config, symbols, index + 1) {
          Err(e) => return Err(e),
          Ok(p1) => p1,
        };
        index = p1.0;
        // od
        index += 1;
        match symbols.get(index) {
          Some(Symbol::Keyword(kw)) if kw == "od" => {}
          Some(s) => {
            return Err(format!(
              "Invalid symbol '{s:?}' in 'while {cv} != 0 do .. {s:?}'. Only 'od' is allowed."
            ))
          }
          None => {
            return Err(format!(
              "Unexpected end of program after 'while {cv} != 0 do .. '. Expected 'od' instead."
            ))
          }
        };
        statement = Some(Statement::While(cv.to_owned(), Box::new(p1.1)));
      }
      // ;
      _ => {
        return Err(format!("Invalid start of statement: {:?}", first));
      }
    }
  }
}

impl std::fmt::Debug for Value {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Variable(arg0) => write!(f, "{}", arg0),
      Self::Constant(arg0) => write!(f, "{}", arg0),
    }
  }
}
