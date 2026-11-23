#!/bin/sh
# Compile and run the C++ example against a fake CLI that writes a report.
set -eu
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
TMP=${TMPDIR:-/tmp}/gr-sdk-cpp-$$
mkdir -p "$TMP/proj/reports"
cat > "$TMP/cli" <<EOF
#!/bin/sh
echo '{"score":70,"stages":[]}' > "$TMP/proj/reports/report.json"
exit 0
EOF
chmod +x "$TMP/cli"
c++ -std=c++17 -Wall -Wextra -I"$ROOT/include" -I"$ROOT/cpp/include" \
  "$ROOT/src/gamerecomp_sdk.c" "$ROOT/cpp/examples/basic.cpp" -o "$TMP/basic"
"$TMP/basic" "$TMP/cli" /etc/hosts "$TMP/proj"
echo "C++ SDK: PASS"
