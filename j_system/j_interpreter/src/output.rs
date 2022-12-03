use j_system_definition::register::*;

pub fn dump_and_panic(panic_msg: String,reg: & RegisterState, stack: & Vec<u64>)
{
    println!("PANIC!");
    //print_register_state(reg);
    print_stack_state(stack);
    println!("----------------------------------------------------------");
    panic!("{}",panic_msg);
}

/* pub fn print_register_state(reg: & RegisterState)
{

    println!("\nRegister state:\n");
    println!("Register:\t{0: <21}\t{1: <18}","DEC:","HEX:");
    println!("----------------------------------------------------------");

    println!("Register a:\t{0: <21}\t{1: <19}",reg.a,format!("0x{:x}",reg.a));
    println!("Register b:\t{0: <21}\t{1: <19}",reg.b,format!("0x{:x}",reg.b));
    println!("Register c:\t{0: <21}\t{1: <19}",reg.c,format!("0x{:x}",reg.c));
    println!("Register d:\t{0: <21}\t{1: <19}",reg.d,format!("0x{:x}",reg.d));
    println!("Register e:\t{0: <21}\t{1: <19}",reg.e,format!("0x{:x}",reg.e));
    println!("Register f:\t{0: <21}\t{1: <19}",reg.f,format!("0x{:x}",reg.f,));
    println!("Register s:\t{0: <21}\t{1: <19}",reg.s,format!("0x{:x}",reg.s));
    println!("Register pc:\t{0: <21}\t{1: <19}",reg.pc,format!("0x{:x}",reg.pc));
    println!("Register tos:\t{0: <21}\t{1: <19}",reg.tos,format!("0x{:x}",reg.tos));
    println!("----------------------------------------------------------");
} */

pub fn print_stack_state(stack: &Vec<u64>)
{
    println!("\nStack state:\n");
    
    
    if stack.len() > 0
    {
        println!("{0: <21}\t{1: <21}\t{2: <19}","Index:","DEC:","HEX:");
        println!("------------------------------------------------------------------");

        for x in 0..stack.len()
        {
            println!("{0: <21}\t{1: <21}\t{2: <19}",x,stack[x],format!("0x{:x}",stack[x]));
        }
    }
    else
    {
        println!("STACK EMPTY");
    }
}