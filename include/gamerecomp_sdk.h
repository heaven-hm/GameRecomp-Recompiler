/*
 * gamerecomp_sdk.h - public C API for the GameReComp recompilation pipeline.
 *
 * This SDK is a thin POSIX C11 client: it validates inputs, runs
 * `game-recomp-cli run <pe> --project-dir <dir>` and checks that
 * `<dir>/reports/report.json` was produced. No recompiler logic lives here.
 *
 * API stability
 * -------------
 * The "legacy" API (gr_client, gr_client_init, gr_recompile, gr_last_error,
 * gr_client_free) is frozen: its struct layout and signatures do not change.
 * The structured API (gr_error, gr_result, *_ex) is additive and was introduced
 * in SDK 1.0.0. Check GR_SDK_VERSION at compile time and gr_sdk_version() at
 * run time. Full contract: docs/C_API_REFERENCE.md.
 */
#ifndef GAMERECOMP_SDK_H
#define GAMERECOMP_SDK_H

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ------------------------------------------------------------------------- *
 * Version
 * ------------------------------------------------------------------------- */

#define GR_SDK_VERSION_MAJOR 1
#define GR_SDK_VERSION_MINOR 0
#define GR_SDK_VERSION_PATCH 0
/* Human-readable, stable for a given release. */
#define GR_SDK_VERSION_STRING "1.0.0"
/* Packed as (major << 16) | (minor << 8) | patch. */
#define GR_SDK_VERSION ((GR_SDK_VERSION_MAJOR << 16) | \
                        (GR_SDK_VERSION_MINOR << 8) | GR_SDK_VERSION_PATCH)

/* Returns the same value as the GR_SDK_VERSION macro the library was built
 * with. Useful for detecting a header/library version mismatch at run time. */
unsigned int gr_sdk_version(void);

/* Returns the same string as GR_SDK_VERSION_STRING. Static storage, valid
 * forever; never free or modify it. */
const char *gr_sdk_version_string(void);

/* ------------------------------------------------------------------------- *
 * Error codes and structured results
 * ------------------------------------------------------------------------- */

/* Explicit, stable numeric error codes. Values are part of the ABI: append
 * new codes, never renumber existing ones. GR_OK is the only success value. */
typedef enum gr_error {
    GR_OK = 0,
    GR_ERR_INVALID_ARGUMENT = 1,  /* NULL/empty required argument */
    GR_ERR_NOT_FOUND = 2,         /* CLI binary or PE32 input does not exist */
    GR_ERR_NOT_DIRECTORY = 3,     /* project_dir is not an existing directory */
    GR_ERR_OUT_OF_MEMORY = 4,     /* internal allocation failed */
    GR_ERR_PIPE_FAILED = 5,       /* pipe(2) failed */
    GR_ERR_FCNTL_FAILED = 6,      /* fcntl(2) failed */
    GR_ERR_FORK_FAILED = 7,       /* fork(2) failed */
    GR_ERR_EXEC_FAILED = 8,       /* execvp of the CLI failed */
    GR_ERR_CLI_FAILED = 9,        /* CLI exited non-zero or was signalled */
    GR_ERR_REPORT_MISSING = 10,   /* no reports/report.json after a zero exit */
    GR_ERR_BUFFER_TOO_SMALL = 11, /* report_path buffer too small */
    GR_ERR_PATH_TOO_LONG = 12,    /* constructed report path exceeds 1023 chars */
    GR_ERR_WAIT_FAILED = 13,      /* waitpid(2) failed */
    GR_ERR_UNKNOWN = 99           /* reserved for future codes */
} gr_error;

/* Stable machine-readable token for a code, e.g. "GR_ERR_NOT_FOUND".
 * Never NULL. Static storage, valid forever. Unknown values map to
 * "GR_ERR_UNKNOWN". */
const char *gr_error_name(gr_error code);

/* Short human-readable description, e.g. "CLI binary or input not found".
 * Never NULL. Static storage, valid forever. */
const char *gr_error_string(gr_error code);

/* Return value of the structured (*_ex) API. Passed and returned by value. */
typedef struct gr_result {
    gr_error code;       /* GR_OK on success, otherwise a gr_error value */
    const char *detail;  /* borrowed message (see ownership rules below) */
} gr_result;

/* Convenience accessors; they never dereference a NULL result pointer. */
gr_error gr_result_code(gr_result r);
const char *gr_result_detail(gr_result r);
int gr_result_ok(gr_result r); /* 1 iff r.code == GR_OK */

/* ------------------------------------------------------------------------- *
 * Client and buffer sizes
 * ------------------------------------------------------------------------- */

/* Size of gr_client.error. Exposed so callers can reason about message
 * truncation when displaying errors. */
#define GR_SDK_ERROR_BUFFER_SIZE 512u

/* A report_path buffer of this many bytes is always large enough for the
 * generated path. Smaller buffers are supported but may return
 * GR_ERR_BUFFER_TOO_SMALL. */
#define GR_SDK_REPORT_PATH_MAX 1024u

/* Caller-allocated value type. Layout is frozen: `cli` at offset 0 and
 * `error` at offset sizeof(char *), with a total size of sizeof(char *)+512.
 * Do not memcpy, serialize, or subclass this struct; use the API instead. */
typedef struct gr_client {
    char *cli;          /* internal copy of the CLI path; owned by the client */
    char error[512];    /* last error message; never contains secrets */
} gr_client;

/* ------------------------------------------------------------------------- *
 * Legacy API (unchanged, kept for source and binary compatibility)
 * ------------------------------------------------------------------------- */

/* Validate and retain a copy of cli_path. Returns 0 on success, -1 on error.
 * Requires c != NULL and a non-empty cli_path that exists on disk.
 * On failure the reason is available from gr_last_error(c). */
int gr_client_init(gr_client *c, const char *cli_path);

/* Recompile pe_path into the existing directory project_dir. Returns 0 on
 * success and, when report_path != NULL and report_path_size > 0, writes a
 * NUL-terminated report path into that caller-owned buffer. Returns -1 on any
 * failure with the reason in gr_last_error(c). Both pe_path and project_dir
 * are borrowed and are not retained. */
int gr_recompile(gr_client *c, const char *pe_path, const char *project_dir,
                 char *report_path, size_t report_path_size);

/* Borrowed pointer to the last error message. Valid until the next operation
 * on c or until gr_client_free(c); never NULL (empty string on success).
 * With c == NULL returns a static "no client" string. */
const char *gr_last_error(const gr_client *c);

/* Release the internal copy of the CLI path. Safe to call with NULL. After
 * this call c must not be used until it is re-initialized. */
void gr_client_free(gr_client *c);

/* ------------------------------------------------------------------------- *
 * Structured API (additive, introduced in 1.0.0)
 * ------------------------------------------------------------------------- */

/* Same contract as the legacy functions, but return a gr_result carrying an
 * explicit gr_error code. On failure r.detail is also stored in c->error, so
 * gr_last_error(c) stays consistent with the structured result. On success
 * r.detail points at the now-empty c->error. With c == NULL, r.detail points
 * at a static message and r.code is GR_ERR_INVALID_ARGUMENT. */
gr_result gr_client_init_ex(gr_client *c, const char *cli_path);
gr_result gr_recompile_ex(gr_client *c, const char *pe_path,
                          const char *project_dir,
                          char *report_path, size_t report_path_size);

/* Ownership rules (apply to both APIs):
 *  - gr_client is caller-allocated. Initialize it exactly once, then call
 *    gr_client_free exactly once. There is no implicit global state.
 *  - cli_path is copied; pe_path and project_dir are borrowed for the call.
 *  - report_path is a caller-owned output buffer; the SDK only writes to it
 *    on success and always NUL-terminates within report_path_size.
 *  - gr_last_error() and gr_result.detail are borrowed views into c->error.
 *    Do not free or retain them past the next call on c or gr_client_free(c).
 *  - gr_error_name(), gr_error_string() and gr_sdk_version_string() return
 *    static literals with process lifetime.
 *  - A single gr_client is not safe for concurrent use. Distinct clients may
 *    be used concurrently; the SDK keeps no mutable global state.
 */

#ifdef __cplusplus
}
#endif
#endif /* GAMERECOMP_SDK_H */
