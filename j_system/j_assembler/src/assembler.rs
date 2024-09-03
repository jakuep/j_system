use crate::label_resolve::*;
use crate::serialization::*;
//use crate::decode_instructons::*;
use crate::debug::*;
use crate::logging::j_log;
use crate::preprocessor::*;

use lazy_static::lazy_static;
use regex::Regex;

use crate::j_system_definition::instructions;
use crate::j_system_definition::register;
use std::collections::HashMap;

// Regex definitions
lazy_static! {
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
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct RomData {}

/// distinguish between Jumps and Rom labels beacuse rom and code section will be split
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum LabelType {
    JumpLabel(u64),
    Rom(u64),
}

#[derive(Clone, PartialEq, Debug)]
pub struct AssembledFile {
    /// contains the filename (and path?) of the original input file
    pub name: String,
    pub instructions: Vec<UnlinkedInstruction>,
    pub rom: Vec<RomData>,

    /// contains the offset in the file that the label points to.
    pub linker_info: HashMap<String, LabelType>,
}
#[derive(Clone, PartialEq, Debug)]
pub struct UnlinkedInstruction {
    /// the line number in the original input file
    pub line: u64,
    pub inst: instructions::InstructionEnum,
    pub param1: Option<UnlinkedParameter>,
    pub param2: Option<UnlinkedParameter>,
}

impl UnlinkedInstruction {
    pub fn size(&self) -> u8 {
        // 1 is for the instruction itself
        1 + Self::param_size(&self.param1) + Self::param_size(&self.param2)
    }

    fn param_size(param: &Option<UnlinkedParameter>) -> u8 {
        if let Some(p) = param {
            p.size()
        } else {
            0
        }
    }
}
#[derive(Clone, PartialEq, Debug)]
pub enum UnlinkedParameter {
    Determined(instructions::Param),

    /// contains the name (1) of the label and the origin file-name (2) that has the jump label / data label.
    LinkerReslovedLabel(LinkerResolvedLabel),
}

impl UnlinkedParameter {
    pub fn size(&self) -> u8 {
        match self {
            Self::Determined(d) => match d {
                instructions::Param::Constant(_) => 1,
                instructions::Param::MemPtr(_) => 1,
                instructions::Param::MemPtrOffset(_, _) => 1,
                instructions::Param::Register(_) => 0,
            },
            Self::LinkerReslovedLabel(l) => match l.teip {
                LabelUse::Deref => 1,

                // deref offset will be added to the label value at compile time.
                // that results in only one value to store
                LabelUse::DerefOffset(_) => 1,
                LabelUse::Raw => 1,
            },
        }
    }
}
#[derive(Clone, PartialEq, Debug)]
pub struct LinkerResolvedLabel {
    pub label_name: String,
    pub teip: LabelUse,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum LabelUse {
    /// eg: jmp .jumplabel
    Raw,

    /// eg: mov a, [.data]
    Deref,

    /// eg: mov a, [.data+8]
    DerefOffset(i64),
}

/*
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

*/

pub fn assemble_file(
    mut input_file: SourceFileRun2,
    file_name: String,
) -> Result<AssembledFile, String> {
    let mut ii = 0;
    // remove comments and empty lines
    // len might change during iteration
    while ii < input_file.content.len() {
        // remove comments
        // search for the first semicolon as begin of comment
        if let Some(index) = input_file.content[ii].content.find(';') {
            // revome everthing behind and including the ';'
            input_file.content[ii].content.truncate(index);
        }

        // only inc ii by one if no line gets removed
        if input_file.content[ii].content.trim().is_empty() {
            let _ = input_file.content.remove(ii);
        } else {
            ii += 1;
        }
    }

    let sections = split_sections(&input_file, &file_name)?;

    // start parsing instructions and parameters
    let mut parsed_instructions = vec![];
    if let Some(code) = sections.code {
        parse_instructions(code, &mut parsed_instructions, &file_name)?;
    }

    let mut parsed_rom = vec![];
    if let Some(rom) = sections.rom {
        parse_rom(rom, &mut parsed_rom, &file_name)?;
    }

    // parse rom section

    Ok(AssembledFile {
        name: file_name,
        instructions: vec![],
        rom: vec![],
        linker_info: HashMap::new(),
    })
}

struct AsmSections {
    code: Option<Vec<RawLine>>,
    rom: Option<Vec<RawLine>>,
}

/// split the sections of one input file
/// it is allowed to not have both if the correct flags are set
fn split_sections(input_file: &SourceFileRun2, file_name: &str) -> Result<AsmSections, String> {
    match (
        input_file.flags.contains_key("nocode"),
        input_file.flags.contains_key("norom"),
    ) {
        // file only contains definitions, flags, ... , but no code or rom
        (true, true) => {
            return Ok(AsmSections {
                code: None,
                rom: None,
            })
        }

        // treat everything as code
        (false, true) => {
            // remove empty lines
            let mut cleaned_content: Vec<_> = input_file
                .content
                .iter()
                .filter(|line| !line.content.chars().all(|c| c.is_whitespace()))
                .map(|e| e.clone())
                .collect();
            // first line should be "_code"
            if cleaned_content[0].content.trim() == "_code" {
                cleaned_content.remove(0);
            } else {
                return Err(format!(
                    "could not find begin of code section, missing '_code' in file {}",
                    file_name
                ));
            }
            return Ok(AsmSections {
                code: Some(cleaned_content),
                rom: None,
            });
        }

        // treat everything as rom
        (true, false) => {
            // remove empty lines
            let mut cleaned_content: Vec<_> = input_file
                .content
                .iter()
                .filter(|line| !line.content.chars().all(|c| c.is_whitespace()))
                .map(|e| e.clone())
                .collect();
            // first line should be "_rom"
            if cleaned_content[0].content.trim() == "_rom" {
                cleaned_content.remove(0);
            } else {
                return Err(format!(
                    "could not find begin of rom section, missing '_rom' in file {}",
                    file_name
                ));
            }
            return Ok(AsmSections {
                code: None,
                rom: Some(cleaned_content),
            });
        }

        // continue with parsing
        _ => {}
    }

    if input_file.content.len() == 0 {
        return Err(format!("could not find rom/code section in file '{}'.\nHint: if you wish not to use code or rom sections in this file, set the 'nocode'/'norom' flag(s)", file_name));
    }

    // after the preprocessor and the removing of empty lines,
    // the first line should be "_rom" or "_code"

    // search for rom
    // TODO: make it so that _code and _rom can have instructions in the same line and do NOT need a newline
    match input_file.content[0].content.trim() {
        "_rom" => {
            // search for "_code"
            for (index, line) in input_file.content.iter().enumerate() {
                if line.content.trim() == "_code" {
                    return Ok(AsmSections {
                        code: Some(input_file.content[1..index].to_vec()),
                        rom: Some(input_file.content[index + 1..].to_vec()),
                    });
                }
            }
            return Err(format!(
                "read _rom but could not find _code section in file: '{}'",
                file_name
            ));
        }
        "_code" => {
            // search for "_rom"
            for (index, line) in input_file.content.iter().enumerate() {
                if line.content.trim() == "_rom" {
                    return Ok(AsmSections {
                        rom: Some(input_file.content[1..index].to_vec()),
                        code: Some(input_file.content[index + 1..].to_vec()),
                    });
                }
            }
            return Err(format!(
                "read _code but could not find _rom section in file: '{}'",
                file_name
            ));
        }
        line => {
            return Err(format!(
                "could not partse: {} in '{}' line {}",
                line, file_name, input_file.content[0].line_number
            ))
        }
    }
}

fn parse_instructions(
    code: Vec<RawLine>,
    parsed_instructions: &mut Vec<UnlinkedInstruction>,
    file_name: &str,
) -> Result<(), String> {
    for line in code {
        // split in parts
        let parts: Vec<_> = line
            .content
            .split(char::is_whitespace)
            .map(|s| s.to_string())
            .collect();
        let parameter_part = parts.iter().skip(1).fold(String::new(), |a, s| a + s);

        // find the instruction in this line
        match parts[0].as_str() {
            "add" => {
                let params = parse_parameters(parameter_part, line.line_number, file_name)?;
                if params.len() == 2 {
                    parsed_instructions.push(UnlinkedInstruction {
                        line: line.line_number,
                        inst: instructions::InstructionEnum::add,
                        param1: Some(params[0].clone()),
                        param2: Some(params[1].clone()),
                    });
                } else {
                    return Err(format!("instruction 'add' needs 2 parameters but only {} could be parsed in line {} in file '{}' ", 
                    params.len(),line.line_number, file_name));
                }
            }

            "sub" => {
                let params = parse_parameters(parameter_part, line.line_number, file_name)?;
                if params.len() == 2 {
                    parsed_instructions.push(UnlinkedInstruction {
                        line: line.line_number,
                        inst: instructions::InstructionEnum::sub,
                        param1: Some(params[0].clone()),
                        param2: Some(params[1].clone()),
                    });
                } else {
                    return Err(format!("instruction 'sub' needs 2 parameters but only {} could be parsed in line {} in file '{}' ", 
                    params.len(),line.line_number, file_name));
                }
            }

            "xor" => {}
            "or" => {}
            "and" => {}
            "shr" => {}
            "shl" => {}
            "jmp" => {}
            "cmp" => {}
            "je" => {}
            "jeg" => {}
            "jel" => {}
            "jg" => {}
            "jl" => {}
            "mov" => {}
            "push" => {}
            "pop" => {}
            "pusha" => {}
            "popa" => {}
            "call" => {}
            "ret" => {}
            "sys" => {}
            x => {
                return Err(format!(
                    "could not parse instruction '{}' in line {} in file '{}'",
                    x, line.line_number, file_name
                ))
            }
        }
    }

    return Ok(());
}

/// return a vector with all found parameter.
/// can only have 0,1 or 2 entries
fn parse_parameters(
    content: String,
    line_number: u64,
    file_name: &str,
) -> Result<Vec<UnlinkedParameter>, String> {
    let parts: Vec<_> = content.trim().split(',').map(|s| s.trim()).collect();

    // maximum of 2 parametes allowed
    assert!(parts.len() <= 2);
    let mut ret = vec![];
    for param in parts {
        // check if value is a number
        if param.chars().all(|c| c.is_ascii_digit()) {
            let maybe_number = param.parse();
            if let Ok(number) = maybe_number {
                ret.push(UnlinkedParameter::Determined(
                    instructions::Param::Constant(number),
                ));
                j_log(&format!("decoded parameter: {:?}\n", ret[ret.len() - 1]), 3);
                continue;
            } else {
                return Err(format!(
                    "found all digits but could not parse value form '{}' in line {} in file '{}'",
                    param, line_number, file_name
                ));
            }
        }

        if let Some(reg) = parse_register(param) {
            ret.push(UnlinkedParameter::Determined(
                instructions::Param::Register(reg),
            ));
            j_log(&format!("decoded parameter: {:?}\n", ret[ret.len() - 1]), 3);
            continue;
        }

        // check for label without deref
        if param.starts_with('.') {
            // remove leading dot
            let label_name: String = param.chars().skip(1).collect();

            // NOTE: do not allow a label to have a offset if its not derefed
            if label_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                ret.push(UnlinkedParameter::LinkerReslovedLabel(
                    LinkerResolvedLabel {
                        label_name,
                        teip: LabelUse::Raw,
                    },
                ));
                j_log(&format!("decoded parameter: {:?}\n", ret[ret.len() - 1]), 3);
                continue;
            } else {
                return Err(format!(
                    "could not parse label name '{}' in line {} in file {}",
                    label_name, line_number, file_name
                ));
            }
        }

        //  [.label], [.label+1], [.label-1], [a] , [a+1] , [a-1]  or [42]
        if param.starts_with('[') && param.ends_with(']') {
            let mut s = param.chars();
            // remove '[' and ']'
            s.next();
            s.next_back();
            let p = s.collect::<String>();
            let param_no_bracket = p.trim();

            // check if the content is just a positive integer
            if param_no_bracket.chars().all(|c| c.is_ascii_digit()) {
                let val: u64 = param_no_bracket.parse().or_else(|_| {
                    Err(format!(
                        "could not parse number in dref in line {} in file {}",
                        line_number, file_name
                    ))
                })?;
                ret.push(UnlinkedParameter::Determined(instructions::Param::MemPtr(
                    val,
                )));
                j_log(&format!("decoded parameter: {:?}\n", ret[ret.len() - 1]), 3);
                continue;
            }

            // if it stats with a dot, that means it is a label.
            if param_no_bracket.starts_with('.') {
                // remove the dot & remove all whitespaces
                let label_with_maybe_offset: String = param_no_bracket
                    .chars()
                    .skip(1)
                    .filter(|c| !c.is_whitespace())
                    .collect();
                // get only the labelname aka split off the offset part if it is present
                // otherwise just get the label as parameter
                if let Some(offset_begins_at) = label_with_maybe_offset.find(['+', '-']) {
                    let offet_part = &label_with_maybe_offset[offset_begins_at..].trim();
                    let label_name = &label_with_maybe_offset[..offset_begins_at].trim();
                    if let Some(param_offset) = get_param_offset(offet_part) {
                        if label_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                            ret.push(UnlinkedParameter::LinkerReslovedLabel(
                                LinkerResolvedLabel {
                                    label_name: label_name.to_string(),
                                    teip: LabelUse::DerefOffset(param_offset),
                                },
                            ));
                            j_log(&format!("decoded parameter: {:?}\n", ret[ret.len() - 1]), 3);
                            continue;
                        } else {
                            return Err(format!(
                                "could not parse parameter '{}' in line {} in file '{}'\n",
                                param, line_number, file_name,
                            ));
                        }
                    } else {
                        return Err(format!("could not parse parameter '{}' in line {} in file '{}', because the offset of the label could not be decoded",param,line_number,file_name));
                    }
                } else {
                    if label_with_maybe_offset
                        .chars()
                        .all(|c| c.is_alphanumeric() || c == '_')
                    {
                        ret.push(UnlinkedParameter::LinkerReslovedLabel(
                            LinkerResolvedLabel {
                                label_name: label_with_maybe_offset.to_string(),
                                teip: LabelUse::Deref,
                            },
                        ));
                        j_log(&format!("decoded parameter: {:?}\n", ret[ret.len() - 1]), 3);
                        continue;
                    }
                }
            }

            // check for register dref oder register dref with offset
            if let Some(offset_part) = param_no_bracket.find(['+', '-']) {
                let reg_part = &param_no_bracket[..offset_part];
                let offset_part = &param_no_bracket[offset_part..];
                if let (Some(register), Some(offset)) = (
                    parse_register(reg_part.trim()),
                    get_param_offset(offset_part),
                ) {
                    ret.push(UnlinkedParameter::Determined(
                        instructions::Param::MemPtrOffset(register, offset),
                    ));
                    j_log(&format!("decoded parameter: {:?}\n", ret[ret.len() - 1]), 3);
                    continue;
                } else {
                    return Err(format!(
                        "could not parse parameter '{}' in line {} in file '{}'\n",
                        param, line_number, file_name
                    ));
                }
            } else {
                if let Some(register) = parse_register(param_no_bracket.trim()) {
                    ret.push(UnlinkedParameter::Determined(
                        instructions::Param::Register(register),
                    ));
                    j_log(&format!("decoded parameter: {:?}\n", ret[ret.len() - 1]), 3);
                    continue;
                } else {
                    return Err(format!(
                        "could not parse parameter '{}' in line {} in file '{}'\n",
                        param, line_number, file_name
                    ));
                }
            }
        }
        return Err(format!(
            "could not parse parameter '{}' in line {} in file '{}'\n",
            param, line_number, file_name
        ));
    }

    Ok(ret)
    //Err(format!("not implemented!"))
}

fn parse_rom(
    code: Vec<RawLine>,
    parsed_rom: &mut Vec<RomData>,
    file_name: &str,
) -> Result<(), String> {
    return Ok(());
}

fn get_param_offset(inp: &str) -> Option<i64> {
    let trimmed = inp.trim();
    if trimmed.starts_with(['+', '-']) {
        // remove the '+' / '-'
        let maybe_number: String = trimmed.chars().skip(1).collect();
        return maybe_number.parse::<i64>().ok();
    }
    None
}

fn parse_register(inp: &str) -> Option<register::Register> {
    match inp {
        // general registers
        "a" => Some(register::Register::a),
        "b" => Some(register::Register::b),
        "c" => Some(register::Register::c),
        "d" => Some(register::Register::d),
        "e" => Some(register::Register::e),
        "f" => Some(register::Register::f),
        // special registers
        "tos" => Some(register::Register::tos),
        "bos" => Some(register::Register::bos),
        "pc" => Some(register::Register::pc),
        "s" => Some(register::Register::s),
        _ => None,
    }
}
