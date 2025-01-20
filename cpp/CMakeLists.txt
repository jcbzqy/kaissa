cmake_minimum_required(VERSION 3.15)
project(kaissa_cpp VERSION 0.1.0 LANGUAGES CXX)

# Add conda lib path
link_directories($ENV{CONDA_PREFIX}/lib)

set(CMAKE_CXX_STANDARD 23)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

find_package(absl REQUIRED)
find_package(Boost REQUIRED)
find_package(fmt REQUIRED)

add_library(kaissa_lib src/uci.cpp)
target_include_directories(kaissa_lib PUBLIC src)
target_link_libraries(kaissa_lib PUBLIC Boost::boost fmt::fmt)
target_compile_options(kaissa_lib PRIVATE -Wall -Wextra -Wpedantic)

add_executable(kaissa_cpp src/main.cpp)
target_link_libraries(kaissa_cpp PRIVATE kaissa_lib)
target_compile_options(kaissa_cpp PRIVATE -Wall -Wextra -Wpedantic)