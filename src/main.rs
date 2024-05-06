use std::io::Read;
use std::time::Instant;

use crate::config::cli;
use crate::parser::parse;
use crate::run::run;
use crate::symbolizer::symbolize;
use clio::*;
use config::Config;

mod config;
mod parser;
mod run;
mod symbolizer;

// TODO: multiplication / IF f=0 then Q else R end

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

  let mut config = Config::from(&args);

  // Parsing the code

  println!("Symbolizing and parsing program...");
  let res = symbolize(&mut config, &code);
  let (_, parsed) = match res {
    Ok(k) => match parse(&config, &k, 0) {
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
      let max_chars = match state.keys().map(|s| s.chars().count()).max() {
        Some(max_k) => max_k,
        None => {
          println!("No variables used.");
          return;
        }
      };
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
