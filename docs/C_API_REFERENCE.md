# C API reference (GameReComp SDK)

This is the normative contract for `include/gamerecomp_sdk.h`, implemented by
`src/gamerecomp_sdk.c`. The C SDK is a **thin client**: it validates inputs,
executes `game-recomp-cli run <pe> --project-dir <dir> --no-table
--no-progress`, and requires `<dir>/reports/report.json` to exist. It contains
no recompiler, lifter or PE parser. For the language-neutral boundary see
[ARCHITECTURE.md](ARCHITECTURE.md); for a short tour see [C.md](C.md).

## 1. Versioning and compatibility

| Macro / function | Meaning |
|---|---|
| `GR_SDK_VERSION_MAJOR`, `GR_SDK_VERSION_MINOR`, `GR_SDK_VERSION_PATCH` | Release components as integers. |
| `GR_SDK_VERSION` | `(major << 16) \| (minor << 8) \| patch`. |
| `GR_SDK_VERSION_STRING` | `"1.0.0"`, stable for a release. |
| `gr_sdk_version(void)` | Packed version the **library** was built with; compare with the macro to detect a header/library mismatch. |
| `gr_sdk_version_string(void)` | Same string as the macro; static storage. |

The current C API version is **1.0.0**. Compatibility rules:

- The legacy API (`gr_client`, `gr_client_init`, `gr_recompile`,
  `gr_last_error`, `gr_client_free`) is frozen. Its signatures and struct
  layout do not change in this major version.
- New error codes are appended; existing numeric values never change.
- New functions are added, never removed or given new semantics.
- Source compiled against this header and the library built from the same
  header are binary-compatible. Do not mix headers from different versions in
  one process.

## 2. Error codes

`gr_error` is an explicit enum with stable values. `GR_OK` (0) is the only
success value; every other value is a failure.

| Code | Value | Meaning / typical cause |
|---|---:|---|
| `GR_OK` | 0 | Operation succeeded. |
| `GR_ERR_INVALID_ARGUMENT` | 1 | NULL/empty required argument, or uninitialized client. |
| `GR_ERR_NOT_FOUND` | 2 | CLI binary or PE32 input does not exist. |
| `GR_ERR_NOT_DIRECTORY` | 3 | `project_dir` is not an existing directory. |
| `GR_ERR_OUT_OF_MEMORY` | 4 | Internal allocation of the CLI path failed. |
| `GR_ERR_PIPE_FAILED` | 5 | `pipe(2)` failed. |
| `GR_ERR_FCNTL_FAILED` | 6 | `fcntl(2)` failed while setting close-on-exec. |
| `GR_ERR_FORK_FAILED` | 7 | `fork(2)` failed. |
| `GR_ERR_EXEC_FAILED` | 8 | `execvp` of the CLI failed (e.g. permission denied). |
| `GR_ERR_CLI_FAILED` | 9 | CLI exited non-zero or was terminated by a signal. |
| `GR_ERR_REPORT_MISSING` | 10 | CLI exited 0 but no `reports/report.json` appeared. |
| `GR_ERR_BUFFER_TOO_SMALL` | 11 | `report_path` buffer could not hold the generated path. |
| `GR_ERR_PATH_TOO_LONG` | 12 | Generated path exceeds `GR_SDK_REPORT_PATH_MAX - 1`. |
| `GR_ERR_WAIT_FAILED` | 13 | `waitpid(2)` failed. |
| `GR_ERR_UNKNOWN` | 99 | Reserved; returned by the name/string helpers for unrecognized values. |

```c
const char *gr_error_name(gr_error code);    /* "GR_ERR_NOT_FOUND", ... */
const char *gr_error_string(gr_error code);  /* short human description */
```

Both return never-NULL static string literals (process lifetime). They are
safe for unknown/out-of-range values: those map to `GR_ERR_UNKNOWN` /
`"unknown error"`.

## 3. Structured results

```c
typedef struct gr_result {
    gr_error code;       /* GR_OK or a gr_error failure code */
    const char *detail;  /* borrowed message, never NULL from the SDK */
} gr_result;

gr_error    gr_result_code(gr_result r);
const char *gr_result_detail(gr_result r);
int         gr_result_ok(gr_result r);   /* 1 iff r.code == GR_OK */
```

The result is returned **by value**; there is nothing to free. On failure,
`detail` is the same text as `gr_last_error(c)` (the captured CLI tail when the
CLI produced output). On success it points at the now-empty `c->error`, so
`detail[0] == '\0'`. When the client pointer itself is NULL, `detail` points at
a static `"no client"` message. `gr_result_detail` also normalizes a manually
constructed result with a NULL `detail` to an empty static string.

Structured operations:

```c
gr_result gr_client_init_ex(gr_client *c, const char *cli_path);
gr_result gr_recompile_ex(gr_client *c, const char *pe_path,
                          const char *project_dir,
                          char *report_path, size_t report_path_size);
```

These have exactly the same behavior as their legacy counterparts except for
the return type: they give you the machine-readable code without string
parsing. They never write outside `report_path_size`.

## 4. Legacy API (unchanged)

```c
typedef struct gr_client {
    char *cli;          /* internal copy of the CLI path */
    char error[512];    /* last error message; never contains secrets */
} gr_client;

int         gr_client_init(gr_client *c, const char *cli_path);
int         gr_recompile(gr_client *c, const char *pe_path,
                         const char *project_dir,
                         char *report_path, size_t report_path_size);
const char *gr_last_error(const gr_client *c);
void        gr_client_free(gr_client *c);
```

`gr_client_init` returns 0 on success, -1 otherwise; it validates that
`cli_path` is non-empty and exists, then makes a private copy.
`gr_recompile` returns 0 only when the CLI exits 0 **and** the report file
exists. On any failure it returns -1 and `gr_last_error(c)` holds the reason
(capped at `GR_SDK_ERROR_BUFFER_SIZE - 1` bytes plus a NUL). The legacy
functions are thin wrappers over the structured ones, so their behavior and
messages stay in sync with `gr_result.code` / `gr_result.detail`.

## 5. Safe buffers

| Macro | Value | Use |
|---|---:|---|
| `GR_SDK_ERROR_BUFFER_SIZE` | 512 | Size of `gr_client.error`; messages are truncated with a trailing NUL. |
| `GR_SDK_REPORT_PATH_MAX` | 1024 | Size of a buffer guaranteed to hold any generated report path. |

- `report_path` is optional: pass `NULL` with size 0 to skip the copy.
- Otherwise the buffer must have at least `strlen(path) + 1` bytes; a smaller
  buffer yields `GR_ERR_BUFFER_TOO_SMALL` and leaves the buffer untouched by
  the SDK. A buffer of `GR_SDK_REPORT_PATH_MAX` bytes always succeeds.
- Captured CLI output is appended byte-by-byte into `c->error` with explicit
  bounds, so hostile/untrusted CLI output cannot overflow it. `EINTR` during
  the read is retried rather than truncating output.
- Paths are passed to `execvp` as separate `argv` entries; no shell is invoked
  and no path is interpolated into a command string.
- `error` never stores CLI configuration or secrets. Treat its content as
  untrusted text and sanitize it before logging or sharing (see
  [SECURITY.md](SECURITY.md)).

## 6. Ownership and lifetime

| Item | Ownership |
|---|---|
| `gr_client` | Caller-allocated value (stack or heap). Initialize once with `gr_client_init*`; call `gr_client_free` exactly once afterwards. There is no opaque allocation and no global state. |
| `cli_path` | Borrowed; the SDK copies it into `c->cli`. |
| `pe_path`, `project_dir` | Borrowed for the duration of the call only. |
| `report_path` | Caller-owned output buffer; written only on success, always NUL-terminated within the given size. |
| `gr_last_error(c)`, `gr_result.detail` | Borrowed views of `c->error`. Valid until the next operation on `c` or `gr_client_free(c)`. Never free or retain them. |
| `gr_error_name`, `gr_error_string`, `gr_sdk_version_string`, `gr_last_error(NULL)` | Static literals with process lifetime. |

A `gr_client` must not be used concurrently by multiple threads. Distinct
clients may be used concurrently: the SDK keeps no mutable global state and
only uses process-level POSIX primitives. Do not re-initialize a client that
still owns a `cli` copy (`gr_client_free` first) or the previous copy leaks.

## 7. Typical usage

```c
#include "gamerecomp_sdk.h"
#include <stdio.h>

int main(int argc, char **argv) {
    if (argc != 4) return 2;

    gr_client c;
    gr_result r = gr_client_init_ex(&c, argv[1]);
    if (!gr_result_ok(r)) {
        fprintf(stderr, "init [%s]: %s\n",
                gr_error_name(r.code), r.detail);
        return 1;
    }

    char report[GR_SDK_REPORT_PATH_MAX] = {0};
    r = gr_recompile_ex(&c, argv[2], argv[3], report, sizeof report);
    if (!gr_result_ok(r))
        fprintf(stderr, "recompile [%s]: %s\n",
                gr_error_name(r.code), r.detail);
    else
        printf("report=%s\n", report);

    gr_client_free(&c);
    return gr_result_ok(r) ? 0 : 1;
}
```

Build and verify:

```sh
cc -std=c11 -Wall -Wextra -Wpedantic -Werror -Iinclude \
  src/gamerecomp_sdk.c c/examples/basic.c -o basic
sh tests/test_c_sdk.sh
```

`tests/test_c_sdk.sh` compiles both the example and a dedicated ABI/API
conformance program with those strict flags, then exercises success, CLI and
validation failures, buffer overflow prevention, every error-name/string
entry, version macros, and the frozen struct layout.

## 8. Limitations

- A successful call means a pipeline report exists, **not** that the game
  renders or plays correctly.
- `gr_last_error` is message-only for ABI reasons; to obtain a numeric code
  for an already-completed legacy call, use the `*_ex` variants. The legacy
  and structured entry points share the same error buffer and behavior.
- PE32+ x64, .NET and Win16/NE are refused by the underlying GameReComp
  pipeline; the SDK reports that as `GR_ERR_CLI_FAILED` with the CLI output.
- The SDK is POSIX C11 (`fork`/`execvp`/pipes); it is not a Windows API.
