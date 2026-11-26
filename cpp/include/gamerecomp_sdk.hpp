// gamerecomp_sdk.hpp - RAII C++ wrapper over the C client.
#pragma once
#include "../../include/gamerecomp_sdk.h"
#include <cctype>
#include <climits>
#include <fstream>
#include <iterator>
#include <stdexcept>
#include <string>

namespace gamerecomp {
namespace detail {

// Minimal, dependency-free JSON integer lookup. Finds the first occurrence of
// the quoted key, then scans forward for ':' and a decimal integer. Malformed
// or out-of-range values yield -1 (the "absent" sentinel).
inline int parse_json_int(const std::string &text, const std::string &key) {
    const std::string needle = "\"" + key + "\"";
    const std::size_t key_pos = text.find(needle);
    if (key_pos == std::string::npos) return -1;

    const std::size_t colon = text.find(':', key_pos + needle.size());
    if (colon == std::string::npos) return -1;
    std::size_t pos = colon + 1;
    while (pos < text.size() &&
           std::isspace(static_cast<unsigned char>(text[pos]))) {
        ++pos;
    }

    int sign = 1;
    if (pos < text.size() && (text[pos] == '-' || text[pos] == '+')) {
        sign = (text[pos] == '-') ? -1 : 1;
        ++pos;
    }

    const std::size_t digits_begin = pos;
    long long magnitude = 0;
    while (pos < text.size() && text[pos] >= '0' && text[pos] <= '9') {
        magnitude = magnitude * 10 + (text[pos] - '0');
        if (magnitude > 2147483648LL) return -1;  // avoid overflow / absurd input
        ++pos;
    }
    if (pos == digits_begin) return -1;  // no integer digits found

    const long long value = sign * magnitude;
    if (value < INT_MIN || value > INT_MAX) return -1;
    return static_cast<int>(value);
}

}  // namespace detail

struct Report {
    std::string path;

    // Read the top-level "score" integer from the report JSON at `path`.
    // Returns -1 when the report is missing, unreadable, or has no integer
    // "score" field. Dependency-free: uses only the C++ standard library.
    int score() const {
        if (path.empty()) return -1;
        std::ifstream in(path.c_str());
        if (!in) return -1;
        const std::string text((std::istreambuf_iterator<char>(in)),
                               std::istreambuf_iterator<char>());
        return detail::parse_json_int(text, "score");
    }
};

class Client {
public:
    /* No path needed: the SDK discovers the engine from GAMERECOMP_CLI/PATH. */
    Client() {
        if (gr_client_init_default(&c_) != 0)
            throw std::runtime_error(gr_last_error(&c_));
    }
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
