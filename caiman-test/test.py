#!/usr/bin/env python3
import argparse
import subprocess
from pathlib import Path
from itertools import chain
from sys import stderr
from typing import List
from dataclasses import dataclass

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

class Compiler:
    def __init__(self, test_dir):
        manifest_path = test_dir / ".." / "Cargo.toml"
        args = [ "cargo", "build", 
            "--manifest-path", manifest_path, 
            "--features", "build-binary" ]
        rv = subprocess.run(args)
        if rv.returncode != 0:
            eprint(f"{COLOR_WARN} using previous caimanc")
        self.test_dir = test_dir

    def _compiler_path(self) -> Path:
        return self.test_dir / ".." / "target" / "debug" / "caimanc"

    def compile(self, input: Path, output: Path) -> subprocess.CompletedProcess:
        args = [ self._compiler_path(), 
            "--input", input, 
            "--output", output ]
        return subprocess.run(args, capture_output=True, encoding="utf8")

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
def process_inputs(compiler: Compiler, test_dir: Path, quiet: bool) -> ProcessStatistics:
    lf = (test_dir / "src" / "lib.rs").open(mode='w')
    lf.write("pub mod util;\n")
    ps = ProcessStatistics(0,0,0)
    for input in chain(test_dir.rglob("*.cair"), test_dir.rglob("*.ron")):
        relativized = input.relative_to(test_dir)
        output =  test_dir / "src" / (input.stem + ".rs")

        rv = compiler.compile(input, output)
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

def build(test_dir: Path, quiet: bool):
    eprint(f"{COLOR_INFO} building caimanc")
    c = Compiler(test_dir)

    eprint(f"{COLOR_INFO} compiling Caiman source files")
    ps = process_inputs(c, test_dir, quiet)
    if (ps.failures > 0):
        eprint(f"{COLOR_WARN} {ps.failures}/{ps.total()} files failed to compile")

def run(test_dir: Path):
    eprint(f"{COLOR_INFO} running Cargo tests")
    manifest_path = test_dir / "Cargo.toml"
    args = [ "cargo", "test", "--manifest-path", manifest_path, ]
    _ = subprocess.run(args)

def clean(test_dir: Path):
    for log in test_dir.rglob("*.log"):
        log.unlink()
    for dbg in test_dir.rglob("*.debug"):
        dbg.unlink()
    gen_dir = test_dir / "src"
    for f in gen_dir.iterdir():
        if f.name != "util.rs":
            f.unlink()
    lf = (gen_dir / "lib.rs").open(mode='w')
    lf.write("pub mod util;\n")

def main():
    test_dir = Path(__file__).resolve().parent
    parser = argparse.ArgumentParser(description="Caiman Test Suite", fromfile_prefix_chars="@")
    parser.add_argument("command", choices=["run", "build", "clean"])
    parser.add_argument("-q", "--quiet", action="store_true", help="Suppress extra info.")
    args = parser.parse_args()
    if args.command == "run":
        build(test_dir, args.quiet)
        run(test_dir)
    elif args.command == "build":
        build(test_dir, args.quiet)
    else:
        clean(test_dir)
        
if __name__ == "__main__":
    main()