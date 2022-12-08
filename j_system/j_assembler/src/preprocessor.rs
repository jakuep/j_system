use crate::decode_instructons::OriginInformation;

use regex::Regex;
use lazy_static::lazy_static;

use std::collections::{HashSet,HashMap};
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

lazy_static!
{
    // detect include statements
    static ref RE_INCLUDE:  Regex = Regex::new(r"^\s*#\s*include\s+((?:[A-Za-z0-9_])+\.asm)\s*;?.*$").unwrap();
    
    // exported labels that should be visable in other files
    static ref RE_EXPORT:   Regex = Regex::new(r"^\s*#\s*export\s+(.*)\s*;?.*$").unwrap();            
}
#[derive(Debug)]
pub struct RawLine
{
    //info: OriginInformation,
    line: u64,
    content: String,
}

#[derive(Debug,Eq,PartialEq,Clone,Hash)]
pub struct Export
{
    teip: ExportType,
    name: String,
}
#[derive(Debug,Eq,PartialEq,Clone,Hash)]
pub enum ExportType
{
    Label,
    Define,
}

#[derive(Debug)]
pub struct SourceFile
{
    pub content: Vec<RawLine>,
    
    /// can both reference jump labels and rom data.
    /// contains the labels that where exported
    pub exports: HashSet<Export>,

    /// Visable Labels in self.content
    /// Key: name of the export and their type
    /// Value: File that holds this label
    pub visable_exports: HashMap<Export,String>
}

pub fn get_file_includes(already_included: &mut HashMap<String, SourceFile>, current_file: &str, path: &str) -> Result<(),String>
{
    // check if this file is already inlcuded
    if already_included.contains_key(current_file)
    {
        return Ok(());
    }
    let mut lines = into_lines(open_file(&current_file, path)?);
    let includes = resolve_includes(&mut lines)?;
    let exports = get_exports(&mut lines)?;

    // add self to set of already includes files.
    // !!! This must happen before iterating over the rest of the includes to prevent double inclusion.
    // the labels that are visable in this file will be addes later
    already_included.insert(current_file.into(), 
            SourceFile{content: lines, exports: exports, visable_exports: HashMap::new()});

    let mut vis_labels = HashMap::new();
    // perform all includes actions for the files that included by this file
    for inlc in includes
    {
        // prevent that exported labels by self appear in vis_labels of self
        if inlc == current_file
        {continue}
        get_file_includes(already_included, &inlc, &path)?;
        // get the exports that were just added to the list of inlcuded files.
        // since the file just got included or was included before,
        // we can just unwrap
        let exports = already_included.get(&inlc).unwrap().exports.clone();
        for label in exports
        {  
            // if the label is already defined the Hashmap returns the filename(path?)
            // of the other include
            if let Some(other_file) = vis_labels.insert(label.clone(), inlc.clone())
            {
                return Err(format!("\ndouble include of label {:#?} in {}. Label is exported in {} and {}\n",
                    label, current_file, other_file, inlc))
            };
        }
        // TODO: path in file is relative to root path
    } 
    // insert visable labels of current file
    already_included.get_mut(current_file).unwrap().visable_exports = vis_labels;
    Ok(())
}

fn get_exports(lines: &mut Vec<RawLine>) -> Result<HashSet<Export>,String>
{
    let mut exports = HashSet::new();
    let mut ii = 0;
    // len changes in loop, because the export statements get removed
    while ii < lines.len()
    {
        if let Some(matches) = RE_EXPORT.captures(&lines[ii].content)
        {
            // Export list, should look like this: "label1,label2, label3,$def1"
            let raw_matches = matches.get(1).unwrap().as_str().trim();
            for exp in raw_matches.split(',')
            {
                let mut exp = exp.trim();

                // decide if it is a definition or a label
                let exp_type = if exp.len()>1 && exp.chars().next().unwrap() == '$'
                {
                    // remove the '$'
                    exp = &exp[1..];
                    ExportType::Define
                }
                else
                {
                    ExportType::Label
                };

                // check valid label name. Skip the first char since it could be a '$',
                // which indicates that it is a definition
                if !exp.chars().all(|char| char.is_ascii_alphanumeric() || char == '_')
                {
                    return Err(format!("label name '{}' is not valid", exp))
                }

                exports.insert(Export{ teip: exp_type, name: exp.into()});
            }
            // remove the line form provided vec
            lines.remove(ii);
        }
        else 
        {
            ii +=1;    
        }
    }
    Ok(exports)
}

fn resolve_includes(lines: &mut Vec<RawLine>) -> Result<Vec<String>,String>
{
    let mut includes = vec![];
    let mut ii = 0;
    // len changes in loop, because the export statements get removed
    while ii < lines.len()
    {
        if let Some(matches) = RE_INCLUDE.captures(&lines[ii].content)
        {
            let file_name = matches.get(1).unwrap().as_str().trim();
            // no path due to '/' being forbidden
            if file_name.contains(['<','>',':','"','/','\\','|','?','*'])
            {
                return Err(format!("Invalid filename: {}",file_name))
            }
            includes.push(file_name.into());
            // remove the line form provided vec
            lines.remove(ii);
        }
        else
        {
            ii +=1;
        }
    }
    Ok(includes)
}

fn open_file(file_path: &str, path: &str) -> Result<String,String>
{
    let mut ret = String::new();
    let p:String = path.to_string() + file_path;
    File::open(&p).expect("").read_to_string(&mut ret).expect("");
    Ok(ret)
}

fn into_lines(content: String) -> Vec<RawLine>
{
    let raw_lines:Vec<_> = content.split('\n').collect();
    let mut lines = vec![];

    for ii in 0..raw_lines.len()
    {
        lines.push(RawLine{line:(ii+1) as u64, content:raw_lines[ii].into()})
    }

    lines
}

#[cfg(test)]
mod tests
{   
    use super::*;
    
    #[test]
    fn test1() 
    {
        let mut files = HashMap::new();
        if let Err(e) = get_file_includes(&mut files, "test1.asm".into(), "./test/test1/".into())
        {
            assert!(false)
        }
        let abc_exp = ["a"];
        let abc_vis = ["b", "c"];

        let test1_exp = ["b","c"];
        let test1_vis = ["a"];
    }

}