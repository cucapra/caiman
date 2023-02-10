import os
import subprocess

def main():
    for filename in os.listdir("."):
        end_string = "_baseline.cair"
        n = len(end_string)
        if filename.endswith(end_string):
            base = filename[:len(filename)-n]
            command = f"cargo run --manifest-path ../../Cargo.toml \
                --features=build-binary -- --input {filename}\
                    --output {base}.out --explicate_only"
            command_list = command.split()
            result = subprocess.run(
                command_list,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
            )
            with open(f"{base}.log", 'w') as outfile:
                outfile.write(result.stdout.decode())
                outfile.write(result.stderr.decode())
            subprocess.run(["turnt", f"{base}.cair"]) # so stupid

if __name__ == "__main__":
    main()