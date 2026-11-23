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

struct Report {
    std::string path;
    int score() const;  // -1 when the score is absent or unreadable
};

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

### `Report::score()`

`int score() const` reads the verified report at `Report::path` and returns the
top-level `"score"` integer. It is dependency-free (standard library only) and
needs no JSON library. The method returns `-1` as an "absent" sentinel when:

- `path` is empty, the file is missing, or the file cannot be read;
- the JSON has no `"score"` key, or its value is not a decimal integer;
- the parsed value does not fit in a 32-bit `int`.

Whitespace around the value and an optional leading `-`/`+` are accepted.
Because `-1` is the sentinel, a genuine score of `-1` is indistinguishable
from an absent score.

`Report` remains an aggregate (`Report{path}` still works), and the `Client`
constructor and `recompile` signatures are unchanged, so existing code keeps
compiling without modification.

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
        std::cout << "score=" << report.score() << "\n";
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
    int score = r.score();          // -1 if the report has no integer score
    // r.path is a real file; parse other fields with your own JSON library.
} catch (const std::runtime_error &e) {
    // e.what() is either the SDK validation message or the CLI output tail.
}
```

## Notes

- `Report` carries the verified report path plus a built-in `score()` extractor.
  For other fields (e.g. the stage list), parse `report.json` yourself.
- `gamerecomp::Client` is RAII; no manual free is needed.
- Thread safety: each `Client` owns its own error buffer; use one per thread.
