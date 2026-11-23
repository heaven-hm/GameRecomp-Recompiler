/* Thin client: executes game-recomp-cli and reads its report. No recompiler
 * logic lives here; all lifting/verification belongs to GameReComp. */
#include "gamerecomp_sdk.h"

#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <unistd.h>

static void seterr(gr_client *c, const char *msg) {
    if (c) snprintf(c->error, sizeof c->error, "%s", msg);
}

static int exists(const char *p) {
    struct stat st;
    return p && stat(p, &st) == 0;
}

int gr_client_init(gr_client *c, const char *cli_path) {
    if (!c || !cli_path || !*cli_path) return -1;
    memset(c, 0, sizeof *c);
    if (!exists(cli_path)) { seterr(c, "CLI binary does not exist"); return -1; }
    c->cli = strdup(cli_path);
    return c->cli ? 0 : -1;
}

int gr_recompile(gr_client *c, const char *pe_path, const char *project_dir,
                 char *report_path, size_t report_path_size) {
    if (!c || !c->cli || !pe_path || !project_dir) { seterr(c, "invalid argument"); return -1; }
    if (!exists(pe_path)) { seterr(c, "PE32 input does not exist"); return -1; }

    int pipefd[2];
    if (pipe(pipefd) != 0) { seterr(c, "pipe failed"); return -1; }
    pid_t pid = fork();
    if (pid < 0) { close(pipefd[0]); close(pipefd[1]); seterr(c, "fork failed"); return -1; }
    if (pid == 0) {
        close(pipefd[0]);
        dup2(pipefd[1], STDOUT_FILENO);
        dup2(pipefd[1], STDERR_FILENO);
        close(pipefd[1]);
        char *argv[] = { c->cli, "run", (char *)pe_path,
                         "--project-dir", (char *)project_dir,
                         "--no-table", "--no-progress", NULL };
        execvp(c->cli, argv);
        _exit(127);
    }
    close(pipefd[1]);
    char buf[4096];
    ssize_t n;
    size_t total = 0;
    while ((n = read(pipefd[0], buf, sizeof buf)) > 0) {
        if (total < sizeof c->error - 1) {
            size_t room = sizeof c->error - 1 - total;
            size_t take = (size_t)n < room ? (size_t)n : room;
            memcpy(c->error + total, buf, take);
            total += take;
            c->error[total] = 0;
        }
    }
    close(pipefd[0]);
    int status = 0;
    if (waitpid(pid, &status, 0) != pid) { seterr(c, "waitpid failed"); return -1; }
    if (!WIFEXITED(status) || WEXITSTATUS(status) != 0) {
        if (!total) seterr(c, "GameReComp CLI exited non-zero");
        return -1;
    }
    char rp[1024];
    snprintf(rp, sizeof rp, "%s/reports/report.json", project_dir);
    if (!exists(rp)) { seterr(c, "pipeline produced no report.json"); return -1; }
    if (report_path && report_path_size) {
        snprintf(report_path, report_path_size, "%s", rp);
    }
    c->error[0] = 0;
    return 0;
}

const char *gr_last_error(const gr_client *c) { return c ? c->error : "no client"; }

void gr_client_free(gr_client *c) {
    if (!c) return;
    free(c->cli);
    c->cli = NULL;
}
