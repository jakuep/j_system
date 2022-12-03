use std::fs::File;
use std::io::prelude::*;

pub struct Binary
{
    pub code:           Vec<u64>,
    pub rom:            Vec<u64>,
    pub start_ptr:      u64
}

impl Binary{

    pub fn new() -> Self
    {
        Self{
            code:vec![],
            rom:vec![],
            start_ptr:0}
    }

    pub fn load_file(&mut self,file_name: String)
    {
        let mut file = File::open(&file_name).unwrap();
        let mut s = String::new();
        file.read_to_string(&mut s).unwrap();

        let mut bin: Vec<u64> = vec![];

        for line in s.lines()
        {
            bin.push(line.parse::<u64>().unwrap())
        }

        // move the whole binary into the sections and set
        // the start pointer
        self.start_ptr = bin.pop().unwrap();

        // get the split point
        let split_point = bin.pop().unwrap();

        // split vec into the sections
        let (rom,code) = bin.split_at(split_point as usize);

        // set sections
        self.rom = rom.to_vec();
        self.code = code.to_vec();
    }

}