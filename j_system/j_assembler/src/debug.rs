use j_system_definition::instructions::AsmLine;
use crate::decode_instructons::LabelPointer;
use std::fs;

pub fn debug_ouput(
    code: &Vec<AsmLine>, 
    instruction_position: Vec<u64>, 
    start: u64, 
    rom_len: u64, 
    labels: Vec<(String,u64)>)
{
    let mut s = String::new();

    let ziped = code.iter().zip(instruction_position.iter());

    s.push_str(&format!("start of execution: {}\n",start));
    s.push_str(&format!("rom size: {}\n\n\n",rom_len));

    for (ins, pos) in ziped
    {
        s.push_str(&format!("{}\t", pos));
        s.push_str(&ins.as_string());
        s.push('\n');
    }
 
    fs::write("debug.txt", s).unwrap();

    let mut s = String::new();
    for label in labels
    {
        s.push_str(&format!("{}\t{}\n",label.1,label.0))
    }
    fs::write("labels.dbg",s).unwrap();
}
