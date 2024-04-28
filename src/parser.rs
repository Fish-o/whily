use crate::symbolizer::Symbol;

#[derive(Debug)]
pub enum Statement {
  S(Box<Statement>, Box<Statement>),
  DeclarePlus(u64, u64, u64),
  DeclareMin(u64, u64, u64),
  DeclareConst(u64, u64),
  While(u64, Box<Statement>),
}

pub fn parse(symbols: &Vec<Symbol>, mut index: usize) -> Result<(usize, Statement), String> {
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
      // xi := xj + xk
      // xi := xj − xk
      // xi := c
      Some(Symbol::Variable(v0)) => {
        if statement.is_some() {
          return Err(format!("Found two statements in a row, the second starting with 'x{v0}', did you miss a ; symbol?"));
        }
        index += 1;
        let second = symbols.get(index);
        if !matches!(second, Some(Symbol::Declare)) {
          return Err(format!(
            "Invalid second symbol '{:?}' after variable 'x{v0}'",
            second
          ));
        }
        index += 1;
        let third = symbols.get(index);
        match third {
          Some(Symbol::Constant(c)) => statement = Some(Statement::DeclareConst(*v0, *c)),
          Some(Symbol::Variable(v1)) => {
            index += 1;
            let operation = symbols.get(index);
            index += 1;
            let right = symbols.get(index);

            let operation = match operation {
              Some(Symbol::Plus) => Symbol::Plus,
              Some(Symbol::Minus) => Symbol::Minus,
              Some(s) => {
                return Err(format!(
                  "Invalid symbol '{s:?}' in 'x{v0} := x{v1} {s:?}'. Only + or - allowed."
                ))
              }
              None => return Err(format!("Unexpected end of program after 'x{v0} := x{v1}'")),
            };

            let v2 = match right {
            Some(Symbol::Variable(v2)) => v2,
            Some(s)  => {
              return Err(format!(
                "Invalid symbol '{s:?}' in  'x{v0} := x{v1} {operation:?} {s:?}'. Only another variable is allowed."
              ))
            }
            None => return Err(format!("Unexpected end of program after 'x{v0} := x{v1} {operation:?}'.")),
          };

            match operation {
              Symbol::Plus => statement = Some(Statement::DeclarePlus(*v0, *v1, *v2)),
              Symbol::Minus => statement = Some(Statement::DeclareMin(*v0, *v1, *v2)),
              _ => unreachable!(),
            }
          }
          Some(t) => {
            return Err(format!(
              "Invalid second symbol '{t:?}' after variable 'x{v0}'"
            ))
          }
          None => return Err(format!("Unexpected end of program after variable 'x{v0}'")),
        }
      }
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
              "Invalid symbol '{s:?}' in 'while x{cv} {s:?}'. Only != is allowed."
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
              "Invalid symbol '{s:?}' in 'while x{cv} != {s:?}'. Only 0 is allowed."
            ))
          }
          None => return Err(format!("Unexpected end of program after 'while x{cv} !='.")),
        };
        // do
        index += 1;
        match symbols.get(index) {
          Some(Symbol::Keyword(kw)) if kw == "do" => {}
          Some(s) => {
            return Err(format!(
              "Invalid symbol '{s:?}' in 'while x{cv} != 0 {s:?}'. Only 'do' is allowed."
            ))
          }
          None => {
            return Err(format!(
              "Unexpected end of program after 'while x{cv} != 0'."
            ))
          }
        };
        // P1
        let p1 = match parse(symbols, index + 1) {
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
              "Invalid symbol '{s:?}' in 'while x{cv} != 0 do .. {s:?}'. Only 'od' is allowed."
            ))
          }
          None => {
            return Err(format!(
              "Unexpected end of program after 'while x{cv} != 0 do .. '. Expected 'od' instead."
            ))
          }
        };
        statement = Some(Statement::While(*cv, Box::new(p1.1)));
      }
      // ;
      _ => {
        return Err(format!("Invalid start of statement: {:?}", first));
      }
    }
  }

  unreachable!()
}
