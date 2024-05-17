#!/usr/bin/env python3
import argparse
import subprocess
from pathlib import Path
from itertools import chain
from sys import stderr
from dataclasses import dataclass
from shutil import rmtree
import os
import time
import csv


# stupid hack to build a test file for each pair of results
def rust_diff(file1: Path, file2: Path):
    return f"""
#[test]
fn compare() -> Result<(), String> {{
    use file_diff::{{diff_files}};
    use std::fs::{{File}};

    let mut file1 = match File::open(r##"{file1}"##) {{
        Ok(f) => f,
        Err(e) => panic!("{{}}", e),
    }};
    let mut file2 = match File::open(r##"{file2}"##) {{
        Ok(f) => f,
        Err(e) => panic!("{{}}", e),
    }};

    crate::expect_returned!(true, Some(diff_files(&mut file1, &mut file2)));
}}
    """

def help():
    print("Help function has not been implemented yet.")

def eprint(*args, **kwargs):
    print(*args, file=stderr, **kwargs)


class Colorizer:
    def cyan(s: str) -> str:
        return f"\033[36m{s}\033[39m"

    def yellow(s: str) -> str:
        return f"\033[93m{s}\033[39m"

    def red(s: str) -> str:
        return f"\033[31m{s}\033[39m"

    def grey(s: str) -> str:
        return f"\033[90m{s}\033[39m"


COLOR_INFO = Colorizer.cyan("[info]")
COLOR_WARN = Colorizer.yellow("[warn]")
COLOR_FAIL = Colorizer.red("[fail]")

"""Pads the start of each line in `s` with `pad`."""


def pad_lines(s: str, pad: str) -> str:
    return pad + s.replace("\n", f"\n{pad}")


class Compiler:
    def __init__(self, test_dir):
        manifest_path = test_dir / ".." / "Cargo.toml"
        args = [
            "cargo",
            "build",
            "--manifest-path",
            manifest_path,
            "--features",
            "build-binary",
        ]
        rv = subprocess.run(args)
        if rv.returncode != 0:
            eprint(f"{COLOR_WARN} using previous caimanc")
        self.test_dir = test_dir

    def _compiler_path(self) -> Path:
        return self.test_dir / ".." / "target" / "debug" / "caimanc"

    def compile(
        self, input: Path, output: Path, explicate_only: bool = False
    ) -> subprocess.CompletedProcess:
        args = [self._compiler_path(), "--input", input, "--output", output] + [
            "--explicate_only"
        ] * explicate_only
        return subprocess.run(
            args, capture_output=True, encoding="utf8", cwd=input.parent
        )


class HighLevelCaiman:
    def __init__(self, test_dir):
        args = ["cargo", "build", "--all"]
        rv = subprocess.run(args)
        if rv.returncode != 0:
            eprint(f"{COLOR_WARN} using previous high-level-caiman")
        self.test_dir = test_dir

    def _compiler_path(self) -> Path:
        return self.test_dir / ".." / "target" / "debug" / "hlc"

    def compile(self, input: Path, output: Path) -> subprocess.CompletedProcess:
        args = [self._compiler_path(), "-o", output, input]
        return subprocess.run(
            args, capture_output=True, encoding="utf8", cwd=input.parent
        )


@dataclass
class ProcessStatistics:
    """Successfully compiled inputs which are associated with a test Rust file."""

    linked: int
    """Successfully compiled inputs."""
    compiled: int
    """Inputs which caimanc failed to compile."""
    failures: int

    """Total number of files processed."""

    def total(self) -> int:
        return self.compiled + self.failures


def compiler_error(
    rv: subprocess.CompletedProcess,
    relativized: Path,
    quiet: bool,
    ps: ProcessStatistics,
) -> bool:
    if rv.returncode != 0:
        eprint(f"    {Colorizer.red('fail:')} {relativized}")
        if not quiet:
            msg = pad_lines(rv.stderr, f"        {Colorizer.red('|')} ")
            print(msg, file=stderr)
        ps.failures += 1
        return True  # return if there was an error
    return False


# returns num failed, num succeeded
def process_inputs(
    compiler: Compiler,
    hlc: HighLevelCaiman,
    test_dir: Path,
    inputs,
    quiet: bool,
) -> ProcessStatistics:
    lf = (test_dir / "src" / "lib.rs").open(mode="w")
    lf.write("pub mod util;\n")
    ps = ProcessStatistics(0, 0, 0)
    if not inputs:
        inputs = chain(
            test_dir.rglob("*test.cair"),
            test_dir.rglob("*test.ron"),
            test_dir.rglob("*test.caiman"),
            test_dir.rglob("*test.cm"),
        )
    for input in inputs:
        if f"{input}".find("turnt") != -1:
            continue
        relativized = input.absolute().relative_to(test_dir)
        output = test_dir / "src" / (input.stem + ".rs")

        input_str = str(input)  # cause I wanna do direct string manipulations
        if input_str.endswith("test.cair"):
            baseline_name = (
                input.name[: input.name.find("_")] + "_baseline.cair"
            )
            baseline = input.parent / baseline_name
            if baseline.is_file():
                # we compile here for explication only
                test_out = output.with_suffix(".txt")
                rv = compiler.compile(input.absolute(), test_out, True)
                if compiler_error(rv, relativized, quiet, ps):
                    continue

                baseline_out = Path(
                    str(output).replace("test.rs", "baseline.txt")
                )  # to explication thing
                rv = compiler.compile(baseline.absolute(), baseline_out, True)
                if compiler_error(rv, relativized, quiet, ps):
                    continue

                eprint(Colorizer.grey(f"    pass: {relativized}"))
                lf.write(f"mod {input.stem};\n")
                ps.compiled += 1

                of = output.open(mode="w", encoding="utf8")
                of.write(rust_diff(test_out, baseline_out))
                of.close()

                ps.linked += 1

                continue

        input_compiler = hlc if input_str.endswith(".cm") else compiler
        rv = input_compiler.compile(input.absolute(), output)

        if rv.returncode == 0:
            eprint(Colorizer.grey(f"    pass: {relativized}"))
            if not quiet and rv.stderr:
                msg = pad_lines(rv.stderr, f"        {Colorizer.grey('|')} ")
                print(msg, file=stderr)
        else:
            eprint(f"    {Colorizer.red('fail:')} {relativized}")
            if not quiet:
                msg = pad_lines(rv.stderr, f"        {Colorizer.red('|')} ")
                print(msg, file=stderr)
            ps.failures += 1
            continue

        lf.write(f"mod {input.stem};\n")
        ps.compiled += 1

        test_file = input.with_suffix(".rs")
        if not test_file.exists():
            if not quiet:
                eprint(Colorizer.grey("        | no Rust test file provided"))
        else:
            input_rs = Path("..") / relativized.with_suffix(".rs")
            of = output.open(mode="a", encoding="utf8")
            of.write(f'\ninclude!(r##"{input_rs}"##);\n')
            of.close()
            ps.linked += 1

    lf.close()
    return ps


#function for building caiman and compiling it
def build(test_dir: Path, inputs, quiet: bool):
    eprint(f"{COLOR_INFO} building caimanc")
    c = Compiler(test_dir)
    print("ALKSDJFADKS")

    eprint(f"{COLOR_INFO} building high-level-caiman")
    hlc = HighLevelCaiman(test_dir)

    eprint(f"{COLOR_INFO} compiling Caiman source files")
    ps = process_inputs(c, hlc, test_dir, inputs, quiet)
    if ps.failures > 0:
        eprint(
            f"{COLOR_WARN} {ps.failures}/{ps.total()} files failed to compile"
        )
    return ps.failures


#function that compiles a .cair source file
def compile(test_dir: Path, file):
    #prints colored messages, fancy 
    eprint(f"{COLOR_INFO} compiling Caiman file with Cargo run")

    #change directory to /caiman main directory.
    os.chdir("../")

    #change the filename to have "./caiman-test/ in front of it"
    f = "./caiman-test/basics/" + file

    #arguments for running
    args = ["cargo", "run", "--", "--input"]
    args += [f]

    #time this part. start time:
    start = time.time()

    #get the output 
    r2 = subprocess.Popen(args, stdout=subprocess.PIPE)
    #end time
    end = time.time()

    #read output into a variable 
    output = r2.stdout.read()

    #runtime to return
    t = end - start

    #if running was not a success, raise an exception 
    if r2.returncode is not None:
        raise Exception(f"compiling caiman file {file} failed! failed with error code{r2.returncode}") 
    #else, return the runtime and output.
    return [t, output]  


#function that runs a file 
def run(test_dir: Path, inputs):
    #prints colored messages, fancy 
    eprint(f"{COLOR_INFO} running Caiman file with cargo test")

    #gets the cargo.toml
    manifest_path = test_dir / "Cargo.toml"

    #arguments for running 
    args = ["cargo", "test", "--manifest-path", manifest_path, "--"]
    args += [Path(input).stem for input in inputs]

    #time this part. start time:
    start = time.time()

    #process to generate stuff with cargo test:
    r = subprocess.run(args).returncode

    #end time
    end = time.time()
    #runtime to return
    t = end - start

    #if running was not a success, raise an exception 
    if r != 0:
        raise Exception("running caiman file {filename} failed!".format(filename = inputs[i])) 
    #else, return the runtime 
    return t


def clean(test_dir: Path):
    for log in test_dir.rglob("*.log"):
        log.unlink()
    for dbg in test_dir.rglob("*.debug"):
        dbg.unlink()
    gen_dir = test_dir / "src"
    for f in gen_dir.iterdir():
        if f.is_dir():
            rmtree(f, ignore_errors=True)
        elif f.name != "util.rs":
            f.unlink()
    lf = (gen_dir / "lib.rs").open(mode="w")
    lf.write("pub mod util;\n")

def write_to_csv(filename, num_iters, command){
    #generate the csv name you will be writing to
    csv_name = filename.split(.)[0] + command + ".csv"

    #find the file in the dummy directory. if file does not exist, create the file. 

}

def main():
    #makes the test directory, in order to get access to the cargo.toml
    test_dir = Path(__file__).resolve().parent

    #some options that apply to the entire parser 
    parser = argparse.ArgumentParser(
        prog="Timing script",
        description="Times a single file's compilation and runtime",
        fromfile_prefix_chars="@"
    )

    #positional argument 1: command type 
    parser.add_argument(
        "command", 
        choices=["compile", "run"],
        help="Choose your command: compile or run"
        )

    #might not include this one?  
    parser.add_argument(
        "-q", "--quiet", 
        action="store_true", 
        help="Suppress extra info."
    )

    #positional argument 2: filename
    parser.add_argument(
        "file", 
        help="The file to compile or run. Directories are not accepted and will cause the script to break."
    )

    '''
    #positional argument 3: number of iterations
    parser.add_argument(
        "NUM_ITERS",
        type=int, 
        help="number of iterations"
        )
    '''

    #parse arguments 
    args = parser.parse_args()

    #print("LAKSDJFLASDKFJASDLKFJKLA")

    #if the file specified is a directory then we walk through it
    #if it's just a file we add it to inputs 
    inputs = []
    '''
    for file in args.file:
        if os.path.isdir(file):
            for path, _, filenames in os.walk(file):
                for filename in filenames:
                    fpath = Path(filename)
                    if fpath.suffix != ".rs" and fpath.stem.endswith("_test"):
                        inputs.append(Path(path) / fpath)
        else:
            '''
    #gets the filename from args
    filename = args.file.split("/")[-1]
    #print(filename)
    inputs.append(filename)

    #control flow for running and compiling 

    #compile command.
    if args.command == "compile":
        #store the compile time in a compile-time variable. 
        compile_info = compile(test_dir, filename)

        #if success, print to console and store compile time.
        print("Successfully compiled file {f}".format(f=filename))
        compile_time = compile_info[0]
        print("Compile time was {c}".format(c=compile_time))

    #run
    elif args.command == "run":
        #assume Caiman is already built. First, compile the file. 
        compile(test_dir, filename)

        #next, run it. Runtime is stored in this variable.
        runtime = run(test_dir, filename)

        print("Successfully ran file {f}".format(f=filename))
        print("Runtime was {r}".format(r=runtime))
        
    #unknown 
    else:
        eprint("Unknown subcommand. Accepted: run, compile")


if __name__ == "__main__":
    main()
