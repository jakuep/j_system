use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

pub fn load_file(file_name: &str) -> Vec<String>
{
    let s = get_programm_from_fs(file_name);
    
    let mut ret =  vec![];
    for line in s.lines()
    {
        //TODO: shouldnt this just stay as str? nah 
        ret.push(line.to_string());
    }
   
    ret
}

fn get_programm_from_fs(file_name: &str) -> String
{
    let path = Path::new(file_name);
    let display = path.display();

    let mut file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", display, why),
        Ok(file) => file,
    };

    let mut s = String::new();
    match file.read_to_string(&mut s){
        Err(why) => panic!("couldn't read {}: {}", display, why),
        Ok(_) => {},
    };
    
    s
}