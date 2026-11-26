cmake_minimum_required(VERSION 3.16)

# Standalone consumer project: configure with
#   cmake -S cpp/examples -B build-example -DCMAKE_PREFIX_PATH=<install-prefix>
# When added as a subdirectory of the SDK build, the in-tree targets are used.
if(CMAKE_SOURCE_DIR STREQUAL CMAKE_CURRENT_SOURCE_DIR)
  project(gamerecomp-cpp-example LANGUAGES CXX)
endif()

if(NOT TARGET gamerecomp::cpp)
  find_package(gamerecomp CONFIG REQUIRED)
endif()

add_executable(basic_cpp basic.cpp)
target_link_libraries(basic_cpp PRIVATE gamerecomp::cpp)
