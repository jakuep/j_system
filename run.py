import build.basic_build as bb
import os
import sys

run_type = "normal"
file_name= "slow.asm"

if len(sys.argv) >= 2:
	file_name = sys.argv[1]

if len(sys.argv) >= 3:
	run_type = sys.argv[2]

# quit if no filename was supplied
if file_name == "":
    print("no filename supplied")
    exit()

bb.copy_asm_to_assembler(file_name)

print("checking assembler build... ",end="")
if bb.check_build("./j_system/j_assembler"):
    print("OK")
else:
    print("failed")
    exit()

print("running assembler on "+ file_name+ "... ", end="")
if bb.check_assembler_output(bb.run_assembler()):
    print("OK")
else:
    print("failed")
    exit()

bb.copy_bin_to_interpreter()

print("checking interpreter build... ",end="")
if bb.check_build("./j_system/j_interpreter"):
    print("OK")
else:
    print("failed")
    exit()

print("running interpreter ...")

if bb.run_interpreter(run_type):
    print("\nOK")
else:
    print("\nfailed")

print("--done--")


bb.copy_asm_to_assembler(file_name)
