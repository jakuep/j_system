#include j_std_lib.asm

_rom

_code
    .start:
    
    ; inputbuffer size
    push 10
    ; allocate buffer
    call .STDmalloc
    call .STDPrintF
    
    call .STDend


    
    
.modulo:
    .div:
    mov b,[tos+1]
    mov a,[tos+2]
    .divLoop:
    ;check if division is finished
    cmp a,b
    jl .divEnd
    ;perfrom next step
    sub a,b
    jmp .divLoop
    .divEnd:
    ; move remaining value to f 
    mov f,a
    ret 2