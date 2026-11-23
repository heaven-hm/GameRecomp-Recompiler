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

    def _run_cli(self, cmd: list[str]) -> str:
        """Run *cmd*, returning stdout or raising RecompileError on failure."""
        proc = subprocess.run(cmd, stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                              text=True, errors='replace', check=False)
        if proc.returncode != 0:
            raise RecompileError(f'CLI exited {proc.returncode}: {proc.stdout[-2000:]}')
        return proc.stdout

    def inspect(self, pe: str | os.PathLike) -> dict:
        """Inspect *pe* with ``<cli> inspect <pe> --json`` and return its JSON."""
        pe = Path(pe)
        if not pe.is_file():
            raise RecompileError(f'PE32 input does not exist: {pe}')
        cmd = [self.cli, 'inspect', str(pe), '--json']
        stdout = self._run_cli(cmd)
        try:
            return json.loads(stdout)
        except json.JSONDecodeError as exc:
            raise RecompileError(f'CLI produced invalid JSON: {stdout[-2000:]}') from exc

    def recompile_to_path(self, pe: str | os.PathLike,
                          project: str | os.PathLike) -> Path:
        """Recompile *pe* and return the path to the produced ``report.json``.

        This runs the same pipeline as :meth:`recompile` but returns the report
        path instead of parsing the report body.
        """
        pe, project = Path(pe), Path(project)
        if not pe.is_file():
            raise RecompileError(f'PE32 input does not exist: {pe}')
        cmd = [self.cli, 'run', str(pe), '--project-dir', str(project),
               '--no-table', '--no-progress']
        self._run_cli(cmd)
        report = project / 'reports' / 'report.json'
        if not report.is_file():
            raise RecompileError('pipeline produced no report.json')
        return report

    def recompile(self, pe: str | os.PathLike, project: str | os.PathLike) -> dict:
        """Recompile *pe* and return the parsed ``reports/report.json`` dict."""
        report = self.recompile_to_path(pe, project)
        return json.loads(report.read_text())


__all__ = ['Client', 'RecompileError']
