#![feature(try_trait_v2)]
#![feature(never_type)]

pub mod fifth;
pub mod sixth;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
