// Port one PE32 app through the C++ SDK with explicit input/target options.
#include "gamerecomp_sdk.hpp"
#include <iostream>
int main(int argc, char **argv) {
    if (argc != 4) { std::cerr << "usage: app <direction> <pe> <project>\n"; return 2; }
    try {
        gamerecomp::Client client;
        gamerecomp::BuildOptions options;
        options.input_os = "windows"; options.input_arch = "x86";
        options.target_os = "host"; options.target_arch = "host";
        options.out_mode = "full"; options.skip_stages = "verify";
        auto report = client.build(argv[2], argv[3], options);
        std::cout << "PORT_OK direction=" << argv[1] << " report=" << report.path
                  << " score=" << report.score() << '\n';
    } catch (const std::exception &e) {
        std::cout << "PORT_DEFERRED direction=" << argv[1] << " reason=" << e.what() << '\n';
    }
    return 0;
}
