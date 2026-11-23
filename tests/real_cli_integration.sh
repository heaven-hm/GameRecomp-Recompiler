#!/bin/sh
# Real GameReComp CLI integration check (inspect-only, across bindings).
#
# This script DOES NOT RECOMPILE ANYTHING. It invokes only
#   <cli> --version
#   <cli> inspect --json <pe32>
# against the REAL game-recomp-cli binary and verifies the returned JSON
# contract. It is a single shared harness that any binding's proof script can
# invoke (``sh tests/real_cli_integration.sh``). Recompilation ("run") is out
# of scope and is covered separately by docs/REAL_RUN.md.
#
# Usage:
#   GAMERECOMP_CLI=/path/to/game-recomp-cli \
#     sh tests/real_cli_integration.sh [PE32_FILE] [OUT_DIR]
#
# Configuration (environment):
#   GAMERECOMP_CLI          path to the real CLI (REQUIRED; absence => SKIP)
#   GR_PE_FILE              PE32 to inspect (default: generate minimal fixture)
#   GR_CLI_INTEGRATION_OUT  artifact directory (default: a new temp dir)
#   GR_EXPECT_IMAGE_BASE    expected image_base for the generated fixture only
#
# Exit status:
#   0  PASS or SKIP (SKIP is always printed explicitly)
#   1  FAIL (real CLI present but the contract check failed)
#   2  usage/setup error internal to this harness
#
# Artifacts (in OUT_DIR): version.txt, inspect.json, inspect.stderr.log,
#   run.log (full transcript) and results.json (machine-readable summary).
# SHA-256 hashes of the CLI, the PE and the captured JSON are always recorded.

set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
FIXTURE="$ROOT/tests/fixtures/minimal_pe.py"

# Exact key set emitted by `game-recomp-cli inspect --json`
# (GameReComp src/cli.rs::run_inspect_cmd). Keep in sync with that source.
EXPECTED_KEYS='path ok error magic machine pe32_x86 image_base size input_os input_arch'

STATUS=""
OUT_DIR=""
PE_PATH=""
CLI_PATH=""
SUMMARY=""

sha256_file() {
  # Portable hashing via python3 (already required for JSON validation).
  python3 -c 'import hashlib,sys;print(hashlib.sha256(open(sys.argv[1],"rb").read()).hexdigest())' "$1"
}

write_summary() {
  # write_summary <status> <json-overrides...>; overrides are KEY=VALUE pairs.
  STATUS=$1
  shift
  mkdir -p "$OUT_DIR"
  {
    printf '{\n'
    printf '  "status": "%s",\n' "$STATUS"
    printf '  "mode": "inspect-only",\n'
    printf '  "cli": "%s",\n' "${CLI_PATH:-}"
    printf '  "pe": "%s",\n' "${PE_PATH:-}"
    if [ -n "${CLI_SHA:-}" ]; then printf '  "cli_sha256": "%s",\n' "$CLI_SHA"; fi
    if [ -n "${PE_SHA:-}" ]; then printf '  "pe_sha256": "%s",\n' "$PE_SHA"; fi
    if [ -n "${JSON_SHA:-}" ]; then printf '  "inspect_json_sha256": "%s",\n' "$JSON_SHA"; fi
    if [ -n "${CLI_VERSION:-}" ]; then printf '  "cli_version": "%s",\n' "$CLI_VERSION"; fi
    for pair in "$@"; do
      key=${pair%%=*}
      value=${pair#*=}
      printf '  "%s": "%s",\n' "$key" "$value"
    done
    # Close with a always-valid final member to avoid dangling commas.
    printf '  "harness": "tests/real_cli_integration.sh"\n'
    printf '}\n'
  } > "$SUMMARY"
}

finish_skip() {
  log "SKIP: $1"
  write_summary SKIP "reason=$1"
  log "  artifacts: $OUT_DIR"
  exit 0
}

finish_fail() {
  log "FAIL: $1"
  write_summary FAIL "reason=$1"
  log "  artifacts: $OUT_DIR"
  exit 1
}

# ---- Setup / artifact directory ------------------------------------------
if [ $# -gt 2 ]; then
  echo "usage: $0 [PE32_FILE] [OUT_DIR]" >&2
  exit 2
fi
if [ -n "${GR_CLI_INTEGRATION_OUT:-}" ]; then
  OUT_DIR=$GR_CLI_INTEGRATION_OUT
elif [ $# -ge 2 ] && [ -n "$2" ]; then
  OUT_DIR=$2
else
  OUT_DIR=${TMPDIR:-/tmp}/gr-real-cli-integration-$(date -u +%Y%m%dT%H%M%SZ)-$$
fi
mkdir -p "$OUT_DIR"
SUMMARY=$OUT_DIR/results.json
RUNLOG=$OUT_DIR/run.log
: > "$RUNLOG"
log() { printf '%s\n' "$*" | tee -a "$RUNLOG"; }

# ---- 1. Discover the real CLI (env-driven; never hard-coded) -------------
CLI_PATH=${GAMERECOMP_CLI:-}
if [ -z "$CLI_PATH" ]; then
  finish_skip "GAMERECOMP_CLI is not set; no real CLI available"
fi
if [ ! -f "$CLI_PATH" ]; then
  finish_skip "GAMERECOMP_CLI does not point to a file: $CLI_PATH"
fi
if [ ! -x "$CLI_PATH" ]; then
  finish_skip "GAMERECOMP_CLI is not executable: $CLI_PATH"
fi
log "MODE: inspect-only (no recompilation; no 'run' subcommand invoked)"
log "CLI:  $CLI_PATH"

# ---- 2. Validate the CLI version -----------------------------------------
if ! "$CLI_PATH" --version > "$OUT_DIR/version.txt" 2>"$OUT_DIR/version.stderr.log"; then
  log "CLI --version stderr:"; cat "$OUT_DIR/version.stderr.log" >> "$RUNLOG" || true
  finish_fail "CLI --version returned a non-zero exit status"
fi
# Strip ANSI colour codes before matching (portable; no sed dependency).
VERSION_LINE=$(python3 -c '
import re, sys
s = sys.stdin.read()
s = re.sub(r"\x1b\[[0-9;]*[A-Za-z]", "", s).replace("\r", "")
lines = s.splitlines()
print(lines[0] if lines else "")
' < "$OUT_DIR/version.txt")
CLI_VERSION=$VERSION_LINE
if ! printf '%s\n' "$VERSION_LINE" | grep -Eiq 'game-recomp-cli.*[0-9]+\.[0-9]+'; then
  finish_fail "unexpected CLI version string: $VERSION_LINE"
fi
log "VERSION: $CLI_VERSION"

# ---- 3. Obtain the PE32 to inspect ---------------------------------------
GENERATED_PE=0
if [ $# -ge 1 ] && [ -n "$1" ]; then
  PE_PATH=$1
elif [ -n "${GR_PE_FILE:-}" ]; then
  PE_PATH=$GR_PE_FILE
else
  PE_PATH=$OUT_DIR/minimal-pe32.exe
  if ! python3 "$FIXTURE" "$PE_PATH"; then
    finish_fail "could not generate minimal PE32 fixture via $FIXTURE"
  fi
  GENERATED_PE=1
fi
if [ ! -f "$PE_PATH" ]; then
  finish_fail "PE32 file not found: $PE_PATH"
fi
log "PE:   $PE_PATH (generated=$GENERATED_PE)"

# ---- 4. Hash inputs -------------------------------------------------------
CLI_SHA=$(sha256_file "$CLI_PATH")
PE_SHA=$(sha256_file "$PE_PATH")
log "CLI_SHA256: $CLI_SHA"
log "PE_SHA256:  $PE_SHA"

# ---- 5. Run the real inspect --json command -------------------------------
# Only inspect is invoked. There is deliberately no 'run'/recompile step here.
if "$CLI_PATH" inspect --json "$PE_PATH" \
      > "$OUT_DIR/inspect.json" 2> "$OUT_DIR/inspect.stderr.log"; then
  :
else
  rc=$?
  if [ -s "$OUT_DIR/inspect.stderr.log" ]; then
    log "inspect stderr:"; cat "$OUT_DIR/inspect.stderr.log" >> "$RUNLOG" || true
  fi
  finish_fail "CLI inspect --json exited non-zero ($rc)"
fi
JSON_SHA=$(sha256_file "$OUT_DIR/inspect.json")
log "INSPECT_JSON_SHA256: $JSON_SHA"
if [ -s "$OUT_DIR/inspect.stderr.log" ]; then
  log "inspect stderr (non-empty, kept for audit):"
  cat "$OUT_DIR/inspect.stderr.log" >> "$RUNLOG" || true
fi

# ---- 6. Validate the exact JSON contract ---------------------------------
# For the generated fixture we also assert the known header values. For a
# caller-supplied PE we only require a supported PE32 x86 verdict.
if ! python3 - "$OUT_DIR/inspect.json" "$PE_PATH" "$GENERATED_PE" \
      "${GR_EXPECT_IMAGE_BASE:-0x00400000}" "$EXPECTED_KEYS" <<'PY'
import json
import os
import sys

path, pe_path, generated, expect_base, keys_arg = sys.argv[1:6]
generated = generated == "1"

# Single source of truth: the EXPECTED_KEYS string defined by this harness.
expected_keys = set(keys_arg.split())

def die(msg):
    print("JSON CHECK FAILED: " + msg, file=sys.stderr)
    raise SystemExit(1)

with open(path, "r", encoding="utf-8") as fh:
    try:
        doc = json.load(fh)
    except json.JSONDecodeError as exc:
        die("inspect output is not valid JSON: %s" % exc)

if not isinstance(doc, dict):
    die("inspect JSON is not an object")

keys = set(doc)
missing = sorted(expected_keys - keys)
unexpected = sorted(keys - expected_keys)
if missing or unexpected:
    die("key mismatch; missing=%s unexpected=%s" % (missing, unexpected))
print("KEYS: exact match (%d keys)" % len(expected_keys))

if doc.get("ok") is not True:
    die("ok is not true: %r" % doc.get("ok"))
if doc.get("pe32_x86") is not True:
    die("pe32_x86 is not true: %r" % doc.get("pe32_x86"))
if doc.get("magic") != "PE32":
    die("magic is not PE32: %r" % doc.get("magic"))
if doc.get("machine") != "i386":
    die("machine is not i386: %r" % doc.get("machine"))
if doc.get("input_os") != "windows":
    die("input_os is not windows: %r" % doc.get("input_os"))
if doc.get("input_arch") != "x86":
    die("input_arch is not x86: %r" % doc.get("input_arch"))
if not isinstance(doc.get("size"), int) or doc["size"] <= 0:
    die("size is not a positive integer: %r" % doc.get("size"))
if doc.get("error") not in ("", None):
    die("error is not empty: %r" % doc.get("error"))
if os.path.realpath(doc.get("path", "")) != os.path.realpath(pe_path):
    die("path does not match the inspected file: %r" % doc.get("path"))

if generated:
    got = str(doc.get("image_base", "")).lower()
    if got != expect_base.lower():
        die("image_base is not %s: %r" % (expect_base, doc.get("image_base")))
    print("IMAGE_BASE: %s (expected)" % expect_base)

print("PE32_X86: true  MAGIC: PE32  MACHINE: i386  OK: true")
PY
then
  finish_fail "inspect JSON contract validation failed"
fi

# ---- 7. Pass --------------------------------------------------------------
log "JSON_SHA256: $JSON_SHA"
log "PASS: real CLI inspect-only contract verified (10 exact keys, supported PE32 x86)"
log "NOTE: no recompilation was performed; only 'inspect --json' was executed"
write_summary PASS
exit 0
