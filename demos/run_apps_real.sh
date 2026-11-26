#!/bin/sh
# Run each SDK's working app against the deterministic fake CLI and record proof.
set -eu
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
OUT="$ROOT/proof/apps-real"
mkdir -p "$OUT"
TMP=$(mktemp -d "${TMPDIR:-/tmp}/gr-sdk-app.XXXXXX")
trap 'rm -rf "$TMP"' EXIT
CLI="$ROOT/demos/real-cli-wrapper.sh"
PE="${GAMERECOMP_REAL_PE:?set GAMERECOMP_REAL_PE to a real PE32 fixture}"
: > "$OUT/exit-codes.tsv"
run() {
  name=$1; shift
  "$@" > "$OUT/$name.stdout.txt" 2> "$OUT/$name.stderr.txt" && rc=0 || rc=$?
  printf '%s\t%s\n' "$name" "$rc" >> "$OUT/exit-codes.tsv"
}
mkdir -p "$TMP/c-project"
cc -std=c11 -Wall -Wextra -I"$ROOT/include" "$ROOT/src/gamerecomp_sdk.c" "$ROOT/demos/apps/c/app.c" -o "$TMP/app_c"
run c "$TMP/app_c" "$CLI" "$PE" "$TMP/c-project"
mkdir -p "$TMP/cpp-project"
c++ -std=c++17 -Wall -Wextra -I"$ROOT/include" -I"$ROOT/cpp/include" "$ROOT/src/gamerecomp_sdk.c" "$ROOT/demos/apps/cpp/app.cpp" -o "$TMP/app_cpp"
run cpp "$TMP/app_cpp" "$CLI" "$PE" "$TMP/cpp-project"
mkdir -p "$TMP/python-project"
run python python3 "$ROOT/demos/apps/python/app.py" "$CLI" "$PE" "$TMP/python-project"
mkdir -p "$TMP/js-project"
run javascript node "$ROOT/demos/apps/js/app.js" "$CLI" "$PE" "$TMP/js-project"
mkdir -p "$TMP/rust-project"
(cd "$ROOT/rust" && blaze run --example app -- "$CLI" "$PE" "$TMP/rust-project") > "$OUT/rust.stdout.txt" 2> "$OUT/rust.stderr.txt" && rrc=0 || rrc=$?
printf 'rust\t%s\n' "$rrc" >> "$OUT/exit-codes.tsv"
python3 - "$OUT" <<'PY'
import hashlib, json, pathlib, sys
out = pathlib.Path(sys.argv[1])
apps = []
for line in (out / "exit-codes.tsv").read_text().splitlines():
    name, rc = line.split("\t")
    stdout = (out / f"{name}.stdout.txt").read_text()
    stderr = (out / f"{name}.stderr.txt").read_text()
    apps.append({
        "sdk": name, "exit_code": int(rc), "stdout": stdout, "stderr": stderr,
        "stdout_sha256": hashlib.sha256(stdout.encode()).hexdigest(),
        "stderr_sha256": hashlib.sha256(stderr.encode()).hexdigest(),
        "app_ok": "APP_OK" in stdout,
    })
report = {"schema": 1, "contract": "one working app per binding against the same fake CLI",
          "scope": "Real GameReComp CLI v0.3.0 recompilation of a real PE32 input; verification skipped because no Windows oracle", "apps": apps}
(out / "results.json").write_text(json.dumps(report, indent=2, sort_keys=True) + "\n")
print(json.dumps([{"sdk": a["sdk"], "exit": a["exit_code"], "app_ok": a["app_ok"]} for a in apps], indent=2))
PY
