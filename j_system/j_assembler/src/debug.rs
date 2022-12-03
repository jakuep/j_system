use j_system_definition::instructions::AsmLine;
use std::fs;

pub fn debug_ouput(code: &Vec<AsmLine>, instruction_position: Vec<u64>, start: u64, rom_len: u64)
{
    let mut s = String::new();

    let ziped = code.iter().zip(instruction_position.iter());

    s.push_str(&format!("start of execution: {}\n",start));
    s.push_str(&format!("rom size: {}\n\n\n",rom_len));

    for (ins, pos) in ziped
    {
        s.push_str(&format!("{}\t", pos));
        s.push_str(&ins.as_string());
    }
 
    fs::write("debug.txt", s).unwrap();
}
