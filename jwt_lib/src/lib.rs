// #[path = "./utils/mod.rs"]
mod utils;

#[macro_use]
extern crate lazy_static;

pub use utils::encryption;
pub use utils::jwt;



#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn it_works() {
    //     let result = add(2, 2);
    //     assert_eq!(result, 4);
    // }
}
