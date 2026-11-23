/* Thin client: executes game-recomp-cli and reads its report. No recompiler
 * logic lives here; all lifting/verification belongs to GameReComp. */
#include "gamerecomp_sdk.h"

#include <errno.h>
#include <fcntl.h>
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

static int is_dir(const char *p) {
    struct stat st;
    return p && stat(p, &st) == 0 && S_ISDIR(st.st_mode);
}

/* strdup is POSIX, not C11; keep the SDK buildable with strict -std=c11. */
static char *dup_string(const char *s) {
    size_t n = strlen(s) + 1u;
    char *out = (char *)malloc(n);
    if (out) memcpy(out, s, n);
    return out;
}

int gr_client_init(gr_client *c, const char *cli_path) {
    if (!c || !cli_path || !*cli_path) return -1;
    memset(c, 0, sizeof *c);
    if (!exists(cli_path)) { seterr(c, "CLI binary does not exist"); return -1; }
    c->cli = dup_string(cli_path);
    return c->cli ? 0 : -1;
}

int gr_recompile(gr_client *c, const char *pe_path, const char *project_dir,
                 char *report_path, size_t report_path_size) {
    if (!c || !c->cli || !pe_path || !project_dir) { seterr(c, "invalid argument"); return -1; }
    if (!exists(pe_path)) { seterr(c, "PE32 input does not exist"); return -1; }
    if (!is_dir(project_dir)) { seterr(c, "project_dir is not an existing directory"); return -1; }

    int pipefd[2];
    if (pipe(pipefd) != 0) { seterr(c, "pipe failed"); return -1; }

    /* A second, close-on-exec pipe carries execvp()'s errno back to the
     * parent. A successful exec closes it, so the parent just sees EOF. */
    int errpipe[2];
    if (pipe(errpipe) != 0) {
        close(pipefd[0]); close(pipefd[1]);
        seterr(c, "pipe failed"); return -1;
    }
    int fdflags = fcntl(errpipe[1], F_GETFD);
    if (fdflags < 0 || fcntl(errpipe[1], F_SETFD, fdflags | FD_CLOEXEC) < 0) {
        close(pipefd[0]); close(pipefd[1]);
        close(errpipe[0]); close(errpipe[1]);
        seterr(c, "fcntl failed"); return -1;
    }

    pid_t pid = fork();
    if (pid < 0) {
        close(pipefd[0]); close(pipefd[1]);
        close(errpipe[0]); close(errpipe[1]);
        seterr(c, "fork failed"); return -1;
    }
    if (pid == 0) {
        close(pipefd[0]);
        close(errpipe[0]);
        dup2(pipefd[1], STDOUT_FILENO);
        dup2(pipefd[1], STDERR_FILENO);
        if (pipefd[1] != STDOUT_FILENO && pipefd[1] != STDERR_FILENO)
            close(pipefd[1]);
        char *argv[] = { c->cli, "run", (char *)pe_path,
                         "--project-dir", (char *)project_dir,
                         "--no-table", "--no-progress", NULL };
        execvp(c->cli, argv);
        int child_errno = errno;
        ssize_t ignored = write(errpipe[1], &child_errno, sizeof child_errno);
        (void)ignored;
        _exit(127);
    }
    close(pipefd[1]);
    close(errpipe[1]);

    char buf[4096];
    ssize_t n;
    size_t total = 0;
    while ((n = read(pipefd[0], buf, sizeof buf)) != 0) {
        if (n < 0) {
            if (errno == EINTR) continue;  /* a signal must not truncate output */
            break;
        }
        if (total < sizeof c->error - 1) {
            size_t room = sizeof c->error - 1 - total;
            size_t take = (size_t)n < room ? (size_t)n : room;
            memcpy(c->error + total, buf, take);
            total += take;
            c->error[total] = 0;
        }
    }
    close(pipefd[0]);

    /* The child either exec'd (EOF here) or wrote its exec errno before
     * exiting. Read after draining the output pipe, so this never blocks. */
    int exec_errno = 0;
    {
        int child_errno = 0;
        ssize_t r;
        do {
            r = read(errpipe[0], &child_errno, sizeof child_errno);
        } while (r < 0 && errno == EINTR);
        if (r == (ssize_t)sizeof child_errno) exec_errno = child_errno;
    }
    close(errpipe[0]);

    int status = 0;
    pid_t w;
    do {
        w = waitpid(pid, &status, 0);
    } while (w < 0 && errno == EINTR);
    if (w != pid) { seterr(c, "waitpid failed"); return -1; }

    if (exec_errno != 0) {
        char msg[sizeof c->error];
        snprintf(msg, sizeof msg, "failed to execute CLI '%s': %s",
                 c->cli, strerror(exec_errno));
        seterr(c, msg);
        return -1;
    }
    if (!WIFEXITED(status) || WEXITSTATUS(status) != 0) {
        if (!total) seterr(c, "GameReComp CLI exited non-zero");
        return -1;
    }

    char rp[1024];
    int rp_len = snprintf(rp, sizeof rp, "%s/reports/report.json", project_dir);
    if (rp_len < 0 || (size_t)rp_len >= sizeof rp) {
        seterr(c, "report path too long"); return -1;
    }
    if (!exists(rp)) { seterr(c, "pipeline produced no report.json"); return -1; }
    if (report_path && report_path_size) {
        int copy_len = snprintf(report_path, report_path_size, "%s", rp);
        if (copy_len < 0 || (size_t)copy_len >= report_path_size) {
            seterr(c, "report_path buffer too small"); return -1;
        }
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
