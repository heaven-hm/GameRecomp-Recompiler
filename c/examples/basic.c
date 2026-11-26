/* Example: call GameReComp from C using the structured (gr_result) API.
 * Usage: basic <cli> <pe> <project>
 *        basic --version                       (print SDK version)          */
#include "gamerecomp_sdk.h"

#include <stdio.h>
#include <string.h>

static int fail(const char *what, gr_result r) {
    fprintf(stderr, "%s failed [%s]: %s\n", what,
            gr_error_name(gr_result_code(r)), gr_result_detail(r));
    return 1;
}

int main(int argc, char **argv) {
    if (argc == 2 && strcmp(argv[1], "--version") == 0) {
        printf("gamerecomp C SDK %s (0x%08x)\n", gr_sdk_version_string(),
               gr_sdk_version());
        return 0;
    }
    if (argc != 4) {
        fprintf(stderr, "usage: basic <cli> <pe> <project>\n");
        return 2;
    }

    gr_client c;
    gr_result r = gr_client_init_ex(&c, argv[1]);
    if (!gr_result_ok(r)) return fail("init", r);

    char report[GR_SDK_REPORT_PATH_MAX] = {0};
    r = gr_recompile_ex(&c, argv[2], argv[3], report, sizeof report);
    if (!gr_result_ok(r)) {
        int rc = fail("recompile", r);
        gr_client_free(&c);
        return rc;
    }

    printf("report=%s\n", report);
    gr_client_free(&c);
    return 0;
}
