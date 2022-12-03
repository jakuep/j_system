import os
import sys
import subprocess

input_name = "in.asm"

if len(sys.argv) == 2:
	input_name = sys.argv[1]
	
#copy code from ./asm to assembler folder
asm_handle = open("./asm/" + input_name , "r")
asm = asm_handle.read()
asm_handle.close()
asm_write_handle = open("./j_system/j_assembler/in.asm", "w")
asm_write_handle.write(asm)
asm_write_handle.close()

print("running assembler ...")
pa = subprocess.Popen(["cargo", "run", "--release"], cwd="./j_system/j_assembler/",stdout=subprocess.PIPE, stderr=subprocess.PIPE)
pa.wait()

stdout_pa = pa.stdout.read()

if len(stdout_pa.splitlines()) < 1:
	print("stderr:\n" + pa.stderr.read().decode("utf-8"))
	exit()

if stdout_pa.splitlines()[-1] != b'Ok':
	print("stderr:\n" + pa.stderr.read().decode("utf-8"))
	print("stdout:\n" + stdout_pa.decode("utf-8"))
	print("early exit from build due to assembler error")
	exit()

# read the binay output from the assembler
binary_handle = open("./j_system/j_assembler/out.bin", "r")
bin = binary_handle.read()
binary_handle.close()

# write the binay
write_handle = open("./j_system/j_interpreter/in.bin", "w")
write_handle.write(bin)
write_handle.close()

#clear the output file
clear_handle = open("./j_system/j_interpreter/output.txt", "w")
clear_handle.write("")
clear_handle.close()

print("running interpreter ...")
p = subprocess.Popen(["cargo", "run","--release"], cwd="./j_system/j_interpreter/")#, stderr=subprocess.PIPE), stdout = subprocess.DEVNULL)
p.wait()

#interpreter_err = p.stderr.read().decode("utf-8")

#if "panic" in interpreter_err:
#	print(interpreter_err)
#	exit()

#read interpreter output
output_handle = open("./j_system/j_interpreter/output.txt", "r")
out = output_handle.read()
output_handle.close()

if out != "":
	print("interpreter output:\n" +  out )#p.stdout.read().decode("utf-8"))
