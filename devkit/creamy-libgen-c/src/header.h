#pragma once
#include <cassert>
#include <stdint.h>

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

#define CMY_MESSAGE_SIZE 32
#define CMY_MESSAGE_PAYLOAD 28

enum CMY_Status {
    Error,
    Allow,
    Deny
};

struct CMY_Message {
    /* -------- HEADER -------- */
    u8 dst;
    u8 group;
    u8 src;
    u8 kind;

    /* -------- PAYLOAD -------- */
    u32 serial;
    u8 _padding0[24];
};

void function() {
    enum CMY_Status status = CMY_Status::Allow;
    struct CMY_Message message = {
        .dst = 0,
        .group = 0,
        .src = 0,
        .kind = 0,
        .serial = 0,
        ._padding0 = {0}
    };
}


struct BucketSmall {
    u64 d;
    u32 c;
    u16 b;
    u8 a;
};

struct NestedValid {
    /* -------- HEADER -------- */
    u8 dst;
    u8 group;
    u8 src;
    u8 kind;
    /* -------- PAYLOAD -------- */
    u8 _padding0[4];
    BucketSmall data;
    u8 extra[8];
};

_Static_assert(sizeof(NestedValid) == CMY_MESSAGE_SIZE, "xddd");
