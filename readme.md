# An interpreter for WHILE-programs

To run, make sure you have rust installed.
The easiest way to run this is to use `cargo`, which can be installed using `rustup`.
Check out this site for more info: [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install).

To run a program, create a text file and then enter `cargo run yourprogram.txt`. The file extension doesn't matter, but i've been using `.while`. 

To view a proper help page, just run it without a file specified, or with the `--help` options. 
That help page will also show you a couple options you can enable to make it less painful to write these programs. You can also enable the options on a per-program basis by putting `#OPTION` at the start of your program, as seen in [example.while](example.while).

