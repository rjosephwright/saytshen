extern crate csv;
extern crate regex;
extern crate rustc_serialize;

#[macro_use]
extern crate serde_derive;

extern crate serde_yaml;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}

pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");

pub mod scan;
