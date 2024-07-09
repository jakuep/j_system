extern crate j_system_definition;
mod assembler;
mod code_parse;
mod debug;
mod decode_instructons;
mod file_save_load;
mod label_resolve;
mod linker;
mod logging;
mod preprocessor;
mod serialization;
mod type_cov_parse;

use crate::assembler::*;
use crate::file_save_load::*;
use crate::preprocessor::*;

use std::fs;
use std::process;

fn main() {
    let main_file_name = "./in.asm";

    let file = load_file(main_file_name);

    let root = "./test/test1/test1.asm";

    let preprocess = preprocess(root);
    match &preprocess {
        Err(s) => print!("err: {}\n", s),
        Ok(output) => {
            for (file_name, content) in output {
                print!("file name: {}\n\n", file_name);
                print!("{:#?}\n", content);
                //print!("---------------------------\n\n");
            }
        }
    }
    print!("\n\n-----------------END PREPROCESS-----------------\n\n");
    if preprocess.is_err() {
        process::exit(0);
    }
    let preprocess = preprocess.unwrap();

    let mut assembled_files = vec![];
    for (file_name, file) in preprocess {
        match assembler::assemble_file(file, file_name) {
            Ok(assembled_file) => assembled_files.push(assembled_file),
            Err(e) => {
                print!("ERROR: {}\n", e);
                process::exit(-1);
            }
        }
    }

    // print assembler stage
    for file in assembled_files {
        print!("{:?}\n", file);
    }

    print!("\n\n-----------------END ASSEMBLER-----------------\n\n");

    //let fin = assemble_into_u64_vec(file, main_file_name.to_string());

    //let mut result = String::new();

    //for x in fin
    //{
    //    result.push_str(&format!("{:0>20}\n",x));
    //}

    //fs::write("./out.bin", &result).unwrap();
    print!("Ok\n");
}
