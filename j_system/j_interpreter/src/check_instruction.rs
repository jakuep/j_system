use j_system_definition::instructions::*;
use j_system_definition::register::Register;
use crate::exec::*;
use crate::machine::*;

pub fn check_two_param(param1: &Option<Param>, param2: &Option<Param>) -> bool
{
    if param1.is_some() && param2.is_some()
    {
        return true;
    }
    false
}

pub fn check_only_one_param(param1: &Option<Param>, param2: &Option<Param>) -> bool
{
    if param1.is_some() && param2.is_none()
    {
        return true;
    }
    false
}

pub fn is_register(param: &Option<Param>) -> bool
{
    if let Some(Param::Register(_)) = param
    {
        return true;
    }
    false
}


// TODO: make this impl in state
pub fn store_in_dest(val:u64, param: &Option<Param>,state: &mut MachineState)
{
    if let Some(param_some) = param
    {    
        match param_some
        {
            Param::Register(dest) => state.reg_state.store(*dest, val),
            
            Param::MemPtr(dest) => 
            {
                // TODO: handle result
                state.mem_state.store(val, *dest).unwrap()
            },

            Param::MemPtrOffset(reg,offset) =>
            {
                let addr = state.reg_state.read(*reg) as i128 + *offset as i128;
                
                // TODO: check the i128 to u64 conversion for loss
                state.mem_state.store(val, addr as u64).unwrap()
            }

            Param::Constant(_) => 
            {
                panic!("a constant can't be a destination");
                //crate::output::dump_and_panic(format!("a constant can't be a destination"), register_state, stack_state);
            },
            //Param::ParamStack(z) => {return stack_state[*z as usize]},
        }
    }
    else
    {
        panic!("no destination defined");
        //crate::output::dump_and_panic(format!("no destination defined"), register_state, stack_state);
    }
}

pub fn pop_stack(state: &mut MachineState) -> Result<u64,String>
{
    // TODO: handle empty stack? aka tos == 0

    let tos = state.reg_state.read(Register::tos);

    let ret = state.mem_state.read(tos).unwrap();

    if tos == state.mem_state.get_mem_size()-1
    {
        state.reg_state.store(Register::tos, 0);
    }
    else
    {
        state.reg_state.change(Register::tos, |x| x+1);
    }
    Ok(ret)
}

pub fn push_stack(state: &mut MachineState, val: u64) -> Result<(),String>
{
    // TODO: handle heap-stack collison

    if state.reg_state.read(Register::tos) == 0
    {
        state.mem_state.store(val, state.mem_state.get_mem_size()-1).unwrap();
        state.reg_state.store(Register::tos, state.mem_state.get_mem_size()-1);
        return Ok(());
    }

    state.reg_state.change(Register::tos, |x| x-1);
    state.mem_state.store(val, state.reg_state.read(Register::tos)).unwrap();
   

    Ok(())
} 


// TODO: make this impl on state
pub fn get_param_value(param: &Option<Param>, state: &mut MachineState)-> u64
{
    if let Some(x) = param
    {
        match x
        {
            Param::Register(z)         => state.reg_state.read(*z),
            Param::Constant(z)              => *z,

            // TODO: do not just unwrap but check if is Some(..)
            // TODO: check for vaild index
            Param::MemPtr(z)            => state.mem_state.read(*z).unwrap(), 
            // TODO: check for save conversion
            Param::MemPtrOffset(reg,of) => state.mem_state.read((state.reg_state.read(*reg) as i64+*of) as u64).expect(&format!("could not read form {}",state.reg_state.read(*reg) as i64+*of))
        }
    }
    else
    {
        //crate::output::dump_and_panic(format!(""), register_state, stack_state);
        panic!("existence of a second parameter should already be checked!");
    }
}