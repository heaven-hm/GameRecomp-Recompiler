"""app.py - Python example for the in-process GameRecomp SDK engine.

Usage: app.py <pe32-x86.exe> <project-dir>

No CLI binary and no subprocess: the recompiler source is vendored in this
repository and runs inside this process through ``Engine``.
"""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[3] / "python"))

from gamerecomp_sdk import BuildOptions, Engine  # noqa: E402


def build_options() -> BuildOptions:
    """Explicit guest/host declaration used by this demo."""
    return BuildOptions(
        input_os="windows",
        input_arch="x86",
        target_os="host",
        target_arch="host",
        out_mode="full",
        skip_stages="verify",  # no original-Windows oracle in this demo
    )


def main(pe_path: str, project_dir: str) -> int:
    report = Engine().build(pe_path, project_dir, build_options())
    stages = report.get("stages") or []
    report_path = Path(project_dir).resolve() / "reports" / "report.json"

    print(
        f"APP_OK sdk=python"
        f" score={report.get('score')}"
        f" stages={len(stages)}"
        f" report={report_path}"
    )
    return 0


if __name__ == "__main__":
    if len(sys.argv) != 3:
        print("usage: app.py <pe> <project>", file=sys.stderr)
        sys.exit(2)
    sys.exit(main(*sys.argv[1:]))
