#!/bin/sh
# Clean CMake configure/build/install into a staging prefix, then consume the
# installed package from independent projects (static C++, shared C/C++).
set -eu
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
TMP=${TMPDIR:-/tmp}/gr-sdk-cmake-$$
trap 'rm -rf "$TMP"' EXIT HUP INT TERM
PREFIX="$TMP/prefix"

# The version must match the canonical one in rust/Cargo.toml.
VERSION=$(sed -n 's/^version[[:space:]]*=[[:space:]]*"\([0-9][0-9.]*\)".*/\1/p' \
  "$ROOT/rust/Cargo.toml" | head -n 1)
[ -n "$VERSION" ] || { echo "cannot read version from Cargo.toml" >&2; exit 1; }

# 1. Configure, build and install from scratch.
cmake -S "$ROOT" -B "$TMP/build" \
  -DCMAKE_INSTALL_PREFIX="$PREFIX" -DCMAKE_BUILD_TYPE=Release
cmake --build "$TMP/build" --parallel
cmake --install "$TMP/build"

# Library directory is platform dependent (GNUInstallDirs).
LIBDIR=$(cd "$PREFIX" && ls -d lib lib64 2>/dev/null | head -n 1)
[ -n "$LIBDIR" ] || { echo "no library directory installed" >&2; exit 1; }
CFGDIR="$PREFIX/$LIBDIR/cmake/gamerecomp"

# 2. Installed headers, both libraries, and the package config files.
test -f "$PREFIX/include/gamerecomp_sdk.h"
test -f "$PREFIX/include/cpp/gamerecomp_sdk.hpp"
test -f "$PREFIX/$LIBDIR/libgamerecomp_sdk.a"
test -f "$PREFIX/$LIBDIR/libgamerecomp_sdk.dylib" || \
  test -f "$PREFIX/$LIBDIR/libgamerecomp_sdk.so"
test -f "$CFGDIR/gamerecomp-config.cmake"
test -f "$CFGDIR/gamerecomp-config-version.cmake"
test -f "$CFGDIR/gamerecomp-targets.cmake"

# 3. Version consistency: installed package version == Cargo.toml version.
grep -q "PACKAGE_VERSION \"$VERSION\"" "$CFGDIR/gamerecomp-config-version.cmake"

# 4. Build the C++ example as an external consumer of the installed package.
cmake -S "$ROOT/cpp/examples" -B "$TMP/consumer" \
  -DCMAKE_PREFIX_PATH="$PREFIX" -DCMAKE_BUILD_TYPE=Release
cmake --build "$TMP/consumer" --parallel

# 5. A C consumer that links the shared library explicitly, pinned to the
#    manifest version (also checks find_package version compatibility).
mkdir -p "$TMP/shared_consumer"
cat > "$TMP/shared_consumer/CMakeLists.txt" <<EOF
cmake_minimum_required(VERSION 3.16)
project(gr_shared_consumer LANGUAGES C)
find_package(gamerecomp $VERSION CONFIG REQUIRED)
add_executable(shared_basic "$ROOT/c/examples/basic.c")
target_link_libraries(shared_basic PRIVATE gamerecomp::c_shared)
EOF
cmake -S "$TMP/shared_consumer" -B "$TMP/shared_build" \
  -DCMAKE_PREFIX_PATH="$PREFIX" -DCMAKE_BUILD_TYPE=Release
cmake --build "$TMP/shared_build" --parallel

# 6. A C++ consumer bound to the shared core via gamerecomp::cpp_shared.
mkdir -p "$TMP/cpp_shared_consumer"
cat > "$TMP/cpp_shared_consumer/CMakeLists.txt" <<EOF
cmake_minimum_required(VERSION 3.16)
project(gr_cpp_shared_consumer LANGUAGES CXX)
find_package(gamerecomp CONFIG REQUIRED)
add_executable(cpp_shared_basic "$ROOT/cpp/examples/basic.cpp")
target_link_libraries(cpp_shared_basic PRIVATE gamerecomp::cpp_shared)
EOF
cmake -S "$TMP/cpp_shared_consumer" -B "$TMP/cpp_shared_build" \
  -DCMAKE_PREFIX_PATH="$PREFIX" -DCMAKE_BUILD_TYPE=Release
cmake --build "$TMP/cpp_shared_build" --parallel

# 7. An incompatible major version must be rejected by the version file.
mkdir -p "$TMP/version_consumer"
cat > "$TMP/version_consumer/CMakeLists.txt" <<'EOF'
cmake_minimum_required(VERSION 3.16)
project(gr_version_consumer LANGUAGES C)
find_package(gamerecomp 99.0 CONFIG REQUIRED)
EOF
if cmake -S "$TMP/version_consumer" -B "$TMP/version_build" \
    -DCMAKE_PREFIX_PATH="$PREFIX" >"$TMP/version.log" 2>&1; then
  echo "expected find_package to reject an incompatible version" >&2
  exit 1
fi
grep -qi 'version' "$TMP/version.log"

# 8. Run all three consumers against a fake CLI that writes a report.
mkdir -p "$TMP/proj/reports"
cat > "$TMP/cli" <<EOF
#!/bin/sh
printf '%s' '{"score":70,"stages":[]}' > "$TMP/proj/reports/report.json"
exit 0
EOF
chmod +x "$TMP/cli"

"$TMP/consumer/basic_cpp" "$TMP/cli" /etc/hosts "$TMP/proj" | grep -q 'score=70'
DYLD_LIBRARY_PATH="$PREFIX/$LIBDIR" LD_LIBRARY_PATH="$PREFIX/$LIBDIR" \
  "$TMP/shared_build/shared_basic" "$TMP/cli" /etc/hosts "$TMP/proj" | grep -q '^report='
DYLD_LIBRARY_PATH="$PREFIX/$LIBDIR" LD_LIBRARY_PATH="$PREFIX/$LIBDIR" \
  "$TMP/cpp_shared_build/cpp_shared_basic" "$TMP/cli" /etc/hosts "$TMP/proj" \
  | grep -q 'score=70'

echo "CMake package: PASS"
