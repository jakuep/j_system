use crate::register::*;

#[derive(PartialEq, Debug, Clone)]
pub struct AsmLine {
    pub line: u64,
    pub instruction: InstructionEnum,
    pub param1: Option<Param>,
    pub param2: Option<Param>,
}

impl AsmLine {
    pub fn size(&self) -> u8 {
        // start with instruction size
        let mut size = 1;

        // add the size of addtional parameters that need 64bit each
        if let Some(param1) = self.param1 {
            size += param1.size();
        }
        if let Some(param2) = self.param2 {
            size += param2.size();
        }

        size
    }

    pub fn as_string(&self) -> String {
        //TODO: add original line
        //      since its 0 atm

        let mut s = String::new();
        //s.push_str(line)!!!!
        s.push_str(&(ins_as_string(self.instruction) + "\t"));

        if let Some(param1_str) = param_as_string(self.get_param1()) {
            s.push_str(" ");
            s.push_str(&param1_str);
        }

        if let Some(param2_str) = param_as_string(self.get_param2()) {
            s.push_str(", ");
            s.push_str(&param2_str);
        }
        s
    }

    pub fn get_param1(&self) -> Option<Param> {
        self.param1.clone()
        // // TODO: cleanup
        // match &self.param1
        // {
        //     Some(x) =>
        //     {
        //         match x
        //         {
        //             Param::Register(y) => Some(Param::Register(*y)),
        //             Param::Constant(y) => Some(Param::Constant(*y)),
        //             Param::MemPtr(y) => Some(Param::MemPtr(*y)),
        //             Param::MemPtrOffset(y,z) => Some(Param::MemPtrOffset(*y,*z)),
        //         }
        //     },
        //     None => None
        // }
    }

    pub fn get_param2(&self) -> Option<Param> {
        self.param2.clone()
        // match &self.param2
        // {
        //     Some(x) =>
        //     {
        //         match x
        //         {
        //             Param::Register(y) => Some(Param::Register(*y)),
        //             Param::Constant(y) => Some(Param::Constant(*y)),
        //             Param::MemPtr(y) => Some(Param::MemPtr(*y)),
        //             Param::MemPtrOffset(y,z) => Some(Param::MemPtrOffset(*y,*z)),
        //         }
        //     },

        //     None => None
        // }
    }
}

fn param_as_string(p: Option<Param>) -> Option<String> {
    let val = p?;

    let s = match val {
        Param::Register(reg) => format!("{}", reg_as_string(reg)),
        Param::Constant(c) => format!("{}", c),
        Param::MemPtr(ptr) => format!("{}", ptr),
        Param::MemPtrOffset(reg, off) => format!(
            "[{}{}{}]",
            reg_as_string(reg),
            if off < 0 { "" } else { "+" },
            off
        ),
    };

    Some(s.into())
}

fn ins_as_string(ins: InstructionEnum) -> String {
    match ins {
        InstructionEnum::add => "add",
        InstructionEnum::sub => "sub",
        InstructionEnum::xor => "xor",
        InstructionEnum::or => "or",
        InstructionEnum::and => "and",
        InstructionEnum::shr => "shr",
        InstructionEnum::shl => "shl",
        InstructionEnum::jmp => "jmp",
        InstructionEnum::cmp => "cmp",
        InstructionEnum::je => "je",
        InstructionEnum::jeg => "jeg",
        InstructionEnum::jel => "jel",
        InstructionEnum::jg => "jg",
        InstructionEnum::jl => "jl",
        InstructionEnum::mov => "mov",
        InstructionEnum::push => "push",
        InstructionEnum::pop => "pop",
        InstructionEnum::pusha => "pusha",
        InstructionEnum::popa => "popa",
        InstructionEnum::call => "call",
        InstructionEnum::ret => "ret",
        InstructionEnum::sys => "sys",
    }
    .to_string()
}

fn reg_as_string(reg: Register) -> String {
    match reg {
        Register::a => "a",
        Register::b => "b",
        Register::c => "c",
        Register::d => "d",
        Register::e => "e",
        Register::f => "f",
        Register::s => "s",
        Register::pc => "pc",
        Register::tos => "tos",
        Register::bos => "bos",
    }
    .to_string()
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Param {
    Register(Register),
    MemPtr(u64),
    MemPtrOffset(Register, i64), // maybe i128?
    Constant(u64),
}

impl Param {
    pub fn size(&self) -> u8 {
        match self {
            Param::Register(_) => 0,
            Param::MemPtr(_) => 1,
            Param::MemPtrOffset(_, _) => 1,
            Param::Constant(_) => 1,
        }
    }
}

pub enum ParamType {
    Register,
    MemPtr,
    MemPtrOffset,
    Constant,
}

#[derive(Copy, Clone, PartialEq, Debug)]
#[allow(non_camel_case_types)]
pub enum InstructionEnum {
    /// ## Addition
    /// ---  
    /// **Description**: performs the addition of the value of \<A\> and \<B\>.
    /// The result will be written to \<A\>.  
    /// TODO: If the result overflows the register, the carry/Overflow bit in the status-register will be set.  
    /// **Usage**: add A,B  
    /// > A: Register  
    /// > B: Any
    add,

    /// ## Subtraction
    /// ---  
    /// **Description**: performs the subtraction of the value of \<A\> by the value \<B\>.
    /// The result will be written to \<A\>.  
    /// If B is greater than A, A will be set to 0.    
    /// **Usage**: sub A,B  
    /// > A: Register  
    /// > B: Any
    sub, // (sub a,b) a - b = (erg) -> a , if b > a -> a - b = 0
    xor, // (xor a,b) a xor b -> a ?
    or,  // (or a,b) a or b -> a ?
    and, // (and a,b) a and b -> a ?

    /// ## Shift Right
    /// ---  
    /// **Description**: shift a Value in Register \<A\> by the value of \<B\> to the right  
    /// **Usage**: shr A,B  
    /// > A: Register  
    /// > B: Any (Value Range 0-64)
    shr,

    /// ## Shift Left
    /// ---  
    /// **Description**: shift a Value in Register \<A\> by the value of \<B\> to the left  
    /// **Usage**: shl A,B  
    /// > A: Register  
    /// > B: Any (Value Range 0-64)
    shl,

    /// (jmp A) jump to line in arg
    jmp,

    /// (cmp X,Y) compare value of a and b.  
    /// Set S(Status-Register) to Result  
    /// X<Y     -> s = 1<< 1  
    /// X>Y     -> s = 1<< 2  
    /// X==Y    -> s = 1<< 3  
    cmp,
    je,  // Jump equal ??
    jeg, // jump equal or greater
    jel, // jump equal or less
    jg,  // jump greater
    jl,  // jump less

    mov,  // a <- b  .. copy b into a
    push, // push arg on stack
    pop,  // get tos and put it in arg

    pusha,
    popa,

    call, // call arg function

    /// ## Return
    /// ---  
    /// **Description**: Return to the caller function by jumping to the adress that is on top of the stack.
    /// The value in \<A\>
    /// **Usage**: ret A
    /// > A: Constant?  
    ///  ### Example
    /// ...  
    /// 0x9  
    /// 0x8 (parameter to called function) <- gets removed  
    /// 0x7 (parameter to called function) <- gets removed  
    /// 0x6 (return adress)  
    /// 0x5 (value in called function)  
    /// 0x4 (value in called function)  
    /// 0x3 (value in called function)  
    /// ...
    ret, // return form current function back to the caller

    //out, // print register to the console
    sys, // syscall
         //end, // end the programm
}

// satus register bit set

/*
    0000000..00000001
    |               |
    first bit (63)  last bit (0)

    0. is set when overflow occurs
    1. is set when compare returns equal -> cmp 1,1



*/
