use std::time::Instant;

use crate::parser::parse;
use crate::run::run;
use crate::symbolizer::symbolize;
use std::env;

mod parser;
mod run;
mod symbolizer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let args: Vec<String> = env::args().collect();
  let wd = std::env::current_dir()?;
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
  let res = symbolize(&code);
  let (index, parsed) = match res {
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
  match run(&parsed) {
    Ok(state) => {
      let elapsed = start.elapsed();
      println!("Success! (time: {:?})", elapsed);
      let max_k = match state.keys().max() {
        Some(max_k) => max_k,
        None => {
          println!("No variables used.");
          return Ok(());
        }
      };
      let max_chars = (*max_k as f64).log10().ceil() as usize + 1;
      let mut keys = state.keys().collect::<Vec<_>>();
      keys.sort();
      for key in keys {
        let key_str = format!("{key}");
        let pad = " ".repeat(max_chars - key_str.len());
        println!("x{key_str}{pad} = {}", state.get(key).unwrap());
      }
    }
    Err(e) => println!("A runtime error occurred: {e:?}"),
  };
  Ok(())
}
