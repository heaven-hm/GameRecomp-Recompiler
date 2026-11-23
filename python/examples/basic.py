#!/usr/bin/env python3
"""Example: inspect and recompile one PE32 game through the GameReComp SDK."""
import argparse
from gamerecomp_sdk import Client

if __name__ == '__main__':
    ap = argparse.ArgumentParser()
    ap.add_argument('--pe', required=True)
    ap.add_argument('--project', required=True,
                    help='project directory (used by recompile / --report-path)')
    ap.add_argument('--cli')
    ap.add_argument('--inspect', action='store_true',
                    help='print the CLI PE inspection JSON and exit')
    ap.add_argument('--report-path', action='store_true',
                    help='print the report.json path instead of the score summary')
    a = ap.parse_args()

    client = Client(a.cli)
    if a.inspect:
        info = client.inspect(a.pe)
        print(f"inspect keys={sorted(info)}")
    elif a.report_path:
        path = client.recompile_to_path(a.pe, a.project)
        print(f"report={path}")
    else:
        report = client.recompile(a.pe, a.project)
        print(f"score={report.get('score')} stages={len(report.get('stages', []))}")
