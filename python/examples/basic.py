#!/usr/bin/env python3
"""Example: recompile one PE32 game through the GameReComp SDK."""
import argparse
from gamerecomp_sdk import Client

if __name__ == '__main__':
    ap = argparse.ArgumentParser()
    ap.add_argument('--pe', required=True)
    ap.add_argument('--project', required=True)
    ap.add_argument('--cli')
    a = ap.parse_args()
    report = Client(a.cli).recompile(a.pe, a.project)
    print(f"score={report.get('score')} stages={len(report.get('stages', []))}")
