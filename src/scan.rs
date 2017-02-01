use std::fs;
use std::io;
use std::path;
use std::process;
use std::string;

use csv;
use regex;
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
    RegexError(regex::Error),
    StringError(string::FromUtf8Error),
    YamlError(serde_yaml::Error),
    NonCompliant,
}

impl From<csv::Error> for AuditError {
    fn from(err: csv::Error) -> AuditError {
        AuditError::CsvError(err)
    }
}

impl From<regex::Error> for AuditError {
    fn from(err: regex::Error) -> AuditError {
        AuditError::RegexError(err)
    }
}

impl From<string::FromUtf8Error> for AuditError {
    fn from(err: string::FromUtf8Error) -> AuditError {
        AuditError::StringError(err)
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

fn is_success(output: &process::Output, expect: &Option<String>)
              -> Result<bool, AuditError> {
    let stdout_matches = match expect {
        &Some(ref pattern) => {
            let re = regex::Regex::new(&pattern)?;
            let stdout = String::from_utf8(output.stdout.clone())?;
            let is_match = re.is_match(&stdout.to_string());
            is_match
        },
        &None => true
    };
    Ok(stdout_matches && output.status.success())
}

fn run_benchmark(step: &Benchmark) -> Result<Vec<BenchmarkResult>, AuditError> {
    let mut outputs = Vec::new();
    for audit in step.audit.iter() {
        let run = audit.run.clone();
        let output = run_script(&run)?;
        let success = is_success(&output, &audit.expect)?;
        outputs.push((run, output, success));
    }

    let mode = step.mode.unwrap_or(Mode::All);
    let should_skip = step.skip.is_some();
    let passed = should_skip || match mode {
        Mode::All => outputs.iter().all(|&(_, _, ref success)| *success),
        Mode::Any => outputs.iter().any(|&(_, _, ref success)| *success),
    };

    let results = outputs.iter().map(|&(ref run, ref output, _)| {
        BenchmarkResult {
            section: step.section.clone(),
            description: step.description.clone(),
            run: run.to_string(),
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

fn write_report(results: &Vec<BenchmarkResult>, output: &str) -> Result<(), AuditError> {
    let mut writer = csv::Writer::from_file(path::Path::new(output))?;
    let headers = vec![
        "Section", "Description", "Run", "Passed", "Output", "Error", "Skip"
    ];
    writer.write(headers.iter())?;
    for result in results {
        writer.encode(result)?;
    }
    Ok(())
}

pub fn run_scan(spec: &str, output: &str) -> Result<(), AuditError> {
    load_benchmarks(spec)
        .and_then(|benchmarks| run_benchmarks(&benchmarks))
        .and_then(|results| write_report(&results, output).and(Ok(results)))
        .and_then(|results| {
            let passed = results.iter().all(|r| r.passed);
            match passed {
                true => Ok(()),
                false => Err(AuditError::NonCompliant)
            }
        })
}
