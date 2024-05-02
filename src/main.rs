use std::time::Instant;

use crate::parser::parse;
use crate::run::run;
use crate::symbolizer::symbolize;
use std::env;

mod parser;
mod run;
mod symbolizer;

// TODO: multiplication / IF f=0 then Q else R end

pub struct Config {
  allow_named_vars: bool,
  strict_underflow: bool,
  allow_constants_in_operations: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let args: Vec<String> = env::args().collect();

  if args.len() != 2 {
    println!("Usage: {} <file>", args[0]);
    return Ok(());
  }
  let path = std::path::Path::new(&args[1]);
  if !path.exists() {
    println!("File '{}' does not exist", path.display());
    return Ok(());
  }
  let code = std::fs::read_to_string(path)?;

  println!("Symbolizing and parsing program...");
  let config = Config {
    allow_named_vars: true,
    strict_underflow: true,
    allow_constants_in_operations: true,
  };
  let res = symbolize(&config, &code);
  let (_, parsed) = match res {
    Ok(k) => match parse(&k, 0) {
      Ok(k) => k,
      Err(e) => {
        println!("Parser error: {e}");
        return Ok(());
      }
    },
    Err(e) => {
      println!("Syntax error: {e:?}");
      return Ok(());
    }
  };
  println!("Done!");
  println!("Running program...");
  let start = Instant::now();

  match run(&config, &parsed) {
    Ok(state) => {
      let elapsed = start.elapsed();
      println!("Success! (time: {:?})", elapsed);
      let max_k = match state.keys().map(|s| s.chars().count()).max() {
        Some(max_k) => max_k,
        None => {
          println!("No variables used.");
          return Ok(());
        }
      };
      let max_chars = max_k + 1;
      let mut keys = state.keys().collect::<Vec<_>>();

      // This complex sorting function is just there so that x2 will be before x12
      keys.sort_by(|a, b| {
        let mut a_number = None;
        let mut b_number = None;
        if a.starts_with("x") {
          let number = a.chars().skip(1).collect::<String>();
          let res = u64::from_str_radix(&number, 10);
          if res.is_ok() {
            a_number = Some(res.unwrap());
          }
        }
        if b.starts_with("x") {
          let number = b.chars().skip(1).collect::<String>();
          let res = u64::from_str_radix(&number, 10);
          if res.is_ok() {
            b_number = Some(res.unwrap());
          }
        }
        match (a_number, b_number) {
          (None, None) => a.partial_cmp(b).unwrap(),
          (None, Some(_)) => std::cmp::Ordering::Less,
          (Some(_), None) => std::cmp::Ordering::Greater,
          (Some(xa), Some(xb)) => xa.partial_cmp(&xb).unwrap(),
        }
      });

      for key in keys {
        let pad = " ".repeat(max_chars - key.len());
        println!("{key}{pad} = {}", state.get(key).unwrap());
      }
    }
    Err(e) => println!("A runtime error occurred: {e:?}"),
  };
  Ok(())
}
