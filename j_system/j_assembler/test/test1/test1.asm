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
mov b,a
