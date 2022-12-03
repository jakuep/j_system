#[derive(Clone,Copy)]
pub struct RegisterState{
    a: u64,
    b: u64,
    c: u64,
    d: u64,
    e: u64,
    f: u64,
    tos: u64, // top of stack
    bos: u64, // bottom of stack or current stack frame

    //read only
    pc: u64, // programm counter
    s: u64, // status register
}

#[allow(non_camel_case_types)]
#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone,Copy)]
pub enum Register
{
    a,
    b,
    c,
    d,
    e,
    f,
    tos,
    bos,

    /// bit 0 -> set when add overflows? TODO!!
    /// bit 1 -> set when cmp a,b & a<b
    /// bit 2 -> set when cmp a,b & a>b
    /// bit 3 -> set when cmp a,b & a==b
    //  TODO: maybe if a shift happens s could contain the value that gest shifted out ???
    s,
    pc,
}

impl RegisterState
{
    pub fn new() -> Self
    {
        RegisterState{a:0,b:0,c:0,d:0,e:0,f:0,s:0,pc:0,tos: 0,bos:0}
    }

    pub fn store(&mut self, reg: Register, val: u64)
    {
        match reg
        {
            Register::a     => self.a=val, 
            Register::b     => self.b=val, 
            Register::c     => self.c=val, 
            Register::d     => self.d=val, 
            Register::e     => self.e=val,
            Register::f     => self.f=val,
            Register::tos   => self.tos=val,
            Register::bos   => self.bos=val, 
            Register::s     => panic!("s is a read only register"), 
            Register::pc    => panic!("pc is a read only register"),
        }
    }

    pub fn store_to_read_only(&mut self, reg: Register, val: u64)
    {
        match reg
        {
            Register::pc    => self.pc=val,
            //Register::tos   => self.tos=val,
            //Register::bos   => self.bos= val,
            Register::s     => self.s=val,
            _               => panic!("this function can only store values in read only registers")
        }
    }

    pub fn read(& self, reg: Register) -> u64
    {
        match reg
        {
            Register::a     => self.a,
            Register::b     => self.b,
            Register::c     => self.c,
            Register::d     => self.d,
            Register::e     => self.e,
            Register::f     => self.f,
            Register::s     => self.s,
            Register::pc    => self.pc,
            Register::tos   => self.tos,
            Register::bos   => self.bos,
        }
    }

    pub fn change(&mut self, reg: Register, f: fn(u64) -> u64)
    {
        match reg
        {
            Register::a     => self.a = f(self.a),
            Register::b     => self.b = f(self.b),
            Register::c     => self.c = f(self.c),
            Register::d     => self.d = f(self.d),
            Register::e     => self.e = f(self.e),
            Register::f     => self.f = f(self.f),
            Register::s     => self.s = f(self.s),
            Register::pc    => self.pc = f(self.pc),
            Register::tos   => self.tos = f(self.tos),
            Register::bos   => self.bos = f(self.bos),
        }
    }

    pub fn change_tos(&mut self, change: i64)
    {
        let new_val = self.read(Register::tos) as i128 + change as i128;

        if new_val < 0 || new_val > u64::MAX as i128
        {
            panic!("cant update tos because of overflow/underflow");
        }

        self.tos = new_val as u64;
    }
}
