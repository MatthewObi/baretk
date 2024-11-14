#ifndef BARETK_H_INCLUDED
#define BARETK_H_INCLUDED

// C FFI library declarations

int baretk_print_strings(const char* path, int min_len, const char* out_path);

typedef struct BARETK_Program* BARETK_Program;
typedef enum BARETK_Endianess {
    LITTLE_ENDIAN = 0x1,
    BIG_ENDIAN = 0x2,
} BARETK_Endianess;

BARETK_Program baretk_load_program(const char* path);
void baretk_free_program(BARETK_Program program);
BARETK_Endianess baretk_get_endianess(BARETK_Program program);
const char* baretk_get_machine_type(BARETK_Program program);

#endif // BARETK_H_INCLUDED

