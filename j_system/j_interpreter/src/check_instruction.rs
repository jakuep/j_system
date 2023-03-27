use j_system_definition::instructions::*;
use j_system_definition::register::Register;
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

pub trait StateHelperFunctions
{
    fn store_in_dest(&mut self, val:u64, param: &Option<Param>);
    fn pop_stack(&mut self) -> Result<u64,String>;
    fn push_stack(&mut self, val: u64) -> Result<(),String>;
    fn get_param_value(&mut self, param: &Option<Param>) -> u64;

}

impl StateHelperFunctions for MachineState {

    fn store_in_dest(&mut self, val:u64, param: &Option<Param>)
    {
        if let Some(param_some) = param
        {    
            match param_some
            {
                Param::Register(dest) => self.reg_state.store(*dest, val),
                
                Param::MemPtr(dest) => 
                {
                    // TODO: handle result
                    self.mem_state.store(val, *dest).unwrap()
                },

                Param::MemPtrOffset(reg,offset) =>
                {
                    let addr = self.reg_state.read(*reg) as i128 + *offset as i128;
                    
                    // TODO: check the i128 to u64 conversion for loss
                    self.mem_state.store(val, addr as u64).unwrap()
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

    fn pop_stack(&mut self) -> Result<u64,String>
    {
        // TODO: handle empty stack? aka tos == 0

        let tos = self.reg_state.read(Register::tos);

        let ret = self.mem_state.read(tos).unwrap();

        if tos == self.mem_state.get_mem_size()-1
        {
            self.reg_state.store(Register::tos, 0);
        }
        else
        {
            self.reg_state.change(Register::tos, |x| x+1);
        }
        Ok(ret)
    }

    fn push_stack(&mut self, val: u64) -> Result<(),String>
    {
        // TODO: handle heap-stack collison

        if self.reg_state.read(Register::tos) == 0
        {
            self.mem_state.store(val, self.mem_state.get_mem_size()-1).unwrap();
            self.reg_state.store(Register::tos, self.mem_state.get_mem_size()-1);
            return Ok(());
        }

        self.reg_state.change(Register::tos, |x| x-1);
        self.mem_state.store(val, self.reg_state.read(Register::tos)).unwrap();
    

        Ok(())
    } 

    fn get_param_value(&mut self, param: &Option<Param>)-> u64
    {
        if let Some(x) = param
        {
            match x
            {
                Param::Register(z)         => self.reg_state.read(*z),
                Param::Constant(z)              => *z,

                // TODO: do not just unwrap but check if is Some(..)
                // TODO: check for vaild index
                Param::MemPtr(z)            => self.mem_state.read(*z).unwrap(), 
                // TODO: check for save conversion
                Param::MemPtrOffset(reg,of) => self.mem_state.read((self.reg_state.read(*reg) as i64+*of) as u64).expect(&format!("could not read form {}",self.reg_state.read(*reg) as i64+*of))
            }
        }
        else
        {
            //crate::output::dump_and_panic(format!(""), register_state, stack_state);
            panic!("existence of a second parameter should already be checked!");
        }
    }
}