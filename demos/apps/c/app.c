/* Working C app: recompile through the SDK and print a verified report card. */
#include "gamerecomp_sdk.h"
#include <stdio.h>
#include <string.h>
int main(int argc, char **argv) {
    if (argc != 4) { fprintf(stderr, "usage: app <cli> <pe> <project>\n"); return 2; }
    gr_client c; char report[1024] = {0};
    gr_result r = gr_client_init_ex(&c, argv[1]);
    if (gr_result_code(r) != GR_OK) { fprintf(stderr, "init: %s\n", gr_result_detail(r)); return 1; }
    r = gr_recompile_ex(&c, argv[2], argv[3], report, sizeof report);
    if (gr_result_code(r) != GR_OK) { fprintf(stderr, "recompile: %s\n", gr_result_detail(r)); gr_client_free(&c); return 1; }
    FILE *f = fopen(report, "rb");
    if (!f) { fprintf(stderr, "report missing\n"); gr_client_free(&c); return 1; }
    fseek(f, 0, SEEK_END); long bytes = ftell(f); fclose(f);
    printf("APP_OK sdk=c report=%s bytes=%ld\n", report, bytes);
    gr_client_free(&c);
    return 0;
}
