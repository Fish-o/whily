const KEYWORDS: [&str; 3] = ["while", "do", "od"];

pub enum Symbol {
  Variable(u64),
  Constant(u64),
  Keyword(String),
  Declare,
  NotEquals,
  Plus,
  Minus,
  EOS,
}

impl std::fmt::Debug for Symbol {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Variable(v) => write!(f, "x{v}"),
      Self::Constant(c) => write!(f, "{c}"),
      Self::Keyword(k) => write!(f, "{k}"),
      Self::Declare => write!(f, ":="),
      Self::NotEquals => write!(f, "!="),
      Self::Plus => write!(f, "+"),
      Self::Minus => write!(f, "-"),
      Self::EOS => write!(f, ";"),
    }
  }
}
pub fn symbolize(input: &str) -> Result<Vec<Symbol>, SymbolError> {
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
      Some('+') => Some(Symbol::Plus),
      Some('-') => Some(Symbol::Minus),
      Some(';') => Some(Symbol::EOS),
      Some('x') => Some(Symbol::Variable(0)),
      Some('\n') => {
        col = 0;
        line += 1;
        continue;
      }

      Some(c) if c.is_whitespace() => continue,
      _ => None,
    };
    let is_var = matches!(symbol, Some(Symbol::Variable(0)));
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
        symbol = Some(Symbol::Variable(val));
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
        let mut c_clone = c.clone();
        while cur_keyword_char.is_some()
          && c_clone.is_some()
          && cur_keyword_char.unwrap() == c_clone.unwrap()
        {
          c_clone = chars_clone.next();
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
        return Err(SymbolError::new(line, col, "Unknown keyword"));
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
