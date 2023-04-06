use j_system_definition::register::*;
use j_system_definition::instructions::*;
use regex::Regex;
use lazy_static::lazy_static;

use crate::type_cov_parse::*;
use crate::file_save_load::*;
use crate::serialization::*;

use std::collections::hash_map::HashMap;

#[derive(Clone,PartialEq,Debug)]
pub struct LabelPointer
{
    pub pos: u64,
    pub identifier: String,
    pub label_type: LabelType,
}

// Defines the type of data that the label points to
// TODO: do i need this? the Label translates to just an adress anyway
#[derive(Copy,Clone,PartialEq,Debug)]
pub enum LabelType
{
    JumpLabel,
    ArdressToRomData
}

pub struct OriginInformation
{
    /// contains the filename (and path?) to the file that defines this datapoint
    pub file: String,
    /// line of the definition of this datapoint
    pub line: u64,
}

pub struct AsmLineLabel
{
    pub info: OriginInformation,
    pub instruction: InstructionEnum,
    pub param1: ParamOrLabel,
    pub param2: ParamOrLabel,
}

#[derive(Copy,Clone,PartialEq,Debug)]
pub enum LabelUse
{
    Raw,
    Deref,
    DerefOffset(i64),
}

#[derive(Clone,PartialEq,Debug)]
pub enum ParamOrLabel
{
    /// contaits the label name and the way it is used
    Label(String,LabelUse),
    
    /// contains the name of the define the Label refers to
    DefineLabel(String),
    
    // a real parameter that can be encoded dirctly
    Param(Param),
    Nothing,
}

// some constants
const MAX_INCLUDE_ACTIONS:u64 = 100;

// Regex definitions
lazy_static!
{
    // define constans
    static ref RE_DEFINE_CONST:         Regex = Regex::new(r"^\s*#\s*define\s+([A-Za-z_][A-Za-z0-9_]*)\s+([0-9]+)\s*(?:\s+;.*)?$").unwrap();
    static ref RE_GET_DEFINE_CONST:     Regex = Regex::new(r"^\s*\$\s*([a-zA-Z_][0-9a-zA-Z_]*)\s*$").unwrap();
    static ref RE_GET_DEFINE_OFFSET:    Regex = Regex::new(r"").unwrap();
    static ref RE_GET_DEFINE_DEREF:     Regex = Regex::new(r"").unwrap();
    static ref RE_DEFINE_GET_LEN:       Regex = unimplemented!();

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

pub fn cleanup_input(code: &mut Vec<String>)
{
    //debug_print_vec_string(&code_lines);
    
    remove_comments(code);
    //print!("after remnoved comments:\n");
    //debug_print_vec_string(&code_lines);
    
    remove_empty_lines(code);
    
    //print!("\nafter removed empty lines:\n");
    //debug_print_vec_string(&code_lines);

    // generate label table for inserting correct jmp values later
    //let mut lable_table: Vec<LabelPointer> = vec![];
    
    //parse_lables_and_remove_labels(& mut lable_table, & mut code_lines);
    
    
    //print!("\nafter parsed and removed labels:\n");
    //debug_print_vec_string(&code_lines);
    //print!("label table:\n");
    //for x in &lable_table
    //{
    //    print!("{} -> {}\n",x.name,x.line);
    //}

    // give owenership of lable_table to fn 
    // beause its not needed anymore
    //set_jump_adresses(lable_table,& mut code_lines);
    //debug_print_vec_string(&code_lines);
}


pub fn parse_lables_and_remove_labels(text_list: & mut Vec<String>, label_list:& mut Vec<LabelPointer>)
{
    let mut ii = 0usize;

    while text_list.len() > ii
    {
        if RE_GET_CODE_LABEL.is_match(&text_list[ii])
        {
            let mut label_name = String::from(""); 
            for cap in RE_GET_CODE_LABEL.captures_iter(&text_list[ii])
            {
                label_name = String::from(&cap[1]);
                break;
            }

            for label_list_elem in label_list.into_iter()
            {
                if label_list_elem.identifier == label_name
                { 
                    panic!("found duplicate label name: {}", label_name);
                }
            }

            label_list.push(
                LabelPointer{
                    identifier: label_name,
                    label_type: LabelType::JumpLabel,
                    // pos just means that it points to the n-th istruction
                    // not to confuse with the pos of rom data where it points
                    // directly to the correct adress in memory
                    pos: ii as u64
                }
            );

            text_list.remove(ii);
        }
        else
        {
            ii+=1;
        }
    }
}

// fn set_jump_adresses(label_list:Vec<Label>, text_list: & mut Vec<String>)
// {
//     let re = Regex::new(r"^\s*jmp\s+\.([A-Za-z0-9]+)\s*$").unwrap();
    
//     for line in text_list
//     {
//         if re.is_match(&line)
//         {
//             let mut label_name = String::from(""); 
//             for cap in re.captures_iter(&line)
//             {
//                 label_name = String::from(&cap[1]);
//                 break;
//             }
            
//             let mut addr:i128 = -1; 
//             for label_list_elem in &label_list
//             {
//                 if label_name == label_list_elem.name
//                 {
//                     addr = label_list_elem.line as i128;
//                 }
//             }
//             if addr > -1 
//             {
//                 // ??
//                 line = & mut "jmp\t".to_string();
//                 line.push_str(&(addr as u64).to_string()); 
//             }
//             else
//             {
//                 panic!("could not find jmp lable: {}",label_name);
//             }
//         }
//     }
// }

fn remove_empty_lines(text_list: &mut Vec<String>)
{
    let re = Regex::new(r"^\s*$").unwrap();

    let mut ii = 0usize;
    while text_list.len() > ii
    {

        if re.is_match(&text_list[ii])
        {
            // remove empty line on match of regex
            text_list.remove(ii);
           

            // do NOT increment ii here because the index
            // has new element after delete
        }
        else
        {
            ii+=1;
        }
    }
}

fn remove_comments(text_list: & mut Vec<String>)
{
    let only_comment = Regex::new(r"^\s*;.*").unwrap();
    
    let mut ii = 0usize;
    while text_list.len() > ii
    {
        if only_comment.is_match(&text_list[ii])
        {
            // remove empty line on match of regex
            text_list.remove(ii);
            
            // do NOT increment ii here because the index
            // has new element after delete
        }
        else
        {
            ii+=1;
        }
    }

    let comment_after_code = Regex::new(r"([^;]+)\s+;.*").unwrap();
    let mut ii2 = 0usize;
    
    while text_list.len() > ii2
    {
        if comment_after_code.is_match(&text_list[ii2])
        {
            let mut code_line_no_comment = String::from("");
            for cap in comment_after_code.captures_iter(&text_list[ii2])
            {
                code_line_no_comment = String::from(&cap[1]);
                break;
            }
            text_list[ii2] = code_line_no_comment;
            ii2 +=1;
        }
        else
        {
            ii2+=1;
        }
    }
}

fn parse_line(line_code: String) -> AsmLineLabel
{
    let mut line_is_ok = false;

    let mut cap1 = String::from("");
    let mut parsed_param1 = ParamOrLabel::Nothing;
    let mut parsed_param2 = ParamOrLabel::Nothing; 

    for cap in RE_INSTRUCTION_CAPTURE.captures_iter(&line_code) {
        line_is_ok = true;
        
        cap1 = cap[1].to_string();

        // checking if there even are parameters
        if let Some(_) = cap.get(2)
        {
            parse_parameters(cap[2].to_string(), &mut parsed_param1, &mut parsed_param2);
        }
    }

    if !line_is_ok{ panic!("could not parse line: {}",line_code);}

    let parsed_instruction = match_instructtion(cap1);

    // TODO: include line number from the original input file for debug output
    AsmLineLabel
    {   
        info: OriginInformation { file: "".into(), line: 0 },
        instruction: parsed_instruction, 
        param1: parsed_param1, 
        param2: parsed_param2,
    }
} 

fn parse_parameters(snippet: String,p1: &mut ParamOrLabel,p2: &mut ParamOrLabel) 
{
    // RE_PARSE_PRAMS:
    // cap 1 -> complete param line with comma if parameters exit
    //              -> a,1
    // cap 2 -> first param
    // cap 3 -> second param

    let mut param1_str = String::from("");
    let mut param2_str = String::from("");    
    
    for cap in RE_PARSE_PRAMS.captures_iter(&snippet) {
       
        // check if there even are parameters
        if let Some(_) = cap.get(2)
        {
            // if there are parameters get the first one 
            param1_str= cap[2].to_string();

            // check if there is a second parameter
            if let Some(_) = cap.get(3)
            {
                param2_str = cap[3].to_string();
            }
        }
    }
    if &param1_str != "" 
    {
        *p1 = parse_one_parameter(param1_str);
        
        if &param2_str != "" 
        {
            *p2 = parse_one_parameter(param2_str);
        }
    }
    //(param1,param2)
}

fn parse_one_parameter(snippet: String) -> ParamOrLabel
{
    let mut p:ParamOrLabel = ParamOrLabel::Nothing;

    // check if the parameter is a register
    for cap in RE_REGISTER.captures_iter(&snippet) 
    { 
        if let Some(_) = cap.get(1)
        {
            p = ParamOrLabel::Param(Param::Register(match_register(cap[1].to_string())));
        }
    }

    // check if the parameter is a constant
    for cap in RE_CONSTANT.captures_iter(&snippet) 
    { 
        if let Some(_) = cap.get(1)
        {
            p = ParamOrLabel::Param(Param::Constant(parse_number_u64(cap[1].to_string()).unwrap())); //TODO: check for save parse
        }
    }

    for cap in RE_MEM_PTR_CONST.captures_iter(&snippet) 
    { 
        if let Some(_) = cap.get(1)
        {
            p = ParamOrLabel::Param(Param::MemPtr(parse_integer_u64(cap[1].to_string()).unwrap())); //TODO: check for save parse
        }
    }

    for cap in RE_MEM_PTR_REG_OFFSET.captures_iter(&snippet) 
    { 
        if let Some(_) = cap.get(1) 
        {
            let reg = match_register(cap[1].to_string());

            if let Some(_) = cap.get(2)
            {
                if let Some(_) = cap.get(3)
                {
                    let val = crate::type_cov_parse::parse_integer_u64(cap[3].to_string()).unwrap();
                    if cap[2].to_string() == "+"
                    {
                        p = ParamOrLabel::Param(Param::MemPtrOffset(reg,val as i64));
                    }
                    else if cap[2].to_string() == "-"
                    {
                        p = ParamOrLabel::Param(Param::MemPtrOffset(reg,-(val as i64)));
                    }
                }
            }
            else
            {
                //TODO: should be ok, right?
                p = ParamOrLabel::Param(Param::MemPtrOffset(match_register(cap[1].to_string()),0));
            }
        }
    }

    // check if parameter contains a label
    if RE_CONTAINS_LABEL.is_match(&snippet)
    {
        for cap in RE_LABEL_AS_POINTER.captures_iter(&snippet)
        {
            p = ParamOrLabel::Label(cap[1].to_string(),LabelUse::Raw);
        }

        for cap in RE_LABEL_DEREF.captures_iter(&snippet)
        {
            p = ParamOrLabel::Label(cap[1].to_string(),LabelUse::Deref);
        }

        for cap in RE_LABEL_DEREF_OFFSET.captures_iter(&snippet)
        {
            if cap[2].to_string() == "-"
            {
                p = ParamOrLabel::Label(cap[1].to_string(),LabelUse::DerefOffset(-(cap[3].parse::<i64>().unwrap())));
            }
            else
            {
                p = ParamOrLabel::Label(cap[1].to_string(),LabelUse::DerefOffset(cap[3].parse::<i64>().unwrap()));
            }
        }
    }

    if RE_GET_DEFINE_CONST.is_match(&snippet)
    {
        for cap in RE_GET_DEFINE_CONST.captures_iter(&snippet)
        {
            p = ParamOrLabel::DefineLabel(cap[1].into())
        }
    }

    // We can just panic here since this method shouldt get called
    // if there were no input that could be a parameter
    if p == ParamOrLabel::Nothing
    {
        panic!("could not match parameter for: {}",snippet);
    }
    p
} 

fn match_register(snippet: String) -> Register
{
    match &snippet[..]
    {
        "a"     => Register::a,
        "b"     => Register::b,
        "c"     => Register::c,
        "d"     => Register::d,
        "e"     => Register::e,
        "f"     => Register::f,
        "s"     => Register::s,
        "pc"    => Register::pc,
        "tos"   => Register::tos,
        "bos"   => Register::bos,
        _       => panic!("could not match register")
    }
}

fn match_instructtion(snippet: String) -> InstructionEnum
{
    match &snippet[..]
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
        _       => panic!("could not mach instruction")
    }
}

pub fn parse_code(mut text: Vec<String>, lable_table: & mut Vec<LabelPointer>) -> Vec<AsmLineLabel>
{
    parse_lables_and_remove_labels(&mut text, lable_table);
    
    let mut parsed_code = vec![];
    for line in text
    {
        parsed_code.push(parse_line(line));
    }

    parsed_code
}

pub fn parse_rom(mut r: Vec<String>) -> (Vec<u64>, Vec<LabelPointer>)
{
    let mut label_list:Vec<LabelPointer> = vec![];
    
    // adding the null value that shouldnt be dereferenced
    let mut rom_mem = vec![0];

    while r.len() >0 
    {
        for cap in RE_ROM_GET_ELEMENT.captures_iter(&r[0])
        {
            if cap.get(1).is_some() && cap.get(2).is_some() && cap.get(3).is_some()
            {
                let label_name = cap[1].to_string();
                
                // dertermine the type of the 
                let rom_data_type = match &cap[2]
                {
                    "i"     => RomDataType::Integer,
                    "s"     => RomDataType::String,
                    "ai"    => RomDataType::IntegerArray,
                    "as"    => RomDataType::StringArray,
                    _       => panic!("could to determine data type of: {}", r[0])
                };
                let label_content = cap[3].to_string();

                let mut sersed = serialize_rom(label_content,rom_data_type);
                
                for x in &label_list
                {
                    if x.identifier == label_name
                    {
                        panic!("duplicate rom data name: {}", label_name);
                    } 
                }
  
                // push on the label list BEFORE adding the sersed data to the rom
                // to set the pointer correctly
                label_list.push(
                    LabelPointer{
                        identifier: label_name,
                        label_type: LabelType::ArdressToRomData,
                        pos:rom_mem.len() as u64
                    });
                
                rom_mem.append(&mut sersed);
            }
            else
            {
                panic!("cant parse rom line: {}",r[0]);
            }
        }
        r.remove(0);
    }
    
    (rom_mem,label_list)
}

pub fn split_sections(mut input: Vec<String>) -> (Vec<String>, Vec<String>)
{
    // TODO: does one really need to define a _rom sections if it is not needed? 
    
    // the first after cleanup line should either be "_code" or "_rom"

    if RE_CODE_SECTION_START.is_match(&input[0])
    {
        // search for the end of code section aka start of rom section
        for ii in 1..input.len()
        {
            if RE_ROM_SECTION_START.is_match(&input[ii])
            {
                // remove ".code" and ".rom" from vec
                input.remove(ii);
                input.remove(0);

                let rom_section = input.split_off(ii-1);
                return (input,rom_section)
            }
        }
    }

    if RE_ROM_SECTION_START.is_match(&input[0])
    {
        // search for the end of rom section aka start of code section
        for ii in 1..input.len()
        {
            if RE_CODE_SECTION_START.is_match(&input[ii])
            {
                // remove ".code" and ".rom" from vec
                input.remove(ii);
                input.remove(0);
                
                let code_section = input.split_off(ii-1);
                return (code_section,input)
            }
        }
    }

    panic!("PARSE ERROR: Could not split sections! Did you define a '_rom' and '_code 'section? Did you define anything outside of these sections?");
}

fn get_includes(input:&mut Vec<String>, already_included: &mut Vec<String>, to_be_included: &mut Vec<String>) 
{
    cleanup_input(input);
    let mut ii = 0;

    // use loop because the size of input might shrink
    loop 
    {
        if input.len() <= ii 
        {
            break;
        }

        if RE_INCLUDE.is_match(&input[ii])
        {
            for cap in RE_INCLUDE.captures_iter(&input[ii])
            {
                let newly_found_include = cap[1].to_string();
                
                // push to include list if it isnt already in it
                if !already_included.contains(&newly_found_include)
                {
                    to_be_included.push(newly_found_include);
                }

                // only take the first match
                break;
            }
            // remove the include statement
            input.remove(ii);
        }
        ii+=1;
    }
}

pub fn get_defines(input: &mut Vec<String>, defines: &mut HashMap<String,u64>)
{
    //let mut defines:HashMap<String,u64> = HashMap::new();
    let mut ii = 0;
    
    loop 
    {
        if input.len() <= ii
        {
            break
        }
        let current_line = input[ii].clone();

        if RE_DEFINE_CONST.is_match(&current_line)
        {
            for elem in RE_DEFINE_CONST.captures_iter(&current_line)
            {
                let def_name    = elem[1].to_string();
                //panic!("{}",elem[2].to_string());
                let value       = (&elem[2]).parse::<u64>().unwrap();

                // check if key is already defined
                if defines.contains_key(&def_name)
                {
                    panic!("double definition of key: {} as define", def_name);
                }
                defines.insert(def_name, value);
            }
            input.remove(ii);
        }
        ii+=1;
    }
}

pub struct PreprocessedInput
{
    pub rom: Vec<String>,
    pub code: Vec<String>,
    pub defines: HashMap<String,u64>,
}

pub fn preprocess_input(mut input: Vec<String>, main_file_name: String) -> PreprocessedInput
{
    // add the first file name to the already inluded vec
    let mut already_inluded = vec![main_file_name];
    let mut to_be_included = vec![];
    let mut count_include_actions:u64 = 0;

    // hashmap with all the #defines
    let mut defines:HashMap<String,u64> = HashMap::new();

    // first iteration outside of the loop to fill to_be_included
    cleanup_input(&mut input);
    get_includes(&mut input, &mut already_inluded, &mut to_be_included);
    get_defines(&mut input, &mut defines);
    let (mut code,mut rom) = split_sections(input);

    while to_be_included.len() !=0 && count_include_actions <= MAX_INCLUDE_ACTIONS
    {
        // get the name of the next include 
        // we can just unwrap it becuase the while condition would catch 
        // the case in that to_be_included would be empty 
        let next_include = to_be_included.pop().unwrap();
        
        // laod new file and push it to vec off already included files
        let mut new_file = load_file(&next_include);
        already_inluded.push(next_include);
        
        // extract includes from NEW file and remove the include statements
        // both is handeld by get_includes(..)
        get_includes(&mut new_file, &mut already_inluded, &mut to_be_included);

        // get the new defines
        get_defines(&mut new_file, &mut defines);

        // merge sections of both old and new file
        cleanup_input(&mut new_file);
        let (mut new_code,mut new_rom) = split_sections(new_file);

        code.append(&mut new_code);
        rom.append(&mut new_rom);

        count_include_actions +=1;
    }

    PreprocessedInput{rom, code, defines}
}
