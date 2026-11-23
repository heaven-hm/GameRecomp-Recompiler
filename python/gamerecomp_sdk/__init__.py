"""Python client SDK for the GameReComp PE32 x86 static recompiler.

This package does not reimplement lifting; it invokes the real CLI and reads
its machine-readable report.
"""
from __future__ import annotations

import json
import os
import subprocess
from pathlib import Path


class RecompileError(RuntimeError):
    """The CLI failed or produced no usable report."""


class Client:
    def __init__(self, cli: str | os.PathLike | None = None):
        cli = cli or os.environ.get('GAMERECOMP_CLI', 'game-recomp-cli')
        self.cli = str(cli)
        if not Path(self.cli).is_file():
            raise RecompileError(f'CLI binary does not exist: {self.cli}')

    def recompile(self, pe: str | os.PathLike, project: str | os.PathLike) -> dict:
        pe, project = Path(pe), Path(project)
        if not pe.is_file():
            raise RecompileError(f'PE32 input does not exist: {pe}')
        cmd = [self.cli, 'run', str(pe), '--project-dir', str(project),
               '--no-table', '--no-progress']
        proc = subprocess.run(cmd, stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                              text=True, errors='replace', check=False)
        if proc.returncode != 0:
            raise RecompileError(f'CLI exited {proc.returncode}: {proc.stdout[-2000:]}')
        report = project / 'reports' / 'report.json'
        if not report.is_file():
            raise RecompileError('pipeline produced no report.json')
        return json.loads(report.read_text())


__all__ = ['Client', 'RecompileError']
