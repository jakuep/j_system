mod check_instruction;
mod deserialization;
mod exec;
mod memory;
mod syscall;
mod type_cov_parse;
mod output;
mod load_bin;
mod debug;
mod machine;

use std::collections::HashSet;

use crate::load_bin::Binary;
use crate::machine::{MachineState,MachineInitInfo};

use clap::Parser;


/// Jan-Interpreter
#[derive(Parser, Debug)]
#[clap(author, about, long_about = None)]
struct Args {
    /// run in debug mode
    #[clap(short, long, action)]
    debug: bool,

    /// wirite the machine output to a file
    #[clap(short='f',long="out", action)]
    output_to_file: bool,

    /// Name of the input binary
    #[clap(short, long,  value_parser)]
    input_file: Option<String>,

    /// Name of the additional debug information file
    #[clap(long, value_parser)]
    debug_information: Option<String>,

    /// size of the memory
    #[clap(short, long, value_parser, default_value_t = 1024)]
    mem_size: u64,

    /// maximum cycles the machine is allowed to execute
    #[clap(short, long,  value_parser, default_value_t = 10_000_000_000)]
    cycle_limit: u128,

    /// maximum cycles the machine is allowed to execute
    #[clap(short, long, action)]
    verbose: bool,
}



fn main() {
    let args = Args::parse();

    // add breakpoints form file
    let mut breakpoints = HashSet::new(); 
    
    // TODO: remove
    
    breakpoints.insert(1);
    //breakpoints.insert(2);
    //breakpoints.insert(3);
    //breakpoints.insert(5);
    //breakpoints.insert(6);
    //breakpoints.insert(7);
    //breakpoints.insert(8);
    //breakpoints.insert(9);
    //breakpoints.insert(10);

    let init = MachineInitInfo{
            max_cycles: args.cycle_limit,
            mem_size: args.mem_size,
            debug_mode: if args.debug {Some(breakpoints)} else {None},
            write_to_file: args.output_to_file
        };
    
    let mut b = Binary::new();
    b.load_file("in.bin".to_string());

    // init machine in debug mode by providing Some(breakpoints)
    let mut machine = MachineState::init(init);
    machine.laod_into_state(b);

    machine.run_program();

    //println!("Hello, world!");
}

