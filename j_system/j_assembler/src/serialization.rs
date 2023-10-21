use j_system_definition::register::*;
use j_system_definition::instructions::*;

use crate::type_cov_parse::*;

/*
    bit 63-56 (8bit):   instruction
    bit 55-52 (4bit):   paramtype of first parameter
    bit 51-48 (4bit):   paramtype of second parameter
    bit 47-32 (16bit):  currently not in use
    bit 31-16 (16bit):  additional values for first parameter
    bit 15-0  (16bit):  additional values for second parameter
*/

pub fn serialize_asm(code: Vec<AsmLine>) -> Vec<u64>
{
    let mut asm_list = vec![];

    // holds the pointers to all instructions
    let mut asm_index_list:Vec<u64> = vec![];

    for inst_struct in code
    {

        let mut inst_ser:u64 = 0;

        let mut par1:Option<u64> = None;
        let mut par2:Option<u64> = None;

        let matched_instruction = serialize_instruction(&inst_struct.instruction);

        // set the instruction 
        inst_ser |= (matched_instruction as u64) << 56;

        if let Some(param1) = &inst_struct.param1
        {
            let (par1_type,par1_adval,par1_vec_entry) = serialize_param(param1);
            par1 = par1_vec_entry;
            
            //partype1 to 55-52
            inst_ser |= (par1_type as u64) << 52;

            //paradval1 to 31-16
            inst_ser |= (par1_adval as u64) << 16;

            if let Some(param2) = &inst_struct.param2
            {
                let (par2_type,par2_adval,par2_vec_entry) = serialize_param(param2);
                par2 = par2_vec_entry;

                //partype2 to 51-48
                inst_ser |= (par2_type as u64) << 48;

                //paradval2 to 15-0
                inst_ser |= par2_adval as u64;
            }
        }

        asm_list.push(inst_ser);

        // save the postions of Instrcutions to match with jump labels
        asm_index_list.push((asm_list.len()-1) as u64);

        // check serperatly from another since par2 can have a value even if par1 dosnt
        if let Some(p1) = par1
        {
            asm_list.push(p1);
        }

        if let Some(p2) = par2
        {
            asm_list.push(p2);
        }
    }

    asm_list
}

#[derive(Debug,PartialEq,Clone)]
pub enum RomDataType
{
    Integer,
    String,
    IntegerArray,
    StringArray,
}

pub fn serialize_rom(r: String,teip: RomDataType) -> Vec<u64>
{
    match teip
    {
        RomDataType::Integer         => number_to_vec_u64(r),
        RomDataType::String          => string_to_vec_u64(r),
        RomDataType::IntegerArray    => 
        {
            let mut res = vec![]; 
            // remove "[" and "]"
            let seperated_ints = r[1..r.len()-1].split(',');

            for e in seperated_ints
            {
                let element =  e.split_whitespace().filter(|x| x.len()>0).collect::<Vec<&str>>();

                // panic of the format is wrong
                if element.len() != 1 {panic!("could not parse {} as int array",r)}

                if let Ok(found_int) = element[0].parse::<u64>()
                {
                    res.push(found_int);
                }
            }

            // check if all chars are ascii
            for c in &res
            {
                if c & 0x80!= 0
                {
                    panic!("only ascii chars are supported");
                }
            }
            
            res
        },
        
        RomDataType::StringArray     => 
        {
            let mut res = vec![];
            for element in r[1..r.len()-1].split(',')
            {
                res.append(&mut element.split('"').collect::<Vec<&str>>()[1].as_bytes().iter().map(|x| *x as u64).collect::<Vec<_>>());
                
                // null terminate the string
                res.push(0);
            }

            // check if all chars are ascii
            for c in &res
            {
                // TODO: this is wrong!!!???
                if c & 0x80 != 0
                {
                    panic!("only ascii chars are supported");
                }
            }

            res
        }
    }


    /*
    // check if the value is an array type, a String or a number
    if r.chars().nth(0) == Some('[') && r.chars().nth(r.len()-1) == Some(']')
    {
        // split the array of elements in the string into a vec of elements
        // the elemnts may still conatin whitespaces that we need to deal with
        let element_array = r.split(',').collect::<Vec<&str>>();

    }
    else if r.chars().nth(0) == Some('"') && r.chars().nth(r.len()-1) == Some('"')
    {
        // wth?
        return r[1..r.len()-2].to_owned().as_bytes().into_iter().map(|x| *x as u64).collect();
    }
    else if let Ok(found_int) = r.parse::<u64>()
    {
        return vec![found_int];
    }

    */
}

fn string_to_vec_u64(s: String) -> Vec<u64> 
{
    if s.chars().nth(0) == Some('"') && s.chars().nth(s.len()-1) == Some('"') 
    {
        let mut ret = s[1..s.len()-1].to_owned().as_bytes().into_iter().map(|x| *x as u64).collect::<Vec<_>>();
        
        // null terminate the string
        ret.push(0);
        ret
    } 
    else
    {
        panic!("could not parse ->{}<- as a string", s)
    }
}

fn number_to_vec_u64(s: String) -> Vec<u64>
{
    // bitwise conversion that allows any number that fits
    // inside a i64 
    // it will be represented as a u64 internaly

    if let Ok(found_int) = s.parse::<u64>()
    {
        return vec![found_int];
    }
    else
    {
        panic!("could not parse ->{}<- as a number", s);
    }
}

/*
fn serialize_param_type(typ: ParamType) -> u8
{
    match typ
    {
        ParamType::Constant         => 0x01,
        ParamType::ParamRegister    => 0x02,
        ParamType::ParamStack       => 0x03,
        ParamType::ParamStackOffset => 0x04,
    }
}
*/

fn serialize_param(param: &Param) -> (u8,u16,Option<u64>)
{
    match param
    {
        Param::Constant(val)            =>  (0x01,0x0,Some(*val)),
        Param::Register(reg)            =>  (0x02,serialize_register(*reg),None),
        Param::MemPtr(val)              =>  (0x03,0x0,Some(*val)),
        Param::MemPtrOffset(reg,offset) =>  (0x04,serialize_register(*reg),Some(i64_to_u64_bitwise(*offset))),

    }
}

fn serialize_register(reg: Register) -> u16 
{
    match reg
    {
        Register::a     => 0x0001,
        Register::b     => 0x0002,
        Register::c     => 0x0003,
        Register::d     => 0x0004,
        Register::e     => 0x0005,
        Register::f     => 0x0006,
        Register::tos   => 0x0007,
        Register::bos   => 0x0008,
        Register::pc    => 0x0009,
        Register::s     => 0x000A,
    }
}

fn serialize_instruction(inst: &InstructionEnum) -> u8
{
    match inst
    {
        InstructionEnum::add    =>  0x01,
        InstructionEnum::sub    =>  0x02,
        InstructionEnum::xor    =>  0x03,
        InstructionEnum::or     =>  0x04,
        InstructionEnum::and    =>  0x05,
        InstructionEnum::shr    =>  0x06,
        InstructionEnum::shl    =>  0x07,
        InstructionEnum::jmp    =>  0x08,
        InstructionEnum::cmp    =>  0x09,
        InstructionEnum::je     =>  0x0A,
        InstructionEnum::jeg    =>  0x0B,
        InstructionEnum::jel    =>  0x0C,
        InstructionEnum::jg     =>  0x0D,
        InstructionEnum::jl     =>  0x0E,
        InstructionEnum::mov    =>  0x0F,
        InstructionEnum::push   =>  0x10,
        InstructionEnum::pop    =>  0x11,
        InstructionEnum::pusha  =>  0x12,
        InstructionEnum::popa   =>  0x13,
        InstructionEnum::call   =>  0x14,
        InstructionEnum::ret    =>  0x15,
        InstructionEnum::sys    =>  0x16,
    }
}


// TODO: crate tests that do not need decode crate

/*
#[cfg(test)]
mod tests
{
    use super::*;
    #[test]
    fn ser_deser_simple()
    {
        let a = AsmLine 
        {
            line:0,
            instruction: InstructionEnum::sub,
            param1: Some(Param::ParamRegister(Register::a)),
            param2: Some(Param::Constant(4000)),
        };
        
        let asm_vec = vec![a.clone()];
        let sersed = serialize_asm(asm_vec);
        let mut mm = MemModel::new(20);
        mm.code = sersed;

        let (insret,_) = deserialize_asm(& mut mm,0);

        assert_eq!(insret,a);
        
    }

    #[test]
    fn ser_deser_two_instr()
    {
        let a = AsmLine 
        {
            line:0,
            instruction: InstructionEnum::sub,
            param1: Some(Param::ParamRegister(Register::a)),
            param2: Some(Param::Constant(4000)),
        };

        let b = AsmLine 
        {
            line:0,
            instruction: InstructionEnum::add,
            param1: Some(Param::ParamRegister(Register::f)),
            param2: Some(Param::ParamStackOffset(Register::a,-87)),
        };
        
        let asm_vec = vec![a.clone(),b.clone()];
        let sersed = serialize_asm(asm_vec);
        let mut mm = MemModel::new(20);
        mm.code = sersed;

        let (insret1,next) = deserialize_asm(& mut mm,0);
        let (insret2,_) = deserialize_asm(&mut mm, next);

        let ok = insret1 == a && insret2 == b;
        assert_eq!(ok,true);
        
    }
}
*/