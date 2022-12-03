#include j_std_lib.asm

#define buffersize 10

_rom
_code
    .start:
        push $buffersize
        call .STDmalloc
        push f
        push $buffersize
        call .STDInput
        push f
        call .STDPrintString
        push f
        call .STDfree
        call .STDend

