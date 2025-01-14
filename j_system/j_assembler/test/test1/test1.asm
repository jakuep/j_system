#include abc.asm
#export b,c
#set norom

_code
add a, $number
add a, .testlabel
add a, [41]
add a, [a]
add a, [a+1]
add a, [a-1]
add a, [.testlabel]
add a, [.testlabel+1]
add a, [.testlabel-1]
add a,32
; test parse all instructions
add a,1
sub a,1
xor a,1
or  a,1
and a,1
shr a,1
shl a,1
jmp a
cmp a,1
je  a
jeg a 
jel a
jg a
jl a
mov a,1
push a
pop a
pusha 
popa 
call a
ret 1
sys 

.test:
mov b,a
