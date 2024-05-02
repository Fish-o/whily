# An interpreter for WHILE-programs

To run, make sure you have rust installed.
The easiest way to run this is to use `cargo`, which can be installed using `rustup`.
Check out this site for more info: [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install).

To run a program, create a text file and then enter `cargo run yourprogram.txt`. The file extension doesn't matter, but i've been using `.while`. 


You can enable named variables (instead of just x0, x1, x2, etc.) by adding the argument `--allow_named_vars`, so your command will look like: `cargo run --allow_named_vars coolprogram.while`. You can also combine this with `--allow_underflow` to have the program set any negative result of a subtraction to 0, instead of stopping the program.