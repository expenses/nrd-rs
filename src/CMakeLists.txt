cmake_minimum_required(VERSION 3.22)
project(nrd LANGUAGES CXX)

set(NRD_STATIC_LIBRARY ON CACHE BOOL "" FORCE)
set(NRD_EMBEDS_DXIL_SHADERS OFF CACHE BOOL "" FORCE)
set(NRD_EMBEDS_DXBC_SHADERS OFF CACHE BOOL "" FORCE)

if(MSVC)
    # NRI and ShaderMakeBlob use C++ exception handling, and upstream
    # CMakeLists treat MSVC warnings as errors (/WX).  Without /EHsc,
    # MSVC emits C4530 ("exception handler used without unwind
    # semantics"), which becomes a fatal error.
    add_compile_options(/EHsc)

    # Link the dynamic release CRT (/MD) always, matching Rust on MSVC.
    set(CMAKE_MSVC_RUNTIME_LIBRARY "MultiThreadedDLL")
endif()

add_subdirectory(${CMAKE_CURRENT_SOURCE_DIR}/../NRD NRD)

set(NRI_STATIC_LIBRARY ON CACHE BOOL "" FORCE)
set(NRI_ENABLE_VK_SUPPORT ON CACHE BOOL "" FORCE)
set(NRI_ENABLE_D3D11_SUPPORT OFF CACHE BOOL "" FORCE)
set(NRI_ENABLE_D3D12_SUPPORT OFF CACHE BOOL "" FORCE)
set(NRI_ENABLE_NONE_SUPPORT OFF CACHE BOOL "" FORCE)
set(NRI_ENABLE_NVAPI OFF CACHE BOOL "" FORCE)
set(NRI_ENABLE_AMDAGS OFF CACHE BOOL "" FORCE)
set(NRI_ENABLE_NVTX_SUPPORT OFF CACHE BOOL "" FORCE)
set(NRI_ENABLE_WAYLAND_SUPPORT OFF CACHE BOOL "" FORCE)

# Manually find X11 so NRI's VK backend gets VK_USE_PLATFORM_XLIB_KHR on Linux
find_path(XLIB_HEADERS NAMES "X11/Xlib.h")
if(XLIB_HEADERS)
    set(NRI_ENABLE_XLIB_SUPPORT ON CACHE BOOL "" FORCE)
    set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -I${XLIB_HEADERS}")
endif()

add_subdirectory(${CMAKE_CURRENT_SOURCE_DIR}/../NRI NRI)

install(TARGETS NRD NRI ARCHIVE DESTINATION lib)
