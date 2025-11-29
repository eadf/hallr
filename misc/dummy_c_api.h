/* A rough estimate of the FFI API as written in C */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

struct Vector3 {
    float x;
    float y;
    float z;
};

struct StringMap {
    char** keys;
    char** values;
    size_t count;
};

struct GeometryOutput {
    struct Vector3* vertices;
    size_t vertex_count;
    c_uint32* indices;
    size_t indices_count;
    float* matrices;
    size_t matrices_count;
};

struct ProcessResult {
    struct GeometryOutput geometry;
    struct StringMap map;
};

void free_process_results(struct ProcessResult* result) {
    for (size_t i = 0; i < result->map.count; i++) {
        free(result->map.keys[i]);
        free(result->map.values[i]);
    }
    free(result->map.keys);
    free(result->map.values);

    free(result->geometry.vertices);
    free(result->geometry.indices);
    free(result->geometry.matrices);
}

struct ProcessResult process_geometry(const struct Vector3* vertices, size_t vertex_count,
                                      const c_uint32* indices, size_t indices_count,
                                      const float* matrices, size_t matrices_count,
                                      const struct StringMap* config) {
    printf("C: Received config of size: %zu\n", config->count);

    struct ProcessResult result;

    // Simulate some processing and populate the result structure

    return result;
}
