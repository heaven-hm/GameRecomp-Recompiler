#!/bin/sh
# Compile and run the C example against fake CLIs; cover the SDK hardening
# paths plus the frozen ABI and the structured gr_result/error-code API.
set -eu
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
TMP=${TMPDIR:-/tmp}/gr-sdk-c-$$
trap 'rm -rf "$TMP"' EXIT HUP INT TERM
mkdir -p "$TMP/proj/reports"
CC=${CC:-cc}
CFLAGS='-std=c11 -Wall -Wextra -Wpedantic -Werror'
cat > "$TMP/cli" <<EOF
#!/bin/sh
echo '{"score":70,"stages":[]}' > "$TMP/proj/reports/report.json"
exit 0
EOF
chmod +x "$TMP/cli"
$CC $CFLAGS -I"$ROOT/include" \
  "$ROOT/src/gamerecomp_sdk.c" "$ROOT/c/examples/basic.c" -o "$TMP/basic"

# Version entry point is callable without a client.
"$TMP/basic" --version | grep -q 'gamerecomp C SDK 1.0.0'

# Happy path: report path is returned to the caller.
"$TMP/basic" "$TMP/cli" /etc/hosts "$TMP/proj" | grep -q '^report='

# project_dir must exist before the SDK attempts to exec the CLI.
if "$TMP/basic" "$TMP/cli" /etc/hosts "$TMP/missing" 2>"$TMP/err"; then
  echo "expected failure for missing project_dir" >&2; exit 1
fi
grep -qi 'directory' "$TMP/err"
grep -q 'GR_ERR_NOT_DIRECTORY' "$TMP/err"

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
grep -q 'GR_ERR_EXEC_FAILED' "$TMP/err"

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
    char report[GR_SDK_REPORT_PATH_MAX] = {0};
    int rc = gr_recompile(&c, argv[2], argv[3], report, sizeof report);
    printf("rc=%d err=%s\n", rc, gr_last_error(&c));
    gr_client_free(&c);
    return 0;
}
EOF
$CC $CFLAGS -I"$ROOT/include" \
  "$ROOT/src/gamerecomp_sdk.c" "$TMP/eintr.c" -o "$TMP/eintr"
"$TMP/eintr" "$TMP/signal_cli" /etc/hosts "$TMP/proj" > "$TMP/eintr.out" 2>&1
grep -q 'SECOND_MARKER' "$TMP/eintr.out"

# A CLI that exits non-zero: captured output is the structured detail.
cat > "$TMP/fail_cli" <<'EOF'
#!/bin/sh
echo "CLI_FAILURE_MARKER" 1>&2
exit 3
EOF
chmod +x "$TMP/fail_cli"

# ABI/API conformance program: frozen struct layout, stable enum values,
# version macros, error-name/string table, result accessors, and both APIs.
cat > "$TMP/abi_api.c" <<'EOF'
#define _POSIX_C_SOURCE 200809L
#include "gamerecomp_sdk.h"
#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/* Frozen ABI: legacy struct offsets/size and enum values never change. */
_Static_assert(offsetof(gr_client, cli) == 0, "gr_client.cli offset changed");
_Static_assert(offsetof(gr_client, error) == sizeof(char *),
               "gr_client.error offset changed");
_Static_assert(sizeof(gr_client) == sizeof(char *) + 512,
               "gr_client size changed");
_Static_assert(GR_SDK_ERROR_BUFFER_SIZE == 512u, "error buffer size changed");
_Static_assert(GR_SDK_REPORT_PATH_MAX == 1024u, "report path max changed");
_Static_assert(offsetof(gr_result, code) == 0, "gr_result.code offset");
_Static_assert(offsetof(gr_result, detail) >= sizeof(gr_error),
               "gr_result.detail offset");
_Static_assert(GR_OK == 0, "GR_OK value changed");
_Static_assert(GR_ERR_INVALID_ARGUMENT == 1, "error code value changed");
_Static_assert(GR_ERR_UNKNOWN == 99, "GR_ERR_UNKNOWN value changed");
_Static_assert(GR_SDK_VERSION_MAJOR == 1, "major version changed");
_Static_assert(GR_SDK_VERSION == 0x00010000u, "packed version changed");

static int failures = 0;
#define CHECK(cond) do { \
    if (!(cond)) { \
        fprintf(stderr, "FAIL %s:%d: %s\n", __FILE__, __LINE__, #cond); \
        ++failures; \
    } \
} while (0)

int main(int argc, char **argv) {
    if (argc != 5) return 2;
    const char *cli = argv[1];
    const char *pe = argv[2];
    const char *project = argv[3];
    const char *fail_cli = argv[4];

    CHECK(gr_sdk_version() == (unsigned int)GR_SDK_VERSION);
    CHECK(strcmp(gr_sdk_version_string(), GR_SDK_VERSION_STRING) == 0);

    /* Every code has stable, non-NULL, unique names and descriptions. */
    const gr_error codes[] = {
        GR_OK, GR_ERR_INVALID_ARGUMENT, GR_ERR_NOT_FOUND,
        GR_ERR_NOT_DIRECTORY, GR_ERR_OUT_OF_MEMORY, GR_ERR_PIPE_FAILED,
        GR_ERR_FCNTL_FAILED, GR_ERR_FORK_FAILED, GR_ERR_EXEC_FAILED,
        GR_ERR_CLI_FAILED, GR_ERR_REPORT_MISSING, GR_ERR_BUFFER_TOO_SMALL,
        GR_ERR_PATH_TOO_LONG, GR_ERR_WAIT_FAILED, GR_ERR_UNKNOWN
    };
    size_t i, j;
    for (i = 0; i < sizeof codes / sizeof codes[0]; ++i) {
        const char *name = gr_error_name(codes[i]);
        const char *text = gr_error_string(codes[i]);
        CHECK(name != NULL && text != NULL);
        CHECK(name != NULL && strncmp(name, "GR_", 3) == 0);
        CHECK(text != NULL && text[0] != '\0');
        for (j = i + 1; j < sizeof codes / sizeof codes[0]; ++j)
            CHECK(strcmp(name, gr_error_name(codes[j])) != 0);
    }
    CHECK(strcmp(gr_error_name((gr_error)12345), "GR_ERR_UNKNOWN") == 0);

    /* Result accessors, including the NULL-detail safety path. */
    gr_result r;
    r.code = GR_ERR_NOT_FOUND;
    r.detail = "detail-text";
    CHECK(gr_result_code(r) == GR_ERR_NOT_FOUND);
    CHECK(strcmp(gr_result_detail(r), "detail-text") == 0);
    CHECK(gr_result_ok(r) == 0);
    r.detail = NULL;
    CHECK(gr_result_detail(r) != NULL && gr_result_detail(r)[0] == '\0');
    r.code = GR_OK;
    CHECK(gr_result_ok(r) == 1);

    /* No-client paths must be safe and structured. */
    CHECK(gr_last_error(NULL) != NULL);
    r = gr_client_init_ex(NULL, cli);
    CHECK(gr_result_code(r) == GR_ERR_INVALID_ARGUMENT);
    CHECK(gr_result_detail(r) != NULL);
    r = gr_recompile_ex(NULL, pe, project, NULL, 0u);
    CHECK(gr_result_code(r) == GR_ERR_INVALID_ARGUMENT);
    CHECK(gr_client_init(NULL, cli) == -1);
    gr_client_free(NULL);

    /* Validation error codes and message consistency. */
    gr_client c;
    r = gr_client_init_ex(&c, "");
    CHECK(gr_result_code(r) == GR_ERR_INVALID_ARGUMENT);
    r = gr_client_init_ex(&c, "/definitely/not/a/real/cli");
    CHECK(gr_result_code(r) == GR_ERR_NOT_FOUND);
    CHECK(strcmp(gr_result_detail(r), gr_last_error(&c)) == 0);
    CHECK(gr_client_init(&c, "/definitely/not/a/real/cli") == -1);

    r = gr_client_init_ex(&c, cli);
    CHECK(gr_result_ok(r));
    CHECK(gr_result_detail(r) != NULL && gr_result_detail(r)[0] == '\0');

    r = gr_recompile_ex(&c, "/definitely/not/a/real/pe", project, NULL, 0u);
    CHECK(gr_result_code(r) == GR_ERR_NOT_FOUND);
    r = gr_recompile_ex(&c, pe, "/definitely/not/a/real/dir", NULL, 0u);
    CHECK(gr_result_code(r) == GR_ERR_NOT_DIRECTORY);

    char report[GR_SDK_REPORT_PATH_MAX] = {0};
    r = gr_recompile_ex(&c, pe, project, report, sizeof report);
    CHECK(gr_result_ok(r));
    CHECK(strstr(report, "reports/report.json") != NULL);
    CHECK(strcmp(gr_result_detail(r), gr_last_error(&c)) == 0);
    CHECK(gr_recompile(&c, pe, project, NULL, 0u) == 0);

    char tiny[4];
    r = gr_recompile_ex(&c, pe, project, tiny, sizeof tiny);
    CHECK(gr_result_code(r) == GR_ERR_BUFFER_TOO_SMALL);
    CHECK(strstr(gr_result_detail(r), "buffer too small") != NULL);
    r = gr_recompile_ex(&c, pe, project, NULL, 0u);
    CHECK(gr_result_ok(r));
    gr_client_free(&c);

    /* CLI failure: explicit code plus captured untrusted output as detail. */
    gr_client fc;
    r = gr_client_init_ex(&fc, fail_cli);
    CHECK(gr_result_ok(r));
    r = gr_recompile_ex(&fc, pe, project, NULL, 0u);
    CHECK(gr_result_code(r) == GR_ERR_CLI_FAILED);
    CHECK(gr_result_detail(r) != NULL);
    CHECK(strstr(gr_result_detail(r), "CLI_FAILURE_MARKER") != NULL);
    CHECK(strcmp(gr_result_detail(r), gr_last_error(&fc)) == 0);
    CHECK(gr_recompile(&fc, pe, project, NULL, 0u) == -1);
    gr_client_free(&fc);

    if (failures != 0) return 1;
    puts("abi_api: PASS");
    return 0;
}
EOF
$CC $CFLAGS -I"$ROOT/include" \
  "$ROOT/src/gamerecomp_sdk.c" "$TMP/abi_api.c" -o "$TMP/abi_api"
"$TMP/abi_api" "$TMP/cli" /etc/hosts "$TMP/proj" "$TMP/fail_cli"

echo "C SDK: PASS"
