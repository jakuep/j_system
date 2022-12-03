pub fn from_float_to_int_bitwise(number :u64) -> Option<f64>
{
    let ret = f64::from_bits(number);

    if ret.is_nan() || ret.is_infinite()
    {
        return None;
    }
    Some(ret)
}

pub fn from_int_to_float_bitwise(number :f64) -> u64
{
    number.to_bits()
}

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

pub fn u64_to_i64_bitwise(n: u64) -> i64 
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