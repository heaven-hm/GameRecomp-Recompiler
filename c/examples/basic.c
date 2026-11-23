/* Example: call GameReComp from C. */
#include "gamerecomp_sdk.h"
#include <stdio.h>

int main(int argc, char **argv) {
    if (argc != 4) { fprintf(stderr, "usage: basic <cli> <pe> <project>\n"); return 2; }
    gr_client c;
    if (gr_client_init(&c, argv[1]) != 0) { fprintf(stderr, "%s\n", gr_last_error(&c)); return 1; }
    char report[1024] = {0};
    int rc = gr_recompile(&c, argv[2], argv[3], report, sizeof report);
    if (rc != 0) fprintf(stderr, "%s\n", gr_last_error(&c));
    else printf("report=%s\n", report);
    gr_client_free(&c);
    return rc;
}
