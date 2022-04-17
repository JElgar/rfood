#![feature(box_patterns)]

pub mod ast;
pub mod transform;
pub mod examples;
pub mod cli;
pub mod context;
pub mod utils;

#[cfg(test)]
pub mod tests {
    #[test]
    fn test_things() {
        assert_eq!(2 + 2, 4);
    }
}
