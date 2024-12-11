import argparse
import json
import subprocess
import tempfile
import sys
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent

# Command to run for compiling the rust project.
COMMAND = "just build"

# JSON FIELDS
PROJECT_DIR = (SCRIPT_DIR / "../").resolve()
SOURCES = ["pool/src/pool/*.rs", "backstop/src/*.rs", "Cargo.toml"]
EXECUTABLES = "target/wasm32-unknown-unknown/release/pool.wasm"

def run_command(command, to_stdout=False):
    """Runs the build command and dumps output to temporary files."""
    print(f"Running {command}")
    try:
        if to_stdout:
            result = subprocess.run(
                command,
                shell=True,
                text=True
            )
            return None, None, result.returncode
        else:
            with tempfile.NamedTemporaryFile(delete=False, mode='w', suffix='.stdout.log') as stdout_file, tempfile.NamedTemporaryFile(delete=False, mode='w', suffix='.stderr.log') as stderr_file:
                # Compile rust project and redirect stdout and stderr to a temp file
                result = subprocess.run(
                    command,
                    shell=True,
                    stdout=stdout_file,
                    stderr=stderr_file,
                    text=True
                )
                return stdout_file.name, stderr_file.name, result.returncode
    except Exception as e:
        print(f"Error running command '{command}': {e}")
        return None, None -1

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
    parser.add_argument("-o", "--output", metavar="FILE", help="Path to output JSON to a file.")
    parser.add_argument("-j", "--json", action="store_true", help="Dump JSON output to the console.")
    parser.add_argument("-l", "--log", action="store_true", help="Show log outputs from cargo build on standard out.")

    args = parser.parse_args()
    to_stdout = args.log
    
    # Compile rust project and dump the logs to tmp files
    stdout_log, stderr_log, return_code = run_command(COMMAND, to_stdout)
    
    print(f"Temporary log file located at: {stdout_log} and {stderr_log}")
    
    # JSON template
    output_data = {
        "project_directory": str(PROJECT_DIR),
        "sources": SOURCES,
        "executables": EXECUTABLES,
        "success": return_code,
        "stdout_log": stdout_log,
        "stderr_log": stderr_log
    }
    
    # Handle output based on the provided argument
    if args.output:
        write_output(output_data, args.output)
    
    if args.json:
        write_output(output_data)
        
    # Needed for mutations: if you run _this_ script inside another script, you can check this returncode and decide what to do
    sys.exit(0 if return_code else 1)

if __name__ == "__main__":
    main()