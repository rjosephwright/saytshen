extern crate csv;
extern crate regex;
extern crate rustc_serialize;
extern crate serde;
extern crate serde_yaml;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}

pub const VERSION: &'static str = "0.1.0";

pub mod scan;
