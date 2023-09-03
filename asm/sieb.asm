#include j_std_lib.asm

_rom

_code
    .start:
    mov a,0
    push 12
    call .getuserinput
    mov b,a

    call .STDend


.getuserinput:
    ; inputbuffer size
    push 10
    ; allocate buffer
    call .STDmalloc



    ; input syscall
    push f
    push 10
    call .STDInput

    ; number placeholder / return value
    mov a,0

    ; count iterations
    mov c,0

    ; check input for digits
    ; loop unitl null termintion
    ; b holds next address, f is base ptr and add iterations from c
    mov b,0
    add b,f
    add b,c
    ; check null termintion
    cmp [b],0
    je .endloopinput
    ; continue if not finished
    ; move char into register d
    mov d,[b]
    
    ; test if the char is ascii
    ; lower boundary
    cmp 48,d
    jg .errinput
    ; upper boundary
    cmp 57,d
    jl .errinput

    ; from here on char is considered valid
    
    ; get numeric-value form ascii char
    sub d,48
    
    shl d,c



    .endloopinput:
    .errinput:
    push f
    call .STDfree
    ret 0


.mulBy10:
    mov a,[tos+1]
    mov f,0
    add f,a
    add f,a
    add f,a
    add f,a
    add f,a
    add f,a
    add f,a
    add f,a
    add f,a
    add f,a
    ret 1


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
