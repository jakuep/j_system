#include j_std_lib.asm;

#define idk 421

_rom 
_code
	.start:
		mov a, $idk
		call .STDPrintA
		call .STDend
