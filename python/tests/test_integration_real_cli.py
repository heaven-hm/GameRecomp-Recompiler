"""Opt-in integration test against a **real** GameReComp CLI.

This module is skipped by default. It only runs when the caller explicitly
opts in, because it invokes the real native recompilation pipeline (slow) and
needs a real CLI binary plus a real PE32 input:

    GAMERECOMP_RUN_REAL_INTEGRATION=1 \
    GAMERECOMP_CLI=/path/to/game-recomp-cli \
    GAMERECOMP_TEST_PE=/path/to/game.exe \
    python3 -m pytest python/tests/test_integration_real_cli.py -q -rs

Optional: ``GAMERECOMP_TEST_PROJECT`` (project dir, else a temp dir) and
``GAMERECOMP_TEST_TIMEOUT`` (seconds). None of these values are hard-coded in
source, and no real CLI/game asset is needed to build or unit-test the SDK.
"""
from __future__ import annotations

import os
import sys
import tempfile
import unittest
from pathlib import Path

_REPO_ROOT = Path(__file__).resolve().parents[2]
_PY_SRC = str(_REPO_ROOT / 'python')
if _PY_SRC not in sys.path:
    sys.path.insert(0, _PY_SRC)

from gamerecomp_sdk import Client, RecompileError  # noqa: E402

OPT_IN_VAR = 'GAMERECOMP_RUN_REAL_INTEGRATION'
CLI_VAR = 'GAMERECOMP_CLI'
PE_VAR = 'GAMERECOMP_TEST_PE'
PROJECT_VAR = 'GAMERECOMP_TEST_PROJECT'
TIMEOUT_VAR = 'GAMERECOMP_TEST_TIMEOUT'

_TRUTHY = {'1', 'true', 'yes', 'on'}
_OPTED_IN = os.environ.get(OPT_IN_VAR, '').strip().lower() in _TRUTHY
_SKIP_REASON = (f'set {OPT_IN_VAR}=1 to run real-CLI integration tests '
                f'(also requires {CLI_VAR} and {PE_VAR})')


def _validate_report(report: object) -> dict:
    """Reference shape check for ``reports/report.json`` (kept local so this
    module stays self-contained). Raises ``AssertionError`` on bad reports."""
    assert isinstance(report, dict), 'report must be a JSON object (dict)'
    assert 'score' in report, "report is missing required field 'score'"
    score = report['score']
    assert isinstance(score, int) and not isinstance(score, bool), \
        "'score' must be an int (not bool)"
    assert 0 <= score <= 100, f"'score' out of range [0, 100]: {score}"
    assert isinstance(report.get('stages', []), list), \
        "'stages' must be a list"
    return report


def _make_client(cli: str) -> Client:
    """Build a Client, passing a timeout when the SDK supports one."""
    import inspect
    timeout_raw = os.environ.get(TIMEOUT_VAR, '').strip()
    accepts_timeout = 'timeout' in inspect.signature(Client.__init__).parameters
    if timeout_raw and accepts_timeout:
        return Client(cli, timeout=float(timeout_raw))
    return Client(cli)


@unittest.skipUnless(_OPTED_IN, _SKIP_REASON)
class RealCliIntegrationTest(unittest.TestCase):
    """Drive the real CLI end-to-end when explicitly enabled."""

    @classmethod
    def setUpClass(cls) -> None:
        cli = os.environ.get(CLI_VAR, '')
        pe = os.environ.get(PE_VAR, '')
        if not cli or not Path(cli).is_file():
            raise unittest.SkipTest(f'{CLI_VAR} must point to a real CLI binary')
        if not pe or not Path(pe).is_file():
            raise unittest.SkipTest(f'{PE_VAR} must point to a real PE32 file')
        cls.cli = cli
        cls.pe = pe

    def test_inspect_real_pe(self):
        info = _make_client(self.cli).inspect(self.pe)
        self.assertIsInstance(info, dict)
        self.assertTrue(info, 'inspect() returned an empty object')

    def test_recompile_real_pe_and_validate_report(self):
        configured = os.environ.get(PROJECT_VAR, '').strip()
        if configured:
            project = Path(configured)
            project.mkdir(parents=True, exist_ok=True)
            self._run_recompile(project)
        else:
            with tempfile.TemporaryDirectory(prefix='gamerecomp-sdk-it-') as tmp:
                self._run_recompile(Path(tmp))

    def _run_recompile(self, project: Path) -> None:
        client = _make_client(self.cli)
        try:
            report = client.recompile(self.pe, project)
        except RecompileError as exc:
            self.fail(f'real CLI recompile failed: {exc}')
        _validate_report(report)

        report_path = project / 'reports' / 'report.json'
        self.assertTrue(report_path.is_file(), 'pipeline wrote no report.json')

        # The path-returning variant must agree with the parsed-report variant.
        self.assertEqual(client.recompile_to_path(self.pe, project), report_path)


if __name__ == '__main__':
    unittest.main()
