use std::fs;
use std::io;
use std::path;
use std::process;

use csv;
use serde_yaml;

include!(concat!(env!("OUT_DIR"), "/serde_types.rs"));

impl Benchmark {
    fn expand(&self) -> Vec<BenchmarkStep> {
        self.audit.iter()
            .map(|a| {
                BenchmarkStep {
                    section: self.section.clone(),
                    description: self.description.clone(),
                    audit: a.clone(),
                    mode: self.mode.clone(),
                    skip: self.skip.clone(),
                }
            })
            .collect::<Vec<_>>()
    }
}

// Like Benchmark except there is just one audit step
// instead of a vector of them.
#[derive(Debug)]
struct BenchmarkStep {
    section: String,
    description: String,
    audit: AuditStep,
    mode: Option<Mode>,
    skip: Option<String>,
}

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

fn run_script(script: String) -> Result<process::Output, AuditError> {
    process::Command::new("/bin/sh")
        .arg("-c")
        .arg(script)
        .stdin(process::Stdio::null())
        .stdout(process::Stdio::piped())
        .stderr(process::Stdio::piped())
        .spawn().map_err(AuditError::IoError)
        .and_then(|child| child.wait_with_output().map_err(AuditError::IoError))
}

fn run_benchmark(step: &BenchmarkStep) -> Result<BenchmarkResult, AuditError> {
    let script = step.audit.run.clone();
    run_script(script).and_then(|output| {
        Ok(BenchmarkResult {
            section: step.section.clone(),
            description: step.description.clone(),
            run: step.audit.run.clone(),
            passed: output.status.success(),
            output: String::from_utf8(output.stdout.clone()).ok(),
            error: String::from_utf8(output.stderr.clone()).ok(),
            skip: step.skip.clone(),
        })
    })
}

fn run_benchmarks(steps: &Vec<BenchmarkStep>) -> Result<Vec<BenchmarkResult>, AuditError> {
    let mut results = Vec::new();
    for step in steps {
        let result = run_benchmark(&step)?;
        results.push(result);
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
        .and_then(|benchmarks| {
            // Flatten benchmarks with multiple steps into a vector of
            // single benchmark steps, for uniform error handling.
            Ok(benchmarks.iter().flat_map(|b| b.expand()).collect::<Vec<_>>())
        })
        .and_then(|steps| run_benchmarks(&steps))
        .and_then(|results| write_report(&results).and(Ok(results)))
        .and_then(|results| {
            let passed = results.iter().all(|r| r.passed);
            match passed {
                true => Ok(()),
                false => Err(AuditError::NonCompliant)
            }
        })
}
