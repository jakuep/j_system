use j_system_definition::register::*;
use j_system_definition::instructions::*;

use crate::memory::*;
use crate::type_cov_parse::*;

/*
    bit 63-56 (8bit):   instruction
    bit 55-52 (4bit):   paramtype of first parameter
    bit 51-48 (4bit):   paramtype of second parameter
    bit 47-32 (16bit):  currently not in use
    bit 31-16 (16bit):  additional values for first parameter
    bit 15-0  (16bit):  additional values for second parameter
*/

/// Retruns the deserialized instruction and the pointer to the next instruction.
/// Retruns None when the instruction cant be read
pub fn deserialize_asm(mem_state: &MemModel, mut ptr:u64) -> Option<(AsmLine,u64)>
{
    let serialized_line = mem_state.read(ptr).ok()?;//.unwrap();
     
    // the first 8 bits contain the instruction.
    // get the masked bits of the instruction and
    // shift them to fit in a u8
    let instruction_bits = ((serialized_line & 0xff00_0000_0000_0000) >> 56) as u8;
    
    let (instr,parameter_amount) = deserialize_instruction(instruction_bits)?;
    
    let mut par1: Option<Param> = None;
    let mut par2: Option<Param> = None;

    // get the parameter types 
    let param_type1 = ((serialized_line & 0x00f0_0000_0000_0000) >> 52) as u8;
	let param_type2 = ((serialized_line & 0x000f_0000_0000_0000) >> 48) as u8;

    //get additional values 
    let param_ad1 = ((serialized_line & 0x0000_0000_ffff_0000) >> 16) as u16;
	let param_ad2 = ((serialized_line & 0x0000_0000_0000_ffff) >> 0) as u16;

    // the next line of code could be ptr+1 if
    // the asmline takes no parameters or
    // it could be ptr+2 or ptr+3 if it takes 1 or
    // 2 paramters
    ptr += 1;
    
    if parameter_amount >= 1 
    {
        let preload_next_line = mem_state.read(ptr);
        let (param,advance_ptr) = get_param_and_insert_val(param_type1,param_ad1,preload_next_line.ok())?;
        par1 = Some(param);

        // only advance if the param needed the value in par
        if advance_ptr
        {
            ptr += 1;
        }

        if parameter_amount >= 2 
        {
            let preload_next_line = mem_state.read(ptr);
            let (param,advance_ptr) = get_param_and_insert_val(param_type2,param_ad2,preload_next_line.ok())?;

            par2 = Some(param);
    
            if advance_ptr
            {
                ptr += 1;
            }
        }
    }

    let ret = AsmLine
    {    
        line: 0,
        instruction: instr,
        param1: par1,
        param2: par2
    };

    Some((ret,ptr))
}

/// gets the first 8 bits of the AsmLine to match and retun the 
/// instruction and the amount of paramters the instruction takes 
fn deserialize_instruction(bits:u8) -> Option<(InstructionEnum,u8)>
{
    // not defined -> 0x00
    // add -> 0x01
    // sub -> 0x02
    // ....
    
    match bits
    {
        0x01    =>  Some((InstructionEnum::add,2)),
        0x02    =>  Some((InstructionEnum::sub,2)),
        0x03    =>  Some((InstructionEnum::xor,2)), 
        0x04    =>  Some((InstructionEnum::or,2)),
        0x05    =>  Some((InstructionEnum::and,2)),
        0x06    =>  Some((InstructionEnum::shr,2)),
        0x07    =>  Some((InstructionEnum::shl,2)),
        0x08    =>  Some((InstructionEnum::jmp,1)),
        0x09    =>  Some((InstructionEnum::cmp,2)),
        0x0A    =>  Some((InstructionEnum::je,1)),
        0x0B    =>  Some((InstructionEnum::jeg,1)),
        0x0C    =>  Some((InstructionEnum::jel,1)),
        0x0D    =>  Some((InstructionEnum::jg,1)),
        0x0E    =>  Some((InstructionEnum::jl,1)),
        0x0F    =>  Some((InstructionEnum::mov,2)),
        0x10    =>  Some((InstructionEnum::push,1)),
        0x11    =>  Some((InstructionEnum::pop,1)),
        0x12    =>  Some((InstructionEnum::pusha,0)),
        0x13    =>  Some((InstructionEnum::popa,0)),
        0x14    =>  Some((InstructionEnum::call,1)),
        0x15    =>  Some((InstructionEnum::ret,1)),
        0x16    =>  Some((InstructionEnum::sys,0)),
        _       =>  None,
    }
}

fn deserialize_param_type(bits: u8) -> Option<ParamType>
{
    match bits
    {
        0x01    =>  Some(ParamType::Constant),
        0x02    =>  Some(ParamType::Register),
        0x03    =>  Some(ParamType::MemPtr),
        0x04    =>  Some(ParamType::MemPtrOffset),
        _       =>  None
    }
}

fn deserialize_check_for_offset_or_constant(pt: &ParamType) -> bool
{
    match pt 
    {
        ParamType::Constant     => true,
        ParamType::Register     => false,
        ParamType::MemPtr       => true,
        ParamType::MemPtrOffset => true
    }
}

fn deserialize_register(bits:u16) -> Option<Register>
{
    match bits
    {
        0x0001  =>  Some(Register::a),
        0x0002  =>  Some(Register::b),
        0x0003  =>  Some(Register::c),
        0x0004  =>  Some(Register::d),
        0x0005  =>  Some(Register::e),
        0x0006  =>  Some(Register::f),
        0x0007  =>  Some(Register::tos),
        0x0008  =>  Some(Register::bos),
        0x0009  =>  Some(Register::pc),
        0x000A  =>  Some(Register::s),
        _       =>  None //panic!("could not match register")
    }
}

pub fn get_param_and_insert_val(param_type_bits:u8, param_additional_val:u16, offset_or_constant:Option<u64>) -> Option<(Param,bool)>
{
    let pt = deserialize_param_type(param_type_bits)?;
    
    // if next val is needed but cant be read from memory
    if deserialize_check_for_offset_or_constant(&pt) && offset_or_constant.is_none()
    {
        return None;
    }

    match pt
    {
        ParamType::Constant     =>  Some((Param::Constant(offset_or_constant.unwrap()),true)),
        ParamType::Register     =>  Some((Param::Register(deserialize_register(param_additional_val)?),false)),
        ParamType::MemPtr       =>  Some((Param::MemPtr(offset_or_constant.unwrap()),true)),
        ParamType::MemPtrOffset =>  Some((Param::MemPtrOffset(deserialize_register(param_additional_val)?,u64_to_i64_bitwise(offset_or_constant.unwrap())),true))
    }
}

#[cfg(test)]
mod tests
{
    // use super::*;

    // #[test]
    // fn basic_param_test_const()
    // {
    //     //const
    //     let param_type = 0x01;
        
    //     //none
    //     let additional_val = 0x0;
        
    //     //42 as val
    //     let offs_or_cons = 42;

    //     let (p,adv) = get_param_and_insert_val(param_type, additional_val, Some(offs_or_cons)).unwrap();

    //     let expected_param = Param::Constant(42);

    //     assert_eq!((p,adv),(expected_param,true))
    // }

    // #[test]
    // fn basic_param_test_stack_const()
    // {
    //     let param_type = 0x03;
    //     let additional_val = 0x0;
    //     let offs_or_cons = 42;

    //     let (p,adv) = get_param_and_insert_val(param_type, additional_val, Some(offs_or_cons)).unwrap();

    //     let expected_param = Param::MemPtr(42);

    //     assert_eq!((p,adv),(expected_param,true))
    // }

    // #[test]
    // fn basic_param_test_stack_register()
    // {
    //     // register
    //     let param_type = 0x02;

    //     // register e
    //     let additional_val = 0x0005;
        
    //     //None
    //     let offs_or_cons = 0;

    //     let (p,adv) = get_param_and_insert_val(param_type, additional_val, Some(offs_or_cons)).unwrap();

    //     let expected_param = Param::Register(Register::e);

    //     assert_eq!((p,adv),(expected_param,false))
    // }

    // #[test]
    // fn basic_param_test_stack_stack_offset()
    // {
    //     // register
    //     let param_type = 0x04;

    //     // register b
    //     let additional_val = 0x0002;
        
    //     //42
    //     let offs_or_cons = 42;

    //     let (p,adv) = get_param_and_insert_val(param_type, additional_val, Some(offs_or_cons)).unwrap();

    //     let expected_param = Param::MemPtrOffset(Register::b,u64_to_i64_bitwise(offs_or_cons));

    //     assert_eq!((p,adv),(expected_param,true))
    // }

    // #[test]
    // fn test_deserial_asm_add_with_const()
    // {
    //     let ins_line:u64 = 0x0121_0000_0001_0000;
    //     let param_const = 42;

    //     let mut mem = MemModel::new(10);

    //     mem.code = vec![0,ins_line,param_const];
    //     let (asm,new_ptr) = deserialize_asm(&mut mem, 1).unwrap();

    //     let expected_asm = 
    //         AsmLine{
    //             line: 0,
    //             instruction: InstructionEnum::add,
    //             param1: Some(Param::Register(Register::a)),
    //             param2: Some(Param::Constant(42))
    //         };

    //     assert_eq!((asm,new_ptr),(expected_asm,3))
    // }

    // #[test]
    // fn test_deserial_asm_mov_to_stack_offset_from_register()
    // {
    //     let ins_line:u64 = 0x0F42_0000_0001_0004;
        
    //     //internal u64 represenation
    //     let param_offset = 42 + 0x8000_0000_0000_0000;

    //     let mut mem = MemModel::new(10);

    //     mem.code = vec![0,ins_line,param_offset];
    //     let (asm,new_ptr) = deserialize_asm(&mut mem, 1).unwrap();

    //     let expected_asm = 
    //         AsmLine{
    //             line: 0,
    //             instruction: InstructionEnum::mov,
    //             param1: Some(Param::MemPtrOffset(Register::a,42)),
    //             param2: Some(Param::Register(Register::d))
    //         };

    //     assert_eq!((asm,new_ptr),(expected_asm,3))
    // }

    // #[test]
    // fn test_deserial_asm_cmp_two_full_value_params()
    // {
    //     let ins_line:u64 = 0x0911_0000_0000_0000;
    //     let param_1 = 1473587;
    //     let param_2 = 11;

    //     let mut mem = MemModel::new(10);

    //     mem.code = vec![0,ins_line,param_1,param_2];
    //     let (asm,new_ptr) = deserialize_asm(&mut mem, 1).unwrap();

    //     let expected_asm = 
    //         AsmLine{
    //             line: 0,
    //             instruction: InstructionEnum::cmp,
    //             param1: Some(Param::Constant(1473587)),
    //             param2: Some(Param::Constant(11))
    //         };

    //     assert_eq!((asm,new_ptr),(expected_asm,4))
    // }

    // #[test]
    // fn test_u64_to_i64_tiny_number()
    // {
    //     assert_eq!(42,i64_to_u64_bitwise(u64_to_i64_bitwise(42)))
    // }

    // #[test]
    // fn test_u64_to_i64_large_number()
    // {
    //     assert_eq!(u64::MAX-4,i64_to_u64_bitwise(u64_to_i64_bitwise(u64::MAX-4)))
    // }

    // #[test]
    // fn test_i64_to_u64_large_number()
    // {
    //     assert_eq!(-874982374,u64_to_i64_bitwise(i64_to_u64_bitwise(-874982374)))
    // }

    // #[test]
    // fn test_i64_to_u64_tiny_number()
    // {
    //     assert_eq!(-5,u64_to_i64_bitwise(i64_to_u64_bitwise(-5)))
    // }
}
