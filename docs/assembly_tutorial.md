# CAIMAN REFERENCE
Author: Meredith Hu

## What is Caiman?
*This guide is for CAIMAN ASSEMBLY and not Original Caiman or Caiman Frontend.
Caiman is a research language that is a project of CU Capra. It is a “specification language” and it aims to reveal the optimality differences between different implementations of the same program. There are many levels of Caiman that one can write in. There is low-level Caiman in which if a user wants to implement anything, they must implement every aspect of the program to a level of detail unsuited for user programming. The developers of Caiman have attempted to mitigate this lack of user-friendliness by making Caiman Assembly, a more high level version of Caiman that is easier to write in but still requires lots of the formal specification of program features that is characteristic to the mission of Caiman as a project. Another project currently being developed is Caiman front end, a version of Caiman that is easier to use still.

Caiman Assembly reveals the differences in different implementations to the user by requiring the user to write a value codeblock and scheduling codeblock for any given program. The schedule codeblock must typecheck to the value code in order for the whole file to compile. The value code serves as a formal description of expressions a program must include to perform what it needs to, but the scheduling language actually implements this with allocations for local variables, function calls, and specification for where everything is placed. 

__Important people:__ Dietrich Geisler, Oliver Daids
__Undergrads:__ Mia, Stephen, Sometimes me (Meredith)

## How do I set up Caiman?
You can obtain caiman by cloning the git repository that Caiman lives at. 

1. [ ] Download Rust/Cargo if you don’t have them
2. [ ] Clone this repo: https://github.com/cucapra/caiman/tree/main 
3. [ ] To check if you have caiman, cd into the caiman directory and type “cargo run” as a command in your terminal, and if it prints an error message, you’re good 
4. [ ] Run cargo build to finish building caiman. 
5. [ ] Then, run the test file with python test.py in that folder, which tests if caiman works with the working test file in the test directory. 
run: https://github.com/cucapra/caiman/blob/main/caiman-test/test.py  

__Optional:__
Download the VSCode extension to add commenting capabilities 

## How do I test Caiman?
__How to create a program:__
1. [ ] Consult with your research advisor to see if you should create your own Caiman branch or if you should checkout to an existing branch to put your files on. For now, new tests can go in the repository `/caiman-test/basics`. 
2. [ ] Create a`.cair` file and a corresponding `.rs` file with the exact same name. Both files have to have `_test` at the end of their names in order to be testable by the python file. Again, make sure the files are somewhere in the `/caiman-test` directory.
3. [ ] Write Caiman Assembly code in the `.cair` file and Rust code in the `.rs` file. Boilerplate has been provided by the explication tool (WIP!!!), but you should familiarize yourself with Caiman 101 <link here> to write a very basic program.
4. [ ] To compile your code, do `cargo run – –input caiman-test\basics\<YOUR-TEST-NAME>_test.cair`.
5. [ ] To test your code, run `python test.py run basics/<YOUR-TEST-NAME>_test.cair`.

__How to debug your file:__
For now, debugging with Rust skills is a very practical option. You may also run the compiler with this flag after “run” to get a lot of information about the entered file:
`--explicate_only`

## Helpful Resources:
[ ] Rust basics help a LOT!! 
[ ] [Basic git branching theory](https://git-scm.com/book/en/v2/Git-Branching-Basic-Branching-and-Merging)

### Caiman References 
[ ] Working Caiman Assembly Examples ()
[ ] [Caiman Reference Guide](https://github.com/cucapra/caiman/blob/main/caiman-spec/src/content.ron)
[ ] [More Caiman Reference](https://github.com/cucapra/caiman/blob/main/caiman-test/reference_untested/example.cair)