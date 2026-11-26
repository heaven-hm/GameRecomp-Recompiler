"""Working Python app: recompile and print a structured report card."""
import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parents[3] / "python"))
from gamerecomp_sdk import Client

def main(cli, pe, project):
    client = Client(cli)
    report = client.recompile(pe, project)
    stages = report.get("stages") or []
    print(f"APP_OK sdk=python score={report.get('score')} stages={len(stages)} "
          f"report={client.recompile_to_path(pe, project)}")
    return 0

if __name__ == "__main__":
    if len(sys.argv) != 4:
        print("usage: app.py <cli> <pe> <project>", file=sys.stderr)
        sys.exit(2)
    sys.exit(main(*sys.argv[1:]))
