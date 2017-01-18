extern crate clap;
extern crate saytshen;

use std::process::exit;

use clap::{App,Arg,SubCommand};

use saytshen::scan;

fn main() {
    let app = App::new("saytshen")
        .version("1.0")
        .author("Joseph Wright <rjosephwright@gmail.com>")
        .about("Scans systems for compliance")
        .subcommand(SubCommand::with_name("scan")
                    .arg(Arg::with_name("spec")
                         .short("s")
                         .long("spec")
                         .takes_value(true)
                         .help("Specification for scan")));

    let matches = app.get_matches();
    matches.subcommand_matches("scan")
        .and_then(|scan| scan.value_of("spec"))
        .map(|spec| {
            match scan::run_scan(spec) {
                Err(scan::AuditError::NonCompliant) => {
                    exit(1)
                },
                Err(e) => {
                    println!("{:?}", e);
                    exit(-1)
                },
                Ok(_) => exit(0)
            }
        });
}
