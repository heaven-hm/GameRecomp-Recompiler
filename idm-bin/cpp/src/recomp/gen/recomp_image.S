/* Auto-generated: the original PE, embedded so the rebuilt binary runs without
 * it. .incbin reads the copy beside this file, by absolute path (the assembler
 * runs from the build directory). The symbol carries the platform's C prefix. */
#if defined(__APPLE__)
#define SYM(x) _##x
    .section __DATA,__const
#else
#define SYM(x) x
    .section .rodata
#endif
    .p2align 4
    .globl SYM(recomp_original_image)
SYM(recomp_original_image):
    .incbin "/private/var/folders/q0/rj0k1xgn28x14thfz4gnbm5c0000gn/T/opencode/sdk-idm/idm-bin/cpp/src/recomp/gen/recomp_image.bin"
