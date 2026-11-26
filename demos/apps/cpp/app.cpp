// Working C++ app: RAII client plus parsed score from the SDK report.
#include "gamerecomp_sdk.hpp"
#include <iostream>
int main(int argc, char **argv) {
    if (argc != 4) { std::cerr << "usage: app <cli> <pe> <project>\n"; return 2; }
    try {
        gamerecomp::Client client(argv[1]);
        auto report = client.recompile(argv[2], argv[3]);
        std::cout << "APP_OK sdk=cpp report=" << report.path
                  << " score=" << report.score() << "\n";
    } catch (const std::exception &e) { std::cerr << "error: " << e.what() << "\n"; return 1; }
    return 0;
}
