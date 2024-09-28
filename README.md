# BARETK

BARETK (Binary Analysis and Reverse Engineering Tool Kit) is a work-in-progress suite of tools that will allow the user to examine and analyze binary files.

## Planned features

* The ability to read information from common executable formats such as ELF and PE.
* The ability to disassemble machine code from executables and binary files.
* The ability to generate appoximate pseudo C code from compiled machine code in executables.
* The ability to search for specific binary sequences inside binaries or executables.
* The ability to run executable files in a sandboxed environment via an interpreter or JIT recompilation.

## Planned supported architectures

* x86 (i386, amd64)
* ARM (AArch32, AArch64)
* RISCV (rv32, rv64)
* More in the future

## Planned supported binary formats

* ELF
* PE/COFF
* Raw Binary
* ROM file
* Disc/Filesystem images (ISO, FAT32, etc.)
