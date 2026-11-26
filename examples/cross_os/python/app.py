"""Port one Windows PE32 app to a declared host through the in-process engine."""
from __future__ import annotations
import json, sys
from pathlib import Path
ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "python"))
from gamerecomp_sdk import BuildOptions, Engine  # noqa: E402

def main(direction_id: str, pe: str, project: str) -> int:
    matrix = json.loads((Path(__file__).parents[1] / "matrix.json").read_text())
    row = next(d for d in matrix["directions"] if d["id"] == direction_id)
    if "dotnet" in direction_id:
        print(f"PORT_DEFERRED direction={direction_id} reason=.NET CLR lift path is absent")
        return 0
    if "win16-ne" in direction_id:
        print(f"PORT_DEFERRED direction={direction_id} reason=Win16 NE orchestration is absent")
        return 0
    options = BuildOptions(input_os=row["input_os"], input_arch=row["input_arch"],
                           target_os=row["target_os"], target_arch=row["target_arch"],
                           out_mode="full", skip_stages="verify")
    try:
        report = Engine().build(pe, project, options)
    except Exception as error:
        print(f"PORT_DEFERRED direction={direction_id} reason={error}")
        return 0
    print(f"PORT_OK direction={direction_id} score={report.get('score')} stages={len(report.get('stages') or [])}")
    return 0

if __name__ == "__main__":
    if len(sys.argv) != 4:
        print("usage: app.py <direction> <pe> <project>", file=sys.stderr); sys.exit(2)
    sys.exit(main(*sys.argv[1:]))
