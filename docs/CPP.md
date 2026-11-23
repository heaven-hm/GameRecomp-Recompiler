# C++ SDK guide

## Files

- Header: `cpp/include/gamerecomp_sdk.hpp` (includes the C header by relative path)
- Example: `cpp/examples/basic.cpp`
- Proof script: `tests/test_cpp_sdk.sh`

The C++ wrapper is header-only over the C implementation. Compile both
`src/gamerecomp_sdk.c` and your C++ source.

## API reference

```cpp
namespace gamerecomp {

struct Report { std::string path; };

class Client {
public:
    explicit Client(const std::string &cli_path);   // throws std::runtime_error
    ~Client();
    Report recompile(const std::string &pe,
                     const std::string &project);   // throws std::runtime_error
};
}
```

- The constructor validates the CLI path; a missing binary throws.
- Copy construction/assignment are deleted; use one client per thread or create
  separate instances.
- `recompile` throws `std::runtime_error` on any failure with the CLI's captured
  error tail. It returns `Report` only when `report.json` exists.

## Full example

```cpp
#include "gamerecomp_sdk.hpp"
#include <iostream>

int main(int argc, char **argv) {
    if (argc != 4) {
        std::cerr << "usage: " << argv[0]
                  << " <cli> <game.exe> <project-dir>\n";
        return 2;
    }
    try {
        gamerecomp::Client client(argv[1]);
        auto report = client.recompile(argv[2], argv[3]);
        std::cout << "report=" << report.path << "\n";
    } catch (const std::exception &e) {
        std::cerr << "GameReComp error: " << e.what() << "\n";
        return 1;
    }
    return 0;
}
```

Compile and run:

```sh
c++ -std=c++17 -Wall -Wextra -Iinclude -Icpp/include \
  src/gamerecomp_sdk.c cpp/examples/basic.cpp -o basic_cpp

./basic_cpp /path/to/game-recomp-cli /path/to/game.exe /tmp/project
```

Quick plumbing proof:

```sh
sh tests/test_cpp_sdk.sh   # prints "C++ SDK: PASS"
```

## Error handling patterns

```cpp
try {
    auto r = client.recompile(pe, project);
    // r.path is a real file; parse it with your JSON library.
} catch (const std::runtime_error &e) {
    // e.what() is either the SDK validation message or the CLI output tail.
}
```

## Notes

- `Report` currently carries the verified report path only. Add your own JSON
  parsing if you need the score or stage list.
- `gamerecomp::Client` is RAII; no manual free is needed.
- Thread safety: each `Client` owns its own error buffer; use one per thread.
