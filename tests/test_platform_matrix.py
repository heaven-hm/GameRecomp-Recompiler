"""Schema test for the recorded platform-verification results.

The evidence lives in docs/PLATFORM_VERIFICATION.md as one ``json`` fenced
block. This test keeps that record honest: it must describe exactly one result
per binding per platform, macOS results must be real executed passes, and
Linux results must be explicitly blocked with no fabricated exit code.

Stdlib only. Runs under pytest or directly:
    python3 tests/test_platform_matrix.py
"""

import json
import re
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "docs" / "PLATFORM_VERIFICATION.md"

BINDINGS = {"python", "rust", "c", "cpp"}
PLATFORMS = {"macos", "linux"}
PASSING_STATUSES = {"passed"}
BLOCKED_STATUSES = {"blocked", "not_tested"}
ALLOWED_STATUSES = PASSING_STATUSES | BLOCKED_STATUSES

EXPECTED_COMMANDS = {
    "python": "PYTHONPATH=python python3 -m pytest python/tests -q",
    "rust": "cd rust && blaze test",
    "c": "sh tests/test_c_sdk.sh",
    "cpp": "sh tests/test_cpp_sdk.sh",
}

REQUIRED_ENTRY_KEYS = {
    "binding",
    "platform",
    "status",
    "command",
    "exit_code",
    "summary",
}
REQUIRED_BLOCKED_KEYS = REQUIRED_ENTRY_KEYS | {"reason", "evidence"}


def _doc_text():
    assert DOC.is_file(), f"missing verification record: {DOC}"
    return DOC.read_text(encoding="utf-8")


def _json_blocks(text):
    return re.findall(r"```json[ \t]*\n(.*?)```", text, flags=re.DOTALL)


def load_record():
    """Parse the single machine-readable block from the verification doc."""
    blocks = _json_blocks(_doc_text())
    assert len(blocks) == 1, f"expected exactly one json block, found {len(blocks)}"
    record = json.loads(blocks[0])
    assert isinstance(record, dict), "record must be a JSON object"
    return record


class PlatformMatrixSchemaTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.text = _doc_text()
        cls.record = load_record()

    def test_record_metadata(self):
        r = self.record
        self.assertEqual(r.get("schema_version"), 1)
        for key in ("record_id", "generated_at_utc", "base_commit", "branch"):
            self.assertIsInstance(r.get(key), str, f"missing/invalid {key}")
            self.assertTrue(r[key].strip(), f"empty {key}")
        self.assertRegex(
            r["generated_at_utc"],
            r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z$",
            "generated_at_utc must be an ISO-8601 UTC timestamp",
        )
        self.assertRegex(
            r["base_commit"], r"^[0-9a-f]{40}$", "base_commit must be a 40-hex SHA"
        )
        self.assertIsInstance(r.get("host"), dict)
        self.assertIsInstance(r.get("toolchain"), dict)
        for key in ("os_family", "os_version", "arch"):
            self.assertTrue(r["host"].get(key), f"missing host.{key}")
        for key in ("python", "pytest", "cargo", "rustc", "cc"):
            self.assertTrue(r["toolchain"].get(key), f"missing toolchain.{key}")

    def test_one_result_per_binding_and_platform(self):
        results = self.record.get("results")
        self.assertIsInstance(results, list)
        self.assertEqual(len(results), len(BINDINGS) * len(PLATFORMS))
        seen = set()
        for entry in results:
            self.assertIsInstance(entry, dict, "each result must be an object")
            binding = entry.get("binding")
            platform = entry.get("platform")
            self.assertIn(binding, BINDINGS, f"unknown binding {binding!r}")
            self.assertIn(platform, PLATFORMS, f"unknown platform {platform!r}")
            self.assertNotIn(
                (binding, platform), seen, f"duplicate result for {binding}/{platform}"
            )
            seen.add((binding, platform))
            self.assertIn(entry.get("status"), ALLOWED_STATUSES)
            self.assertIsInstance(entry.get("command"), str)
            self.assertTrue(entry["command"].strip(), "empty command")
            self.assertIsInstance(entry.get("summary"), str)
            self.assertTrue(entry["summary"].strip(), "empty summary")
            exit_code = entry.get("exit_code")
            self.assertTrue(
                exit_code is None
                or (isinstance(exit_code, int) and not isinstance(exit_code, bool)),
                "exit_code must be int or null (booleans are not exit codes)",
            )
        self.assertEqual(
            seen,
            {(b, p) for b in BINDINGS for p in PLATFORMS},
            "missing binding/platform combinations",
        )

    def test_macos_cells_are_executed_passes(self):
        results = self.record["results"]
        for entry in results:
            if entry["platform"] != "macos":
                continue
            self.assertIn(entry["status"], PASSING_STATUSES)
            self.assertEqual(entry["exit_code"], 0)
            self.assertEqual(
                entry["command"],
                EXPECTED_COMMANDS[entry["binding"]],
                "macOS command must match the CI matrix command",
            )

    def test_linux_cells_are_blocked_without_fabricated_results(self):
        results = self.record["results"]
        linux = [e for e in results if e["platform"] == "linux"]
        self.assertEqual(len(linux), len(BINDINGS))
        for entry in linux:
            self.assertIn(entry["status"], BLOCKED_STATUSES)
            self.assertIsNone(
                entry["exit_code"], "a blocked Linux cell must not claim an exit code"
            )
            missing = REQUIRED_BLOCKED_KEYS - entry.keys()
            self.assertFalse(missing, f"missing blocked-result keys: {sorted(missing)}")
            self.assertTrue(entry["reason"].strip(), "empty Linux blocker reason")
            self.assertIsInstance(entry["evidence"], list)
            self.assertGreaterEqual(len(entry["evidence"]), 1)
            for item in entry["evidence"]:
                self.assertIsInstance(item, str)
                self.assertTrue(item.strip(), "empty evidence string")
            self.assertIn(
                "NOT TESTED",
                entry["summary"].upper(),
                "Linux summary must state that the check was not run",
            )

    def test_doc_states_linux_status_explicitly(self):
        self.assertIn("BLOCKED", self.text)
        self.assertIn("NOT TESTED", self.text)
        self.assertIn("Linux matrix", self.text)
        self.assertIn("docker version", self.text)
        self.assertIn("colima status", self.text)
        self.assertIn("limactl list", self.text)


if __name__ == "__main__":
    unittest.main(verbosity=2)
