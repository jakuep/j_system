#include j_std_lib.asm;

_rom
_code
	.start:
	; push the size that should be allocated
	push 2
	call .STDmalloc
	; f now contains the pointer to the heap-allocated memory
	;write to the heap
	mov [f],42
	; read from the heap
	mov b,[f]

	; print the value
	call .STDPrintB

	;save the pointer to the allocation 
	push f
	
	;create a new allocation
	
	push 1
	call .STDmalloc
	push f
	;xor a,b
	push f
	call .STDfree
	call .STDend
