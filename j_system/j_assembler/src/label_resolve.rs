use j_system_definition::instructions::*;
use crate::decode_instructons::*;

use std::collections::HashMap;

pub fn remove_labels_from_asm(
    code_with_labels: Vec<AsmLineLabel>, 
    lable_table: &mut Vec<LabelPointer>, 
    defines: HashMap<String,u64>,
    debug_symbols: &mut Vec<(String,u64)>,
    rom_size:u64) -> (Vec<AsmLine>, u64, Vec<u64>)
{
    let instruction_position = calc_sersed_code_positions(&code_with_labels, rom_size);
    let mut code_without_labels:Vec<AsmLine> = vec![];
    let start_of_execution_ptr = label_to_adress("start".to_string(), lable_table, &instruction_position);

    // get position of labels for debug output
    // INFO: the label tabel seems to hold the incorrect adress
    for lab in &*lable_table
    {
        // get the identifier/name of the label
        let name = lab.identifier.clone();
        let addr = label_to_adress(name.clone(), lable_table, &instruction_position);
        debug_symbols.push((name,addr));
    }
    
    for instr in code_with_labels
    {
        let instr_type = instr.instruction;
        let info = instr.info;
        let p1 = resolve_labels(&instr.param1, lable_table, &defines, &instruction_position);
        let p2 = resolve_labels(&instr.param2, lable_table, &defines, &instruction_position);

        code_without_labels.push(
            AsmLine{
                line: info.line,
                instruction: instr_type,
                param1: p1,
                param2: p2,
            });
    }

    // returns code without labels and the adress where execution stars
    (code_without_labels, start_of_execution_ptr, instruction_position)
}

fn resolve_labels(param:& ParamOrLabel,lable_table: &Vec<LabelPointer>, defines: &HashMap<String,u64>, instruction_position:&Vec<u64>) -> Option<Param>
{
    match param
    {
        ParamOrLabel::Nothing                                   => None,
        ParamOrLabel::Param(x)                                  => Some(*x),
        ParamOrLabel::Label(x,LabelUse::Raw)                    => Some(Param::Constant(label_to_adress(x.clone(), lable_table, instruction_position))),
        ParamOrLabel::Label(x,LabelUse::Deref)                  => Some(Param::MemPtr(label_to_adress(x.clone(), lable_table, instruction_position))),
        ParamOrLabel::Label(x,LabelUse::DerefOffset(offset))    => Some(Param::MemPtr((label_to_adress(x.clone(), lable_table, instruction_position)as i64 + offset)as u64)), //TODO: is this right? and check for save parse
        ParamOrLabel::DefineLabel(x)                            => Some(Param::Constant(define_to_constant(x.clone(), defines)))
    }
}

fn define_to_constant(define_name: String, defines: &HashMap<String,u64>) -> u64
{
    if let Some(val) = defines.get(&define_name)
    {
        *val
    }
    else
    {
        panic!("cound not find definition for {}", define_name)
    }
}

fn label_to_adress(label_use: String, lable_table:& Vec<LabelPointer>, instruction_position:&Vec<u64>) -> u64
{
    for label in lable_table
    {  
        if label.identifier == label_use
        {
            if label.label_type == LabelType::ArdressToRomData
            {
                // the pos is the correct ptr to memory
                return label.pos;
            }
            else
            {
                return instruction_position[label.pos as usize];
            }
        }
    }

    // print label table:
    print!("label table:\n");
    lable_table.into_iter().for_each(|x| print!("{:#?}\n",x));
    panic!("clound not find label: {}", label_use);

}

fn calc_sersed_code_positions(code: &Vec<AsmLineLabel>, start_of_code_section: u64) -> Vec<u64>
{
    // we need to know the positons of the code in the binary to 
    // correctly resolve the jump labels
    
    // contains the adress to the n-th instruction indexed
    // the first instruction is at the start of code section
    let mut instruction_position = vec![start_of_code_section];

    for inst in code
    {
        // the instruction itself is always one vec-entry (u64) 
        // add the size of both parameters
        let size:u64 = 1+instruction_size(inst.param1.clone())+instruction_size(inst.param2.clone());
        
        // add the size of the instruction to the last position
        instruction_position.push(instruction_position[instruction_position.len()-1]+size);
    }

    // remove the last entry since it points to a instruction that does not exsist
    instruction_position.pop();

    instruction_position
}

fn instruction_size(ins: ParamOrLabel) -> u64
{
    match ins
    {
        ParamOrLabel::Nothing                           => 0,
        ParamOrLabel::Label(_,LabelUse::Raw)            => 1,
        ParamOrLabel::Label(_,LabelUse::Deref)          => 1,
        ParamOrLabel::Label(_,LabelUse::DerefOffset(_)) => 1,
        ParamOrLabel::Param(Param::Register(_))         => 0,
        ParamOrLabel::Param(Param::MemPtr(_))           => 1,
        ParamOrLabel::Param(Param::MemPtrOffset(_,_))   => 1,
        ParamOrLabel::Param(Param::Constant(_))         => 1,
        ParamOrLabel::DefineLabel(_)                    => 1,
    }
}