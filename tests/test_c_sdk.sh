#!/bin/sh
# Compile and run the C example against fake CLIs; cover the SDK hardening paths.
set -eu
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
TMP=${TMPDIR:-/tmp}/gr-sdk-c-$$
trap 'rm -rf "$TMP"' EXIT HUP INT TERM
mkdir -p "$TMP/proj/reports"
cat > "$TMP/cli" <<EOF
#!/bin/sh
echo '{"score":70,"stages":[]}' > "$TMP/proj/reports/report.json"
exit 0
EOF
chmod +x "$TMP/cli"
cc -std=c11 -Wall -Wextra -I"$ROOT/include" \
  "$ROOT/src/gamerecomp_sdk.c" "$ROOT/c/examples/basic.c" -o "$TMP/basic"

# Happy path: report path is returned to the caller.
"$TMP/basic" "$TMP/cli" /etc/hosts "$TMP/proj" | grep -q '^report='

# project_dir must exist before the SDK attempts to exec the CLI.
if "$TMP/basic" "$TMP/cli" /etc/hosts "$TMP/missing" 2>"$TMP/err"; then
  echo "expected failure for missing project_dir" >&2; exit 1
fi
grep -qi 'directory' "$TMP/err"

# project_dir must be a directory, not just an existing path.
printf 'not a dir\n' > "$TMP/notadir"
if "$TMP/basic" "$TMP/cli" /etc/hosts "$TMP/notadir" 2>"$TMP/err"; then
  echo "expected failure for file project_dir" >&2; exit 1
fi
grep -qi 'directory' "$TMP/err"

# A failed exec reports the OS reason instead of a generic exit status.
printf '#!/bin/sh\nexit 0\n' > "$TMP/noexec"
chmod 644 "$TMP/noexec"
if "$TMP/basic" "$TMP/noexec" /etc/hosts "$TMP/proj" 2>"$TMP/err"; then
  echo "expected failure for non-executable CLI" >&2; exit 1
fi
grep -qi 'execute' "$TMP/err"
grep -qiE 'denied|permission|exec' "$TMP/err"

# EINTR during the output read must not truncate captured CLI output.
cat > "$TMP/signal_cli" <<'EOF'
#!/bin/sh
echo "FIRST"
kill -ALRM "$PPID"
sleep 0.05
echo "SECOND_MARKER"
exit 7
EOF
chmod +x "$TMP/signal_cli"
cat > "$TMP/eintr.c" <<'EOF'
/* POSIX feature test macros must precede ANY header on glibc. */
#define _POSIX_C_SOURCE 200809L
#include "gamerecomp_sdk.h"
#include <signal.h>
#include <stdio.h>
#include <string.h>
static void on_alrm(int sig) { (void)sig; }
int main(int argc, char **argv) {
    struct sigaction sa;
    if (argc != 4) return 2;
    memset(&sa, 0, sizeof sa);
    sa.sa_handler = on_alrm;
    sigaction(SIGALRM, &sa, NULL);
    gr_client c;
    if (gr_client_init(&c, argv[1]) != 0) { puts(gr_last_error(&c)); return 1; }
    char report[1024] = {0};
    int rc = gr_recompile(&c, argv[2], argv[3], report, sizeof report);
    printf("rc=%d err=%s\n", rc, gr_last_error(&c));
    gr_client_free(&c);
    return 0;
}
EOF
cc -std=c11 -Wall -Wextra -I"$ROOT/include" \
  "$ROOT/src/gamerecomp_sdk.c" "$TMP/eintr.c" -o "$TMP/eintr"
"$TMP/eintr" "$TMP/signal_cli" /etc/hosts "$TMP/proj" > "$TMP/eintr.out" 2>&1
grep -q 'SECOND_MARKER' "$TMP/eintr.out"

echo "C SDK: PASS"
