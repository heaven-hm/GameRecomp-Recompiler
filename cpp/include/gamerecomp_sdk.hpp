// gamerecomp_sdk.hpp - RAII C++ wrapper over the C client.
#pragma once
#include "../../include/gamerecomp_sdk.h"
#include <stdexcept>
#include <string>

namespace gamerecomp {

struct Report {
    std::string path;
};

class Client {
public:
    explicit Client(const std::string &cli_path) {
        if (gr_client_init(&c_, cli_path.c_str()) != 0)
            throw std::runtime_error(gr_last_error(&c_));
    }
    ~Client() { gr_client_free(&c_); }
    Client(const Client &) = delete;
    Client &operator=(const Client &) = delete;

    Report recompile(const std::string &pe, const std::string &project) {
        char path[1024] = {0};
        if (gr_recompile(&c_, pe.c_str(), project.c_str(), path, sizeof path) != 0)
            throw std::runtime_error(gr_last_error(&c_));
        return Report{path};
    }

private:
    gr_client c_{};
};

}  // namespace gamerecomp
