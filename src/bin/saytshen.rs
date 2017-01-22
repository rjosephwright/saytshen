extern crate clap;
extern crate saytshen;

use std::process::exit;

use clap::{App,Arg,SubCommand};

use saytshen::scan;

fn main() {
    let app = App::new("saytshen")
        .version(saytshen::VERSION)
        .author("Joseph Wright <rjosephwright@gmail.com>")
        .about("Scans systems for compliance")
        .subcommand(SubCommand::with_name("scan")
                    .arg(Arg::with_name("spec")
                         .short("s")
                         .long("spec")
                         .takes_value(true)
                         .help("Specification for scan"))
                    .arg(Arg::with_name("output")
                         .short("o")
                         .long("output")
                         .takes_value(true)
                         .default_value("report.csv")
                         .help("Output file for report")));

    let matches = app.get_matches();
    matches.subcommand_matches("scan")
        .and_then(|scan| {
            // unwrap() should be safe here because clap handles
            // missing arguments or provides default values.
            let spec = scan.value_of("spec").unwrap();
            let output = scan.value_of("output").unwrap();
            Some((spec, output))
        })
        .map(|(spec, output)| {
            match scan::run_scan(spec, output) {
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
