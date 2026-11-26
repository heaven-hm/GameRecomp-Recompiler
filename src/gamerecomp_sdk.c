/* Thin client: executes game-recomp-cli and reads its report. No recompiler
 * logic lives here; all lifting/verification belongs to GameReComp. */
#ifndef _POSIX_C_SOURCE
#define _POSIX_C_SOURCE 200809L
#endif

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

/* Static, process-lifetime strings used when there is no client to borrow
 * from (gr_last_error(NULL), *_ex with a NULL client). */
static const char k_no_client[] = "no client";
static const char k_empty_message[] = "";

static void seterr(gr_client *c, const char *msg) {
    if (c) snprintf(c->error, sizeof c->error, "%s", msg);
}

/* Build a structured result. msg == NULL means "keep whatever is already in
 * c->error" (used when the captured CLI tail is the actual error message). */
static gr_result make_result(gr_client *c, gr_error code, const char *msg) {
    gr_result r;
    r.code = code;
    r.detail = c ? c->error : k_no_client;
    if (c && msg) seterr(c, msg);
    return r;
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

unsigned int gr_sdk_version(void) { return (unsigned int)GR_SDK_VERSION; }
const char *gr_sdk_version_string(void) { return GR_SDK_VERSION_STRING; }

const char *gr_error_name(gr_error code) {
    switch (code) {
    case GR_OK: return "GR_OK";
    case GR_ERR_INVALID_ARGUMENT: return "GR_ERR_INVALID_ARGUMENT";
    case GR_ERR_NOT_FOUND: return "GR_ERR_NOT_FOUND";
    case GR_ERR_NOT_DIRECTORY: return "GR_ERR_NOT_DIRECTORY";
    case GR_ERR_OUT_OF_MEMORY: return "GR_ERR_OUT_OF_MEMORY";
    case GR_ERR_PIPE_FAILED: return "GR_ERR_PIPE_FAILED";
    case GR_ERR_FCNTL_FAILED: return "GR_ERR_FCNTL_FAILED";
    case GR_ERR_FORK_FAILED: return "GR_ERR_FORK_FAILED";
    case GR_ERR_EXEC_FAILED: return "GR_ERR_EXEC_FAILED";
    case GR_ERR_CLI_FAILED: return "GR_ERR_CLI_FAILED";
    case GR_ERR_REPORT_MISSING: return "GR_ERR_REPORT_MISSING";
    case GR_ERR_BUFFER_TOO_SMALL: return "GR_ERR_BUFFER_TOO_SMALL";
    case GR_ERR_PATH_TOO_LONG: return "GR_ERR_PATH_TOO_LONG";
    case GR_ERR_WAIT_FAILED: return "GR_ERR_WAIT_FAILED";
    case GR_ERR_UNKNOWN: return "GR_ERR_UNKNOWN";
    default: return "GR_ERR_UNKNOWN";
    }
}

const char *gr_error_string(gr_error code) {
    switch (code) {
    case GR_OK: return "success";
    case GR_ERR_INVALID_ARGUMENT: return "invalid argument";
    case GR_ERR_NOT_FOUND: return "CLI binary or input not found";
    case GR_ERR_NOT_DIRECTORY: return "project_dir is not an existing directory";
    case GR_ERR_OUT_OF_MEMORY: return "out of memory";
    case GR_ERR_PIPE_FAILED: return "pipe failed";
    case GR_ERR_FCNTL_FAILED: return "fcntl failed";
    case GR_ERR_FORK_FAILED: return "fork failed";
    case GR_ERR_EXEC_FAILED: return "failed to execute CLI";
    case GR_ERR_CLI_FAILED: return "GameReComp CLI exited non-zero";
    case GR_ERR_REPORT_MISSING: return "pipeline produced no report.json";
    case GR_ERR_BUFFER_TOO_SMALL: return "report_path buffer too small";
    case GR_ERR_PATH_TOO_LONG: return "report path too long";
    case GR_ERR_WAIT_FAILED: return "waitpid failed";
    case GR_ERR_UNKNOWN: return "unknown error";
    default: return "unknown error";
    }
}

gr_error gr_result_code(gr_result r) { return r.code; }
const char *gr_result_detail(gr_result r) {
    return r.detail ? r.detail : k_empty_message;
}
int gr_result_ok(gr_result r) { return r.code == GR_OK; }

gr_result gr_client_init_ex(gr_client *c, const char *cli_path) {
    if (!c || !cli_path || !*cli_path)
        return make_result(c, GR_ERR_INVALID_ARGUMENT, "invalid argument");
    memset(c, 0, sizeof *c);
    if (!exists(cli_path))
        return make_result(c, GR_ERR_NOT_FOUND, "CLI binary does not exist");
    c->cli = dup_string(cli_path);
    if (!c->cli)
        return make_result(c, GR_ERR_OUT_OF_MEMORY, "out of memory");
    return make_result(c, GR_OK, NULL);
}

static gr_result run_argv(gr_client *c, char *const argv[],
                          const char *project_dir,
                          char *report_path, size_t report_path_size) {
    int pipefd[2];
    if (pipe(pipefd) != 0)
        return make_result(c, GR_ERR_PIPE_FAILED, "pipe failed");

    /* A second, close-on-exec pipe carries execvp()'s errno back to the
     * parent. A successful exec closes it, so the parent just sees EOF. */
    int errpipe[2];
    if (pipe(errpipe) != 0) {
        close(pipefd[0]); close(pipefd[1]);
        return make_result(c, GR_ERR_PIPE_FAILED, "pipe failed");
    }
    int fdflags = fcntl(errpipe[1], F_GETFD);
    if (fdflags < 0 || fcntl(errpipe[1], F_SETFD, fdflags | FD_CLOEXEC) < 0) {
        close(pipefd[0]); close(pipefd[1]);
        close(errpipe[0]); close(errpipe[1]);
        return make_result(c, GR_ERR_FCNTL_FAILED, "fcntl failed");
    }

    pid_t pid = fork();
    if (pid < 0) {
        close(pipefd[0]); close(pipefd[1]);
        close(errpipe[0]); close(errpipe[1]);
        return make_result(c, GR_ERR_FORK_FAILED, "fork failed");
    }
    if (pid == 0) {
        close(pipefd[0]);
        close(errpipe[0]);
        dup2(pipefd[1], STDOUT_FILENO);
        dup2(pipefd[1], STDERR_FILENO);
        if (pipefd[1] != STDOUT_FILENO && pipefd[1] != STDERR_FILENO)
            close(pipefd[1]);
        /* Casts keep this translation unit warning-free when a C++ consumer
         * compiles it directly (tests/test_cpp_sdk.sh does). */
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
    if (w != pid)
        return make_result(c, GR_ERR_WAIT_FAILED, "waitpid failed");

    if (exec_errno != 0) {
        char msg[sizeof c->error];
        snprintf(msg, sizeof msg, "failed to execute CLI '%s': %s",
                 c->cli, strerror(exec_errno));
        return make_result(c, GR_ERR_EXEC_FAILED, msg);
    }
    if (!WIFEXITED(status) || WEXITSTATUS(status) != 0) {
        if (!total) seterr(c, "GameReComp CLI exited non-zero");
        /* Keep the captured CLI tail as the detail when there is one. */
        return make_result(c, GR_ERR_CLI_FAILED, NULL);
    }

    char rp[GR_SDK_REPORT_PATH_MAX];
    int rp_len = snprintf(rp, sizeof rp, "%s/reports/report.json", project_dir);
    if (rp_len < 0 || (size_t)rp_len >= sizeof rp)
        return make_result(c, GR_ERR_PATH_TOO_LONG, "report path too long");
    if (!exists(rp))
        return make_result(c, GR_ERR_REPORT_MISSING,
                           "pipeline produced no report.json");
    if (report_path && report_path_size) {
        int copy_len = snprintf(report_path, report_path_size, "%s", rp);
        if (copy_len < 0 || (size_t)copy_len >= report_path_size)
            return make_result(c, GR_ERR_BUFFER_TOO_SMALL,
                               "report_path buffer too small");
    }
    c->error[0] = 0;
    return make_result(c, GR_OK, NULL);
}

gr_result gr_recompile_ex(gr_client *c, const char *pe_path,
                          const char *project_dir,
                          char *report_path, size_t report_path_size) {
    if (!c || !c->cli || !pe_path || !project_dir)
        return make_result(c, GR_ERR_INVALID_ARGUMENT, "invalid argument");
    if (!exists(pe_path))
        return make_result(c, GR_ERR_NOT_FOUND, "PE32 input does not exist");
    if (!is_dir(project_dir))
        return make_result(c, GR_ERR_NOT_DIRECTORY,
                           "project_dir is not an existing directory");

    char *argv[] = { c->cli, (char *)"run", (char *)pe_path,
                     (char *)"--project-dir", (char *)project_dir,
                     (char *)"--no-table", (char *)"--no-progress", NULL };
    return run_argv(c, argv, project_dir, report_path, report_path_size);

}


gr_result gr_build(gr_client *c, const char *pe_path, const char *project_dir,
                   const gr_build_options *opt,
                   char *report_path, size_t report_path_size) {
    if (!c || !c->cli || !pe_path || !project_dir)
        return make_result(c, GR_ERR_INVALID_ARGUMENT, "invalid argument");
    if (!exists(pe_path))
        return make_result(c, GR_ERR_NOT_FOUND, "PE32 input does not exist");
    if (!is_dir(project_dir))
        return make_result(c, GR_ERR_NOT_DIRECTORY,
                           "project_dir is not an existing directory");
    char *argv[20];
    size_t n = 0;
    argv[n++] = c->cli;
    argv[n++] = (char *)"run";
    argv[n++] = (char *)pe_path;
    argv[n++] = (char *)"--project-dir";
    argv[n++] = (char *)project_dir;
    argv[n++] = (char *)"--no-table";
    argv[n++] = (char *)"--no-progress";
    if (opt) {
        if (opt->input_os)      { argv[n++] = (char *)"--input-os";      argv[n++] = (char *)opt->input_os; }
        if (opt->input_arch)    { argv[n++] = (char *)"--input-arch";    argv[n++] = (char *)opt->input_arch; }
        if (opt->target_os)     { argv[n++] = (char *)"--target-os";     argv[n++] = (char *)opt->target_os; }
        if (opt->target_arch)   { argv[n++] = (char *)"--target-arch";   argv[n++] = (char *)opt->target_arch; }
        if (opt->out_mode)      { argv[n++] = (char *)"--out-mode";      argv[n++] = (char *)opt->out_mode; }
        if (opt->skip_stages)   { argv[n++] = (char *)"--skip-stages";   argv[n++] = (char *)opt->skip_stages; }
    }
    argv[n] = NULL;
    return run_argv(c, argv, project_dir, report_path, report_path_size);
}

/* Legacy wrappers: preserve the original int 0/-1 contract exactly. */
int gr_client_init(gr_client *c, const char *cli_path) {
    return gr_result_ok(gr_client_init_ex(c, cli_path)) ? 0 : -1;
}

int gr_client_init_default(gr_client *c) {
    const char *path = getenv("GAMERECOMP_CLI");
    if (!path || !*path) path = "game-recomp-cli";
    return gr_client_init(c, path);
}

int gr_recompile(gr_client *c, const char *pe_path, const char *project_dir,
                 char *report_path, size_t report_path_size) {
    return gr_result_ok(gr_recompile_ex(c, pe_path, project_dir,
                                        report_path, report_path_size)) ? 0 : -1;
}

const char *gr_last_error(const gr_client *c) { return c ? c->error : k_no_client; }

void gr_client_free(gr_client *c) {
    if (!c) return;
    free(c->cli);
    c->cli = NULL;
}
