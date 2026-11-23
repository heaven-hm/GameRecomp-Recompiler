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
        pe = self.root / 'game.exe'
        pe.write_bytes(b'MZ')
        proj = self.root / 'proj'
        (proj / 'reports').mkdir(parents=True)
        (proj / 'reports' / 'report.json').write_text(json.dumps({'score': 70}))
        with patch('gamerecomp_sdk.subprocess.run') as run:
            run.return_value.returncode = 0
            run.return_value.stdout = 'ok'
            data = Client(self.cli).recompile(pe, proj)
        self.assertEqual(data['score'], 70)
        cmd = run.call_args.args[0]
        self.assertEqual(cmd[:3], [str(self.cli), 'run', str(pe)])

    def test_cli_failure_is_explicit(self):
        pe = self.root / 'game.exe'
        pe.write_bytes(b'MZ')
        with patch('gamerecomp_sdk.subprocess.run') as run:
            run.return_value.returncode = 7
            run.return_value.stdout = 'failure detail'
            with self.assertRaises(RecompileError) as ctx:
                Client(self.cli).recompile(pe, self.root / 'proj')
        self.assertIn('failure detail', str(ctx.exception))


if __name__ == '__main__':
    unittest.main()
