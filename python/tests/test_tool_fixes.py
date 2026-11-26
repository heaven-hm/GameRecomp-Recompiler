"""Deterministic regressions for the SDK tool fixes.

These tests exercise the two tool bugs directly, without live game data:
the compatibility gate must return INCOMPLETE (a defined constant) when a
required input is absent, and the linear-sweep entry finder must decode an
E8 call target using the stdlib `struct` module.
"""

import struct
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
if str(ROOT) not in sys.path:
    sys.path.insert(0, str(ROOT))

from tools import compatibility
from tools.lift import generate


class CompatibilityGateTest(unittest.TestCase):
    def _doc(self, *, present, observed):
        return {
            "inputs": {
                "required-input": {
                    "path": "reports/games/results.json",
                    "present": present,
                    "fixture_id": None,
                    "expected_result": compatibility.PASS,
                    "observed_result": observed,
                },
            },
            "families": [],
        }

    @staticmethod
    def _baseline():
        # No families, no reports in the temporary repo: the only gate input
        # is the one record in doc["inputs"], so the outcome is deterministic.
        return {
            "families": [],
            "fixture_ids": [],
            "known_unresolved_critical_imports": [],
            "critical_import_dlls": [],
        }

    def test_missing_required_input_returns_incomplete(self):
        with tempfile.TemporaryDirectory() as tmp:
            gate = compatibility.make_gate(
                self._doc(present=False, observed=compatibility.NOT_RUN),
                self._baseline(), Path(tmp), [])
        self.assertEqual(compatibility.INCOMPLETE, "INCOMPLETE")
        self.assertEqual(gate["verdict"], compatibility.INCOMPLETE)
        self.assertEqual(gate["checks"]["missing_inputs"],
                         ["reports/games/results.json"])
        self.assertEqual(gate["reason"],
                         "required input absent: reports/games/results.json")

    def test_present_required_input_does_not_return_incomplete(self):
        with tempfile.TemporaryDirectory() as tmp:
            gate = compatibility.make_gate(
                self._doc(present=True, observed=compatibility.PASS),
                self._baseline(), Path(tmp), [])
        self.assertEqual(gate["verdict"], compatibility.PASS)
        self.assertEqual(gate["checks"]["missing_inputs"], [])


class LiftGenerateTest(unittest.TestCase):
    def test_find_entries_decodes_e8_call_target(self):
        code_start = 0x1000
        code_end = 0x2000
        call_rel = 0x10
        code = bytearray(0x40)
        code[0:5] = b"\xe8" + struct.pack("<i", call_rel)
        code[5:8] = b"\x55\x8b\xec"  # push ebp; mov ebp, esp

        entries = generate.find_entries(bytes(code), code_start, code_end)

        # E8 at offset 0 calls code_start + 5 + rel; the prologue at offset 5
        # is a second, independent entry point.
        self.assertIn(code_start + 5 + call_rel, entries)
        self.assertIn(code_start + 5, entries)
        self.assertEqual(entries, [code_start + 5, code_start + 5 + call_rel])


if __name__ == "__main__":
    unittest.main()
