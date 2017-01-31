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

pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");

pub mod scan;
