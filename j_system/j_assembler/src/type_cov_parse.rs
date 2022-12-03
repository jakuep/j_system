pub fn parse_number_u64(snippet: String) -> Option<u64>
{
    
    // TODO:    match decimal, hex and flaot 
    //          and parse it into u64

    match snippet.parse::<u64>()
    {
        Ok(x) => Some(x),
        Err(_) => None
    }

}

pub fn parse_integer_u64(snippet: String) -> Option<u64>
{
    
    // TODO:    match decimal, hex
    //          and parse it into u64

    match snippet.parse::<u64>()
    {
        Ok(x) => Some(x),
        Err(_) => None
    }
}

pub fn _u64_to_i64_bitwise(n: u64) -> i64 
{
    let sign:u64 = n & 0x8000_0000_0000_0000;
    let ret:u64 = n & 0x7fff_ffff_ffff_ffff;
    
    // TODO: check bit representaion
    if sign > 0
    {
        ret as i64
    }
    else
    {
        -(ret as i64)
    }
}

/// used to convert the signed offset into the internal u64 representation
pub fn i64_to_u64_bitwise(n: i64) -> u64 
{
    let mut ret = n.abs() as u64 & 0x7fff_ffff_ffff_ffff;
    
    // TODO: check bit representaion
    if n.is_negative()
    {
        ret 
    }
    else
    {
        ret += 0x8000_0000_0000_0000;
        ret
    }
}