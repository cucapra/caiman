import sys
import subprocess

def main():
    if len(sys.argv) != 2 or sys.argv[1] not in {"build", "run"}:
        print("Must build or run")
        return
    
    if sys.argv[1] == "build":
        result = subprocess.run(["..\\target\\debug\\hlc.exe", "select_sum.cm"], capture_output=True, encoding="utf8")
        if len(result.stderr) > 0:
            # with open('results.txt', 'w') as ofile:
            #     ofile.write(result.stdout)
            #     ofile.write('\n')
            #     ofile.write(result.stderr)
            print(result.stderr)
        else:
            to_write = result.stdout
            to_write = to_write[to_write.find("//"):]
            print("compilation succeeded")
            with open("src/caiman_out.rs", 'w') as ofile:
                ofile.write(to_write)

    else:
        subprocess.run(["cargo", "run"])

if __name__ == "__main__":
    main()