import argparse
import json
import subprocess
import tempfile
import sys

# Commands to run for compiling the rust project.
COMMANDS = [
    'RUSTFLAGS="-C strip=none" cargo build -p blend-contract-sdk',
    'RUSTFLAGS="-C strip=none --emit=llvm-ir" cargo rustc --manifest-path=emitter/Cargo.toml --crate-type=cdylib --target=wasm32-unknown-unknown --release --features certora',
    'RUSTFLAGS="-C strip=none --emit=llvm-ir" cargo rustc --manifest-path=pool-factory/Cargo.toml --crate-type=cdylib --target=wasm32-unknown-unknown --release --features certora',
    'RUSTFLAGS="-C strip=none --emit=llvm-ir" cargo rustc --manifest-path=backstop/Cargo.toml --crate-type=cdylib --target=wasm32-unknown-unknown --release --features certora',
    'RUSTFLAGS="-C strip=none --emit=llvm-ir" cargo rustc --manifest-path=pool/Cargo.toml --crate-type=cdylib --target=wasm32-unknown-unknown --release --features certora',
]

def run_command(command):
    """Runs `cargo build` commands and dumps output to a temporary file."""
    with tempfile.NamedTemporaryFile(delete=False, mode='w', suffix='.log') as tmp_file:
        try:
            # Compile rust project and redirect stdout and stderr to a temp file
            result = subprocess.run(
                command,
                shell=True,
                stdout=tmp_file,
                stderr=subprocess.STDOUT,
                text=True
            )
            return tmp_file.name, result.returncode
        except Exception as e:
            print(f"Error running command '{command}': {e}")
            return None, -1

def write_output(output_data, output_file=None):
    """Writes the JSON output either to a file or dumps it to the console."""
    if output_file:
        with open(output_file, 'w') as f:
            json.dump(output_data, f, indent=4)
        print(f"Output written to {output_file}")
    else:
        print(json.dumps(output_data, indent=4))

def main():
    parser = argparse.ArgumentParser(description="Compile rust projects and generate JSON output to be used by Certora Prover.")
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument("-o", "--output", metavar="FILE", help="Path to output JSON to a file.")
    group.add_argument("-j", "--json", action="store_true", help="Dump JSON output to the console.")
    group.add_argument("-m", "--mutate", action="store_true", help="Return 0 if cargo build succeeds, 1 if it fails.")
    
    args = parser.parse_args()
    
    # Compile rust project and dump the logs to tmp files
    # Also saving some information for each comamnd in `results`
    log_files = []
    results = []
    for command in COMMANDS:
        log_file, return_code = run_command(command)
        print(f"Temporary log file located at: {log_file}")
        log_files.append(log_file)
        results.append({"command": command, "return_code": return_code, "log_file": log_file})

    # Determine overall success or failure of the rust project build process
    all_succeeded = all(res["return_code"] == 0 for res in results)
    
    # JSON template
    output_data = {
        "project_directory":".",
        "success": all_succeeded,
        "sources": ["pool/src/pool/*.rs", "backstop/src/*.rs", "Cargo.toml"],
        "executables": "target/wasm32-unknown-unknown/release/pool.wasm"
    }
    
    # Handle output based on the provided argument
    if args.output:
        write_output(output_data, args.output)
    elif args.json:
        write_output(output_data)
    # Needed for mutations: if you run _this_ script inside another script, you can check this returncode and decide what to do
    elif args.mutate:
        sys.exit(0 if all_succeeded else 1)

if __name__ == "__main__":
    main()