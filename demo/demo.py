import sys
import subprocess

def main():
    if len(sys.argv) < 2 or sys.argv[1] not in {"build", "buildc", "run"}:
        print("Must build and provide a file name or run")
        return
    
    if sys.argv[1] == "build":
        assert len(sys.argv) > 1, "must provide path"
        result = subprocess.run(["..\\target\\debug\\hlc.exe", sys.argv[2]], capture_output=True, encoding="utf8")
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

    elif sys.argv[1] == "buildc":
        assert len(sys.argv) > 1, "must provide path"
        result = subprocess.run(["..\\target\\debug\\caimanc.exe", 
                                    "--input", sys.argv[2], 
                                    "--output", "src/caiman_out.rs"], 
                                 capture_output=True, encoding="utf8")
        if len(result.stderr) > 0:
            print(result.stderr)
        else:
            print("compilation succeeded")
    else:
        subprocess.run(["cargo", "run"])

if __name__ == "__main__":
    main()