import sys
import re
import copy


def check_scope_str(s: str) -> bool:
    stack = []
    for c in s:
        if c == "{" or c == "[" or c == "(":
            stack.append(c)

        if c == "}" or c == "]" or c == ")":
            r = stack.pop()
            if (
                (c == "}" and r != "{")
                or (c == "]" and r != "[")
                or (c == ")" and r != "(")
            ):
                return False
    return len(stack) == 0


if len(sys.argv) != 2 and len(sys.argv) != 3:
    print("Usage: fcheck <src_file> <ir_file>")
    print("       fcheck <src_file>")
    exit(1)


src_file = sys.argv[1]
f_out = open(sys.argv[2]) if len(sys.argv) == 3 else sys.stdin


with open(src_file) as f_src:
    s_in = f_src.read()
    s_out = f_out.read()
    last_src_pos = -1

    for match in re.finditer(
        re.compile(r"#CHECK(-([a-zA-Z_-]+))?:\s*((.|\s)*?)#END", re.MULTILINE),
        s_in,
    ):
        txt = match.group(3).rstrip()
        opt = match.group(2)
        orig = copy.deepcopy(txt)
        txt = re.sub(re.compile(r"\+"), r"\\+", txt)
        txt = re.sub(re.compile(r"\("), r"\\(", txt)
        txt = re.sub(re.compile(r"\)"), r"\\)", txt)
        txt = re.sub(re.compile(r"\*"), r"\\*", txt)
        txt = re.sub(re.compile(r"\["), r"\\[", txt)
        txt = re.sub(re.compile(r"\]"), r"\\]", txt)
        txt = re.sub(re.compile(r"\""), r"\"", txt)
        txt = re.sub(re.compile(r"\s+"), r"(\\s+|\\b)", txt)
        txt = re.sub(re.compile(r"\.\*"), r"(\\s|.)*?", txt)
        dots = 0
        while "..." in txt:
            txt = txt.replace("...", f"(?P<scope{dots}>(\\s|.)*?)", 1)
            dots += 1

        success = opt == "NOT"
        for out_match in re.finditer(re.compile(txt), s_out):
            if out_match.start() >= last_src_pos:
                cont = False
                for i in range(0, dots):
                    group = out_match.group(f"scope{i}")
                    if not check_scope_str(group):
                        cont = True
                if cont:
                    continue
                if opt == "NOT":
                    success = False
                else:
                    last_src_pos = out_match.end()
                    success = True
                break

        if not success:
            st = "Found" if opt == "NOT" else "Failed to find"
            print(f"{st}:\n {orig}", file=sys.stderr)
            # print(f"\n\nWith regex:\n{txt}")
            exit(1)
