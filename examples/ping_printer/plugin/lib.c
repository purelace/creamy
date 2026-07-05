#include <creamy/limits.h>
#include <creamy/export.h>
#include <creamy/types.h>
#include "tlsf.h"
#include "bus.h"

extern unsigned char __heap_base;
tlsf_t heap = NULL;

// Можно вернуть 0 - это будет интерпретировано как ошибка плагина
u8 init_plugin(u64 max_heap) {
    unsigned char* start = &__heap_base;
    heap = tlsf_create_with_pool(start, max_heap);
    return 1;
}

void* malloc(size_t size) {
    return tlsf_malloc(heap, size);
}

void free(void* ptr) {
    tlsf_free(heap, ptr);
}

// Можно вернуть 0 - это будет интерпретировано как ошибка плагина
u64 export_buffer(u32 length, u32 serial_region, u32 serial) {
    u32 buffer_size = length * CMY_MESSAGE_SIZE + serial_region;
    cmy_buffer = malloc(buffer_size);
    u32 *slice = &cmy_buffer[buffer_size - serial_region];
    cmy_serial_var = slice;
    *slice = serial;
    return (u64)cmy_buffer;
}

void notify(u32 count, u32 serial) {
    *cmy_serial_var = count;
    *cmy_serial_var = serial;

}
