cmake_minimum_required(VERSION 3.2)
project(fancy_boot)

set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -std=c++11")

set(SOURCE_FILES main.cpp)
add_executable(fancy_boot ${SOURCE_FILES})