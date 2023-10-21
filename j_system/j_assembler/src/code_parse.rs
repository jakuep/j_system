use crate::assembler::{LabelType, UnlinkedInstruction,UnlinkedParameter};
use crate::preprocessor::{RawLine};
use crate::type_cov_parse;

use crate::j_system_definition::instructions::{InstructionEnum,Param};
use crate::j_system_definition::register::Register;

use std::collections::HashMap;

fn parse_code_section(input: &Vec<RawLine>,file_name: String, linker_info: &mut HashMap<String,LabelType>) -> Result<(),String>
{
    
    // keep track of the offset of the current instruction to insert this value into the label tabel
    let mut offset_pointer = 0; 

    for line in input
    {
        let parts:Vec<_> = line.content
            .trim()
            .split(char::is_whitespace)
            .filter(|x| !x.is_empty())
            .collect();

        // the first part should be a instruction or a label definition
        if parts[0].starts_with('.') && parts[0].ends_with(':')
        {
            let mut chars = parts[0].chars();
            chars.next();
            chars.next_back();
            let label_name = chars.as_str().to_string();

            if let Some(doubledef) = linker_info.insert(label_name.clone(), LabelType::JumpLabel(offset_pointer))
            {
                return Err(format!("double definition of label '{}' in '{}'. second definition in line {}",label_name,file_name,line.line));
            }

            // the line cloud still contain a instruction after the label definition
            if parts.len() > 1
            {
                let inst = parse_instruction(parts[1..].to_vec(), line.line)?;
            }
        }
        else 
        {
            // if it is not a label it should be instruction

        }

    }

    Ok(())
}

fn parse_instruction(parts: Vec<&str>,line: u64) -> Result<UnlinkedInstruction,String>
{
    // match the instruction
    let inst = match parts[0]
    {
        "add"   => InstructionEnum::add,
        "sub"   => InstructionEnum::sub,
        "xor"   => InstructionEnum::xor,
        "or"    => InstructionEnum::or,
        "and"   => InstructionEnum::and,
        "shr"   => InstructionEnum::shr,
        "shl"   => InstructionEnum::shl,
        "jmp"   => InstructionEnum::jmp,
        "cmp"   => InstructionEnum::cmp,
        "je"    => InstructionEnum::je,
        "jeg"   => InstructionEnum::jeg,
        "jel"   => InstructionEnum::jel,
        "jg"    => InstructionEnum::jg,
        "jl"    => InstructionEnum::jl,
        "mov"   => InstructionEnum::mov,
        "push"  => InstructionEnum::push,
        "pop"   => InstructionEnum::pop,
        "pusha" => InstructionEnum::pusha,
        "popa"  => InstructionEnum::popa,
        "call"  => InstructionEnum::call,
        "ret"   => InstructionEnum::ret,
        "sys"   => InstructionEnum::sys,
        _       => return Err(format!("could not mach instruction")),
    };

    let mut param1 = None;
    let mut param2 = None;

    // since all whitespaces have been removed proir to this it should look like this: a,[b+1]
    let fused = parts[1..].join("");

    if !fused.is_empty()
    {
        if let Some(split_point) = fused.find(',')
        {
            let (first,second) = fused.split_at(split_point);
            parse_parameter(first);

            if !second.is_empty()
            {
                param2 = Some(parse_parameter(second)?)
            }
        }
    }

    Ok(UnlinkedInstruction{line,inst, param1, param2})
}

fn parse_parameter(mut param: &str) -> Result<UnlinkedParameter,String>
{
    // if the parameter string only conatis a number there is nothing more to do
    if let Ok(constant) = param.parse::<i64>()
    {
        return Ok(UnlinkedParameter::Determined(Param::Constant(type_cov_parse::i64_to_u64_bitwise(constant))));
    }

    // jump-/datalabel -> will be resolved by the linker
    if param.starts_with('.')
    {}
    
    
    // check for deref
    let deref = param.starts_with('[') && param.ends_with(']');

    // remove '[' and ']'
    if deref
    {
        let mut chars = param.chars();
        chars.next();
        chars.next_back();
        param = chars.as_str();
    }

    // is number?
    



    Err("".into())
}