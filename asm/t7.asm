#include j_std_lib.asm;
_rom
_code
    .start:
        mov  a, tos
        call .STDPrintA
        call .STDend
