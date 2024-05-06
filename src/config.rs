use clap::{arg, ArgMatches, Command};
use clio::Input;

pub fn cli() -> Command {
  Command::new("git")
    .about("A simple interpreter for WHILE-programs")
    .arg_required_else_help(true)
    .arg(
      arg!(<FILE> "The file path of the program to run").value_parser(clap::value_parser!(Input)),
    )
    .args([
      arg!(--allow_named_vars "Enabled named variables"),
      arg!(--allow_underflow "Allows subtraction to underflow, setting the result to max(0,res)"),
      arg!(--allow_constants_everywhere "Allows the use of constants everywhere a variable is used for its value (and vice versa)"),
      arg!(--extra_operators "Enabled extra operations, right now that is just * for multiplication"),
    ])
}
pub struct Config {
  pub allow_named_vars: bool,
  pub allow_underflow: bool,
  pub allow_constants_everywhere: bool,
  pub extra_operators: bool,
}

impl Config {
  pub fn from(args: &ArgMatches) -> Self {
    Self {
      allow_named_vars: *args
        .get_one("allow_named_vars")
        .expect("Missing arg allow_named_vars"),
      allow_underflow: *args
        .get_one("allow_underflow")
        .expect("Missing arg allow_underflow"),
      allow_constants_everywhere: *args
        .get_one("allow_constants_everywhere")
        .expect("Missing arg allow_underflow"),
      extra_operators: *args
        .get_one("extra_operators")
        .expect("Missing arg allow_underflow"),
    }
  }
  pub fn enable(&mut self, arg: &str) -> Result<(), ()> {
    match arg {
      "allow_named_vars" => self.allow_named_vars = true,
      "allow_underflow" => self.allow_named_vars = true,
      "allow_constants_everywhere" => self.allow_constants_everywhere = true,
      "extra_operators" => self.extra_operators = true,
      _ => return Err(()),
    }
    Ok(())
  }
}
