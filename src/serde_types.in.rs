#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
enum Mode {
    All,
    Any,
}

#[derive(Serialize, Deserialize, Debug)]
struct AuditStep {
    run: String,
    expect: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Benchmark {
    section: String,
    description: String,
    audit: Vec<AuditStep>,
    mode: Option<Mode>,
    skip: Option<String>,
}
