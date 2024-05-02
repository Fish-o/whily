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
  allow_underflow: bool,
  // allow_constants_in_operations: bool,
}
impl Config {
  pub fn from(args: &Vec<String>) -> Result<Config, String> {
    let mut config = Config {
      allow_named_vars: false,
      allow_underflow: false,
    };
    for arg in args {
      match arg.chars().skip(2).collect::<String>().as_str() {
        "allow_named_vars" => config.allow_named_vars = true,
        "allow_underflow" => config.allow_underflow = true,
        _ => return Err(format!("Unknown config variable: {arg}")),
      }
    }
    Ok(config)
  }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let args: Vec<String> = env::args().collect();
  let config_args = args
    .iter()
    .filter(|s| s.starts_with("--"))
    .cloned()
    .collect::<Vec<_>>();
  let file_args = args
    .iter()
    .filter(|s| !s.starts_with("--"))
    .cloned()
    .collect::<Vec<_>>();

  let config = Config::from(&config_args)?;

  if file_args.len() != 2 {
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
