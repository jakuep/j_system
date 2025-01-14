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
#[derive(Clone, PartialEq, Debug, Default)]
pub struct RomData {
    // the raw encooded data
    raw_data: Vec<u64>,

    // the offset into raw_data for the given label
    label_offset: HashMap<String, usize>,
}

/// distinguish between Jumps and Rom labels beacuse rom and code section will be split
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum LabelType {
    JumpLabel(usize),
    Rom(usize),
}

#[derive(Clone, PartialEq, Debug)]
pub struct AssembledFile {
    /// contains the filename (and path?) of the original input file
    pub name: String,
    pub instructions: Vec<UnlinkedInstruction>,
    pub instruction_real_size: usize,
    pub rom: Vec<u64>,
    pub rom_real_size: usize,

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
    pub fn is_register(&self) -> bool {
        if let UnlinkedParameter::Determined(instructions::Param::Register(_)) = self {
            true
        } else {
            false
        }
    }

    pub fn is_constant(&self) -> bool {
        if let UnlinkedParameter::Determined(instructions::Param::Constant(_)) = self {
            true
        } else {
            false
        }
    }

    pub fn is_not_constant(&self) -> bool {
        // check if the parameter is a constant.
        // that is the case when a prameter is directly determined to be a constant
        // or when it is a unresolved label corrosponing to a cosnstant aka not using dref
        match self {
            UnlinkedParameter::Determined(instructions::Param::Constant(_))
            | UnlinkedParameter::LinkerReslovedLabel(LinkerResolvedLabel {
                label_name: _,
                teip: LabelUse::Raw,
            }) => false,
            _ => true,
        }
    }

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

#[derive(Clone, PartialEq, Debug, Default)]
pub struct ParsedCodeSection {
    parsed_instructions: Vec<UnlinkedInstruction>,
    // the offset into final raw data of the instructions for the given label
    label_offset: HashMap<String, usize>,
    // size of the final raw data in amounts of u64s
    size: usize,
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
    // TODO: make remove the default
    let mut parsed_code_section = ParsedCodeSection::default();
    if let Some(code) = sections.code {
        parsed_code_section = parse_instructions(code, &file_name)?;
    }

    // TODO: make remove the default
    let mut parsed_rom = RomData::default();
    if let Some(rom) = sections.rom {
        parsed_rom = parse_rom(rom, &file_name)?;
    }

    // merge the hashmaps
    // this is ok because a label must be unique in one file and cannot be used seperately for rom and jump
    let mut offset_map = HashMap::new();
    parsed_rom
        .label_offset
        .into_iter()
        .for_each(|(label_name, offset)| {if let Some(_) = offset_map.insert(label_name, LabelType::Rom(offset)){
            panic!("double definition in transfer of original hashmap from rom offsets - this should NOT FAIL!")
        }});

    for (label_name, offset) in parsed_code_section.label_offset {
        if let Some(_) = offset_map.insert(label_name.clone(), LabelType::JumpLabel(offset)) {
            return Err(format!("double definition of label '{}' in file '{}' - is defined once in code section and once in rom section. Label names must be unique per file regardless of the section they are used in",label_name, file_name));
        }
    }

    Ok(AssembledFile {
        name: file_name,
        instructions: parsed_code_section.parsed_instructions,
        instruction_real_size: parsed_code_section.size,
        rom_real_size: parsed_rom.raw_data.len(),
        rom: parsed_rom.raw_data,
        linker_info: offset_map,
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

fn parse_instructions(code: Vec<RawLine>, file_name: &str) -> Result<ParsedCodeSection, String> {
    // current offset of the final raw data to keep track of where the labels point to
    let mut current_offset: usize = 0;
    let mut parsed_instructions = vec![];

    // keep track of all labels and their offsets in the code section
    let mut label_offset = HashMap::new();

    for line in code {
        // check for label
        let line_t = line.content.trim();
        if line_t.starts_with('.') && line_t.ends_with(':') {
            let mut line_t = line_t.chars();
            // remove the '.' and the ':'
            line_t.next();
            line_t.next_back();
            let label_name: String = line_t.collect();
            if let Some(_) = label_offset.insert(label_name.clone(), current_offset) {
                return Err(format!(
                    "double definition of label: {}, second definition in line {} in file {}",
                    label_name, line.line_number, file_name
                ));
            } else {
                j_log(
                    &format!(
                        "label parsed {} with offset: {}",
                        label_name, current_offset
                    ),
                    3,
                );
            }
            // prevent the label being interoreted as a instruction AND prevent the addition of a wrong offset
            continue;
        }

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
                    if params[0].is_register() {
                        parsed_instructions.push(UnlinkedInstruction {
                            line: line.line_number,
                            inst: instructions::InstructionEnum::add,
                            param1: Some(params[0].clone()),
                            param2: Some(params[1].clone()),
                        });
                    } else {
                        return Err(format!("instruction 'add' has 2 parameters but the first has to be a register line {} in file '{}' ", 
                    line.line_number, file_name));
                    }
                } else {
                    return Err(format!("instruction 'add' needs 2 parameters but {} were parsed in line {} in file '{}' ", 
                    params.len(),line.line_number, file_name));
                }
            }

            "sub" => {
                let params = parse_parameters(parameter_part, line.line_number, file_name)?;
                if params.len() == 2 {
                    if params[0].is_register() {
                        parsed_instructions.push(UnlinkedInstruction {
                            line: line.line_number,
                            inst: instructions::InstructionEnum::sub,
                            param1: Some(params[0].clone()),
                            param2: Some(params[1].clone()),
                        });
                    } else {
                        return Err(format!("instruction 'sub' has 2 parameters but the first has to be a register line {} in file '{}' ", 
                    line.line_number, file_name));
                    }
                } else {
                    return Err(format!("instruction 'sub' needs 2 parameters but {} were parsed in line {} in file '{}' ", 
                    params.len(),line.line_number, file_name));
                }
            }

            "xor" => {
                let params = parse_parameters(parameter_part, line.line_number, file_name)?;
                if params.len() == 2 {
                    if params[0].is_register() {
                        parsed_instructions.push(UnlinkedInstruction {
                            line: line.line_number,
                            inst: instructions::InstructionEnum::xor,
                            param1: Some(params[0].clone()),
                            param2: Some(params[1].clone()),
                        });
                    } else {
                        return Err(format!("instruction 'xor' has 2 parameters but the first has to be a register line {} in file '{}' ", 
                    line.line_number, file_name));
                    }
                } else {
                    return Err(format!("instruction 'xor' needs 2 parameters but {} were parsed in line {} in file '{}' ", 
                    params.len(),line.line_number, file_name));
                }
            }
            "or" => {
                let params = parse_parameters(parameter_part, line.line_number, file_name)?;
                if params.len() == 2 {
                    if params[0].is_register() {
                        parsed_instructions.push(UnlinkedInstruction {
                            line: line.line_number,
                            inst: instructions::InstructionEnum::or,
                            param1: Some(params[0].clone()),
                            param2: Some(params[1].clone()),
                        });
                    } else {
                        return Err(format!("instruction 'or' has 2 parameters but the first has to be a register line {} in file '{}' ", 
                    line.line_number, file_name));
                    }
                } else {
                    return Err(format!("instruction 'or' needs 2 parameters but {} were parsed in line {} in file '{}' ", 
                    params.len(),line.line_number, file_name));
                }
            }
            "and" => {
                let params = parse_parameters(parameter_part, line.line_number, file_name)?;
                if params.len() == 2 {
                    if params[0].is_register() {
                        parsed_instructions.push(UnlinkedInstruction {
                            line: line.line_number,
                            inst: instructions::InstructionEnum::and,
                            param1: Some(params[0].clone()),
                            param2: Some(params[1].clone()),
                        });
                    } else {
                        return Err(format!("instruction 'and' has 2 parameters but the first has to be a register line {} in file '{}' ", 
                    line.line_number, file_name));
                    }
                } else {
                    return Err(format!("instruction 'and' needs 2 parameters but {} were parsed in line {} in file '{}' ", 
                    params.len(),line.line_number, file_name));
                }
            }
            "shr" => {
                let params = parse_parameters(parameter_part, line.line_number, file_name)?;
                if params.len() == 2 {
                    if params[0].is_register() && params[1].is_constant() {
                        parsed_instructions.push(UnlinkedInstruction {
                            line: line.line_number,
                            inst: instructions::InstructionEnum::shr,
                            param1: Some(params[0].clone()),
                            param2: Some(params[1].clone()),
                        });
                    } else {
                        return Err(format!("instruction 'shr' has 2 parameters but the first has to be a register line {} in file '{}' ", 
                    line.line_number, file_name));
                    }
                } else {
                    return Err(format!("instruction 'shr' needs 2 parameters but {} were parsed in line {} in file '{}' ", 
                    params.len(),line.line_number, file_name));
                }
            }
            "shl" => {
                let params = parse_parameters(parameter_part, line.line_number, file_name)?;
                if params.len() == 2 {
                    if params[0].is_register() && params[1].is_constant() {
                        parsed_instructions.push(UnlinkedInstruction {
                            line: line.line_number,
                            inst: instructions::InstructionEnum::shl,
                            param1: Some(params[0].clone()),
                            param2: Some(params[1].clone()),
                        });
                    } else {
                        return Err(format!("instruction 'shl' has 2 parameters but the first has to be a register line {} in file '{}' ", 
                    line.line_number, file_name));
                    }
                } else {
                    return Err(format!("instruction 'shl' needs 2 parameters but {} were parsed in line {} in file '{}' ", 
                    params.len(),line.line_number, file_name));
                }
            }
            "jmp" => {
                let params = parse_parameters(parameter_part, line.line_number, file_name)?;
                if params.len() == 1 {
                    parsed_instructions.push(UnlinkedInstruction {
                        line: line.line_number,
                        inst: instructions::InstructionEnum::jmp,
                        param1: Some(params[0].clone()),
                        param2: None,
                    });
                } else {
                    return Err(format!("instruction 'jmp' needs 1 parameter but {} were parsed in line {} in file '{}' ", 
                    params.len(),line.line_number, file_name));
                }
            }
            "cmp" => {
                let params = parse_parameters(parameter_part, line.line_number, file_name)?;
                if params.len() == 2 {
                    parsed_instructions.push(UnlinkedInstruction {
                        line: line.line_number,
                        inst: instructions::InstructionEnum::cmp,
                        param1: Some(params[0].clone()),
                        param2: Some(params[1].clone()),
                    });
                } else {
                    return Err(format!("instruction 'cmp' needs 2 parameters but {} were parsed in line {} in file '{}' ", 
                    params.len(),line.line_number, file_name));
                }
            }
            "je" => {
                let params = parse_parameters(parameter_part, line.line_number, file_name)?;
                if params.len() == 1 {
                    parsed_instructions.push(UnlinkedInstruction {
                        line: line.line_number,
                        inst: instructions::InstructionEnum::je,
                        param1: Some(params[0].clone()),
                        param2: None,
                    });
                } else {
                    return Err(format!("instruction 'je' needs 1 parameter but {} were parsed in line {} in file '{}' ", 
                    params.len(),line.line_number, file_name));
                }
            }
            "jeg" => {
                let params = parse_parameters(parameter_part, line.line_number, file_name)?;
                if params.len() == 1 {
                    parsed_instructions.push(UnlinkedInstruction {
                        line: line.line_number,
                        inst: instructions::InstructionEnum::jeg,
                        param1: Some(params[0].clone()),
                        param2: None,
                    });
                } else {
                    return Err(format!("instruction 'jeg' needs 1 parameter but {} were parsed in line {} in file '{}' ", 
                    params.len(),line.line_number, file_name));
                }
            }
            "jel" => {
                let params = parse_parameters(parameter_part, line.line_number, file_name)?;
                if params.len() == 1 {
                    parsed_instructions.push(UnlinkedInstruction {
                        line: line.line_number,
                        inst: instructions::InstructionEnum::jel,
                        param1: Some(params[0].clone()),
                        param2: None,
                    });
                } else {
                    return Err(format!("instruction 'jel' needs 1 parameter but {} were parsed in line {} in file '{}' ", 
                    params.len(),line.line_number, file_name));
                }
            }
            "jg" => {
                let params = parse_parameters(parameter_part, line.line_number, file_name)?;
                if params.len() == 1 {
                    parsed_instructions.push(UnlinkedInstruction {
                        line: line.line_number,
                        inst: instructions::InstructionEnum::jg,
                        param1: Some(params[0].clone()),
                        param2: None,
                    });
                } else {
                    return Err(format!("instruction 'jg' needs 1 parameter but {} were parsed in line {} in file '{}' ", 
                    params.len(),line.line_number, file_name));
                }
            }
            "jl" => {
                let params = parse_parameters(parameter_part, line.line_number, file_name)?;
                if params.len() == 1 {
                    parsed_instructions.push(UnlinkedInstruction {
                        line: line.line_number,
                        inst: instructions::InstructionEnum::jl,
                        param1: Some(params[0].clone()),
                        param2: None,
                    });
                } else {
                    return Err(format!("instruction 'jl' needs 1 parameter but {} were parsed in line {} in file '{}' ", 
                    params.len(),line.line_number, file_name));
                }
            }
            "mov" => {
                let params = parse_parameters(parameter_part, line.line_number, file_name)?;
                if params.len() == 2 {
                    if params[0].is_not_constant() {
                        parsed_instructions.push(UnlinkedInstruction {
                            line: line.line_number,
                            inst: instructions::InstructionEnum::mov,
                            param1: Some(params[0].clone()),
                            param2: Some(params[1].clone()),
                        });
                    } else {
                        return Err(format!("instruction 'mov' has 2 parameters but the first can not be a constant line {} in file '{}' ", 
                    line.line_number, file_name));
                    }
                } else {
                    return Err(format!("instruction 'mov' needs 2 parameters but {} were parsed in line {} in file '{}' ", 
                    params.len(),line.line_number, file_name));
                }
            }
            "push" => {
                let params = parse_parameters(parameter_part, line.line_number, file_name)?;
                if params.len() == 1 {
                    if params[0].is_register() {
                        parsed_instructions.push(UnlinkedInstruction {
                            line: line.line_number,
                            inst: instructions::InstructionEnum::push,
                            param1: Some(params[0].clone()),
                            param2: None,
                        });
                    } else {
                        return Err(format!("instruction 'push' has 1 parameter but the first has to be a register line {} in file '{}' ", 
                line.line_number, file_name));
                    }
                } else {
                    return Err(format!("instruction 'push' needs 1 parameter but {} were parsed in line {} in file '{}' ", 
                    params.len(),line.line_number, file_name));
                }
            }
            "pop" => {
                let params = parse_parameters(parameter_part, line.line_number, file_name)?;
                if params.len() == 1 {
                    if params[0].is_register() {
                        parsed_instructions.push(UnlinkedInstruction {
                            line: line.line_number,
                            inst: instructions::InstructionEnum::pop,
                            param1: Some(params[0].clone()),
                            param2: None,
                        });
                    } else {
                        return Err(format!("instruction 'pop' has 1 parameter but the first has to be a register line {} in file '{}' ", 
                line.line_number, file_name));
                    }
                } else {
                    return Err(format!("instruction 'pop' needs 1 parameter but {} were parsed in line {} in file '{}' ", 
                    params.len(),line.line_number, file_name));
                }
            }
            "pusha" => {
                let params = parse_parameters(parameter_part, line.line_number, file_name)?;
                if params.len() == 0 {
                    parsed_instructions.push(UnlinkedInstruction {
                        line: line.line_number,
                        inst: instructions::InstructionEnum::pusha,
                        param1: None,
                        param2: None,
                    });
                } else {
                    return Err(format!("instruction 'pusha' needs 0 parameters but {} were parsed in line {} in file '{}' ", 
                    params.len(),line.line_number, file_name));
                }
            }
            "popa" => {
                let params = parse_parameters(parameter_part, line.line_number, file_name)?;
                if params.len() == 0 {
                    parsed_instructions.push(UnlinkedInstruction {
                        line: line.line_number,
                        inst: instructions::InstructionEnum::popa,
                        param1: None,
                        param2: None,
                    });
                } else {
                    return Err(format!("instruction 'popa' needs 0 parameters but {} were parsed in line {} in file '{}' ", 
                    params.len(),line.line_number, file_name));
                }
            }
            "call" => {
                let params = parse_parameters(parameter_part, line.line_number, file_name)?;
                if params.len() == 1 {
                    parsed_instructions.push(UnlinkedInstruction {
                        line: line.line_number,
                        inst: instructions::InstructionEnum::push,
                        param1: Some(params[0].clone()),
                        param2: None,
                    });
                } else {
                    return Err(format!("instruction 'call' needs 1 parameter but {} were parsed in line {} in file '{}' ", 
                    params.len(),line.line_number, file_name));
                }
            }
            "ret" => {
                let params = parse_parameters(parameter_part, line.line_number, file_name)?;
                if params.len() == 1 {
                    parsed_instructions.push(UnlinkedInstruction {
                        line: line.line_number,
                        inst: instructions::InstructionEnum::ret,
                        param1: Some(params[0].clone()),
                        param2: None,
                    });
                } else {
                    return Err(format!("instruction 'ret' needs 1 parameter but {} were parsed in line {} in file '{}' ", 
                    params.len(),line.line_number, file_name));
                }
            }
            "sys" => {
                let params = parse_parameters(parameter_part, line.line_number, file_name)?;
                if params.len() == 0 {
                    parsed_instructions.push(UnlinkedInstruction {
                        line: line.line_number,
                        inst: instructions::InstructionEnum::sys,
                        param1: None,
                        param2: None,
                    });
                } else {
                    return Err(format!("instruction 'sys' needs 0 parameters but {} were parsed in line {} in file '{}' ", 
                    params.len(),line.line_number, file_name));
                }
            }
            x => {
                return Err(format!(
                    "could not parse instruction '{}' in line {} in file '{}'",
                    x, line.line_number, file_name
                ))
            }
        }
        // add the size of the last added instruction
        // IMPORTANT: only do this if a instruction was parsed - not when a lebel was parsed
        current_offset += parsed_instructions[parsed_instructions.len() - 1].size() as usize;

        j_log(
            &format!(
                "decoded instruction: {:?}",
                parsed_instructions[parsed_instructions.len() - 1].inst
            ),
            3,
        );
    }

    return Ok(ParsedCodeSection {
        size: current_offset,
        parsed_instructions,
        label_offset,
    });
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
        if param.is_empty() {
            continue;
        }
        // check if value is a number
        if param.chars().all(|c| c.is_ascii_digit()) {
            let maybe_number = param.parse();
            if let Ok(number) = maybe_number {
                ret.push(UnlinkedParameter::Determined(
                    instructions::Param::Constant(number),
                ));
                j_log(&format!("decoded parameter: {:?}", ret[ret.len() - 1]), 3);
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
            j_log(&format!("decoded parameter: {:?}", ret[ret.len() - 1]), 3);
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
                j_log(&format!("decoded parameter: {:?}", ret[ret.len() - 1]), 3);
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
                j_log(&format!("decoded parameter: {:?}", ret[ret.len() - 1]), 3);
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
                            j_log(&format!("decoded parameter: {:?}", ret[ret.len() - 1]), 3);
                            continue;
                        } else {
                            return Err(format!(
                                "could not parse parameter '{}' in line {} in file '{}'",
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
                        j_log(&format!("decoded parameter: {:?}", ret[ret.len() - 1]), 3);
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
                    j_log(&format!("decoded parameter: {:?}", ret[ret.len() - 1]), 3);
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
                    j_log(&format!("decoded parameter: {:?}", ret[ret.len() - 1]), 3);
                    continue;
                } else {
                    return Err(format!(
                        "could not parse parameter '{}' in line {} in file '{}'",
                        param, line_number, file_name
                    ));
                }
            }
        }
    }

    Ok(ret)
}

fn parse_rom(rom: Vec<RawLine>, file_name: &str) -> Result<RomData, String> {
    let mut rom_data = RomData::default();
    // iterate over lines
    for line in rom {
        let line_number = line.line_number;
        let rom_line = line.content;

        // find the ':' for the definiton
        let doublepoint_char_idx = rom_line.find(':').ok_or(format!(
            "could not find definition of label in line {} in file '{}'\n",
            line_number, file_name
        ))?;
        let label_name = rom_line[..doublepoint_char_idx].trim();

        // +1 to get rid of the ':'
        let remaining = rom_line[doublepoint_char_idx + 1..].trim();

        // check for correct naming of the label
        if !label_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(format!(
                "could not parse the label name ->{}<- in line {} in file '{}'\n",
                label_name, line_number, file_name
            ));
        }

        // find the type of rom data
        // i -> integer
        // s -> string encoded as one char per u64 entry aka one char per adress
        // ps -> packed string -> string encoded as up to 8 chars per u64 aka 8 chars per adress
        // as -> string array -> an array of multiple strings sperated by null-termination of each element of the array, one char is encoded per u64
        // ai -> ineger array
    }
    return Ok(rom_data);
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
