/* Port one PE32 app through the C SDK with explicit input/target options. */
#include "gamerecomp_sdk.h"
#include <stdio.h>
#include <stdlib.h>
int main(int argc, char **argv) {
    if (argc != 4) { fprintf(stderr, "usage: app <direction> <pe> <project>\n"); return 2; }
    gr_client c; char report[GR_SDK_REPORT_PATH_MAX] = {0};
    const char *cli = getenv("GAMERECOMP_CLI");
    gr_result r = gr_client_init_ex(&c, cli && *cli ? cli : "game-recomp-cli");
    if (!gr_result_ok(r)) { fprintf(stderr, "init: %s\n", gr_result_detail(r)); return 1; }
    gr_build_options o = GR_BUILD_OPTIONS_DEFAULT;
    o.input_os = "windows"; o.input_arch = "x86";
    o.target_os = "host"; o.target_arch = "host";
    o.out_mode = "full"; o.skip_stages = "verify";
    r = gr_build(&c, argv[2], argv[3], &o, report, sizeof report);
    if (!gr_result_ok(r)) {
        printf("PORT_DEFERRED direction=%s reason=%s\n", argv[1], gr_result_detail(r));
        gr_client_free(&c); return 0;
    }
    printf("PORT_OK direction=%s report=%s\n", argv[1], report);
    gr_client_free(&c); return 0;
}
