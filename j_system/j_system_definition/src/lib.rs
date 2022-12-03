pub mod instructions;
pub mod register;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        let a = register::Register::a;
        assert_eq!(2 + 2, 4);
    }
}
