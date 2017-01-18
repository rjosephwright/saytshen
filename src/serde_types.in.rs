#[derive(Serialize, Deserialize, Debug, Clone)]
enum Mode {
    All,
    Any,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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
