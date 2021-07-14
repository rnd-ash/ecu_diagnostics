cmake_minimum_required(VERSION 3.9)

project(ecu_diag_c C CXX)

set (CMAKE_MODULE_PATH ${CMAKE_MODULE_PATH} ${CMAKE_CURRENT_SOURCE_DIR}/cmake)

file(GLOB SRC_FILES RELATIVE "${CMAKE_CURRENT_SOURCE_DIR}" REGEX
        "${CMAKE_CURRENT_SOURCE_DIR}/src/*.c"
        "${CMAKE_CURRENT_SOURCE_DIR}/src/*.h"
        "${CMAKE_CURRENT_SOURCE_DIR}/src/*.cpp"
        "${CMAKE_CURRENT_SOURCE_DIR}/src/*.hpp"
)

add_executable(ecu_diag_c ${SRC_FILES})

if(WIN32) # WINDOWS
    target_link_libraries(ecu_diag_c ${CMAKE_CURRENT_SOURCE_DIR}/libecu_diagnostics_ffi.dll)
elseif(CMAKE_SYSTEM_NAME STREQUAL Darwin) # OSX
    target_link_libraries(ecu_diag_c ${CMAKE_CURRENT_SOURCE_DIR}/libecu_diagnostics_ffi.dylib)
else() # LINUX
    target_link_libraries(ecu_diag_c ${CMAKE_CURRENT_SOURCE_DIR}/libecu_diagnostics_ffi.so)
endif()