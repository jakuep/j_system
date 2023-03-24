use j_system_definition::register::*;
use j_system_definition::instructions::*;

use crate::deserialization::*;
use crate::memory::MemModel;
use crate::load_bin::Binary;
use crate::debug::{ContinueAfterDebug,MachineDebug,DebugInformation};

use std::fs;
use std::time::Instant;
use std::collections::HashSet;


pub trait Exec
{
    fn run_instruction(&mut self, inst: AsmLine) -> InstructionReturn;
    fn add(&mut self, p1: Option<Param>, p2: Option<Param>) -> InstructionReturn;
    fn mov(&mut self, p1: Option<Param>, p2: Option<Param>) -> InstructionReturn;
    fn jmp(&mut self, p1: Option<Param>, p2: Option<Param>) -> InstructionReturn;
    fn cmp(&mut self, p1: Option<Param>, p2: Option<Param>) -> InstructionReturn;
    fn push(&mut self, p1: Option<Param>, p2: Option<Param>) -> InstructionReturn;
    fn pop(&mut self, p1: Option<Param>, p2: Option<Param>) -> InstructionReturn;
    fn sub(&mut self, p1: Option<Param>, p2: Option<Param>) -> InstructionReturn;
    fn and(&mut self, p1: Option<Param>, p2: Option<Param>) -> InstructionReturn;
    fn je(&mut self, p1: Option<Param>, p2: Option<Param>) -> InstructionReturn;
    fn jl(&mut self, p1: Option<Param>, p2: Option<Param>) -> InstructionReturn;
    fn jel(&mut self, p1: Option<Param>, p2: Option<Param>) -> InstructionReturn;
    fn shr(&mut self, p1: Option<Param>, p2: Option<Param>) -> InstructionReturn;
    fn shl(&mut self, p1: Option<Param>, p2: Option<Param>) -> InstructionReturn;
    fn sys(&mut self, p1: Option<Param>, p2: Option<Param>) -> InstructionReturn;
    fn ret(&mut self, p1: Option<Param>, p2: Option<Param>) -> InstructionReturn;
    fn call(&mut self, p1: Option<Param>, p2: Option<Param>) -> InstructionReturn;
    fn or(&mut self, p1: Option<Param>, p2: Option<Param>) -> InstructionReturn;
    // unimplemented!("TODO: popa"),
    // unimplemented!("TODO: pusha"),
    fn jeg(&mut self, p1: Option<Param>, p2: Option<Param>) -> InstructionReturn;
    fn jg(&mut self, p1: Option<Param>, p2: Option<Param>) -> InstructionReturn;
    fn xor(&mut self, p1: Option<Param>, p2: Option<Param>) -> InstructionReturn;
}

pub struct MachineInitInfo
{
    pub max_cycles: u128,
    pub mem_size: u64,
    /// is Some when machine is in debug mode,
    /// contains a list of all breakpoints
    /// and a list of "debug Symbols"?
    pub debug_mode: Option<HashSet<u64>>,
    pub write_to_file: bool,
}

pub struct MachineState
{
    pub machine_information: MachineInformation,
    pub mem_state: MemModel,
    pub reg_state: RegisterState,
    pub next_ptr: u64,
    pub debug: DebugInformation,
}


pub struct MachineInformation
{
    write_to_file: bool,
    max_cycles: u128,

    /// contains the outpout that should be printed to stdout.
    /// it also contains the cycle count in that the print was triggered
    output: Vec<(u128,String)>,

    /// keep track of how many istructions have been run
    cycle_count:u128,
}

impl MachineInformation
{
    pub fn inc_cycle(&mut self)
    {
        self.cycle_count +=1;
    }

    pub fn get_cycles(&self) -> u128
    {
        self.cycle_count
    } 

    pub fn push_str(&mut self, s: String)
    {
        if self.write_to_file
        {
            self.output.push((self.get_cycles(),s))
        }
        else
        {
            print!("{}",s)
        }
    }

    pub fn get_output(self) -> Vec<(u128,String)>
    {
        self.output
    }

    pub fn print_output(&self, print_with_cycle: bool) -> String
    {
        let mut output = String::from("");

        // no newline at the end of the prints since the get added in the syscall already
        if print_with_cycle
        {
            (&self.output).into_iter().for_each(|(cy,s)| output.push_str(&format!("{cy}: {s}")));
        }
        else
        {
            (&self.output).into_iter().for_each(|(_,s)| output.push_str(&format!("{s}")));
        }

        output
    }
}

pub enum InstructionReturn
{
    /// contains the error code that lead to failure
    Err(String),
    End,
    Next,
    //AwaitInput,
    JumpTo(u64),
}
const PRINT_STACK:bool = false;

impl MachineState
{
    pub fn init(config: MachineInitInfo) -> Self
    {
        let MachineInitInfo{max_cycles, mem_size, debug_mode, write_to_file} = config;

        // mem_size: u64,breakpoints: Option< HashSet<u64>>

        //crate a new machine state instance
        let mem = MemModel::new(mem_size);
        let reg: RegisterState = RegisterState::new();  //{a:0,b:0,c:0,d:0,e:0,f:0,s:0,pc:0,tos: u64::MAX,bos: u64::MAX};
        
        // return initial machine state
        Self{
            machine_information: MachineInformation{output: vec![], cycle_count: 0,max_cycles,write_to_file},
            mem_state:mem, 
            reg_state:reg,
            // will be set when loading of binary
            next_ptr: 0,

            // TODO: add the breakpoints/parse from debug output file
            debug: DebugInformation{debug_mode,debug_step: None, symbols: None}
        }
    }

    pub fn laod_into_state(&mut self, input: Binary)
    {   
        let Binary{code,rom,start_ptr} = input;

        self.mem_state.prepare_mem(rom,code);
        self.next_ptr = start_ptr;
    }

    pub fn run_program(mut self)
    {

        let now = Instant::now();

        // set pc to the pointer of the start label
        self.reg_state.store_to_read_only(Register::pc, self.next_ptr);
        
        loop
        {   
            // get the pointer of the next instruction
            let inst_ptr = self.reg_state.read(Register::pc);

            // TODO: should the breakpoint hit before the inst in executed?
            // hit a breakpoint?
            
            if self.debug.debug_mode.is_some()
            {
                match self.breakpoint()
                {
                    ContinueAfterDebug::Continue    => {},
                    ContinueAfterDebug::Quit        => break,
                }
            }

            
            // decode the instruction to run and get the pointer of 
            // the next instruction to in line
            // next_ptr may be altered by a jump or a call command
            // since it does just increases the pointer by 1,2 or 3 
            // depending on the size of the instruction
            let (inst,next_ptr) = deserialize_asm(&mut self.mem_state, inst_ptr).unwrap();

            self.next_ptr = next_ptr;

            if self.machine_information.get_cycles() >= self.machine_information.max_cycles
            {
                panic!("cycle count surpassed");
                //crate::output::dump_and_panic(format!("maximum amount of allowed cycles surpassed"), register_state, stack_state);
            }

            /*
            if current_line >= code.len() as u64
            {
                crate::output::dump_and_panic(format!("No more programm code to execute and no end statement"), register_state, stack_state);
            }
            */

            let ret = self.run_instruction(inst);
            
            match ret
            {
                InstructionReturn::Err(msg) => 
                {
                    panic!("{}",msg);
                    //crate::output::dump_and_panic(format!("instruction returned with error in code line: {}",current_line), register_state, stack_state);
                },

                InstructionReturn::End => break,
                InstructionReturn::Next => self.reg_state.store_to_read_only(Register::pc, self.next_ptr),
                InstructionReturn::JumpTo(ptr) => self.reg_state.store_to_read_only(Register::pc, ptr),
                //crate::instructions::InstructionReturn::AwaitInput => break // TODO: how should input work?
            }
            
            self.machine_information.inc_cycle();
        }

        let mut output = String::new();

        if self.debug.debug_mode.is_some(){
            //print_register_state(&register_state);
            //print_stack_state(&stack_state);
            output.push_str(&format!("program output:\n"));

            // print the machine output with cycle information
            output.push_str(&self.machine_information.print_output(true));

            output.push_str(&format!("-------------------\n"));
            //output.push_str(&format!("---end---\n"));
            output.push_str(&format!("cycles used: {}\n",self.machine_information.get_cycles()));
            output.push_str(&format!("stack pointer: {}\n",self.reg_state.read(Register::tos)));
            output.push_str(&format!("heap-cutoff pointer: {}\n",self.mem_state.get_heap_cutoff()));
            let exec_time = now.elapsed();
            output.push_str(&format!("program run time: {}s \n", exec_time.as_secs()));

            let cy_per_milli = if exec_time.as_millis() == 0 {self.machine_information.get_cycles()} else { self.machine_information.get_cycles() / exec_time.as_millis()};
            output.push_str(&format!("cycels per milli: {}\n", cy_per_milli));
            if PRINT_STACK 
            {
                output.push_str("stack:\n");
                // TOD0: print stack
                
                //output.push_str(&self.mem_state.stack.into_iter().map(|x| format!("{x}\n")).reduce(|a,b| a+&b).unwrap_or("".to_string()));
            }
        }

        if self.machine_information.write_to_file
        {
            fs::write("output.txt",&output).expect("err lulw");
        }
        else
        {
            self.machine_information.push_str(output)
        }
    }
}
