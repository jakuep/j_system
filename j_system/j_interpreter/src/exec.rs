use j_system_definition::register::*;
use j_system_definition::instructions::*;

use crate::check_instruction::*;
use crate::deserialization::*;
use crate::memory::MemModel;
use crate::syscall::*;
use crate::load_bin::Binary;
use crate::debug::{ContinueAfterDebug,MachineDebug,DebugInformation};

use std::fs;
use std::time::Instant;
use std::collections::HashSet;

pub trait Exec
{
    fn run_instruction(&mut self, inst: AsmLine);
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

            let ret = run_instruction(&mut self, inst);
            
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

// TODO: make this as impl of Machine State
pub fn run_instruction(state: &mut MachineState, inst: AsmLine) -> InstructionReturn
{
    match inst.instruction
    {
        InstructionEnum::add    => add(inst.get_param1(),inst.get_param2(),state),
        InstructionEnum::mov    => mov(inst.get_param1(),inst.get_param2(),state),
        InstructionEnum::jmp    => jmp(inst.get_param1(),inst.get_param2(),state),
        InstructionEnum::cmp    => cmp(inst.get_param1(),inst.get_param2(),state),
        InstructionEnum::push   => push(inst.get_param1(),inst.get_param2(),state),
        InstructionEnum::pop    => pop(inst.get_param1(),inst.get_param2(),state),
        InstructionEnum::sub    => sub(inst.get_param1(),inst.get_param2(),state),
        InstructionEnum::and    => and(inst.get_param1(),inst.get_param2(),state),
        InstructionEnum::je     => je(inst.get_param1(),inst.get_param2(),state),
        InstructionEnum::jl     => jl(inst.get_param1(),inst.get_param2(),state),
        InstructionEnum::jel    => jel(inst.get_param1(),inst.get_param2(),state),
        InstructionEnum::shr    => shr(inst.get_param1(),inst.get_param2(),state),
        InstructionEnum::shl    => shl(inst.get_param1(),inst.get_param2(),state),
        InstructionEnum::sys    => sys(inst.get_param1(),inst.get_param2(),state),
        InstructionEnum::ret    => ret(inst.get_param1(),inst.get_param2(),state),
        InstructionEnum::call   => call(inst.get_param1(),inst.get_param2(),state),
        InstructionEnum::or     => or(inst.get_param1(),inst.get_param2(),state),
        InstructionEnum::popa   => unimplemented!("TODO: popa"),
        InstructionEnum::pusha  => unimplemented!("TODO: pusha"),
        InstructionEnum::jeg    => jeg(inst.get_param1(),inst.get_param2(),state),
        InstructionEnum::jg     => jg(inst.get_param1(),inst.get_param2(),state),
        InstructionEnum::xor    => xor(inst.get_param1(),inst.get_param2(),state),
    }
}

pub fn jg(param1: Option<Param>, param2: Option<Param>,state: &mut MachineState) -> InstructionReturn
{
    if !check_only_one_param(&param1, &param2)
    {
        return InstructionReturn::Err("jg needs 1 parameter".to_string());
    }

    if state.reg_state.read(Register::s) & 1<<2 != 0
    {
        InstructionReturn::JumpTo(get_param_value(&param1, state))
    }
    else
    {
        InstructionReturn::Next
    }
}

pub fn jeg(param1: Option<Param>, param2: Option<Param>,state: &mut MachineState) -> InstructionReturn
{
    if !check_only_one_param(&param1, &param2)
    {
        return InstructionReturn::Err("jeg needs 1 parameter".to_string());
    }

    if state.reg_state.read(Register::s) & (1<<3 | 1<<2) != 0
    {
        InstructionReturn::JumpTo(get_param_value(&param1, state))
    }
    else
    {
        InstructionReturn::Next
    }
}

fn xor(param1: Option<Param>, param2: Option<Param>,state: &mut MachineState) -> InstructionReturn
{
    let op1 = get_param_value(&param1, state);
    let op2 = get_param_value(&param2, state);
    let dest = param1.unwrap();

    let res = op1 ^ op2;

    if let Param::Register(reg) = dest
    {
        state.reg_state.store(reg, res);
        InstructionReturn::Next
    }
    else
    {
        InstructionReturn::Err("first parameter of xor must be a register".to_string())
    }   
}

fn or(param1: Option<Param>, param2: Option<Param>,state: &mut MachineState) -> InstructionReturn
{
    let op1 = get_param_value(&param1, state);
    let op2 = get_param_value(&param2, state);
    let dest = param1.unwrap();

    let res = op1 | op2;

    if let Param::Register(reg) = dest
    {
        state.reg_state.store(reg, res);
        InstructionReturn::Next
    }
    else
    {
        InstructionReturn::Err("first parameter of or must be a register".to_string())
    }   
}

pub fn add(param1: Option<Param>, param2: Option<Param>,state: &mut MachineState) -> InstructionReturn
{
    
    if !crate::check_instruction::check_two_param(&param1,&param2) || !crate::check_instruction::is_register(&param1)
    {
        return InstructionReturn::Err("wrong paramters for add".to_string());
    }

    let val = get_param_value(&param2, state);

    if let Some(reg_some) = param1
    {
        if let Param::Register(reg) = reg_some
        {
            let read_register_val = state.reg_state.read(reg);
            let write_to_register;
            if (read_register_val as u128 + val as u128) > u64::MAX as u128
            {
                let overflowed_value = read_register_val as u128 + val as u128;
                
                // a addition of two u64 can be at maximum 65 bit large
                // since the sum is bigger than u64::MAX the 65. must be set
                // (u64::MAX as u128 +1) does only have the 65 bit set
                // by subracting the 65 bit the result is the lower 8 byte of the number
                let lower_8_byte_of_overflowed_value = overflowed_value - (u64::MAX as u128 +1); 
                write_to_register = lower_8_byte_of_overflowed_value as u64;
                //TODO: Set carry or overflow bit in status register
            }
            else
            {
                write_to_register = read_register_val +val;
            }

            state.reg_state.store(reg, write_to_register);

        }

        InstructionReturn::Next
    }
    else
    {
        InstructionReturn::Err("paramter 1 in add must be a register".to_string())
    }

    
}

pub fn sub(param1: Option<Param>, param2: Option<Param>,state: &mut MachineState) -> InstructionReturn
{
    if !crate::check_instruction::check_two_param(&param1,&param2) && !crate::check_instruction::is_register(&param1)
    {
        return InstructionReturn::Err("wrong paramters for add".to_string());
    }

    let val = crate::check_instruction::get_param_value(&param2, state);

    if let Some(reg_some) = param1
    {
        if let Param::Register(reg) = reg_some
        {
            let read_register_val = state.reg_state.read(reg);
            let store:u64;
            if read_register_val < val
            {
                store = 0;
            }
            else
            {
                store = read_register_val - val;
            }
            state.reg_state.store(reg, store); //TODO: check for overflow
        }
    }
    else
    {
        return InstructionReturn::Err("paramter 1 in add must be a register".to_string());
    }

    InstructionReturn::Next
}

pub fn mov(param1: Option<Param>, param2: Option<Param>,state: &mut MachineState) -> InstructionReturn
{
    //TODO: Does at least one of the parameters have to be a register?
    if !crate::check_instruction::check_two_param(&param1,&param2)
    {
        return InstructionReturn::Err("mov needs 2 paramters".to_string());
    }
    
    let val = crate::check_instruction::get_param_value(&param2,state);
    crate::check_instruction::store_in_dest(val, &param1,state);

    InstructionReturn::Next
}

pub fn jmp(param1: Option<Param>, param2: Option<Param>,state: &mut MachineState) -> InstructionReturn
{
    if check_only_one_param(&param1, &param2)
    {
        let val = crate::check_instruction::get_param_value(&param1, state);
        InstructionReturn::JumpTo(val)
    }
    else
    {
        InstructionReturn::Err("jmp needs 1 paramter".to_string())
    }   
}

pub fn cmp(param1: Option<Param>, param2: Option<Param>,state: &mut MachineState) -> InstructionReturn
{
    let val1 = crate::check_instruction::get_param_value(&param1, state);
    let val2 = crate::check_instruction::get_param_value(&param2, state);

    // reset the staus register
    state.reg_state.store_to_read_only(Register::s, 0);
    
    match val1 as i128 - val2 as i128
    {
        x if x<0    => state.reg_state.store_to_read_only(Register::s, 1<<1),
        x if x>0    => state.reg_state.store_to_read_only(Register::s, 1<<2),
        x if x==0   => state.reg_state.store_to_read_only(Register::s, 1<<3),
        _           => unreachable!()
    }
    
    InstructionReturn::Next
}

pub fn push(param1: Option<Param>, param2: Option<Param>,state: &mut MachineState) -> InstructionReturn
{
    if !check_only_one_param(&param1, &param2)
    {
        return InstructionReturn::Err("push needs 1 paramter".to_string())
    }

    let val = crate::check_instruction::get_param_value(&param1, state);
    
    // TODO: handle result
    push_stack(state,val).unwrap();
    
    InstructionReturn::Next
}

pub fn pop(param1: Option<Param>, param2: Option<Param>,state: &mut MachineState) -> InstructionReturn
{
    if !is_register(&param1) || !check_only_one_param(&param1, &param2)
    {
        return InstructionReturn::Err("pop needs 1 paramter".to_string())
    }

    match pop_stack(state)
    {
        Ok(val) => store_in_dest(val, &param1, state),
        Err(msg) => return InstructionReturn::Err(msg)
        //crate::output::dump_and_panic(format!("Stack is empty"), register_state, stack_state);
    }

    InstructionReturn::Next
}

pub fn je(param1: Option<Param>, param2: Option<Param>,state: &mut MachineState) -> InstructionReturn
{
    if !check_only_one_param(&param1, &param2)
    {
        return InstructionReturn::Err("je needs 1 parameter".to_string());
    }

    if state.reg_state.read(Register::s) & 1<<3 != 0
    {
        InstructionReturn::JumpTo(get_param_value(&param1, state))
    }
    else
    {
        InstructionReturn::Next
    }
}

pub fn jel(param1: Option<Param>, param2: Option<Param>,state: &mut MachineState) -> InstructionReturn
{
    if !check_only_one_param(&param1, &param2)
    {
        return InstructionReturn::Err("jel needs 1 parameter".to_string());
    }

    if state.reg_state.read(Register::s) & (1<<3|1<<1) != 0
    {
        InstructionReturn::JumpTo(get_param_value(&param1, state))
    }
    else
    {
        InstructionReturn::Next
    }
}

pub fn jl(param1: Option<Param>, param2: Option<Param>,state: &mut MachineState) -> InstructionReturn
{
    if !check_only_one_param(&param1, &param2)
    {
        return InstructionReturn::Err("jl needs 1 parameter".to_string());
    }

    if state.reg_state.read(Register::s) & 1<<1 != 0
    {
        InstructionReturn::JumpTo(get_param_value(&param1, state))
    }
    else
    {
        InstructionReturn::Next
    }
}

pub fn ret(param1: Option<Param>, param2: Option<Param>,state: &mut MachineState) -> InstructionReturn
{
    if !(param1.is_some() && param2.is_none())
    {
        return InstructionReturn::Err("ret needs 1 paramter".to_string())
    }

    let stack_clean_amount = get_param_value(&param1, state);

    // get the retrun adress
    // add 1 to not jump to the call again but the instruction after that
    // TODO: handle empty stack
    let jmp_adress = pop_stack(state).unwrap();

    // remove arguments on stack
    for _ in 0..stack_clean_amount //as usize
    { 
        // add the ammount 
        let _ = pop_stack(state);
    }

    InstructionReturn::JumpTo(jmp_adress)
}

pub fn and(param1: Option<Param>, param2: Option<Param>,state: &mut MachineState) -> InstructionReturn
{
    // TODO: check for valid parameters

    let op1 = get_param_value(&param1, state);
    let op2 = get_param_value(&param2, state);
    let dest = param1.unwrap();

    let res = op1 & op2;

    if let Param::Register(reg) = dest
    {
        state.reg_state.store(reg, res);
        InstructionReturn::Next
    }
    else
    {
        InstructionReturn::Err("first parameter of and must be a register".to_string())
    }
}

pub fn shr(param1: Option<Param>, param2: Option<Param>,state: &mut MachineState) -> InstructionReturn
{
    if !check_two_param(&param1, &param2)
    {
        return InstructionReturn::Err("shr needs 2 parameters".to_string());
    }

    if !is_register(&param1)
    {
        return InstructionReturn::Err("the first parameter of shr must be a register".to_string())
    }

    let shift_amount = get_param_value(&param2, state);

    if shift_amount > 64
    {
        return InstructionReturn::Err("cannot shift by more than 64 bits".to_string());
    }

    let mut val = get_param_value(&param1, state);

    val = val >> shift_amount;

    store_in_dest(val, &param1, state);

    InstructionReturn::Next
}

pub fn shl(param1: Option<Param>, param2: Option<Param>,state: &mut MachineState) -> InstructionReturn
{
    if !check_two_param(&param1, &param2)
    {
        return InstructionReturn::Err("shl needs 2 parameters".to_string());
    }

    if !is_register(&param1)
    {
        return InstructionReturn::Err("the first parameter of shl must be a register".to_string())
    }

    let shift_amount = get_param_value(&param2, state);

    if shift_amount > 64
    {
        return InstructionReturn::Err("cannot shift by more than 64 bits".to_string());
    }

    let mut val = get_param_value(&param1, state);

    val = val << shift_amount;

    store_in_dest(val, &param1, state);

    InstructionReturn::Next
}

pub fn call(param1: Option<Param>, param2: Option<Param>,state: &mut MachineState) -> InstructionReturn
{
    if !check_only_one_param(&param1, &param2)
    {
        return InstructionReturn::Err("call only needs 1 parameter".to_string());
    }

    //TODO: save registers?, stackframe?
    
    // save return adress
    push_stack(state, state.next_ptr).unwrap();

    InstructionReturn::JumpTo(get_param_value(&param1, state))
}

pub fn sys(param1: Option<Param>, param2: Option<Param>,state: &mut MachineState) -> InstructionReturn
{
    if !(param1.is_none() && param2.is_none())
    {
        return InstructionReturn::Err("sys does not get any parameters".into())
    }

    // TODO:    for now sys only prints the register state.
    //          -> change it to call the syscall handler

    match syscall(state)
    {
        SysCallAction::End      => InstructionReturn::End,
        SysCallAction::Err(x)   => InstructionReturn::Err(x),
        SysCallAction::Ok       => InstructionReturn::Next,
        SysCallAction::Ptr(x)   => { state.reg_state.store(Register::f, x); InstructionReturn::Next}
    }

}
