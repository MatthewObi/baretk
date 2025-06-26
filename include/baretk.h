#ifndef BARETK_H_INCLUDED
#define BARETK_H_INCLUDED

#include <stdint.h>
#include <stdlib.h>

// C FFI library declarations

/// Prints ASCII strings located in a binary file.
/// @param path The path to the binary file to analyze.
/// @param min_len The minimum number of characters for an ASCII string to be included in the result.
/// @param printable Whether to only include strings that only contain printable ASCII characters (0x20-0x7f). 1 = true. 0 = false.
/// @param out_path Path to output file to write result to or NULL to print to stdout.
/// @return 1 if the action was successful. 0 otherwise.
int baretk_print_strings(const char* path, int min_len, int printable, const char* out_path);

/// Disassembles an executable file.
/// @param path The path to the executable file.
/// @param out_path Path to output file to write result to or NULL to print to stdout.
/// @return 1 if the action was successful. 0 otherwise.
int baretk_disassemble_file(const char* path, const char* out_path);

/// @brief Opaque pointer to program.
typedef struct BARETK_Program* BARETK_Program;

/// @brief Opaque pointer to disassembly.
typedef struct BARETK_Disassembly* BARETK_Disassembly;

/// @brief Opaque pointer to decompilation.
typedef struct BARETK_Decomp* BARETK_Decomp;

/// @brief Endianess enum value
typedef enum BARETK_Endianess {
    BARETK_LITTLE_ENDIAN = 0x1,
    BARETK_BIG_ENDIAN = 0x2,
} BARETK_Endianess;

/// @brief Permission enum value
typedef enum BARETK_Perm {
    PERM_RWX_EXEC = 0x1,
    PERM_RWX_WRITE = 0x2,
    PERM_RWX_READ = 0x4,
} BARETK_Perm;

/// @brief Language enum value
typedef enum BARETK_Lang {
    BARETK_LANG_PSEUDO = 0,
    BARETK_LANG_C = 1,
} BARETK_Lang;

typedef struct BARETK_Segment {
    uint8_t perm;
    uint64_t offset;
    uint64_t vaddr;
    uint64_t paddr;
    size_t size;
} BARETK_Segment;

typedef struct BARETK_SegmentArray {
    const BARETK_Segment* segments;
    size_t size;
} BARETK_SegmentArray;

typedef struct BARETK_U8Array {
    const uint8_t* bytes;
    size_t size;
} BARETK_U8Array;

typedef struct BARETK_Section {
    uint64_t addr;
    BARETK_U8Array bytes;
} BARETK_Section;

/// Loads a program from a file.
/// @param path The path to the binary file to load.
/// @return An opaque pointer to the program data structure, or NULL if load was unsuccessful. Must free the program with ``baretk_free_program()`` 
/// when done.
BARETK_Program baretk_load_program(const char* path);

/// Frees the data of a previously loaded program.
/// @param program The pointer to the program.
/// @note Checks for NULL, but does not check for previously freed program!
void baretk_free_program(BARETK_Program program);

/// Clones the data of an existing program.
/// @param program The pointer to the program to clone.
/// @return An opaque pointer to the clone, or NULL if the clone was unsuccessful. Must free the program with ``baretk_free_program()`` 
/// when done.
/// @note Checks for NULL, but does not check for previously freed program!
BARETK_Program baretk_clone_program(const BARETK_Program program);

/// Get the endianess of the program.
/// @param program The pointer to the program.
/// @return The endianess value of the program.
/// @note Checks for NULL, but does not check for previously freed program!
BARETK_Endianess baretk_get_endianess(const BARETK_Program program);

/// Get the machine type string of the program.
/// @param program The pointer to the program.
/// @return The string value of the machine type of the program.
/// @note Checks for NULL, but does not check for previously freed program!
const char* baretk_get_machine_type(const BARETK_Program program);

/// Get the segment data of the program.
/// @param program The pointer to the program.
/// @return A struct containing a pointer to a contiguous array of segments and the number of segments in the array.
/// @note Checks for NULL, but does not check for previously freed program!
BARETK_SegmentArray baretk_get_segments(const BARETK_Program program);

/// Get the section data for section named key of the program.
/// @param program The pointer to the program.
/// @return A struct containing an address and a pointer to the section's data bytes.
/// @note Checks for NULL, but does not check for previously freed program!
BARETK_Section baretk_get_section(const BARETK_Program program, const char* key);

/// Disassembles a program.
/// @param program The pointer to the program.
/// @return An opaque pointer to the disassembly data structure, or NULL if load was unsuccessful. Must free the disassembly with 
/// ``baretk_free_disassembly()`` when done. 
/// @note The disassembly will take ownership of the program passed into this function, and the
/// pointer will become invalid. To get the program data again, pass the disassembly into ``baretk_get_program_from_disassembly()``
BARETK_Disassembly baretk_disassemble_from_program(BARETK_Program program);

/// Loads a program from a file and disassembles it.
/// @param path The path to the binary file to load.
/// @return An opaque pointer to the disassembly data structure, or NULL if load was unsuccessful. Must free the disassembly with 
/// ``baretk_free_disassembly()`` when done.
BARETK_Disassembly baretk_disassemble_from_file(const char* path);

/// Returns an opaque pointer to the program associated with a disassembly.
/// @param disasm The pointer to the disassembly.
/// @return An opaque pointer to the program, or NULL if the disassembly is NULL.
/// @note The pointer returned by this function is owned by the disassembly. Do not pass it to ``baretk_free_program()``!
const BARETK_Program baretk_get_program_from_disassembly(const BARETK_Disassembly disasm);

/// Frees the data of a disassembly.
/// @param disasm The pointer to the disassembly.
/// @note Checks for NULL, but does not check for previously freed disassembly!
void baretk_free_disassembly(BARETK_Disassembly disasm);

/// Decompiles a disassembly to a specified language.
/// @param disasm The pointer to the disassembly.
/// @return An opaque pointer to the decomp data structure, or NULL if load was unsuccessful. Must free the decomp with 
/// ``baretk_free_decomp()`` when done. 
/// @note The decomp will take ownership of the disassembly passed into this function, and the
/// pointer will become invalid. To get the disassembly data again, pass the decomp into ``baretk_get_disassembly_from_decomp()``
BARETK_Decomp baretk_decomp_disassembly(BARETK_Disassembly disasm, BARETK_Lang lang);

/// Loads a program from a file and decompiles it.
/// @param path The path to the binary file to load.
/// @return An opaque pointer to the decomp data structure, or NULL if load was unsuccessful. Must free the decomp with 
/// ``baretk_free_decomp()`` when done.
BARETK_Disassembly baretk_decomp_from_file(const char* file);

/// Returns an opaque pointer to the disassembly associated with a decomp.
/// @param decomp The pointer to the decomp.
/// @return An opaque pointer to the disassembly, or NULL if the decomp is NULL.
/// @note The pointer returned by this function is owned by the decomp. Do not pass it to ``baretk_free_disassembly()``!
const BARETK_Disassembly baretk_get_disassembly_from_decomp(const BARETK_Decomp decomp);

/// Frees the data of a decomp.
/// @param decomp The pointer to the decomp.
/// @note Checks for NULL, but does not check for previously freed decomp!
void baretk_free_decomp(BARETK_Decomp disasm);

#endif // BARETK_H_INCLUDED

