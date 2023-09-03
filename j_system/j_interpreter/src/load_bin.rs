use std::fs::{File, self};
use std::io::prelude::*;
use std::collections::HashMap;

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

pub fn load_symbols() -> Option<HashMap<u64,Vec<String>>>
{
    if let Ok(inp) = fs::read_to_string("labels.dbg")
    {
        let mut map:HashMap<u64, Vec<String>> = HashMap::new();
        
        for line in inp.trim().split('\n')
        {
            if let Some((addr,label_name)) =parse_parts(line)
            {
                // check if there is already a entry pointing to this address
                if let Some(handle) = map.get_mut(&addr)
                {
                    // if already present, add label name to address
                    handle.push(label_name);
                }
                else
                {
                    // add new vec if nothing yet points to this address
                    let _ = map.insert(addr, vec![label_name]);
                }
            }
            

        }

        Some(map)
    }
    else
    {
        None
    }
}

fn parse_parts(inp: &str) -> Option<(u64,String)>
{
    let parts:Vec<&str> = inp.trim().split('\t').collect();
    if parts.len() < 2
    {return None}
    let addr:u64 = parts[0].parse().ok()?;
    let label_name = parts[1];
    
    // check for valid label name
    if !label_name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
    {return None}

    return Some((addr,label_name.into()));

}