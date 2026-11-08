# SPDX-License-Identifier: MIT OR Apache-2.0
import json
import os
from pathlib import Path
import subprocess
import tempfile


ROOT = Path(__file__).resolve().parents[1]
EXPORTS = ROOT / "src/api/runtime/exports.rs"
FIXTURE_SOURCE = ROOT / "tests/fixtures/napi/addon.c"


def run(command):
    subprocess.run(command, cwd=ROOT, check=True)


def main():
    compiler = os.environ.get("CC", "cc")
    cargo = os.environ.get("CARGO", "cargo")
    dlltool = os.environ.get("DLLTOOL", "dlltool")
    artifact_output = subprocess.run(
        [cargo, "test", "--test", "napi_loader", "--no-run", "--message-format=json"],
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    )
    executable = next(
        Path(message["executable"])
        for line in artifact_output.stdout.splitlines()
        if (message := json.loads(line)).get("reason") == "compiler-artifact"
        and message.get("target", {}).get("name") == "napi_loader"
        and message.get("executable")
    )

    symbols = [
        line.strip().split(" as *const (),", 1)[0]
        for line in EXPORTS.read_text().splitlines()
        if " as *const ()," in line
    ]
    with tempfile.TemporaryDirectory() as temp_dir:
        temp_dir = Path(temp_dir)
        definition = temp_dir / "napi_host.def"
        import_library = temp_dir / "libnapi_host.a"
        definition.write_text(
            f"LIBRARY {executable.name}\nEXPORTS\n"
            + "".join(f"  {symbol}\n" for symbol in symbols)
        )
        run(
            [
                dlltool,
                "--dllname",
                executable.name,
                "--def",
                str(definition),
                "--output-lib",
                str(import_library),
            ]
        )
        run(
            [
                compiler,
                "-O2",
                "-shared",
                "-pthread",
                "-o",
                str(ROOT / "tests/fixtures/napi/addon.node"),
                str(FIXTURE_SOURCE),
                str(import_library),
            ]
        )


if __name__ == "__main__":
    main()
