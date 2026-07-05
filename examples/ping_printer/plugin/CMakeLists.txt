cmake_minimum_required(VERSION 3.10)

set(CMAKE_C_COMPILER "clang")
set(CMAKE_CXX_COMPILER "clang++")
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)
set(CMAKE_C_COMPILER_TARGET wasm32)

project(ping_printer C)

set(PROTOCOL_XML "${CMAKE_CURRENT_SOURCE_DIR}/protocols/ping.xml")
set(PING_HEADER_OUTPUT "${CMAKE_CURRENT_SOURCE_DIR}/generated/ping/messages.h")
set(HEADER_OUTPUT "${CMAKE_CURRENT_BINARY_DIR}/generated")
set(TLSF_DIR "${CMAKE_CURRENT_SOURCE_DIR}/third_party/tlsf")

add_custom_command(
  OUTPUT "${PING_HEADER_OUTPUT}"
  COMMAND ${CMAKE_COMMAND} -E make_directory "${CMAKE_CURRENT_BINARY_DIR}/generated"
  COMMAND creamy generate "${PROTOCOL_XML}" -r -o "${HEADER_OUTPUT}"
  DEPENDS "${PROTOCOL_XML}"
  COMMENT "Generating headers from XML"
  VERBATIM
)


add_executable(ping_printer lib.c "${PING_HEADER_OUTPUT}" "${TLSF_DIR}/tlsf.c")

#set_target_properties(ping_printer PROPERTIES C_DEPENDS_USE_COMPILER OFF)

target_compile_options(ping_printer PRIVATE
  -O3
  -nostdlib
  -flto
  -Wall
  -Wextra
  -isystem /usr/local/include
)

target_link_options(ping_printer PRIVATE
  -nostdlib
  -flto
  -Wl,--no-entry
  -Wl,--export-all
  -Wl,--lto-O3
  # Если ошибка не уйдет, можно добавить принудительно:
  # -Wl,--allow-unknown-driver
)

set_target_properties(ping_printer PROPERTIES SUFFIX ".wasm")


target_include_directories(
  ping_printer
  PRIVATE
  "${CMAKE_CURRENT_BINARY_DIR}/generated"
  "${TLSF_DIR}"
)
