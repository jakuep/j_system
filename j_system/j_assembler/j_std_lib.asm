_code
    ;syscall mask function for malloc
    .STDmalloc:
        mov a,[tos+1]
        push a
		push 1
        sys
        ret 1

    ;syscall mask function for free
    .STDfree:
		mov a,[tos+1]
        push a
		push 2
        sys
        ret 1
		
	;syscall mask function for Quittig the program
	.STDend:
		push 9
		sys
		ret 0
		
	.STDPrintA:
		push 1
		push 1
		push 8
		sys
		ret 0

	.STDPrintB:
		push 2
		push 1
		push 8
		sys
		ret 0
		
	.STDPrintC:
		push 3
		push 1
		push 8
		sys
		ret 0

	.STDPrintD:
		push 4
		push 1
		push 8
		sys
		ret 0
	
	.STDPrintE:
		push 5
		push 1
		push 8
		sys
		ret 0

	.STDPrintF:
		push 6
		push 1
		push 8
		sys
		ret 0

	.STDPrintTOS:
		push 7
		push 1
		push 8
		sys
		ret 0

	.STDPrintBOS:
		push 8
		push 1
		push 8
		sys
		ret 0

	.STDPrintPC:
		push 9
		push 1
		push 8
		sys
		ret 0

	.STDPrintS:
		push 10
		push 1
		push 8
		sys
		ret 0

	.STDPrintString:
		mov f, [tos+1]
		push f
		push 2
		push 8
		sys
		ret 1

	; first push the buffer ptr
	; second push the buffer size
	; size includes the null termination
	.STDInput:
		mov a, [tos+2]
		mov b, [tos+1]
		push a
		push b
		push 7
		sys
		ret 2	

_rom
