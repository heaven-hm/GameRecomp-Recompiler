#!/bin/sh
# Compile and run the C++ example against a fake CLI that writes a report.
# Also exercises Report::score() for both a present and an absent score.
set -eu
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
TMP=${TMPDIR:-/tmp}/gr-sdk-cpp-$$
mkdir -p "$TMP/proj/reports"

# Recreate the fake CLI so the next run writes the given JSON document.
write_cli() {
  cat > "$TMP/cli" <<EOF
#!/bin/sh
printf '%s' '$1' > "$TMP/proj/reports/report.json"
exit 0
EOF
  chmod +x "$TMP/cli"
}

c++ -std=c++17 -Wall -Wextra -I"$ROOT/include" -I"$ROOT/cpp/include" \
  "$ROOT/src/gamerecomp_sdk.c" "$ROOT/cpp/examples/basic.cpp" -o "$TMP/basic"

# Score present: the extractor must read the integer from the JSON report.
write_cli '{"score":70,"stages":[]}'
out=$("$TMP/basic" "$TMP/cli" /etc/hosts "$TMP/proj")
printf '%s\n' "$out"
echo "$out" | grep -q 'score=70'

# Score absent: the extractor must fall back to the -1 sentinel.
write_cli '{"stages":[]}'
out=$("$TMP/basic" "$TMP/cli" /etc/hosts "$TMP/proj")
printf '%s\n' "$out"
echo "$out" | grep -q 'score=-1'

echo "C++ SDK: PASS"
