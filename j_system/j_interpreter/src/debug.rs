use crate::machine::MachineState;
use crate::deserialization::{self, deserialize_asm};

use j_system_definition::register::Register;

use std::collections::HashSet;
use std::io::{self, Write};

const ASM_DISPLAY_SIZE:u64 = 4;
const STACK_DISPLAY_SIZE:u64 = 4;

/// how the Interpreter should continue after leaving
/// the debugger
pub enum ContinueAfterDebug
{
    Quit,
    Continue,
}

pub struct DebugInformation
{
    /// is `None` when not in Debug mode.
    /// Contains all pointers that hava a breakpoint (and symbols?)
    pub debug_mode: Option<HashSet<u64>>,

    pub symbols: Option<Vec<String>>,
    
    /// is `None` when the machine is not stepping in the debugger.
    /// is `Some(<val>)` when the debugger is stepping val-amount of
    /// instructions until it re-hits the debugger
    pub debug_step: Option<u64>,

    // ???
    //pub debug_output: String,
}

pub trait MachineDebug
{
    fn breakpoint(&mut self) -> ContinueAfterDebug;
    fn print_state(&mut self);
    fn print_current_asm(&mut self);
    fn get_regs(&mut self);
    fn peek_stack(&mut self);
    fn peek_instructions(&mut self);
    fn read_mem(&mut self, mem: u64);

    /// returns true if `PC` points to a adress that has a breakpoint
    fn check_breakpoint(&mut self) -> bool;
    
    /// if a breakpoint was hit, this function provides a CI to the user 
    /// in which the user can request information about the machine state
    fn breakpoint_cli(&mut self, rehit: bool) -> ContinueAfterDebug;
}

impl MachineDebug for MachineState
{
    fn check_breakpoint(&mut self) -> bool
    {
        self.debug.debug_mode.as_ref().unwrap().contains(&self.reg_state.read(Register::pc))
    }

    fn read_mem(&mut self, ptr:u64) 
    {
        let s;
        if let Ok(val) = self.mem_state.read(ptr)
        {
            s = format!("{}\n",val);
        }
        else 
        {
            s = format!("could not read address: {}\n", ptr);
        }
        self.machine_information.push_str(s)
    }

    /// displays the breakpoint CLI when a breakpoint is set for this adress
    fn breakpoint(&mut self) -> ContinueAfterDebug
    {
        if let Some(steps) = self.debug.debug_step
        {
            if steps == 0
            {
                self.debug.debug_step = None;
                return self.breakpoint_cli(true); 
            }
            else 
            {
                self.debug.debug_step = Some(steps-1);
            }
        }

        if !self.check_breakpoint()
        {
            return ContinueAfterDebug::Continue
        }

        self.breakpoint_cli(false)
    }

    fn breakpoint_cli(&mut self, rehit: bool) -> ContinueAfterDebug
    {
        // pointer to instruction that is about to be executed
        let addr = self.reg_state.read(Register::pc);

        if rehit
        {   
            print!("resume after step at {}\n",addr);
        }
        else{
            print!("hit breakpoint at: {}\n", addr);
        }

        // initial breakpoint print
        self.print_state();

        // get user command input
        loop
        {
            // no newline??
            print!("(dbg) ");
            // flush due to no newline
            io::stdout().flush().unwrap();

            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            if let Some(command) = get_debug_command(&input)
            {
                match command
                {
                    DebugCommand::PrintRegisters    => {},
                    DebugCommand::MemRead(ptr)      => self.read_mem(ptr),
                    DebugCommand::PrintState        => self.print_state(),
                    DebugCommand::PrintCurrentAsm   => self.print_current_asm(),
                    DebugCommand::Exit              => return ContinueAfterDebug::Quit,
                    DebugCommand::Dump              => {todo!()},
                    DebugCommand::Step(steps)       => {self.debug.debug_step = Some(steps); return ContinueAfterDebug::Continue},
                    DebugCommand::Continue          => return ContinueAfterDebug::Continue,
                }
            }
            else 
            {
                self.machine_information.push_str(format!("unknown command: {}", input));    
            }
        }
    }

    fn print_current_asm(&mut self)
    {
        let pc = self.reg_state.read(Register::pc);

        // the instruction that is about to be executed
        if let Some((inst,_)) = deserialization::deserialize_asm(&mut self.mem_state, pc)
        {
            self.machine_information.push_str(format!("{}\t{}",pc,inst.as_string()))
        }
        else 
        {
            self.machine_information.push_str(format!("cannot read instruction at: {}",pc))    
        }
    }

    fn print_state(&mut self)
    {
        self.machine_information.push_str("----------------------------------\n".into());
        self.get_regs();
        self.peek_instructions();
        self.peek_stack();
    }

    fn get_regs(&mut self)
    {
        let mut s = "".to_string();

        s.push_str(&format!("a: {}\n",self.reg_state.read(Register::a)));
        s.push_str(&format!("b: {}\n",self.reg_state.read(Register::b)));
        s.push_str(&format!("c: {}\n",self.reg_state.read(Register::c)));
        s.push_str(&format!("d: {}\n",self.reg_state.read(Register::d)));
        s.push_str(&format!("e: {}\n",self.reg_state.read(Register::e)));
        s.push_str(&format!("f: {}\n",self.reg_state.read(Register::f)));
        s.push_str(&format!("s: {}\n",self.reg_state.read(Register::s)));
        s.push_str(&format!("pc: {}\n",self.reg_state.read(Register::pc)));
        s.push_str(&format!("tos: {}\n",self.reg_state.read(Register::tos)));
        s.push_str(&format!("bos: {}\n",self.reg_state.read(Register::bos)));

        self.machine_information.push_str(s);

    }

    fn peek_instructions(&mut self)
    {
        self.machine_information.push_str("Instructions:\n".into());

        // pointer to instruction that is about to be executed
        let mut addr = self.reg_state.read(Register::pc);
        
        // no \n
        self.machine_information.push_str("->".into());
        let mut found_next_instriction = false;
        for _ in 0..=ASM_DISPLAY_SIZE
        {
            if let Some((asm,new_addr)) = deserialize_asm(&self.mem_state, addr)
            {
                self.machine_information.push_str(format!("\t{}\t{}",addr, &asm.as_string()));
                addr = new_addr;
                found_next_instriction = true;
            }
        }

        if !found_next_instriction
        {
            self.machine_information.push_str(format!("can not fetch next instruction from: {}\t{:#?}\n",addr,self.mem_state.read(addr)))
        } 
    }

    /// peek at top of stack
    fn peek_stack(&mut self)
    {
        self.machine_information.push_str("Stack:\n".into());
        let tos = self.reg_state.read(Register::tos);
        let stack_size = self.mem_state.get_mem_size() -tos;  // use highest adress for this

        if tos==0 || stack_size==0 
        {
            self.machine_information.push_str("Stack is Empty\n".into());
            return;
        }

        let display_size = if STACK_DISPLAY_SIZE> stack_size {stack_size} else {STACK_DISPLAY_SIZE};

        for ii in (0..display_size).rev()
        {
            // Display data from stack 
            if ii==0
            {
                self.machine_information.push_str("->".into())
            }

            // TODO: -1 until tos fix
            let addr = self.reg_state.read(Register::tos)+ ii;
            // TODO: handle fail 
            let val = self.mem_state.read(addr).unwrap();

            self.machine_information.push_str(format!("\t{:#x}\t{}\n",addr,val));

        }
    }
    
}

pub enum DebugCommand
{
    MemRead(u64),
    PrintRegisters,
    PrintCurrentAsm,
    PrintState,
    Step(u64),
    Continue,
    Dump,
    Exit,
}

fn get_debug_command(command: &str) -> Option<DebugCommand>
{
    let parts:Vec<_> = command.trim().split_ascii_whitespace().filter(|x| !x.is_empty()).collect();

    // check if the input only contained whitespaces
    if parts.len() == 0
    {
        return None
    }

    match (parts[0], parts.len())
    {
        ("exit",1)  => Some(DebugCommand::Exit),
        ("ps",1)    => Some(DebugCommand::PrintState),
        ("ins",1)   => Some(DebugCommand::PrintCurrentAsm),
        
        ("s",1)     => Some(DebugCommand::Step(0)), 
        ("s",2)     => Some(DebugCommand::Step(parts[1].parse::<u64>().ok()?)),

        ("c",1)     => Some(DebugCommand::Continue),
        ("dump",1)  => Some(DebugCommand::Dump),
        ("m",2)     => Some(DebugCommand::MemRead(parts[1].parse::<u64>().ok()?)),

        _ => None 
    }
}