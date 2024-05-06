use crate::Config;

const KEYWORDS: [&str; 3] = ["while", "do", "od"];

pub enum Symbol {
  Variable(String),
  Constant(u64),
  Keyword(String),
  Operator(Operator),
  Declare,
  NotEquals,
  EOS,
}

#[derive(Debug, Clone)]
pub enum Operator {
  Subtract,
  Add,
  Multiply,
}
impl std::fmt::Debug for Symbol {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Variable(v) => write!(f, "{v}"),
      Self::Constant(c) => write!(f, "{c}"),
      Self::Keyword(k) => write!(f, "{k}"),
      Self::Declare => write!(f, ":="),
      Self::NotEquals => write!(f, "!="),
      Self::Operator(Operator::Add) => write!(f, "+"),
      Self::Operator(Operator::Subtract) => write!(f, "-"),
      Self::Operator(Operator::Multiply) => write!(f, "*"),
      Self::EOS => write!(f, ";"),
    }
  }
}

pub fn symbolize(config: &mut Config, input: &str) -> Result<Vec<Symbol>, SymbolError> {
  let mut chars = input.chars().peekable().into_iter();
  let mut line = 1;
  let mut col = 0;
  let mut skip = 0;
  let mut symbols = vec![];

  let mut in_comment = false;
  loop {
    let mut c = chars.next();
    col += 1;
    if c.is_none() {
      break;
    }
    if in_comment {
      match c.unwrap() {
        ']' => in_comment = false,
        '\n' => {
          col = 0;
          line += 1;
          continue;
        }
        _ => {}
      }
      continue;
    }
    let mut symbol = match &c {
      Some('#') => {
        let mut flag = String::new();
        while matches!(&chars.peek(), Some('a'..='z' | 'A'..='Z' | '_' | '-')) {
          c = chars.next();
          flag.push(c.unwrap());
          col += 1;
        }
        let res = config.enable(&flag);
        if res.is_err() {
          return Err(SymbolError::new(
            line,
            col,
            &format!("Invalid configuration flag: #{}. Run whily with --help to see the different possible options.",flag)
          ));
        }
        continue;
      }
      Some('[') => {
        in_comment = true;
        continue;
      }
      Some(':') if matches!(&chars.peek(), Some('=')) => {
        skip = 1;
        Some(Symbol::Declare)
      }
      Some('!') if matches!(&chars.peek(), Some('=')) => {
        skip = 1;
        Some(Symbol::NotEquals)
      }
      Some('+') => Some(Symbol::Operator(Operator::Add)),
      Some('-') => Some(Symbol::Operator(Operator::Subtract)),
      Some('*') => Some(Symbol::Operator(Operator::Multiply)),
      Some(';') => Some(Symbol::EOS),
      Some('x') => Some(Symbol::Variable("".to_owned())),
      Some('\n') => {
        col = 0;
        line += 1;
        continue;
      }

      Some(c) if c.is_whitespace() => continue,
      _ => None,
    };
    let is_var = matches!(symbol, Some(Symbol::Variable(_)));
    if !is_var && symbol.is_some() {
      for _ in 0..skip {
        chars.next();
        col += 1;
      }
      skip = 0;
    } else if is_var || matches!(&c, Some('0'..='9')) {
      // 0..=9
      if is_var {
        assert!(matches!(&chars.peek(), Some('0'..='9')));
        c = chars.next();
        col += 1;
      }
      let mut val = 0u64;
      loop {
        val *= 10;
        val += (c.unwrap() as u8).wrapping_sub('0' as u8) as u64;
        if matches!(&chars.peek(), Some('0'..='9')) {
          c = chars.next();
          col += 1;
        } else {
          break;
        }
      }
      if is_var {
        symbol = Some(Symbol::Variable(format!("x{val}")));
      } else {
        symbol = Some(Symbol::Constant(val));
      }
    } else {
      // KEYWORDS
      for k in KEYWORDS {
        let mut keyword_chars = k.chars().into_iter();
        let mut cur_keyword_char = keyword_chars.next();
        let mut chars_clone = chars.clone();
        let mut col_clone = col.clone();
        let mut c_clone = c.as_ref();

        if !(cur_keyword_char.is_some()
          && c_clone.is_some()
          && &cur_keyword_char.unwrap() == c_clone.unwrap())
        {
          continue;
        }

        c_clone = chars_clone.peek();
        col_clone += 1;
        cur_keyword_char = keyword_chars.next();

        while cur_keyword_char.is_some()
          && c_clone.is_some()
          && &cur_keyword_char.unwrap() == c_clone.unwrap()
        {
          chars_clone.next();
          c_clone = chars_clone.peek();
          col_clone += 1;
          cur_keyword_char = keyword_chars.next();
        }
        if cur_keyword_char.is_none() {
          col = col_clone;
          chars = chars_clone;
          symbol = Some(Symbol::Keyword(k.to_string()));
          break;
        }
      }
      if symbol.is_none() {
        if !config.allow_named_vars {
          return Err(SymbolError::new(
            line,
            col,
            "Unknown keyword. Are you using named variables without 'allow_named_vars' enabled?",
          ));
        }

        // make it a variable
        match c {
          Some('A'..='Z') | Some('a'..='z') | Some('0'..='9') | Some('_') => {}
          _ => {
            return Err(SymbolError::new(
              line,
              col,
              "Unknown keyword or invalid variable name",
            ))
          }
        }
        let mut variable_name = format!("{}", c.unwrap());
        loop {
          match chars.peek() {
            None => break,
            Some('A'..='Z') | Some('a'..='z') | Some('0'..='9') | Some('_') => {}
            _ => break,
          }
          c = chars.next();
          variable_name = format!("{variable_name}{}", c.unwrap());
        }
        symbol = Some(Symbol::Variable(variable_name));
      }
    }
    assert!(symbol.is_some());
    symbols.push(symbol.unwrap());
  }

  Ok(symbols)
}

pub struct SymbolError {
  msg: String,
  line: usize,
  pos: usize,
}
impl SymbolError {
  fn new(line: usize, pos: usize, msg: &str) -> Self {
    Self {
      line,
      pos,
      msg: msg.to_string(),
    }
  }
}

impl std::fmt::Debug for SymbolError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "Syntax error on line {}:{}: {}",
      self.line, self.pos, self.msg
    )
  }
}
