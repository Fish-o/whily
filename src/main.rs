use std::time::Instant;

use crate::parser::parse;
use crate::run::run;
use crate::symbolizer::symbolize;
use clap::{arg, Command};
use clio::*;
use std::io::Read;

mod parser;
mod run;
mod symbolizer;

fn cli() -> Command {
  Command::new("git")
    .about("A simple interpreter for WHILE-programs")
    .arg_required_else_help(true)
    .arg(
      arg!(<FILE> "The file path of the program to run").value_parser(clap::value_parser!(Input)),
    )
    .args([
      arg!(--allow_named_vars "Enabled named variables"),
      arg!(--allow_underflow "Allows subtraction to underflow, setting the result to max(0,res)"),
    ])
}

// TODO: multiplication / IF f=0 then Q else R end

pub struct Config {
  allow_named_vars: bool,
  allow_underflow: bool,
  // allow_constants_in_operations: bool,
}

fn main() {
  // Getting command line args and setting the config

  let mut args = cli().get_matches();
  let mut path = args.remove_one::<Input>("FILE").expect("No file path");
  let mut code = String::new();
  let read_results = path.read_to_string(&mut code);
  match read_results {
    Err(e) => {
      panic!("Error occurred while reading file:\n{}", e)
    }
    _ => {}
  }

  let config = Config {
    allow_named_vars: *args.get_one("allow_named_vars").expect("Missing arg 1"),
    allow_underflow: *args.get_one("allow_underflow").expect("Missing arg 2"),
  };

  // Parsing the code

  println!("Symbolizing and parsing program...");
  let res = symbolize(&config, &code);
  let (_, parsed) = match res {
    Ok(k) => match parse(&k, 0) {
      Ok(k) => k,
      Err(e) => {
        eprintln!("\nA parser error ocurred.\n{e}");
        return;
      }
    },
    Err(e) => {
      eprintln!("\nAn error ocurred.\n{e:?}");
      return;
    }
  };
  println!("Done!");

  // Running the code

  println!("\nRunning program...");
  let start = Instant::now();

  match run(&config, &parsed) {
    Ok(state) => {
      let elapsed = start.elapsed();
      println!("Success! (time: {:?})\n\nFinished state:", elapsed);
      let max_k = match state.keys().map(|s| s.chars().count()).max() {
        Some(max_k) => max_k,
        None => {
          println!("No variables used.");
          return;
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
}
