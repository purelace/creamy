#pragma once

#include <cstdint>
typedef float  f32;
typedef double f64;

typedef uint8_t  u8;
typedef uint16_t u16;
typedef uint32_t u32;
typedef uint64_t u64;
typedef __uint128_t u128;

typedef int8_t  i8;
typedef int16_t i16;
typedef int32_t i32;
typedef int64_t i64;
typedef __int128_t i128;

struct Message {
    public:
        u8 dst;
        u8 group;
        u8 src;
        u8 kind;
} ;


void mes() {
}
