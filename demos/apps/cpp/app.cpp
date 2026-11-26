// app.cpp - C++ example for the GameRecomp SDK.
//
// Usage: app <game-recomp-cli> <pe32-x86.exe> <existing-project-dir>
//
// RAII client, typed report and explicit build options. The CLI path is
// supplied by this demo harness; production callers use Client() and let the
// SDK discover the engine.

#include "gamerecomp_sdk.hpp"

#include <iostream>
#include <string>

namespace {

gamerecomp::BuildOptions default_options() {
    gamerecomp::BuildOptions options;
    options.input_os = "windows";
    options.input_arch = "x86";
    options.target_os = "host";
    options.target_arch = "host";
    options.out_mode = "full";
    return options;
}

}  // namespace

int main(int argc, char **argv) {
    if (argc != 4) {
        std::cerr << "usage: app <cli> <pe> <project>\n";
        return 2;
    }

    try {
        gamerecomp::Client client(argv[1]);
        const auto report = client.build(argv[2], argv[3], default_options());

        std::cout << "APP_OK sdk=cpp"
                  << " report=" << report.path
                  << " score=" << report.score() << '\n';
        return 0;
    } catch (const std::exception &error) {
        std::cerr << "error: " << error.what() << '\n';
        return 1;
    }
}
