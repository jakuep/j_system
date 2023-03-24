use j_system_definition::register::*;
use j_system_definition::instructions::*;

use crate::check_instruction::*;
use crate::syscall::*;
use crate::machine::{MachineState, InstructionReturn, Exec};

impl Exec for MachineState{
fn run_instruction(&mut self, inst: AsmLine) -> InstructionReturn
{
    match inst.instruction
    {
        InstructionEnum::add    => self.add(inst.get_param1(),inst.get_param2()),
        InstructionEnum::mov    => self.mov(inst.get_param1(),inst.get_param2()),
        InstructionEnum::jmp    => self.jmp(inst.get_param1(),inst.get_param2()),
        InstructionEnum::cmp    => self.cmp(inst.get_param1(),inst.get_param2()),
        InstructionEnum::push   => self.push(inst.get_param1(),inst.get_param2()),
        InstructionEnum::pop    => self.pop(inst.get_param1(),inst.get_param2()),
        InstructionEnum::sub    => self.sub(inst.get_param1(),inst.get_param2()),
        InstructionEnum::and    => self.and(inst.get_param1(),inst.get_param2()),
        InstructionEnum::je     => self.je(inst.get_param1(),inst.get_param2()),
        InstructionEnum::jl     => self.jl(inst.get_param1(),inst.get_param2()),
        InstructionEnum::jel    => self.jel(inst.get_param1(),inst.get_param2()),
        InstructionEnum::shr    => self.shr(inst.get_param1(),inst.get_param2()),
        InstructionEnum::shl    => self.shl(inst.get_param1(),inst.get_param2()),
        InstructionEnum::sys    => self.sys(inst.get_param1(),inst.get_param2()),
        InstructionEnum::ret    => self.ret(inst.get_param1(),inst.get_param2()),
        InstructionEnum::call   => self.call(inst.get_param1(),inst.get_param2()),
        InstructionEnum::or     => self.or(inst.get_param1(),inst.get_param2()),
        InstructionEnum::popa   => unimplemented!("TODO: popa"),
        InstructionEnum::pusha  => unimplemented!("TODO: pusha"),
        InstructionEnum::jeg    => self.jeg(inst.get_param1(),inst.get_param2()),
        InstructionEnum::jg     => self.jg(inst.get_param1(),inst.get_param2()),
        InstructionEnum::xor    => self.xor(inst.get_param1(),inst.get_param2()),
    }
}

fn jg(&mut self, param1: Option<Param>, param2: Option<Param>) -> InstructionReturn
{
    if !check_only_one_param(&param1, &param2)
    {
        return InstructionReturn::Err("jg needs 1 parameter".to_string());
    }

    if self.reg_state.read(Register::s) & 1<<2 != 0
    {
        InstructionReturn::JumpTo(get_param_value(&param1, self))
    }
    else
    {
        InstructionReturn::Next
    }
}

fn jeg(&mut self, param1: Option<Param>, param2: Option<Param>) -> InstructionReturn
{
    if !check_only_one_param(&param1, &param2)
    {
        return InstructionReturn::Err("jeg needs 1 parameter".to_string());
    }

    if self.reg_state.read(Register::s) & (1<<3 | 1<<2) != 0
    {
        InstructionReturn::JumpTo(get_param_value(&param1, self))
    }
    else
    {
        InstructionReturn::Next
    }
}

fn xor(&mut self, param1: Option<Param>, param2: Option<Param>) -> InstructionReturn
{
    let op1 = get_param_value(&param1, self);
    let op2 = get_param_value(&param2, self);
    let dest = param1.unwrap();

    let res = op1 ^ op2;

    if let Param::Register(reg) = dest
    {
        self.reg_state.store(reg, res);
        InstructionReturn::Next
    }
    else
    {
        InstructionReturn::Err("first parameter of xor must be a register".to_string())
    }   
}

fn or(&mut self, param1: Option<Param>, param2: Option<Param>) -> InstructionReturn
{
    let op1 = get_param_value(&param1, self);
    let op2 = get_param_value(&param2, self);
    let dest = param1.unwrap();

    let res = op1 | op2;

    if let Param::Register(reg) = dest
    {
        self.reg_state.store(reg, res);
        InstructionReturn::Next
    }
    else
    {
        InstructionReturn::Err("first parameter of or must be a register".to_string())
    }   
}

fn add(&mut self, param1: Option<Param>, param2: Option<Param>) -> InstructionReturn
{
    
    if !crate::check_instruction::check_two_param(&param1,&param2) || !crate::check_instruction::is_register(&param1)
    {
        return InstructionReturn::Err("wrong paramters for add".to_string());
    }

    let val = get_param_value(&param2, self);

    if let Some(reg_some) = param1
    {
        if let Param::Register(reg) = reg_some
        {
            let read_register_val = self.reg_state.read(reg);
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

            self.reg_state.store(reg, write_to_register);

        }

        InstructionReturn::Next
    }
    else
    {
        InstructionReturn::Err("paramter 1 in add must be a register".to_string())
    }

    
}

fn sub(&mut self, param1: Option<Param>, param2: Option<Param>) -> InstructionReturn
{
    if !crate::check_instruction::check_two_param(&param1,&param2) && !crate::check_instruction::is_register(&param1)
    {
        return InstructionReturn::Err("wrong paramters for add".to_string());
    }

    let val = crate::check_instruction::get_param_value(&param2, self);

    if let Some(reg_some) = param1
    {
        if let Param::Register(reg) = reg_some
        {
            let read_register_val = self.reg_state.read(reg);
            let store:u64;
            if read_register_val < val
            {
                store = 0;
            }
            else
            {
                store = read_register_val - val;
            }
            self.reg_state.store(reg, store); //TODO: check for overflow
        }
    }
    else
    {
        return InstructionReturn::Err("paramter 1 in add must be a register".to_string());
    }

    InstructionReturn::Next
}

fn mov(&mut self, param1: Option<Param>, param2: Option<Param>) -> InstructionReturn
{
    //TODO: Does at least one of the parameters have to be a register?
    if !crate::check_instruction::check_two_param(&param1,&param2)
    {
        return InstructionReturn::Err("mov needs 2 paramters".to_string());
    }
    
    let val = crate::check_instruction::get_param_value(&param2,self);
    crate::check_instruction::store_in_dest(val, &param1, self);

    InstructionReturn::Next
}

fn jmp(&mut self, param1: Option<Param>, param2: Option<Param>) -> InstructionReturn
{
    if check_only_one_param(&param1, &param2)
    {
        let val = crate::check_instruction::get_param_value(&param1, self);
        InstructionReturn::JumpTo(val)
    }
    else
    {
        InstructionReturn::Err("jmp needs 1 paramter".to_string())
    }   
}

fn cmp(&mut self, param1: Option<Param>, param2: Option<Param>) -> InstructionReturn
{
    let val1 = crate::check_instruction::get_param_value(&param1, self);
    let val2 = crate::check_instruction::get_param_value(&param2, self);

    // reset the staus register
    self.reg_state.store_to_read_only(Register::s, 0);
    
    match val1 as i128 - val2 as i128
    {
        x if x<0    => self.reg_state.store_to_read_only(Register::s, 1<<1),
        x if x>0    => self.reg_state.store_to_read_only(Register::s, 1<<2),
        x if x==0   => self.reg_state.store_to_read_only(Register::s, 1<<3),
        _           => unreachable!()
    }
    
    InstructionReturn::Next
}

fn push(&mut self, param1: Option<Param>, param2: Option<Param>) -> InstructionReturn
{
    if !check_only_one_param(&param1, &param2)
    {
        return InstructionReturn::Err("push needs 1 paramter".to_string())
    }

    let val = crate::check_instruction::get_param_value(&param1, self);
    
    // TODO: handle result
    push_stack(self, val).unwrap();
    
    InstructionReturn::Next
}

fn pop(&mut self, param1: Option<Param>, param2: Option<Param>) -> InstructionReturn
{
    if !is_register(&param1) || !check_only_one_param(&param1, &param2)
    {
        return InstructionReturn::Err("pop needs 1 paramter".to_string())
    }

    match pop_stack(self)
    {
        Ok(val) => store_in_dest(val, &param1, self),
        Err(msg) => return InstructionReturn::Err(msg)
        //crate::output::dump_and_panic(format!("Stack is empty"), register_state, stack_state);
    }

    InstructionReturn::Next
}

fn je(&mut self, param1: Option<Param>, param2: Option<Param>) -> InstructionReturn
{
    if !check_only_one_param(&param1, &param2)
    {
        return InstructionReturn::Err("je needs 1 parameter".to_string());
    }

    if self.reg_state.read(Register::s) & 1<<3 != 0
    {
        InstructionReturn::JumpTo(get_param_value(&param1, self))
    }
    else
    {
        InstructionReturn::Next
    }
}

fn jel(&mut self, param1: Option<Param>, param2: Option<Param>) -> InstructionReturn
{
    if !check_only_one_param(&param1, &param2)
    {
        return InstructionReturn::Err("jel needs 1 parameter".to_string());
    }

    if self.reg_state.read(Register::s) & (1<<3|1<<1) != 0
    {
        InstructionReturn::JumpTo(get_param_value(&param1, self))
    }
    else
    {
        InstructionReturn::Next
    }
}

fn jl(&mut self, param1: Option<Param>, param2: Option<Param>) -> InstructionReturn
{
    if !check_only_one_param(&param1, &param2)
    {
        return InstructionReturn::Err("jl needs 1 parameter".to_string());
    }

    if self.reg_state.read(Register::s) & 1<<1 != 0
    {
        InstructionReturn::JumpTo(get_param_value(&param1, self))
    }
    else
    {
        InstructionReturn::Next
    }
}

fn ret(&mut self, param1: Option<Param>, param2: Option<Param>) -> InstructionReturn
{
    if !(param1.is_some() && param2.is_none())
    {
        return InstructionReturn::Err("ret needs 1 paramter".to_string())
    }

    let stack_clean_amount = get_param_value(&param1, self);

    // get the retrun adress
    // add 1 to not jump to the call again but the instruction after that
    // TODO: handle empty stack
    let jmp_adress = pop_stack(self).unwrap();

    // remove arguments on stack
    for _ in 0..stack_clean_amount //as usize
    { 
        // add the ammount 
        let _ = pop_stack(self);
    }

    InstructionReturn::JumpTo(jmp_adress)
}

fn and(&mut self, param1: Option<Param>, param2: Option<Param>) -> InstructionReturn
{
    // TODO: check for valid parameters

    let op1 = get_param_value(&param1, self);
    let op2 = get_param_value(&param2, self);
    let dest = param1.unwrap();

    let res = op1 & op2;

    if let Param::Register(reg) = dest
    {
        self.reg_state.store(reg, res);
        InstructionReturn::Next
    }
    else
    {
        InstructionReturn::Err("first parameter of and must be a register".to_string())
    }
}

fn shr(&mut self, param1: Option<Param>, param2: Option<Param>) -> InstructionReturn
{
    if !check_two_param(&param1, &param2)
    {
        return InstructionReturn::Err("shr needs 2 parameters".to_string());
    }

    if !is_register(&param1)
    {
        return InstructionReturn::Err("the first parameter of shr must be a register".to_string())
    }

    let shift_amount = get_param_value(&param2, self);

    if shift_amount > 64
    {
        return InstructionReturn::Err("cannot shift by more than 64 bits".to_string());
    }

    let mut val = get_param_value(&param1, self);

    val = val >> shift_amount;

    store_in_dest(val, &param1, self);

    InstructionReturn::Next
}

fn shl(&mut self, param1: Option<Param>, param2: Option<Param>) -> InstructionReturn
{
    if !check_two_param(&param1, &param2)
    {
        return InstructionReturn::Err("shl needs 2 parameters".to_string());
    }

    if !is_register(&param1)
    {
        return InstructionReturn::Err("the first parameter of shl must be a register".to_string())
    }

    let shift_amount = get_param_value(&param2, self);

    if shift_amount > 64
    {
        return InstructionReturn::Err("cannot shift by more than 64 bits".to_string());
    }

    let mut val = get_param_value(&param1, self);

    val = val << shift_amount;

    store_in_dest(val, &param1, self);

    InstructionReturn::Next
}

fn call(&mut self, param1: Option<Param>, param2: Option<Param>) -> InstructionReturn
{
    if !check_only_one_param(&param1, &param2)
    {
        return InstructionReturn::Err("call only needs 1 parameter".to_string());
    }

    //TODO: save registers?, stackframe?
    
    // save return adress
    push_stack(self, self.next_ptr).unwrap();

    InstructionReturn::JumpTo(get_param_value(&param1, self))
}

fn sys(&mut self, param1: Option<Param>, param2: Option<Param>) -> InstructionReturn
{
    if !(param1.is_none() && param2.is_none())
    {
        return InstructionReturn::Err("sys does not get any parameters".into())
    }

    // TODO:    for now sys only prints the register state.
    //          -> change it to call the syscall handler

    match syscall(self)
    {
        SysCallAction::End      => InstructionReturn::End,
        SysCallAction::Err(x)   => InstructionReturn::Err(x),
        SysCallAction::Ok       => InstructionReturn::Next,
        SysCallAction::Ptr(x)   => { self.reg_state.store(Register::f, x); InstructionReturn::Next}
    }

}
}