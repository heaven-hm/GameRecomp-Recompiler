"""Tests for the Python SDK using a fake CLI (no real recompilation)."""
import json
import os
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from gamerecomp_sdk import Client, RecompileError


class SdkTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.cli = self.root / 'game-recomp-cli'
        self.cli.write_text('#!/bin/sh\nexit 0\n')
        self.cli.chmod(0o755)
        self.pe = self.root / 'game.exe'
        self.pe.write_bytes(b'MZ')

    def tearDown(self):
        self.tmp.cleanup()

    def test_missing_cli_is_rejected(self):
        with self.assertRaises(RecompileError):
            Client(self.root / 'missing')

    def test_missing_pe_is_rejected(self):
        client = Client(self.cli)
        with self.assertRaises(RecompileError):
            client.recompile(self.root / 'missing.exe', self.root / 'proj')

    def test_successful_report_is_returned(self):
        proj = self.root / 'proj'
        (proj / 'reports').mkdir(parents=True)
        (proj / 'reports' / 'report.json').write_text(json.dumps({'score': 70}))
        with patch('gamerecomp_sdk.subprocess.run') as run:
            run.return_value.returncode = 0
            run.return_value.stdout = 'ok'
            data = Client(self.cli).recompile(self.pe, proj)
        self.assertEqual(data['score'], 70)
        cmd = run.call_args.args[0]
        self.assertEqual(cmd[:3], [str(self.cli), 'run', str(self.pe)])

    def test_cli_failure_is_explicit(self):
        with patch('gamerecomp_sdk.subprocess.run') as run:
            run.return_value.returncode = 7
            run.return_value.stdout = 'failure detail'
            with self.assertRaises(RecompileError) as ctx:
                Client(self.cli).recompile(self.pe, self.root / 'proj')
        self.assertIn('failure detail', str(ctx.exception))

    # --- inspect() -------------------------------------------------------

    def test_inspect_returns_parsed_json(self):
        expected = {'machine': 'i386', 'sections': 5, 'entry': 4096}
        with patch('gamerecomp_sdk.subprocess.run') as run:
            run.return_value.returncode = 0
            run.return_value.stdout = json.dumps(expected)
            data = Client(self.cli).inspect(self.pe)
        self.assertEqual(data, expected)
        cmd = run.call_args.args[0]
        self.assertEqual(cmd, [str(self.cli), 'inspect', str(self.pe), '--json'])

    def test_inspect_missing_pe_is_rejected(self):
        with self.assertRaises(RecompileError) as ctx:
            Client(self.cli).inspect(self.root / 'missing.exe')
        self.assertIn('PE32 input does not exist', str(ctx.exception))

    def test_inspect_cli_failure_is_explicit(self):
        with patch('gamerecomp_sdk.subprocess.run') as run:
            run.return_value.returncode = 9
            run.return_value.stdout = 'inspect failed detail'
            with self.assertRaises(RecompileError) as ctx:
                Client(self.cli).inspect(self.pe)
        self.assertIn('inspect failed detail', str(ctx.exception))

    def test_inspect_invalid_json_is_rejected(self):
        with patch('gamerecomp_sdk.subprocess.run') as run:
            run.return_value.returncode = 0
            run.return_value.stdout = 'not json at all'
            with self.assertRaises(RecompileError) as ctx:
                Client(self.cli).inspect(self.pe)
        self.assertIn('invalid JSON', str(ctx.exception))

    # --- recompile_to_path() ---------------------------------------------

    def test_recompile_to_path_returns_report_path(self):
        proj = self.root / 'proj'
        report = proj / 'reports' / 'report.json'
        report.parent.mkdir(parents=True)
        report.write_text(json.dumps({'score': 70}))
        with patch('gamerecomp_sdk.subprocess.run') as run:
            run.return_value.returncode = 0
            run.return_value.stdout = 'ok'
            result = Client(self.cli).recompile_to_path(self.pe, proj)
        self.assertIsInstance(result, Path)
        self.assertEqual(result, report)
        cmd = run.call_args.args[0]
        self.assertEqual(cmd, [str(self.cli), 'run', str(self.pe),
                               '--project-dir', str(proj),
                               '--no-table', '--no-progress'])

    def test_recompile_to_path_missing_pe_is_rejected(self):
        with self.assertRaises(RecompileError) as ctx:
            Client(self.cli).recompile_to_path(self.root / 'missing.exe',
                                               self.root / 'proj')
        self.assertIn('PE32 input does not exist', str(ctx.exception))

    def test_recompile_to_path_cli_failure_is_explicit(self):
        with patch('gamerecomp_sdk.subprocess.run') as run:
            run.return_value.returncode = 7
            run.return_value.stdout = 'failure detail'
            with self.assertRaises(RecompileError) as ctx:
                Client(self.cli).recompile_to_path(self.pe, self.root / 'proj')
        self.assertIn('failure detail', str(ctx.exception))

    def test_recompile_to_path_without_report_is_rejected(self):
        with patch('gamerecomp_sdk.subprocess.run') as run:
            run.return_value.returncode = 0
            run.return_value.stdout = 'ok'
            with self.assertRaises(RecompileError) as ctx:
                Client(self.cli).recompile_to_path(self.pe, self.root / 'proj')
        self.assertIn('no report.json', str(ctx.exception))

    def test_recompile_still_returns_parsed_report(self):
        """The existing recompile() behavior is preserved through the delegation."""
        proj = self.root / 'proj'
        (proj / 'reports').mkdir(parents=True)
        (proj / 'reports' / 'report.json').write_text(json.dumps({'score': 42}))
        with patch('gamerecomp_sdk.subprocess.run') as run:
            run.return_value.returncode = 0
            run.return_value.stdout = 'ok'
            data = Client(self.cli).recompile(self.pe, proj)
        self.assertEqual(data, {'score': 42})


if __name__ == '__main__':
    unittest.main()
