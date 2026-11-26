# C SDK guide

## Files

- Public header: `include/gamerecomp_sdk.h`
- Implementation: `src/gamerecomp_sdk.c`
- Runnable example: `c/examples/basic.c`
- Compile-and-run proof: `tests/test_c_sdk.sh`

The C SDK is portable POSIX C11. It uses `fork`/`execvp` and a pipe to capture
the CLI's output; there is no shell interpolation of paths.

## API reference

```c
typedef struct gr_client {
    char *cli;          /* internal copy of the CLI path */
    char error[512];    /* last error message, never a secret */
} gr_client;

int  gr_client_init(gr_client *c, const char *cli_path);
int  gr_recompile(gr_client *c, const char *pe_path, const char *project_dir,
                  char *report_path, size_t report_path_size);
const char *gr_last_error(const gr_client *c);
void gr_client_free(gr_client *c);
```

### `gr_client_init`

Validates that `cli_path` exists and stores a copy. Returns `0` on success or
`-1` if the client pointer, path, or file is invalid. Call `gr_last_error()` for
the reason.

### `gr_recompile`

Runs:

```
<cli> run <pe_path> --project-dir <project_dir> --no-table --no-progress
```

Returns `0` only when the CLI exits 0 **and** `<project_dir>/reports/report.json`
exists. On success, the report path is copied into `report_path` (if provided).
On failure, returns `-1` and the captured CLI tail is available from
`gr_last_error()`.

### `gr_last_error` / `gr_client_free`

`gr_last_error` returns a pointer valid until the client is modified or freed.
`gr_client_free` releases the internal path copy; always call it.

## Full example

```c
#include "gamerecomp_sdk.h"
#include <stdio.h>

int main(int argc, char **argv) {
    if (argc != 4) {
        fprintf(stderr, "usage: %s <cli> <game.exe> <project-dir>\n", argv[0]);
        return 2;
    }
    gr_client client;
    if (gr_client_init(&client, argv[1]) != 0) {
        fprintf(stderr, "init failed: %s\n", gr_last_error(&client));
        return 1;
    }
    char report[1024];
    if (gr_recompile(&client, argv[2], argv[3], report, sizeof report) != 0) {
        fprintf(stderr, "recompile failed: %s\n", gr_last_error(&client));
        gr_client_free(&client);
        return 1;
    }
    printf("report written: %s\n", report);
    gr_client_free(&client);
    return 0;
}
```

Compile and run:

```sh
cc -std=c11 -Wall -Wextra -Iinclude \
  src/gamerecomp_sdk.c c/examples/basic.c -o basic

./basic /path/to/game-recomp-cli /path/to/game.exe /tmp/project
```

To verify plumbing without a long build:

```sh
sh tests/test_c_sdk.sh
```

This creates a temporary fake CLI that writes a report, compiles with
`-Wall -Wextra`, runs the example, and prints `C SDK: PASS`.

## Error cases worth testing

| Condition | Result |
|---|---|
| CLI path missing | `gr_client_init` returns `-1`, error "CLI binary does not exist" |
| PE file missing | `gr_recompile` returns `-1`, error "PE32 input does not exist" |
| CLI exits non-zero | `-1`, error contains captured CLI output |
| CLI exits 0 but no report | `-1`, error "pipeline produced no report.json" |

## Notes

- The SDK does not parse the report; C callers can open the returned JSON path
  with their own parser.
- A successful return means a pipeline report exists, not that the game runs.
