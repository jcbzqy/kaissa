cmake_minimum_required(VERSION 3.15)
project(kaissa_cpp VERSION 0.1.0 LANGUAGES CXX)

enable_testing()

set(CMAKE_CXX_STANDARD 23)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

link_directories($ENV{CONDA_PREFIX}/lib)

find_package(absl REQUIRED)
find_package(fmt REQUIRED)
find_package(GTest REQUIRED)

add_library(kaissa_lib
    src/board.cpp
    src/engine.cpp
    src/search.cpp
    src/uci.cpp
)
target_include_directories(kaissa_lib
    PUBLIC
        src
)
target_link_libraries(kaissa_lib
    PUBLIC
        fmt::fmt
        absl::inlined_vector
        absl::flat_hash_map
)
target_compile_options(kaissa_lib
    PRIVATE
        -Wall
        -Wextra
        -Wpedantic
)

add_executable(kaissa_tests
    tests/test_position_utils.cpp
    tests/test_search.cpp
)
target_link_libraries(kaissa_tests
    PRIVATE
        kaissa_lib
        GTest::GTest
        GTest::Main
)
target_compile_options(kaissa_tests
    PRIVATE
        -Wall
        -Wextra
        -Wpedantic
)

add_executable(kaissa_cpp
    src/main.cpp
)
target_link_libraries(kaissa_cpp
    PRIVATE
        kaissa_lib
)
target_compile_options(kaissa_cpp
    PRIVATE
        -Wall
        -Wextra
        -Wpedantic
)

include(GoogleTest)
gtest_discover_tests(kaissa_tests)