cmake_minimum_required(VERSION 3.10)
project(bubble_sort)

set(CMAKE_CXX_STANDARD 11)
set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -Wall -Wextra")

# 调试模式选项
option(DEBUG_MODE "Enable debug output" OFF)

if(DEBUG_MODE)
    add_definitions(-DDEBUG)
endif()

add_executable(bubble_sort
    main.cpp
    bubblesort.cpp
    bubblesort.hpp
)