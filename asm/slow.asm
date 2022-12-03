 
#include j_std_lib.asm;

_rom
    ;data: ai [25]

_code
    .start:
	mov a, 77031
	mov c,0
	.loop:
	;print the current value
	call .STDPrintA
	cmp a,1
	je .end
	; test if a is even
	mov b,a
	and b,1
	cmp b,0
	je .divsetup
	;if not jumped multiply by 3 and add 1
	mov b,a
	add a,b
	add a,b
	add a,1
	; add 1 to the step count
	add c,1
	jmp .loop
	.divsetup:
	push a
	push 2
	call .div
	;remove stack params
	mov a,f
	; add 1 to the step count
	add c,1
	jmp .loop
	.end:
	;print the steps:
	call .STDPrintC
    call .STDend

	.div:
		mov b,[tos+1]
		mov a,[tos+2]
		mov f,0
		.divLoop:
		;check if division is finished
		cmp a,b
		jl .divEnd
		;perfrom next step
		sub a,b
		add f,1
		jmp .divLoop
		.divEnd:
		ret 2

	.divBy2:
		mov f,[tos+1]
		shr f,1
		ret 1

