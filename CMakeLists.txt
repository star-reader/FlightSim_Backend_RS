cmake_minimum_required(VERSION 3.20)

project(sim_flight_backend
    VERSION 0.1.0
    LANGUAGES NONE
)

find_program(CARGO_EXECUTABLE cargo REQUIRED)
find_program(RUSTUP_EXECUTABLE rustup)
find_program(ZIG_EXECUTABLE zig)

set(RUST_PACKAGE_NAME "sim_flight_backend")
set(RUST_BINARY_NAME "sim_flight_backend")
set(CARGO_TARGET_DIR "${CMAKE_BINARY_DIR}/cargo-target")
set(DIST_DIR "${CMAKE_BINARY_DIR}/dist")
set(DIST_BIN_DIR "${DIST_DIR}/bin")
set(DIST_PACKAGE_DIR "${DIST_DIR}/packages")
set(ZIG_LOCAL_CACHE_DIR "${CMAKE_BINARY_DIR}/zig-cache/local")
set(ZIG_GLOBAL_CACHE_DIR "${CMAKE_BINARY_DIR}/zig-cache/global")

if(APPLE)
    if(CMAKE_SYSTEM_PROCESSOR MATCHES "arm64|aarch64")
        set(NATIVE_RUST_TARGET "aarch64-apple-darwin")
    else()
        set(NATIVE_RUST_TARGET "x86_64-apple-darwin")
    endif()
elseif(CMAKE_SYSTEM_NAME STREQUAL "Linux")
    if(CMAKE_SYSTEM_PROCESSOR MATCHES "aarch64|arm64")
        set(NATIVE_RUST_TARGET "aarch64-unknown-linux-gnu")
    else()
        set(NATIVE_RUST_TARGET "x86_64-unknown-linux-gnu")
    endif()
else()
    message(FATAL_ERROR "Unsupported host OS: ${CMAKE_SYSTEM_NAME}. This package supports macOS and Linux.")
endif()

set(PACKAGE_FILES
    "${CMAKE_SOURCE_DIR}/README.md"
    "${CMAKE_SOURCE_DIR}/LICENSE"
    "${CMAKE_SOURCE_DIR}/.env.example"
)

function(add_rust_release_target target_name rust_target)
    set(binary_path "${CARGO_TARGET_DIR}/${rust_target}/release/${RUST_BINARY_NAME}")
    set(staged_binary_dir "${DIST_BIN_DIR}/${rust_target}")
    set(staged_binary "${staged_binary_dir}/${RUST_BINARY_NAME}")
    set(package_name "${RUST_PACKAGE_NAME}-${PROJECT_VERSION}-${rust_target}")
    set(package_root "${DIST_PACKAGE_DIR}/${package_name}")
    set(package_archive "${DIST_PACKAGE_DIR}/${package_name}.tar.gz")
    set(cargo_env "CARGO_TARGET_DIR=${CARGO_TARGET_DIR}")

    if(rust_target STREQUAL "x86_64-unknown-linux-gnu" AND ZIG_EXECUTABLE)
        set(linux_cc_wrapper "${CMAKE_SOURCE_DIR}/cmake/zig-cc-x86_64-linux-gnu.sh")
        list(APPEND cargo_env
            "ZIG_LOCAL_CACHE_DIR=${ZIG_LOCAL_CACHE_DIR}"
            "ZIG_GLOBAL_CACHE_DIR=${ZIG_GLOBAL_CACHE_DIR}"
            "CC_x86_64_unknown_linux_gnu=${linux_cc_wrapper}"
            "CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=${linux_cc_wrapper}"
        )
    elseif(rust_target STREQUAL "aarch64-unknown-linux-gnu" AND ZIG_EXECUTABLE)
        set(linux_cc_wrapper "${CMAKE_SOURCE_DIR}/cmake/zig-cc-aarch64-linux-gnu.sh")
        list(APPEND cargo_env
            "ZIG_LOCAL_CACHE_DIR=${ZIG_LOCAL_CACHE_DIR}"
            "ZIG_GLOBAL_CACHE_DIR=${ZIG_GLOBAL_CACHE_DIR}"
            "CC_aarch64_unknown_linux_gnu=${linux_cc_wrapper}"
            "CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=${linux_cc_wrapper}"
        )
    endif()

    add_custom_command(
        OUTPUT "${binary_path}"
        COMMAND "${CMAKE_COMMAND}" -E make_directory "${ZIG_LOCAL_CACHE_DIR}" "${ZIG_GLOBAL_CACHE_DIR}"
        COMMAND "${CMAKE_COMMAND}" -E env
            ${cargo_env}
            "${CARGO_EXECUTABLE}" build --release --locked --target "${rust_target}"
        WORKING_DIRECTORY "${CMAKE_SOURCE_DIR}"
        COMMENT "Building Rust release target ${rust_target}"
        VERBATIM
    )

    add_custom_command(
        OUTPUT "${staged_binary}"
        COMMAND "${CMAKE_COMMAND}" -E make_directory "${staged_binary_dir}"
        COMMAND "${CMAKE_COMMAND}" -E copy "${binary_path}" "${staged_binary}"
        DEPENDS "${binary_path}"
        COMMENT "Staging ${rust_target} binary"
        VERBATIM
    )

    add_custom_target("${target_name}" DEPENDS "${staged_binary}")

    add_custom_command(
        OUTPUT "${package_archive}"
        COMMAND "${CMAKE_COMMAND}" -E make_directory "${DIST_PACKAGE_DIR}"
        COMMAND "${CMAKE_COMMAND}" -E rm -rf "${package_root}"
        COMMAND "${CMAKE_COMMAND}" -E make_directory "${package_root}"
        COMMAND "${CMAKE_COMMAND}" -E copy "${staged_binary}" "${package_root}/${RUST_BINARY_NAME}"
        COMMAND "${CMAKE_COMMAND}" -E copy ${PACKAGE_FILES} "${package_root}"
        COMMAND "${CMAKE_COMMAND}" -E chdir "${DIST_PACKAGE_DIR}"
            "${CMAKE_COMMAND}" -E tar "czf" "${package_archive}" --format=gnutar "${package_name}"
        DEPENDS "${staged_binary}" ${PACKAGE_FILES}
        COMMENT "Packaging ${package_name}.tar.gz"
        VERBATIM
    )

    add_custom_target("package-${target_name}" DEPENDS "${package_archive}")
endfunction()

add_custom_target(check
    COMMAND "${CMAKE_COMMAND}" -E env
        "CARGO_TARGET_DIR=${CARGO_TARGET_DIR}"
        "${CARGO_EXECUTABLE}" check --locked
    WORKING_DIRECTORY "${CMAKE_SOURCE_DIR}"
    COMMENT "Running cargo check"
    VERBATIM
)

add_custom_target(test
    COMMAND "${CMAKE_COMMAND}" -E env
        "CARGO_TARGET_DIR=${CARGO_TARGET_DIR}"
        "${CARGO_EXECUTABLE}" test --locked
    WORKING_DIRECTORY "${CMAKE_SOURCE_DIR}"
    COMMENT "Running cargo test"
    VERBATIM
)

add_rust_release_target("release-native" "${NATIVE_RUST_TARGET}")
add_rust_release_target("release-macos-aarch64" "aarch64-apple-darwin")
add_rust_release_target("release-macos-x86_64" "x86_64-apple-darwin")
add_rust_release_target("release-linux-x86_64" "x86_64-unknown-linux-gnu")
add_rust_release_target("release-linux-aarch64" "aarch64-unknown-linux-gnu")

add_custom_target(release)
add_dependencies(release release-native)

add_custom_target(package-native)
add_dependencies(package-native package-release-native)

add_custom_target(package-macos)
add_dependencies(package-macos
    package-release-macos-aarch64
    package-release-macos-x86_64
)

add_custom_target(package-linux)
add_dependencies(package-linux package-release-linux-x86_64)

add_custom_target(package-linux-all)
add_dependencies(package-linux-all
    package-release-linux-x86_64
    package-release-linux-aarch64
)

add_custom_target(package-all)
add_dependencies(package-all package-macos package-linux)

if(RUSTUP_EXECUTABLE)
    add_custom_target(list-rust-targets
        COMMAND "${RUSTUP_EXECUTABLE}" target list --installed
        COMMENT "Installed Rust targets"
        VERBATIM
    )
endif()

message(STATUS "Native Rust target: ${NATIVE_RUST_TARGET}")
message(STATUS "Cargo target dir: ${CARGO_TARGET_DIR}")
message(STATUS "Release binaries: ${DIST_BIN_DIR}")
message(STATUS "Release packages: ${DIST_PACKAGE_DIR}")
