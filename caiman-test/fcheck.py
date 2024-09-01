"""
This is similar to LLVM's FileCheck.

In a source file, place patterns between #CHECK and #END (multiline allowed).
The text between these directives will be searched for in the IR file. The search
keys must be found in the ir file in sequential order to be successful.

Except for the following notes, this pretty much matches what you'd expect from
FileCheck for the parts of FileCheck that this script supports.

Notes:
    * spaces are important, but any amount of whitespace can match 1 or more whitespace
         in the test file. Don't put a space/newline if a
         space may not be there in the test file.
    * `...` is the same as the regex (.|\n)*?, except that the match must contain
        balanced braces
    * you cannot use a variable in the same #CHECK #END section as the one in which
        you defined it
    * globals can be defined on the command line with the -d flag (Ex: `-d name=val`)
        and can be used in your search key with the syntax `${name}`.

Supported Features:
    * CHECK-NOT (partial, checks for no match after the last match only)
    * CHECK-LABEL
    * Regexes
    * Substitution blocks
"""

import sys
import re
from typing import TextIO


def check_scope_str(s: str) -> None | str:
    """
    Ensures that `s` is a string where all openining braces are correctly closed.
    Returns an error string when `s` is invalid, otherwise `None` when `s` is valid.
    """
    stack = []
    for c in s:
        if c == "{" or c == "[" or c == "(":
            stack.append(c)

        if c == "}" or c == "]" or c == ")":
            try:
                r = stack.pop()
            except:
                return (
                    f"Encountered closing brace: {c} without a matching opening"
                )
            if (
                (c == "}" and r != "{")
                or (c == "]" and r != "[")
                or (c == ")" and r != "(")
            ):
                return f"Mismatch closing brace {c} for {r}"
    if len(stack) > 0:
        return f"Missing closing braces for {stack}"
    return None


# Global constants
vars = {}


def get_src_ir_file() -> tuple[str, TextIO]:
    """
    Gets the name of the src file and the IO for the ir file to check.
    Handles command line arguments and processes global constants in the
    command line arguments specified with `-d name=val`
    """
    # Is the next cmd line arg a definition
    is_def = False
    # List of arguments without global constants or other options
    new_args = []
    for cmd in sys.argv:
        if is_def:
            is_def = False
            if not "=" in cmd:
                print("An argument definition must follow -d", file=sys.stderr)
            var = cmd.split("=")
            vars[var[0]] = var[1]
        elif cmd == "-d":
            is_def = True
        else:
            new_args.append(cmd)

    if not (len(new_args) == 2 or len(new_args) == 3):
        print("Usage: fcheck <src_file> <ir_file>")
        print("       fcheck <src_file>")
        exit(1)

    src_file = new_args[1]
    f_out = open(new_args[2]) if len(new_args) == 3 else sys.stdin
    return src_file, f_out


src_file, f_ir = get_src_ir_file()


def in_any_group(start_pos: int, end_pos: int, groups: list[dict]) -> bool:
    """
    Returns `True` if `start_pos` and `end_pos` overlap with any group in `groups`
    where a group is defined by a dict with a `start` and `end` key-value pair
    """
    for group in groups:
        if start_pos >= group["start"] and (
            "end" not in group or end_pos < group["end"]
        ):
            return True
    return False


with open(src_file) as f_src:
    s_in = f_src.read()
    s_out = f_ir.read()
    groups: list[dict] = []
    last_src_pos = -1
    groups.append({"start": 0})
    names = {}

    for match in re.finditer(
        re.compile(r"#CHECK(-([a-zA-Z_-]+))?:\s*((.|\s)*?)#END", re.MULTILINE),
        s_in,
    ):
        txt = match.group(3).rstrip()  # the text between the #CHECK and #END
        opt = match.group(2)  # any option applies to the check command
        # the list of parts of the text between the #CHECK and #END
        # made up of literal strings to match and regexes that are surrounded with `{{}}`
        parts: list[str] = []
        # the last index of the last part in the list
        last_part_idx = 0
        # any names defined with the syntax [[name:regex]]
        defined_names = []
        for regex_or_name in re.finditer(
            re.compile(
                r"{{(.*)}}|\[\[([a-zA-Z][a-zA-Z0-9_]*:)?(.*)\]\]|\${(.+)}"
            ),
            txt,
        ):
            parts.append(txt[last_part_idx : regex_or_name.start()])
            last_part_idx = regex_or_name.end()
            if regex_or_name.group(0).startswith("{"):
                # we're a regex
                parts.append(regex_or_name.group(0))
            elif regex_or_name.group(0).startswith("${"):
                # we're a global constant
                var_name = regex_or_name.group(4)
                assert var_name in vars, f"Undefined global constant {var_name}"
                parts.append(vars[var_name])
            else:
                # we must be a variable use or decl
                def_name = regex_or_name.group(2)
                if def_name is not None and len(def_name) > 0:
                    # turn a var definition into a named group
                    parts.append(
                        r"{{"
                        + f"(?P<{def_name[:-1]}>({regex_or_name.group(3)}))"
                        + r"}}"
                    )
                    defined_names.append(def_name[:-1])
                else:
                    # we're a use, replace with the literal string and append to parts
                    name = regex_or_name.group(3)
                    if not name in names:
                        print(
                            f"Undefined use of {name} Defined names: {names}",
                            file=sys.stderr,
                        )
                        exit(2)
                    parts.append(names.get(name))
        # append leftovers
        parts.append(txt[last_part_idx:])
        dots = 0
        for i in range(0, len(parts)):
            part = parts[i]
            if part.startswith("{{"):
                # the part is a regex, trim the starting and ending brackets
                assert part.endswith("}}")
                part = part[2:-2]
            else:
                # the part is literal text, sanitize it
                part = re.sub(re.compile(r"\+"), r"\\+", part)
                part = re.sub(re.compile(r"\("), r"\\(", part)
                part = re.sub(re.compile(r"\)"), r"\\)", part)
                part = re.sub(re.compile(r"\*"), r"\\*", part)
                part = re.sub(re.compile(r"\["), r"\\[", part)
                part = re.sub(re.compile(r"\]"), r"\\]", part)
                part = re.sub(re.compile(r"\""), r"\"", part)
                part = re.sub(re.compile(r"\s+"), r"\\s+", part)
                # convert ... into a named group so we can check the brackets
                while "..." in part:
                    part = part.replace(
                        "...", f"(?P<_scope{dots}>(\\n|.)*?)", 1
                    )
                    dots += 1
            parts[i] = part

        regex = ""
        for part in parts:
            regex += part

        # regex uses lookahead so it won't consume input and allow us to go through
        # overlapping regexes
        regex = f"(?=({regex}))"
        success = opt == "NOT"
        if opt == "LABEL":
            groups[-1]["end"] = last_src_pos
            last_src_pos = -1
        for out_match in re.finditer(re.compile(regex), s_out):
            # Allowing overlapping matches means that a match technically
            # has 0 length. Thus we do the end calculation manually
            match_end = out_match.start() + len(out_match.group(1))
            if out_match.start() >= last_src_pos and not in_any_group(
                out_match.start(), match_end, groups[:-1]
            ):
                for name in defined_names:
                    # for any defined names in this pattern, add them to the
                    # variable environment
                    names[name] = out_match.group(name)
                cont = False
                for i in range(0, dots):
                    group = out_match.group(f"_scope{i}")
                    err = check_scope_str(group)
                    if err is not None:
                        # print(
                        #     f"{err} in match\n{group}",
                        #     file=sys.stderr,
                        # )
                        cont = True
                if cont:
                    continue
                if opt == "NOT":
                    # print(out_match.start(), file=sys.stderr)
                    success = False
                else:
                    last_src_pos = match_end
                    success = True
                    if opt == "LABEL":
                        groups.append({"start": out_match.start()})
                break

        if not success:
            st = "Found" if opt == "NOT" else "Failed to find"
            print(f"{st}:\n {txt}", file=sys.stderr)
            print(f"\n\nWith regex:\n{regex}", file=sys.stderr)
            exit(1)
