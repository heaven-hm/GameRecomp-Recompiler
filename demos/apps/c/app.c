/*
 * app.c - C example for the GameRecomp SDK.
 *
 * Usage: app <game-recomp-cli> <pe32-x86.exe> <existing-project-dir>
 *
 * Recompiles the PE through the SDK and prints a one-line proof card.
 * The CLI path is supplied only by this demo harness; SDK callers normally
 * use gr_client_init_default() and pass just the game and options.
 */

#include "gamerecomp_sdk.h"

#include <stdio.h>

static int usage(void) {
    fprintf(stderr, "usage: app <cli> <pe> <project>\n");
    return 2;
}

static int fail(const char *step, gr_result result) {
    fprintf(stderr, "%s: %s\n", step, gr_result_detail(result));
    return 1;
}

static int report_size(const char *path, long *size) {
    FILE *file = fopen(path, "rb");
    if (file == NULL) {
        return -1;
    }
    if (fseek(file, 0, SEEK_END) != 0) {
        fclose(file);
        return -1;
    }
    *size = ftell(file);
    fclose(file);
    return *size < 0 ? -1 : 0;
}

int main(int argc, char **argv) {
    if (argc != 4) {
        return usage();
    }

    const char *cli_path = argv[1];
    const char *pe_path = argv[2];
    const char *project_dir = argv[3];

    gr_client client;
    gr_result result = gr_client_init_ex(&client, cli_path);
    if (!gr_result_ok(result)) {
        return fail("init", result);
    }

    char report_path[GR_SDK_REPORT_PATH_MAX] = {0};
    const gr_build_options options = GR_BUILD_OPTIONS_DEFAULT;
    result = gr_build(&client, pe_path, project_dir, &options,
                      report_path, sizeof report_path);
    if (!gr_result_ok(result)) {
        const int status = fail("build", result);
        gr_client_free(&client);
        return status;
    }

    long bytes = 0;
    if (report_size(report_path, &bytes) != 0) {
        fprintf(stderr, "report: cannot read %s\n", report_path);
        gr_client_free(&client);
        return 1;
    }

    printf("APP_OK sdk=c report=%s bytes=%ld\n", report_path, bytes);
    gr_client_free(&client);
    return 0;
}
