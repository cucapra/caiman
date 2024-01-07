## High-Level Caiman

This is the project for the Caiman frontend. It is currently maintained as a separate
binary for now, but will be merged in the future. It's separate for now since
the cli for high-level caiman uses a newer version of clap which results in a
lot cleaner code, so when it is merged I would like to rewrite the old CLI.

Caiman frontend programs are currently defined in `.cm` files. Right now
a whole program must reside within a single `.cm` file. See 
`caiman-test/high-level-caiman/simple-lower` for examples.

To compile a program with the new frontend, simply pass the file as
an argument to the `hlc` binary. The compiler will return the generated
Rust code on `stdout`. Pass `--help` to see more options available such as
only running a particular stage of the compiler.
