# CMake package (C and C++ SDK)

The repository is a standard CMake project that builds, installs and exports
the C core library and the header-only C++ wrapper as a relocatable CMake
package. No source file is special-cased: the C client in `src/` is compiled
unchanged into both a static and a shared library.

## Build and install

```sh
cmake -S . -B build -DCMAKE_INSTALL_PREFIX=/path/to/prefix -DCMAKE_BUILD_TYPE=Release
cmake --build build --parallel
cmake --install build
```

This produces:

| Installed path | Contents |
|---|---|
| `<prefix>/include/gamerecomp_sdk.h` | Public C header |
| `<prefix>/include/cpp/gamerecomp_sdk.hpp` | Header-only C++ wrapper |
| `<prefix>/lib/libgamerecomp_sdk.a` | Static C library |
| `<prefix>/lib/libgamerecomp_sdk.{dylib,so}` (+ versioned symlinks) | Shared C library |
| `<prefix>/lib/cmake/gamerecomp/` | `gamerecomp-config.cmake`, `gamerecomp-config-version.cmake`, `gamerecomp-targets.cmake` |

(The library directory may be `lib64` on some Linux distributions.) The C++
header is installed one directory deeper than the C header on purpose: its
relative include `"../../include/gamerecomp_sdk.h"` resolves correctly only
with this layout.

## Consume from another CMake project

```cmake
cmake_minimum_required(VERSION 3.16)
project(my_app LANGUAGES CXX)

find_package(gamerecomp 0.1 CONFIG REQUIRED)

add_executable(my_app main.cpp)
target_link_libraries(my_app PRIVATE gamerecomp::cpp)   # C++ wrapper + static C core
```

Point CMake at the install prefix with `-DCMAKE_PREFIX_PATH=<prefix>` (or set
`gamerecomp_DIR` to the `lib/cmake/gamerecomp` directory). Available imported
targets:

| Target | Use |
|---|---|
| `gamerecomp::c` | C core, static library, `<prefix>/include` on the include path |
| `gamerecomp::c_shared` | C core, shared library (same headers) |
| `gamerecomp::cpp` | Header-only C++ wrapper; adds `<prefix>/include/cpp` and links `gamerecomp::c` |
| `gamerecomp::cpp_shared` | Same wrapper, linked against `gamerecomp::c_shared` |

The two `cpp` targets differ only in which C core they link, so switching
between a static and a shared build requires no source changes; both also
propagate the required C++17 standard to consumers. Both C
libraries export the same `gr_*` C API. The C client uses POSIX
process APIs (`fork`, `execvp`, `waitpid`) and therefore targets macOS/Linux.

A complete consumer is checked in at `cpp/examples/`:

```sh
cmake -S cpp/examples -B build-example -DCMAKE_PREFIX_PATH=<prefix>
cmake --build build-example
./build-example/basic_cpp <cli> <pe> <project>
```

The same example is also built against the in-tree targets as part of the main
build; pass `-DGAMERECOMP_BUILD_EXAMPLES=OFF` to skip it.

## Version consistency

`rust/Cargo.toml` is the single source of truth. The root `CMakeLists.txt`
reads its `[package] version` at configure time and uses it as the CMake
project version, which in turn feeds the shared-library `VERSION`/`SOVERSION`
and the generated `gamerecomp-config-version.cmake`. Requests such as
`find_package(gamerecomp 0.1 CONFIG REQUIRED)` are checked with
`SameMajorVersion` compatibility. `tests/test_cmake_install.sh` asserts that
the installed `PACKAGE_VERSION` still equals the manifest version, that a
versioned `find_package` request succeeds, and that an incompatible major
version is rejected.

## Verification

```sh
sh tests/test_cmake_install.sh
```

The script performs a clean configure, build and install into a temporary
prefix, checks the installed layout and version, builds the C++ example and a
second consumer that links the shared library against the installed package,
and runs both against a fake CLI. The existing `test_c_sdk.sh` and
`test_cpp_sdk.sh` continue to verify direct-from-source compilation.
