use crate::label_resolve::*;
use crate::serialization::*;
use crate::decode_instructons::*;
use crate::debug::*;
use crate::preprocessor::*;

use lazy_static::lazy_static;
use regex::Regex;

use std::collections::HashMap;
use crate::j_system_definition::instructions;

// Regex definitions
lazy_static!
{
    // define constans
    // static ref RE_DEFINE_CONST:         Regex = Regex::new(r"^\s*#\s*define\s+([A-Za-z_][A-Za-z0-9_]*)\s+([0-9]+)\s*(?:\s+;.*)?$").unwrap();
    // static ref RE_GET_DEFINE_CONST:     Regex = Regex::new(r"^\s*\$\s*([a-zA-Z_][0-9a-zA-Z_]*)\s*$").unwrap();
    // static ref RE_GET_DEFINE_OFFSET:    Regex = Regex::new(r"").unwrap();
    // static ref RE_GET_DEFINE_DEREF:     Regex = Regex::new(r"").unwrap();
    // static ref RE_DEFINE_GET_LEN:       Regex = unimplemented!();

    // detect include statements
    static ref RE_INCLUDE:              Regex = Regex::new(r"^\s*#\s*include\s+((?:[A-Za-z0-9_])+\.asm)\s*;?.*$").unwrap();

    // detect rom and code sections
    static ref RE_ROM_SECTION_START:    Regex = Regex::new(r"^\s*_rom\s*(?:\s+;.*)?$").unwrap();
    static ref RE_CODE_SECTION_START:   Regex = Regex::new(r"^\s*_code\s*(?:\s+;.*)?$").unwrap();

    // label should look like this: .start: .fun:
    static ref RE_GET_CODE_LABEL:       Regex = Regex::new(r"^\s*\.([A-Za-z][A-Za-z0-9_]*):\s*$").unwrap();

    // parse instruction from line of code and save the rest of the line for later
    static ref RE_INSTRUCTION_CAPTURE:  Regex = Regex::new(r"^\s*(add|sub|xor|or|and|shr|shl|jmp|cmp|je|jeg|jel|jg|jl|mov|push|pop|pusha|popa|call|ret|sys)(?:\s+(.*))?\s*$").unwrap();

    // label rom detection
    static ref RE_ROM_GET_ELEMENT:      Regex = Regex::new(r"^\s*([a-zA-Z][0-9a-zA-Z_]*)\s*:\s*(i|s|ai|as)\s*(.+)\s*$").unwrap();

    // parse parameter list aka none,one or two parameters
    static ref RE_PARSE_PRAMS:          Regex = Regex::new(r"^\s*(([\$0-9a-zA-Z\[\]\+\-\.]*)(?:\s*,\s*([\$0-9a-zA-Z\[\]\+\-\.]*))?)?\s*$").unwrap();

    // match the register
    static ref RE_REGISTER:             Regex = Regex::new(r"^(a|b|c|d|e|f|s|tos|bos|pc)$").unwrap();

    // match the constant
    static ref RE_CONSTANT:             Regex = Regex::new(r"^([0-9]+)$").unwrap();

    // match constant pointer to memony
    static ref RE_MEM_PTR_CONST:        Regex = Regex::new(r"^\s*\[\s*([0-9]+)\s*\]\s*$").unwrap();

    // match register with offset pointer to memory
    static ref RE_MEM_PTR_REG_OFFSET:   Regex = Regex::new(r"^\s*\[\s*(a|b|c|d|e|f|bos|tos|s|pc)(?:(\+|\-)([0-9]*))?\s*\]\s*$").unwrap();

    // check if line contains a dot
    // use this for checking if a parameter refers to a label
    static ref RE_CONTAINS_LABEL:       Regex = Regex::new(r"\.[a-zA-Z0-9]+").unwrap();

    // match label 
    static ref RE_LABEL_AS_POINTER:     Regex = Regex::new(r"^\s*\.([a-zA-Z0-9]+)\s*$").unwrap();
    static ref RE_LABEL_DEREF:          Regex = Regex::new(r"^\s*\[\s*\.([a-zA-Z0-9]+)\s*\]\s*$").unwrap();
    static ref RE_LABEL_DEREF_OFFSET:   Regex = Regex::new(r"^\s*\[\s*\.([a-zA-Z0-9]+)\s*(\+|\-)\s*([0-9]+)\s*\]\s*$").unwrap();
}

pub struct AssembledFile
{
    /// contains the filename (and path?) of the original input file
    name: String,
    instructions: Vec<UnlinkedInstruction>
}

pub struct UnlinkedInstruction
{
    /// the line number in the original input file
    line: u64,
    inst: instructions::InstructionEnum,
    param1: Option<UnlinkedParameter>,
    param2: Option<UnlinkedParameter>,
}

pub enum UnlinkedParameter
{
    Determined(instructions::Param),

    /// contains the name (1) of the label and the origin file-name (2) that has the jump label / data label.
    LinkerReslovedLabel(String,String),
}

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

    let mut debug_symbols = vec![];

    let (final_code, start_of_execution_ptr, instruction_position) = remove_labels_from_asm(code_with_labels, &mut rom_table, defines, &mut debug_symbols ,rom_len);
    
    // create debug output
    debug_ouput(&final_code, instruction_position, start_of_execution_ptr, rom_len, debug_symbols);

    let mut binary: Vec<u64> = rom_raw;

    binary.append(&mut serialize_asm(final_code));

    // instert point of seclection split
    binary.push(rom_len);

    // inset start of execution
    binary.push(start_of_execution_ptr);

    binary
}

fn assemble_file(mut input_file: SourceFileRun2,file_name: String) -> Result<AssembledFile,String>
{
    let mut ii = 0;
    // remove comments and empty lines
    // len might change during iteration
    while ii < input_file.content.len()
    {
        // remove comments
        // search for ';' as begin of comment
        if let Some(index) = input_file.content[ii].content.find(';')
        {
            // revome everthing behind and including the ';'
            input_file.content[ii].content.truncate(index);
        }

        // only inc ii by one if no line gets removed
        if  input_file.content[ii].content.trim().is_empty()
        {
            let _ = input_file.content.remove(ii);
        }
        else 
        {
            ii += 1;    
        }
    }

    let sections = split_sections(&input_file, &file_name)?;

    // parse instruction and parameters

    Ok(AssembledFile { name: file_name, instructions: vec![]})

}

struct AsmSections
{
    code: Option<Vec<RawLine>>,
    rom: Option<Vec<RawLine>>,
}

/// split the sections of one input file
/// it is allowed to not have both if the correct flags are set
fn split_sections(input_file: &SourceFileRun2, file_name: &str) -> Result<AsmSections,String>
{
    match (input_file.flags.contains_key("romonly"),
        input_file.flags.contains_key("codeonly")) 
    {
        // only one of those flags should be set
        (true,true) => return Err(format!("file '{}' contains both the 'codeonly' and 'romonly' flag",file_name)),
        
        // treat everything as code
        (false,true) => return Ok(AsmSections { code: Some(input_file.content.clone()), rom: None}),
        
        // treat everything as code
        (true,false) => return Ok(AsmSections { code: None, rom: Some(input_file.content.clone())}),
        
        // continue with parsing
        _ =>{},
    }

    Err("could not parse code/rom section...".into())
}