#!/usr/bin/env python3
import argparse
import subprocess
from pathlib import Path
from itertools import chain
from sys import stderr
from dataclasses import dataclass
from shutil import rmtree

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
    return pad + s.replace('\n', f'\n{pad}')

class FrontendCompiler:
    def __init__(self, test_dir):
        manifest_path = test_dir / ".." / "caiman-frontend" / "Cargo.toml"
        args = [ "cargo", "build"] 
        rv = subprocess.run(args)
        if rv.returncode != 0:
            eprint(f"{COLOR_WARN} using previous caiman-frontend")
        self.test_dir = test_dir

    def _compiler_path(self) -> Path:
        return self.test_dir / ".." / "target" / "debug" / "caiman-frontend"

    def compile(self, input: Path, output: Path) -> subprocess.CompletedProcess:
        args = [ self._compiler_path(), 
            "--run",
            "--output", output,
            input ] 
        return subprocess.run(args, capture_output=True, encoding="utf8", cwd=input.parent)

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

# returns num failed, num succeeded
def process_inputs(
        #compiler: Compiler, 
    compiler: FrontendCompiler, 
    test_dir: Path,
    inputs,
    quiet: bool
) -> ProcessStatistics:
    lf = (test_dir / "src" / "lib.rs").open(mode='w')
    lf.write("pub mod util;\n")
    ps = ProcessStatistics(0,0,0)
    if not inputs:
        inputs = chain(test_dir.rglob("*test.cair"), test_dir.rglob("*test.ron"))
    for input in inputs:
        relativized = input.absolute().relative_to(test_dir)
        output =  test_dir / "src" / (input.stem + ".rs")

        rv = compiler.compile(input.absolute(), output)
        if (rv.returncode != 0):
            eprint(f"    {Colorizer.red('fail:')} {relativized}")
            if not quiet:
                msg = pad_lines(rv.stderr, f"        {Colorizer.red('|')} ")
                print(msg, file=stderr)
            ps.failures += 1
            continue

        eprint(Colorizer.grey(f"    pass: {relativized}"))
        lf.write(f"mod {input.stem};\n")
        ps.compiled += 1

        test_file = input.with_suffix(".rs")
        if not test_file.exists():
            if not quiet:
                eprint(Colorizer.grey("        | no Rust test file provided"))
        else:
            input_rs = Path("..") /  relativized.with_suffix(".rs")
            of = output.open(mode='a', encoding="utf8")
            of.write(f"\ninclude!(r##\"{input_rs}\"##);\n")
            of.close()
            ps.linked += 1
            
    lf.close()
    return ps

def build(test_dir: Path, inputs, quiet: bool):
    eprint(f"{COLOR_INFO} building caimanc")
    c = FrontendCompiler(test_dir)

    eprint(f"{COLOR_INFO} compiling Caiman source files")
    ps = process_inputs(c, test_dir, inputs, quiet)
    if (ps.failures > 0):
        eprint(f"{COLOR_WARN} {ps.failures}/{ps.total()} files failed to compile")

def run(test_dir: Path, inputs):
    eprint(f"{COLOR_INFO} running Cargo tests")
    manifest_path = test_dir / "Cargo.toml"
    args = ["cargo", "test", "--manifest-path", manifest_path, "--"]
    args += [Path(input).stem for input in inputs]
    _ = subprocess.run(args)

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
    lf = (gen_dir / "lib.rs").open(mode='w')
    lf.write("pub mod util;\n")

def main():
    test_dir = Path(__file__).resolve().parent
    parser = argparse.ArgumentParser(description="Caiman Test Suite", fromfile_prefix_chars="@")
    parser.add_argument("command", choices=["run", "build", "clean"])
    parser.add_argument("-q", "--quiet", action="store_true", help="Suppress extra info.")
    # Just going to hard-code this in here for now
    files = ["high_level_frontend/foo"]
    args = parser.parse_args()
    inputs = [Path(file) for file in files]
    if args.command == "run":
        build(test_dir, inputs, args.quiet)
        run(test_dir, files)
    elif args.command == "build":
        build(test_dir, inputs, args.quiet)
    elif args.command == "clean":
        clean(test_dir)
    else:
        eprint("Unknown subcommand. Accepted: run, build, clean")
        
if __name__ == "__main__":
    main()