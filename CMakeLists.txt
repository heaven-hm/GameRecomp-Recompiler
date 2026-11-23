cmake_minimum_required(VERSION 3.16)

# rust/Cargo.toml is the single source of truth for the SDK version. Deriving
# the CMake project version from it keeps every binding on the same version.
file(STRINGS "${CMAKE_CURRENT_SOURCE_DIR}/rust/Cargo.toml" _gr_version_line
     REGEX "^version[ ]*=[ ]*\"[0-9]+\\.[0-9]+\\.[0-9]+\"")
if(NOT _gr_version_line)
  message(FATAL_ERROR "Could not read the SDK version from rust/Cargo.toml")
endif()
string(REGEX MATCH "[0-9]+\\.[0-9]+\\.[0-9]+" GAMERECOMP_VERSION
       "${_gr_version_line}")

project(gamerecomp VERSION "${GAMERECOMP_VERSION}" LANGUAGES C CXX)

include(GNUInstallDirs)
include(CMakePackageConfigHelpers)

# The C core: one implementation, exported as both a static and a shared
# library. Both consume the same public header and have no third-party deps.
add_library(gamerecomp_c STATIC src/gamerecomp_sdk.c)
add_library(gamerecomp::c ALIAS gamerecomp_c)
set_target_properties(gamerecomp_c PROPERTIES
  OUTPUT_NAME gamerecomp_sdk
  EXPORT_NAME c
  C_STANDARD 11
  C_STANDARD_REQUIRED ON)

add_library(gamerecomp_c_shared SHARED src/gamerecomp_sdk.c)
add_library(gamerecomp::c_shared ALIAS gamerecomp_c_shared)
set_target_properties(gamerecomp_c_shared PROPERTIES
  OUTPUT_NAME gamerecomp_sdk
  EXPORT_NAME c_shared
  VERSION "${PROJECT_VERSION}"
  SOVERSION "${PROJECT_VERSION_MAJOR}"
  C_STANDARD 11
  C_STANDARD_REQUIRED ON)

# MSVC uses the same .lib name for a static library and a DLL import library.
if(MSVC)
  set_target_properties(gamerecomp_c PROPERTIES OUTPUT_NAME gamerecomp_sdk_static)
endif()

foreach(target gamerecomp_c gamerecomp_c_shared)
  target_include_directories(${target} PUBLIC
    $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/include>
    $<INSTALL_INTERFACE:${CMAKE_INSTALL_INCLUDEDIR}>)
endforeach()

# The C++ wrapper is header only. It adds cpp/include to the include path and
# pulls in the static C core; gamerecomp::cpp_shared below is the shared variant.
add_library(gamerecomp_cpp INTERFACE)
add_library(gamerecomp::cpp ALIAS gamerecomp_cpp)
set_target_properties(gamerecomp_cpp PROPERTIES EXPORT_NAME cpp)
target_compile_features(gamerecomp_cpp INTERFACE cxx_std_17)
target_link_libraries(gamerecomp_cpp INTERFACE gamerecomp_c)
target_include_directories(gamerecomp_cpp INTERFACE
  $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/cpp/include>
  $<INSTALL_INTERFACE:${CMAKE_INSTALL_INCLUDEDIR}/cpp>)

# Same wrapper, bound to the shared C core for consumers that want one copy of
# the implementation shared across multiple libraries in the same process.
add_library(gamerecomp_cpp_shared INTERFACE)
add_library(gamerecomp::cpp_shared ALIAS gamerecomp_cpp_shared)
set_target_properties(gamerecomp_cpp_shared PROPERTIES EXPORT_NAME cpp_shared)
target_compile_features(gamerecomp_cpp_shared INTERFACE cxx_std_17)
target_link_libraries(gamerecomp_cpp_shared INTERFACE gamerecomp_c_shared)
target_include_directories(gamerecomp_cpp_shared INTERFACE
  $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/cpp/include>
  $<INSTALL_INTERFACE:${CMAKE_INSTALL_INCLUDEDIR}/cpp>)

# Install rules: keep the source layout so gamerecomp_sdk.hpp's relative
# include "../../include/gamerecomp_sdk.h" still resolves after installation.
install(FILES include/gamerecomp_sdk.h
        DESTINATION "${CMAKE_INSTALL_INCLUDEDIR}")
install(FILES cpp/include/gamerecomp_sdk.hpp
        DESTINATION "${CMAKE_INSTALL_INCLUDEDIR}/cpp")

set(GR_CMAKE_DIR "${CMAKE_INSTALL_LIBDIR}/cmake/gamerecomp")
install(TARGETS gamerecomp_c gamerecomp_c_shared
                 gamerecomp_cpp gamerecomp_cpp_shared
        EXPORT gamerecompTargets
        ARCHIVE DESTINATION "${CMAKE_INSTALL_LIBDIR}"
        LIBRARY DESTINATION "${CMAKE_INSTALL_LIBDIR}"
        RUNTIME DESTINATION "${CMAKE_INSTALL_BINDIR}")

configure_package_config_file(
  "${CMAKE_CURRENT_SOURCE_DIR}/cmake/gamerecomp-config.cmake.in"
  "${CMAKE_CURRENT_BINARY_DIR}/gamerecomp-config.cmake"
  INSTALL_DESTINATION "${GR_CMAKE_DIR}")
write_basic_package_version_file(
  "${CMAKE_CURRENT_BINARY_DIR}/gamerecomp-config-version.cmake"
  VERSION "${PROJECT_VERSION}"
  COMPATIBILITY SameMajorVersion)

install(EXPORT gamerecompTargets
        FILE gamerecomp-targets.cmake
        NAMESPACE gamerecomp::
        DESTINATION "${GR_CMAKE_DIR}")
install(FILES
  "${CMAKE_CURRENT_BINARY_DIR}/gamerecomp-config.cmake"
  "${CMAKE_CURRENT_BINARY_DIR}/gamerecomp-config-version.cmake"
  DESTINATION "${GR_CMAKE_DIR}")

# Build the packaged example against the in-tree targets as a smoke test. The
# same example is built against the installed package by test_cmake_install.sh.
if(CMAKE_SOURCE_DIR STREQUAL CMAKE_CURRENT_SOURCE_DIR)
  option(GAMERECOMP_BUILD_EXAMPLES "Build the C++ example" ON)
  if(GAMERECOMP_BUILD_EXAMPLES)
    add_subdirectory(cpp/examples)
  endif()
endif()
