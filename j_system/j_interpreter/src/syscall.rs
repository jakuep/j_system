use j_system_definition::register::Register;
use crate::machine::*;


use std::io;

// const definition of param offsets
const PARAM1:u64 = 0;
const PARAM2:u64 = 1;
const PARAM3:u64 = 2;
const PARAM4:u64 = 3;

/// syscall types that can be called from the programm 
/// running in the interpreter 
/// can be accessed by the "sys" command
pub enum SysCallType
{
    /// takes a Ineger as value that dictates the size
    /// of the memory block.
    /// returns a pointer to the allocated block
    /// retruns 0 when given a size of 0 or a size that
    /// cant be fitted in the remaining free space 
    Malloc,
    Free,
    MemCopy,
    SysInfo,
    Rand,
    Time,
    ReadFromStdIn,
    PrintToStdOut,
    End,
}

pub enum SysCallAction
{
    Ok,
    End,
    Ptr(u64),
    Err(String),
}

pub fn identify_syscall(state: &mut MachineState) -> SysCallType
{
    // get the type of syscall from the top of the stack
    // the syscall type is the first parameter and due to
    // this calling convention it is on the top of the stack

    match state.mem_state.read(state.reg_state.read(Register::tos)+PARAM1).ok()
    {
        Some(1) => SysCallType::Malloc,
        Some(2) => SysCallType::Free,
        Some(3) => SysCallType::MemCopy,
        Some(4) => SysCallType::SysInfo,
        Some(5) => SysCallType::Rand,
        Some(6) => SysCallType::Time,
        Some(7) => SysCallType::ReadFromStdIn,
        Some(8) => SysCallType::PrintToStdOut,
        Some(9) => SysCallType::End,
        x       => panic!("unkonwn syscall {:#?}",x)
    }
}

pub fn syscall(state: &mut MachineState) -> SysCallAction
{
    // update heap-cutoff
    state.mem_state.set_heap_cutoff(state.reg_state.read(Register::tos)-1);

    match identify_syscall(state)
    {
        SysCallType::PrintToStdOut  => print_to_std_out(state),
        SysCallType::End            => end(state),
        SysCallType::Malloc         => malloc(state),
        SysCallType::Free           => free(state),
        SysCallType::ReadFromStdIn  => input(state),

        // TODO: other syscalls
        _                           => SysCallAction::Err("could not identify syscall".to_string()),
    }
}

pub fn malloc(state: &mut MachineState) -> SysCallAction
{
    if let Ok(arg) = state.mem_state.read(state.reg_state.read(Register::tos)+PARAM2)
    {
        if let Some(ptr) = state.mem_state.malloc(arg)
        {
            let _ = remove_stack_entries(state, 2);
            return SysCallAction::Ptr(ptr);
        }
    }

    SysCallAction::Err("could not allocate memory".to_string())
}

pub fn free(state: &mut MachineState) -> SysCallAction
{
    if let Ok(arg) = state.mem_state.read(state.reg_state.read(Register::tos)+PARAM2)
    {
        let _ = remove_stack_entries(state, 2);
        state.mem_state.heap_free(arg)
    }

    // TODO: handle not ok??
    SysCallAction::Ok 
}

pub fn mem_copy(state: &mut MachineState) -> SysCallAction
{
    let mut size =      0u64;
    let mut from_ptr =  0u64;
    let mut to_ptr =    0u64;

    SysCallAction::Ok
}

pub fn end(state: &mut MachineState) -> SysCallAction
{
    // remoev the syscall number
    // NOTE: the return adress will remain on the stack if end is inside a std wrapper function. 
    let _ = remove_stack_entries(state, 1);
    SysCallAction::End
}

pub fn print_to_std_out(state: &mut MachineState) -> SysCallAction
{

    if let (Ok(print_type), Ok(content)) 
    =   (state.mem_state.read(state.reg_state.read(Register::tos)+PARAM2)
        ,state.mem_state.read(state.reg_state.read(Register::tos)+PARAM3))
    {
        match print_type
        {
            // print a register
            1 => 
            {
                if let Err(x) = print_register(state,content)
                {
                    return SysCallAction::Err(x)
                }
            },

            // print a null terminated String that is uncompressed
            // (uncompressed: can only contain one char in the 64bit value)
            // content is the pointer to the string?
            2 => 
            {
                if let Err(x) = print_string_terminated_uncompressed(state,content)
                {
                    return SysCallAction::Err(x)
                }
            }

            // TODO: more print types ...
            x => return SysCallAction::Err(format!("non defined print-type: {}", x)),
        }
    }
    // remove the stack params
    // TODO: handle result
    let _ = remove_stack_entries(state, 3);

    SysCallAction::Ok
}

pub fn print_string_terminated_uncompressed(state: &mut MachineState, mut adress: u64) -> Result<(),String>
{
    let mut s = String::from("");

    loop 
    {
        let c = state.mem_state.read(adress)?;

        // check if the value can be representetd as ascii symbol
        if c <= 127
        {
            // stop when the null termination is reached
            if c == 0
            {
                break
            }
            s.push(c as u8 as char)
        }
        else
        {
            return Err(format!("non acii value {} at {}", c, adress))
        }
        adress += 1;
    }
    // push string and add newline
    // TODO: maybe the newline should be included in the string?
    //       the assembler cant do that yet
    state.machine_information.push_str(s+ "\n");
    Ok(())
}

pub fn input(state: &mut MachineState) -> SysCallAction
{
    if let (Ok(buffer_size), Ok(buffer_ptr)) 
    =   (state.mem_state.read(state.reg_state.read(Register::tos)+PARAM2)
        ,state.mem_state.read(state.reg_state.read(Register::tos)+PARAM3))
    {
        let mut user_input = String::new();
        io::stdin().read_line(&mut user_input).unwrap();

        user_input = user_input.trim().to_string();

        let input_len = user_input.len();

        let chars = user_input.chars();

        //let mut idk:Vec<_> = user_input.chars().collect();

        // all chars have to be ascii
        if !user_input.chars().all(|x| x.is_ascii())
        {
            return SysCallAction::Err("all input characters have to be ascii".into())
        }

        // since at this point all chars are ascii
        // the len of the string equals the amount of u8 elements?
        // +1 because of Null termination to the input that will be added later
        if input_len + 1 > buffer_size as usize
        {
            return SysCallAction::Err("provided buffer is to small to store the input".into())
        }

        // convert chars to u64
        let vec_buffer: Vec<_> = chars.map(|c| c as u8 as u64).collect();

        // write chars to buffer
        for ii in 0..input_len as u64
        {
            state.mem_state.store(vec_buffer[ii as usize], buffer_ptr+ii).unwrap();
        }

        // add null termination
        state.mem_state.store(0, buffer_ptr + buffer_size -1).unwrap();
    }

    let _ = remove_stack_entries(state, 3);

    SysCallAction::Ok
}

pub fn print_register(state: &mut MachineState, content: u64) -> Result<(),String>
{
    match content
    {
        1   => state.machine_information.push_str(format!("a:{}\n",state.reg_state.read(Register::a))),
        2   => state.machine_information.push_str(format!("b:{}\n",state.reg_state.read(Register::b))),
        3   => state.machine_information.push_str(format!("c:{}\n",state.reg_state.read(Register::c))),
        4   => state.machine_information.push_str(format!("d:{}\n",state.reg_state.read(Register::d))),
        5   => state.machine_information.push_str(format!("e:{}\n",state.reg_state.read(Register::e))),
        6   => state.machine_information.push_str(format!("f:{}\n",state.reg_state.read(Register::f))),
        7   => state.machine_information.push_str(format!("tos:{}\n",state.reg_state.read(Register::tos))),
        8   => state.machine_information.push_str(format!("bos:{}\n",state.reg_state.read(Register::bos))),
        9   => state.machine_information.push_str(format!("pc:{}\n",state.reg_state.read(Register::pc))),
        10  => state.machine_information.push_str(format!("s:{}\n",state.reg_state.read(Register::s))),
        _   => return Err("could not find register to print".into())
    }
    Ok(())
}


/// removes the given amount of stack entries below(lower in the stack).  
/// ### Example with: ret 2  
///  
/// ...  
/// 0x9  
/// 0x8 (parameter to called function) <- gets removed  
/// 0x7 (parameter to called function) <- gets removed  
/// 0x6 (return adress)  
/// 0x5 (value in called function)  
/// 0x4 (value in called function)  
/// 0x3 (value in called function)  
/// ...  
pub fn remove_stack_entries(state: &mut MachineState, amount: u8) -> Result<(),String>
{
    for _ in 0..amount
    {
        //state.reg_state.tos +=1;
        state.reg_state.change(Register::tos, |x| x+1)
    } 

    Ok(())
}