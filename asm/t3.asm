#include j_std_lib.asm;

_rom
    stuff: s "abcdefghijklmnopqrstuvwxyz"
    data: s "hello"

_code
    .start:
        ; push the constant value that point to the string
        mov a, .stuff
        add a,2
        push a
        call .STDPrintString
        call .STDend
