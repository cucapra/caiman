# CAIMAN README
Author: Meredith Hu

## What is Caiman?
Caiman is a research language that is a project of CU Capra. Caiman programs consist of several "specification languages" and an "implementation" of those specification languages. The intention of this divide in responsibility is to make it easier to explore performance tradeoffs while maintaining program semantics.
This repository contains implementations of three levels of abstraction for working with the Caiman compiler:

- The raw IR, which you can write directly as a Rust .ron file
- Caiman Assembly, which gives you exacting control over the generation of the Caiman IR (and is essentially just a .ron file with node names and some small quality of life changes)
- High Level Caiman (HLC), which lets you write Caiman with slightly higher-level semantics. Note that we are in the process of updating names so that HLC is just called Caiman, since this is the intended representation of the Caiman language.

The structure of the Caiman compiler, at the moment, is roughly as follows:

HLC -> Caiman Assembly -> Explication (currently empty) -> Caiman IR -> Rust
but you may also view a detailed visual diagram here: https://github.com/cucapra/caiman/blob/main/docs/compiler_structure.svg

Where each of these arrows can be "bypassed" by writing a lower level of abstraction. Note that typechecking is done on the Caiman IR, though HLC and Caiman Assembly also do some lowering work and may error as a result.

## How do I set up Caiman?
You can obtain caiman by cloning the git repository that Caiman lives at. 

1. [ ] Download Rust/Cargo if you don’t have them
2. [ ] Clone this repo: https://github.com/cucapra/caiman/tree/main 
3. [ ] To check if you have caiman, cd into the caiman directory and type “cargo run” as a command in your terminal, and if it prints an error message, you’re good 
4. [ ] Run cargo build to finish building caiman. 
5. [ ] Then, run the test file with python test.py in that folder, which tests if Caiman works with the working test file in the test directory. 
run: https://github.com/cucapra/caiman/blob/main/caiman-test/test.py  

__Optional:__
Download the VSCode extension to add commenting capabilities. It can be found [here](https://github.com/Checkmate50/caiman-vsc).

## How do I test Caiman?
__How to create a program:__
1. [ ] Create your own Caiman branch to put your files on. For now, new tests can go in the repository `/caiman-test/basics`. 
2. [ ] Create a`.cair` file and a corresponding `.rs` file with the exact same name. Both files have to have `_test` at the end of their names in order to be testable by the python file. Again, make sure the files are somewhere in the `/caiman-test` directory.
3. [ ] Write Caiman Assembly code in the `.cair` file and Rust code in the `.rs` file. You should familiarize yourself with Caiman 101 <link here> to write a basic program.
4. [ ] To compile your code, do `cargo run – –input caiman-test\basics\<YOUR-TEST-NAME>_test.cair`.
5. [ ] To test your code, run `python test.py run basics/<YOUR-TEST-NAME>_test.cair`.

__How to debug your file:__ <br>
For now, debugging with Rust skills is a very practical option. You may also run the compiler with the this flag to see the translation to the intermediate representation, including any explication that was done:
`--explicate_only`

## Helpful Resources:
- [ ] [Rust basics]() 
- [ ] [Cargo basics](https://doc.rust-lang.org/rust-by-example/cargo.html)
- [ ] [Basic git branching theory](https://git-scm.com/book/en/v2/Git-Branching-Basic-Branching-and-Merging)

### Caiman References 
- [ ] Working Caiman Assembly Examples ()
- [ ] [Diagram of Caiman Compiler](https://github.com/cucapra/caiman/blob/main/docs/compiler_structure.svg)
- [ ] [Caiman Reference Guide](https://github.com/cucapra/caiman/blob/main/caiman-spec/src/content.ron)
- [ ] [More Caiman Reference](https://github.com/cucapra/caiman/blob/main/caiman-test/reference_untested/example.cair)