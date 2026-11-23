/* gamerecomp_sdk.h - minimal C client for the GameReComp CLI. */
#ifndef GAMERECOMP_SDK_H
#define GAMERECOMP_SDK_H

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct gr_client {
    char *cli;          /* path to game-recomp-cli */
    char error[512];    /* last error, never contains secrets */
} gr_client;

/* Validate and retain the CLI path. Returns 0 on success, -1 on error. */
int gr_client_init(gr_client *c, const char *cli_path);

/* Recompile a PE32 x86 file into project_dir.
 * Returns 0 on success and copies the report path into report_path when it
 * exists; returns -1 on any failure with the reason in gr_last_error(). */
int gr_recompile(gr_client *c, const char *pe_path, const char *project_dir,
                 char *report_path, size_t report_path_size);

const char *gr_last_error(const gr_client *c);
void gr_client_free(gr_client *c);

#ifdef __cplusplus
}
#endif
#endif
