// Example: call GameReComp from C++ with RAII and exceptions.
#include "gamerecomp_sdk.hpp"
#include <iostream>

int main(int argc, char **argv) {
    if (argc != 4) { std::cerr << "usage: basic <cli> <pe> <project>\n"; return 2; }
    try {
        gamerecomp::Client client(argv[1]);
        auto report = client.recompile(argv[2], argv[3]);
        std::cout << "report=" << report.path << "\n";
    } catch (const std::exception &e) {
        std::cerr << e.what() << "\n";
        return 1;
    }
    return 0;
}
