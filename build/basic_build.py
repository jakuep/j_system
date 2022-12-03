import os
import sys
import subprocess


def copy_asm_to_assembler(file_name):
    asm_handle = open("./asm/" + file_name , "r")
    asm = asm_handle.read()
    asm_handle.close()
    asm_write_handle = open("./j_system/j_assembler/in.asm", "w")
    asm_write_handle.write(asm)
    asm_write_handle.close()


def run_assembler():
    #print("running assembler ...")
    pa = subprocess.Popen(["cargo", "run", "--release"], cwd="./j_system/j_assembler/",stdout=subprocess.PIPE, stderr=subprocess.PIPE)

    # tuple (outs, errs)
    return pa.communicate()

def check_assembler_output(inp):
    outs, _ = inp
    return len(outs)>0 and outs.splitlines()[-1] == b'Ok'

def copy_bin_to_interpreter():
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

def run_interpreter(mode):
    if mode == "debug":
        p = subprocess.Popen(["cargo", "run","--release", "--", "-d"], cwd="./j_system/j_interpreter/",stderr=subprocess.DEVNULL)
        p.wait()
        return p.returncode == 0
    else:
        p = subprocess.Popen(["cargo", "run","--release"], cwd="./j_system/j_interpreter/",stderr=subprocess.DEVNULL)
        p.wait()
        return p.returncode == 0

# check for successful build
def check_build(path):
    pa = subprocess.Popen('cargo build --release', shell=True, cwd=path,stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    pa.wait()
    return pa.returncode == 0

def _check_build(path):
    pa = subprocess.Popen('cargo build --message-format json-diagnostic-short > output.json', shell=True, cwd=path,stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    pa.wait()
    #pa = subprocess.run('cargo build > output.json', shell=True, cwd="./whatever")
    # ["cargo", "build", "--release", ">" , "output.json"]

    h = open(path+"/output.json", "r")
    json_output = h.read()
    h.close()
    os.remove(path+"/output.json")
    return json_output.splitlines()[-1] == "{\"reason\":\"build-finished\",\"success\":true}"
