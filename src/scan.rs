use std::collections::HashMap;
use std::fs;
use std::io;
use std::path;
use std::process;

use csv;
use serde_yaml;

include!(concat!(env!("OUT_DIR"), "/serde_types.rs"));

#[derive(Debug, RustcEncodable)]
struct BenchmarkResult {
    section: String,
    description: String,
    run: String,
    passed: bool,
    output: Option<String>,
    error: Option<String>,
    skip: Option<String>,
}

#[derive(Debug)]
pub enum AuditError {
    CsvError(csv::Error),
    IoError(io::Error),
    YamlError(serde_yaml::Error),
    NonCompliant,
}

impl From<csv::Error> for AuditError {
    fn from(err: csv::Error) -> AuditError {
        AuditError::CsvError(err)
    }
}

fn open(path: &str) -> Result<fs::File, AuditError> {
    fs::File::open(path).map_err(AuditError::IoError)
}

fn parse(file: fs::File) -> Result<Vec<Benchmark>, AuditError> {
    serde_yaml::from_reader(file).map_err(AuditError::YamlError)
}

fn load_benchmarks(path: &str) -> Result<Vec<Benchmark>, AuditError> {
    open(path).and_then(parse)
}

fn run_script(script: &String) -> Result<process::Output, AuditError> {
    process::Command::new("/bin/sh")
        .arg("-c")
        .arg(script)
        .stdin(process::Stdio::null())
        .stdout(process::Stdio::piped())
        .stderr(process::Stdio::piped())
        .spawn().map_err(AuditError::IoError)
        .and_then(|child| child.wait_with_output().map_err(AuditError::IoError))
}

fn run_benchmark(step: &Benchmark) -> Result<Vec<BenchmarkResult>, AuditError> {
    let scripts = step.audit.iter().map(|a| &a.run);
    let mut outputs = HashMap::new();
    for script in scripts {
        let output = run_script(script)?;
        outputs.insert(script, output);
    }

    let mode = step.mode.unwrap_or(Mode::All);
    let should_skip = step.skip.is_some();
    let passed = should_skip || match mode {
        Mode::All => outputs.values().all(|o| o.status.success()),
        Mode::Any => outputs.values().any(|o| o.status.success()),
    };

    let results = outputs.iter().map(|(script, output)| {
        BenchmarkResult {
            section: step.section.clone(),
            description: step.description.clone(),
            run: script.to_string(),
            passed: passed,
            output: String::from_utf8(output.stdout.clone()).ok(),
            error: String::from_utf8(output.stderr.clone()).ok(),
            skip: step.skip.clone(),
        }
    }).collect::<Vec<_>>();

    Ok(results)
}

fn run_benchmarks(benchmarks: &Vec<Benchmark>) -> Result<Vec<BenchmarkResult>, AuditError> {
    let mut results = Vec::new();
    for benchmark in benchmarks {
        for result in run_benchmark(&benchmark)? {
            results.push(result);
        }
    }
    Ok(results)
}

fn write_report(results: &Vec<BenchmarkResult>) -> Result<(), AuditError> {
    let mut writer = csv::Writer::from_file(path::Path::new("results.csv"))?;
    let headers = vec![
        "Section", "Description", "Run", "Passed", "Output", "Error", "Skip"
    ];
    writer.write(headers.iter())?;
    for result in results {
        writer.encode(result)?;
    }
    Ok(())
}

pub fn run_scan(spec: &str) -> Result<(), AuditError> {
    load_benchmarks(spec)
        .and_then(|benchmarks| run_benchmarks(&benchmarks))
        .and_then(|results| write_report(&results).and(Ok(results)))
        .and_then(|results| {
            let passed = results.iter().all(|r| r.passed);
            match passed {
                true => Ok(()),
                false => Err(AuditError::NonCompliant)
            }
        })
}
