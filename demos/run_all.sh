#!/bin/sh
# Run one end-to-end demo per SDK binding against the deterministic fake CLI.
# This proves SDK plumbing only: no recompiler and no gameplay are claimed.
set -eu
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
OUT="$ROOT/proof/demos"
mkdir -p "$OUT"
TMP=$(mktemp -d "${TMPDIR:-/tmp}/gr-sdk-demo.XXXXXX")
trap 'rm -rf "$TMP"' EXIT
CLI="$ROOT/demos/fake-cli.sh"
PE="$TMP/game.exe"
printf 'MZ synthetic PE32 fixture for SDK demos\n' > "$PE"
run() {
  name=$1; shift
  "$@" > "$OUT/$name.stdout.txt" 2> "$OUT/$name.stderr.txt" && rc=0 || rc=$?
  printf '%s\t%s\n' "$name" "$rc" >> "$OUT/exit-codes.tsv"
}
: > "$OUT/exit-codes.tsv"
cc -std=c11 -Wall -Wextra -I"$ROOT/include" "$ROOT/src/gamerecomp_sdk.c" "$ROOT/c/examples/basic.c" -o "$TMP/basic_c"
mkdir -p "$TMP/c-project"
run c "$TMP/basic_c" "$CLI" "$PE" "$TMP/c-project"
c++ -std=c++17 -Wall -Wextra -I"$ROOT/include" -I"$ROOT/cpp/include" "$ROOT/src/gamerecomp_sdk.c" "$ROOT/cpp/examples/basic.cpp" -o "$TMP/basic_cpp"
mkdir -p "$TMP/cpp-project"
run cpp "$TMP/basic_cpp" "$CLI" "$PE" "$TMP/cpp-project"
mkdir -p "$TMP/python-project"
run python env PYTHONPATH="$ROOT/python" python3 "$ROOT/python/examples/basic.py" --cli "$CLI" --pe "$PE" --project "$TMP/python-project"
mkdir -p "$TMP/js-project"
run javascript node "$ROOT/js/examples/basic.js" "$CLI" "$PE" "$TMP/js-project"
mkdir -p "$TMP/rust-project"
(cd "$ROOT/rust" && blaze run --example basic -- "$CLI" "$PE" "$TMP/rust-project") > "$OUT/rust.stdout.txt" 2> "$OUT/rust.stderr.txt" && rrc=0 || rrc=$?
printf 'rust\t%s\n' "$rrc" >> "$OUT/exit-codes.tsv"
python3 - "$OUT" <<'PY'
import hashlib,json,pathlib,sys
out=pathlib.Path(sys.argv[1])
rows=[]
for line in (out/'exit-codes.tsv').read_text().splitlines():
 name,rc=line.split('\t')
 side={}
 for suffix in ('stdout.txt','stderr.txt'):
  p=out/f'{name}.{suffix}'
  side[suffix]=p.read_text()
 rows.append({'sdk':name,'exit_code':int(rc),'stdout':side['stdout.txt'],'stderr':side['stderr.txt'],
              'stdout_sha256':hashlib.sha256(side['stdout.txt'].encode()).hexdigest(),
              'stderr_sha256':hashlib.sha256(side['stderr.txt'].encode()).hexdigest()})
report={'schema':1,'contract':'all bindings invoked the same fake CLI and parsed report.json',
        'scope':'SDK plumbing proof only; no recompiler or gameplay claim','demos':rows}
(out/'results.json').write_text(json.dumps(report,indent=2,sort_keys=True)+'\n')
print(json.dumps([{'sdk':r['sdk'],'exit':r['exit_code'],'sha':r['stdout_sha256'][:12]} for r in rows], indent=2))
PY
