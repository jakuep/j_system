use crate::label_resolve::*;
use crate::serialization::*;
use crate::decode_instructons::*;
use crate::debug::*;

pub fn assemble_into_u64_vec(input: Vec<String>, main_file_name: String) -> Vec<u64>
{   
    let preprocessed = preprocess_input(input, main_file_name);
    let code_section = preprocessed.code; 
    let rom_section  = preprocessed.rom;
    let defines      = preprocessed.defines;

    // TODO: why is this called rom table? doesnt it incluce all labels???
    let (rom_raw, mut rom_table) = parse_rom(rom_section);

    // save len for later insertion since it will be "moved" into binary
    let rom_len = rom_raw.len() as u64;

    let code_with_labels = parse_code(code_section, &mut rom_table);

    let (final_code,start_of_execution_ptr, instruction_position) = remove_labels_from_asm(code_with_labels, &mut rom_table, defines, rom_len);
    
    // create debug output
    debug_ouput(&final_code, instruction_position, start_of_execution_ptr, rom_len, rom_table);

    let mut binary: Vec<u64> = rom_raw;

    binary.append(&mut serialize_asm(final_code));

    // instert point of seclection split
    binary.push(rom_len);

    // inset start of execution
    binary.push(start_of_execution_ptr);

    binary
}
