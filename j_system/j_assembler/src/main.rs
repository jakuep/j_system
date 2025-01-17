extern crate j_system_definition;
mod decode_instructons;
mod label_resolve;
mod serialization;
mod type_cov_parse;
mod assembler;
mod file_save_load;
mod debug;
mod preprocessor;

use crate::file_save_load::*;
use crate::assembler::*;
use crate::preprocessor::*;



use std::fs;

fn main() {
    let main_file_name = "./in.asm";
    
    let file = load_file(main_file_name);
    
    let fin = assemble_into_u64_vec(file, main_file_name.to_string());
    
    let mut result = String::new(); 

    for x in fin
    {
        result.push_str(&format!("{:0>20}\n",x));
    }
    
    fs::write("./out.bin", &result).unwrap();
    print!("Ok\n");

   
}
